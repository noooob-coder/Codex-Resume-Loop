use crate::codex::{
    build_resume_prompt, prepare_new_session_exec_command, prepare_resume_command,
    resolve_codex_launch,
};
use crate::diagnostics::append_log;
use crate::model::{LogEntry, LogStream, WorkspaceRunRequest};
use chrono::Local;
use crossbeam_channel::Sender;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(test)]
use crossbeam_channel::unbounded;

#[cfg(test)]
use std::collections::VecDeque;

#[cfg(test)]
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum TaskOutcome {
    Completed,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    Log {
        workspace_id: u64,
        entry: LogEntry,
    },
    OutputChunk {
        workspace_id: u64,
        stream: LogStream,
        chunk: String,
    },
    RoundStarted {
        workspace_id: u64,
        current_round: u32,
        total_rounds: u32,
    },
    Finished {
        workspace_id: u64,
        outcome: TaskOutcome,
    },
}

pub struct TaskHandle {
    stop_flag: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
    _join: thread::JoinHandle<()>,
}

impl TaskHandle {
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.child.lock()
            && let Some(child) = guard.as_mut()
        {
            let _ = child.kill();
        }
    }
}

pub fn spawn_workspace_runner(
    request: WorkspaceRunRequest,
    sender: Sender<RuntimeEvent>,
) -> TaskHandle {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let child_slot = Arc::new(Mutex::new(None));
    let worker_stop = Arc::clone(&stop_flag);
    let worker_child = Arc::clone(&child_slot);
    let join = thread::spawn(move || {
        run_workspace_loop(request, sender, worker_stop, worker_child);
    });

    TaskHandle {
        stop_flag,
        child: child_slot,
        _join: join,
    }
}

pub fn spawn_new_session_runner(
    workspace_id: u64,
    path: PathBuf,
    sender: Sender<RuntimeEvent>,
) -> TaskHandle {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let child_slot = Arc::new(Mutex::new(None));
    let worker_stop = Arc::clone(&stop_flag);
    let worker_child = Arc::clone(&child_slot);
    let join = thread::spawn(move || {
        run_new_session(workspace_id, path, sender, worker_stop, worker_child);
    });

    TaskHandle {
        stop_flag,
        child: child_slot,
        _join: join,
    }
}

const NEW_SESSION_MAX_ATTEMPTS: u32 = 2;

#[derive(Default)]
struct CodexStdoutFilter {
    pending_line: String,
    keeping_line: bool,
    suppress_next_line: bool,
    suppress_next_numeric_line: bool,
    suppress_shell_block: bool,
}

fn run_workspace_loop(
    request: WorkspaceRunRequest,
    sender: Sender<RuntimeEvent>,
    stop_flag: Arc<AtomicBool>,
    child_slot: Arc<Mutex<Option<Child>>>,
) {
    let round_prompt = build_resume_prompt(&request.prompt);
    let mut failed_rounds = Vec::new();

    for current_round in 1..=request.rounds {
        append_log(&format!(
            "runtime loop workspace_id={} round={}/{}",
            request.workspace_id, current_round, request.rounds
        ));
        if stop_flag.load(Ordering::SeqCst) {
            let _ = sender.send(RuntimeEvent::Finished {
                workspace_id: request.workspace_id,
                outcome: TaskOutcome::Stopped,
            });
            return;
        }

        let _ = sender.send(RuntimeEvent::RoundStarted {
            workspace_id: request.workspace_id,
            current_round,
            total_rounds: request.rounds,
        });
        let _ = sender.send(RuntimeEvent::Log {
            workspace_id: request.workspace_id,
            entry: LogEntry {
                timestamp: Local::now(),
                stream: LogStream::System,
                text: format!(
                    "开始第 {current_round}/{total} 轮，目标会话：{}",
                    request.session_id,
                    total = request.rounds
                ),
            },
        });

        let launch = match resolve_codex_launch() {
            Ok(launch) => launch,
            Err(error) => {
                append_log(&format!("resolve_codex_launch failed: {error}"));
                let _ = sender.send(RuntimeEvent::Finished {
                    workspace_id: request.workspace_id,
                    outcome: TaskOutcome::Error(error.to_string()),
                });
                return;
            }
        };
        append_log(&format!(
            "resolved codex launch workspace_id={} -> {}",
            request.workspace_id,
            launch.describe()
        ));
        let mut command = prepare_resume_command(&launch, &request.session_id, &round_prompt);
        command
            .current_dir(&request.path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        append_log(&format!(
            "spawning codex process workspace_id={} cwd={}",
            request.workspace_id,
            request.path.display()
        ));

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                append_log(&format!("command.spawn failed: {error}"));
                let _ = sender.send(RuntimeEvent::Finished {
                    workspace_id: request.workspace_id,
                    outcome: TaskOutcome::Error(format!("无法启动 codex 进程：{error}")),
                });
                return;
            }
        };
        append_log(&format!(
            "spawned codex process workspace_id={} pid={}",
            request.workspace_id,
            child.id()
        ));

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        if let Ok(mut guard) = child_slot.lock() {
            *guard = Some(child);
        }

        let stdout_join = stdout.map(|stdout| {
            spawn_stream_reader(
                stdout,
                request.workspace_id,
                LogStream::Stdout,
                sender.clone(),
            )
        });
        let stderr_join = stderr.map(|stderr| {
            spawn_stream_reader(
                stderr,
                request.workspace_id,
                LogStream::Stderr,
                sender.clone(),
            )
        });

        let exit_status = loop {
            if stop_flag.load(Ordering::SeqCst)
                && let Ok(mut guard) = child_slot.lock()
                && let Some(child) = guard.as_mut()
            {
                let _ = child.kill();
            }

            let maybe_status = {
                let mut guard = match child_slot.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        let _ = sender.send(RuntimeEvent::Finished {
                            workspace_id: request.workspace_id,
                            outcome: TaskOutcome::Error("无法锁定正在运行的任务句柄".to_owned()),
                        });
                        return;
                    }
                };

                if let Some(child) = guard.as_mut() {
                    match child.try_wait() {
                        Ok(status) => status,
                        Err(error) => {
                            let _ = sender.send(RuntimeEvent::Finished {
                                workspace_id: request.workspace_id,
                                outcome: TaskOutcome::Error(format!("无法读取进程状态：{error}")),
                            });
                            return;
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(status) = maybe_status {
                break status;
            }

            thread::sleep(Duration::from_millis(120));
        };

        if let Ok(mut guard) = child_slot.lock() {
            guard.take();
        }
        if let Some(join) = stdout_join {
            let _ = join.join();
        }
        if let Some(join) = stderr_join {
            let _ = join.join();
        }

        if stop_flag.load(Ordering::SeqCst) {
            let _ = sender.send(RuntimeEvent::Finished {
                workspace_id: request.workspace_id,
                outcome: TaskOutcome::Stopped,
            });
            return;
        }

        if !exit_status.success() {
            let code = exit_status
                .code()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "未知".to_owned());
            append_log(&format!(
                "runtime non-zero exit workspace_id={} round={} code={}",
                request.workspace_id, current_round, code
            ));
            failed_rounds.push((current_round, code.clone()));
            let _ = sender.send(RuntimeEvent::Log {
                workspace_id: request.workspace_id,
                entry: LogEntry {
                    timestamp: Local::now(),
                    stream: LogStream::System,
                    text: format!("第 {current_round} 轮失败，退出码：{code}；继续下一轮。"),
                },
            });
            continue;
        }

        let _ = sender.send(RuntimeEvent::Log {
            workspace_id: request.workspace_id,
            entry: LogEntry {
                timestamp: Local::now(),
                stream: LogStream::System,
                text: format!("第 {current_round} 轮已完成。"),
            },
        });
    }

    append_log(&format!(
        "runtime completed workspace_id={}",
        request.workspace_id
    ));
    if failed_rounds.is_empty() {
        let _ = sender.send(RuntimeEvent::Finished {
            workspace_id: request.workspace_id,
            outcome: TaskOutcome::Completed,
        });
        return;
    }

    let summary = failed_rounds
        .iter()
        .map(|(round, code)| format!("{round}:{code}"))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = sender.send(RuntimeEvent::Finished {
        workspace_id: request.workspace_id,
        outcome: TaskOutcome::Error(format!("已尝试全部轮次，但以下轮次失败：{summary}")),
    });
}

fn run_new_session(
    workspace_id: u64,
    path: PathBuf,
    sender: Sender<RuntimeEvent>,
    stop_flag: Arc<AtomicBool>,
    child_slot: Arc<Mutex<Option<Child>>>,
) {
    let launch = match resolve_codex_launch() {
        Ok(launch) => launch,
        Err(error) => {
            append_log(&format!("resolve_codex_launch failed: {error}"));
            let _ = sender.send(RuntimeEvent::Finished {
                workspace_id,
                outcome: TaskOutcome::Error(error.to_string()),
            });
            return;
        }
    };

    for attempt in 1..=NEW_SESSION_MAX_ATTEMPTS {
        let _ = sender.send(RuntimeEvent::RoundStarted {
            workspace_id,
            current_round: attempt,
            total_rounds: NEW_SESSION_MAX_ATTEMPTS,
        });
        let _ = sender.send(RuntimeEvent::Log {
            workspace_id,
            entry: LogEntry {
                timestamp: Local::now(),
                stream: LogStream::System,
                text: if attempt == 1 {
                    "Creating a new Codex conversation.".to_owned()
                } else {
                    format!(
                        "Retrying the new Codex conversation after a transient stream disconnect ({attempt}/{NEW_SESSION_MAX_ATTEMPTS})."
                    )
                },
            },
        });

        let mut command = prepare_new_session_exec_command(&launch);
        command.current_dir(&path).stdout(Stdio::piped()).stderr(Stdio::piped());
        append_log(&format!(
            "spawning new-session codex process workspace_id={} cwd={} attempt={}",
            workspace_id,
            path.display(),
            attempt
        ));

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                append_log(&format!("new-session command.spawn failed: {error}"));
                let _ = sender.send(RuntimeEvent::Finished {
                    workspace_id,
                    outcome: TaskOutcome::Error(format!(
                        "Failed to start a new Codex conversation: {error}"
                    )),
                });
                return;
            }
        };

        let capture = Arc::new(Mutex::new(String::new()));
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        if let Ok(mut guard) = child_slot.lock() {
            *guard = Some(child);
        }

        let stdout_join = stdout.map(|stdout| {
            spawn_stream_reader_with_capture(
                stdout,
                workspace_id,
                LogStream::Stdout,
                sender.clone(),
                Arc::clone(&capture),
            )
        });
        let stderr_join = stderr.map(|stderr| {
            spawn_stream_reader_with_capture(
                stderr,
                workspace_id,
                LogStream::Stderr,
                sender.clone(),
                Arc::clone(&capture),
            )
        });

        let exit_status = loop {
            if stop_flag.load(Ordering::SeqCst)
                && let Ok(mut guard) = child_slot.lock()
                && let Some(child) = guard.as_mut()
            {
                let _ = child.kill();
            }

            let maybe_status = {
                let mut guard = match child_slot.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        let _ = sender.send(RuntimeEvent::Finished {
                            workspace_id,
                            outcome: TaskOutcome::Error(
                                "Failed to lock the running new-session process.".to_owned(),
                            ),
                        });
                        return;
                    }
                };

                if let Some(child) = guard.as_mut() {
                    match child.try_wait() {
                        Ok(status) => status,
                        Err(error) => {
                            let _ = sender.send(RuntimeEvent::Finished {
                                workspace_id,
                                outcome: TaskOutcome::Error(format!(
                                    "Failed to read the new-session process state: {error}"
                                )),
                            });
                            return;
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(status) = maybe_status {
                break status;
            }

            thread::sleep(Duration::from_millis(120));
        };

        if let Ok(mut guard) = child_slot.lock() {
            guard.take();
        }
        if let Some(join) = stdout_join {
            let _ = join.join();
        }
        if let Some(join) = stderr_join {
            let _ = join.join();
        }

        if stop_flag.load(Ordering::SeqCst) {
            let _ = sender.send(RuntimeEvent::Finished {
                workspace_id,
                outcome: TaskOutcome::Stopped,
            });
            return;
        }

        if exit_status.success() {
            let _ = sender.send(RuntimeEvent::Log {
                workspace_id,
                entry: LogEntry {
                    timestamp: Local::now(),
                    stream: LogStream::System,
                    text: "New Codex conversation created.".to_owned(),
                },
            });
            let _ = sender.send(RuntimeEvent::Finished {
                workspace_id,
                outcome: TaskOutcome::Completed,
            });
            return;
        }

        let captured_output = capture
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default();
        if attempt < NEW_SESSION_MAX_ATTEMPTS
            && is_transient_stream_disconnect(&captured_output)
        {
            thread::sleep(Duration::from_millis(600));
            continue;
        }

        let code = exit_status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_owned());
        let _ = sender.send(RuntimeEvent::Finished {
            workspace_id,
            outcome: TaskOutcome::Error(format!(
                "The new Codex conversation exited with code {code}."
            )),
        });
        return;
    }
}

impl CodexStdoutFilter {
    fn push(&mut self, chunk: &str) -> String {
        let mut output = String::new();

        for ch in chunk.chars() {
            if self.keeping_line {
                output.push(ch);
                if ch == '\n' {
                    self.keeping_line = false;
                    self.pending_line.clear();
                }
                continue;
            }

            if ch == '\r' {
                continue;
            }

            if ch == '\n' {
                if !self.should_suppress_completed_line() {
                    output.push_str(&self.pending_line);
                    output.push('\n');
                }
                self.finish_line();
                continue;
            }

            self.pending_line.push(ch);
            if !self.should_hold_line() {
                output.push_str(&self.pending_line);
                self.pending_line.clear();
                self.keeping_line = true;
            }
        }

        output
    }

    fn finish(&mut self) -> String {
        if self.keeping_line {
            self.keeping_line = false;
            self.pending_line.clear();
            return String::new();
        }

        if self.pending_line.is_empty() {
            return String::new();
        }

        if self.should_suppress_completed_line() {
            self.finish_line();
            return String::new();
        }

        let line = self.pending_line.clone();
        self.finish_line();
        line
    }

    fn should_hold_line(&self) -> bool {
        if self.suppress_shell_block || self.suppress_next_line || self.suppress_next_numeric_line {
            return true;
        }
        is_possible_filtered_prefix(&self.pending_line)
    }

    fn should_suppress_completed_line(&mut self) -> bool {
        let trimmed = self.pending_line.trim();
        if trimmed.is_empty() {
            return false;
        }

        if self.suppress_shell_block {
            if trimmed.eq_ignore_ascii_case("</user_shell_command>") {
                self.suppress_shell_block = false;
            }
            return true;
        }

        if self.suppress_next_line {
            self.suppress_next_line = false;
            return true;
        }
        if self.suppress_next_numeric_line {
            self.suppress_next_numeric_line = false;
            return trimmed
                .chars()
                .all(|ch| ch.is_ascii_digit() || ch == ',' || ch == ' ');
        }

        let lowered = trimmed.to_ascii_lowercase();
        if lowered == "<user_shell_command>" {
            self.suppress_shell_block = true;
            return true;
        }
        if lowered == "user" {
            self.suppress_next_line = true;
            return true;
        }
        if lowered == "codex" || lowered == "--------" || lowered == "tokens used" {
            if lowered == "tokens used" {
                self.suppress_next_numeric_line = true;
            }
            return true;
        }

        starts_with_filtered_prefix(&lowered) || is_internal_transcript_line(trimmed, &lowered)
    }

    fn finish_line(&mut self) {
        self.pending_line.clear();
        self.keeping_line = false;
    }
}

fn starts_with_filtered_prefix(line: &str) -> bool {
    [
        "openai codex",
        "workdir:",
        "model:",
        "provider:",
        "approval:",
        "sandbox:",
        "reasoning effort:",
        "reasoning summaries:",
        "session id:",
        "mcp startup:",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

fn is_possible_filtered_prefix(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    [
        "openai codex",
        "workdir:",
        "model:",
        "provider:",
        "approval:",
        "sandbox:",
        "reasoning effort:",
        "reasoning summaries:",
        "session id:",
        "mcp startup:",
        "user",
        "codex",
        "--------",
        "tokens used",
        "<user_shell_command>",
        "! exec",
        "! \"",
        "! succeeded in",
        "! exited",
        "! e:\\",
    ]
    .iter()
    .any(|prefix| prefix.starts_with(&lowered))
}

fn is_internal_transcript_line(trimmed: &str, lowered: &str) -> bool {
    lowered == "<command>"
        || lowered == "</command>"
        || lowered == "<result>"
        || lowered == "</result>"
        || lowered == "<user_shell_command>"
        || lowered == "</user_shell_command>"
        || lowered == "! exec"
        || lowered.starts_with("! \"")
        || lowered.starts_with("!  succeeded in")
        || lowered.starts_with("! succeeded in")
        || lowered.starts_with("!  exited")
        || lowered.starts_with("! exited")
        || lowered.starts_with("! e:\\")
        || lowered.starts_with("! c:\\")
        || (trimmed.starts_with("At line:") && lowered.contains("char:"))
        || lowered.starts_with("+ ! ")
        || lowered.starts_with("+   ~")
        || lowered.starts_with("missing expression after unary operator")
        || lowered.starts_with("unexpected token")
        || lowered.starts_with("not all parse errors were reported")
        || lowered.starts_with("    + categoryinfo")
        || lowered.starts_with("    + fullyqualifiederrorid")
}

fn spawn_stream_reader<R>(
    reader: R,
    workspace_id: u64,
    stream: LogStream,
    sender: Sender<RuntimeEvent>,
) -> thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    spawn_stream_reader_impl(reader, workspace_id, stream, sender, None)
}

fn spawn_stream_reader_with_capture<R>(
    reader: R,
    workspace_id: u64,
    stream: LogStream,
    sender: Sender<RuntimeEvent>,
    capture: Arc<Mutex<String>>,
) -> thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    spawn_stream_reader_impl(reader, workspace_id, stream, sender, Some(capture))
}

fn spawn_stream_reader_impl<R>(
    reader: R,
    workspace_id: u64,
    stream: LogStream,
    sender: Sender<RuntimeEvent>,
    capture: Option<Arc<Mutex<String>>>,
) -> thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        forward_stream_chunks(reader, workspace_id, stream, sender, capture);
    })
}

fn forward_stream_chunks<R>(
    mut reader: R,
    workspace_id: u64,
    stream: LogStream,
    sender: Sender<RuntimeEvent>,
    capture: Option<Arc<Mutex<String>>>,
) where
    R: Read,
{
    let mut buffer = [0_u8; 4096];
    let mut decoder = Utf8ChunkDecoder::default();
    let mut stdout_filter =
        matches!(stream, LogStream::Stdout | LogStream::Stderr).then(CodexStdoutFilter::default);

    loop {
        let read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(_) => return,
        };

        let raw_chunk = decoder.push(&buffer[..read]);
        let chunk = if let Some(filter) = stdout_filter.as_mut() {
            filter.push(&raw_chunk)
        } else {
            raw_chunk
        };
        if !chunk.is_empty() {
            if let Some(capture) = capture.as_ref()
                && let Ok(mut transcript) = capture.lock()
            {
                transcript.push_str(&chunk);
            }
            let _ = sender.send(RuntimeEvent::OutputChunk {
                workspace_id,
                stream,
                chunk,
            });
        }
    }

    let raw_final_chunk = decoder.finish();
    let final_chunk = if let Some(filter) = stdout_filter.as_mut() {
        let mut combined = String::new();
        if !raw_final_chunk.is_empty() {
            combined.push_str(&filter.push(&raw_final_chunk));
        }
        combined.push_str(&filter.finish());
        combined
    } else {
        raw_final_chunk
    };
    if !final_chunk.is_empty() {
        if let Some(capture) = capture.as_ref()
            && let Ok(mut transcript) = capture.lock()
        {
            transcript.push_str(&final_chunk);
        }
        let _ = sender.send(RuntimeEvent::OutputChunk {
            workspace_id,
            stream,
            chunk: final_chunk,
        });
    }
}

fn is_transient_stream_disconnect(output: &str) -> bool {
    let lowered = output.to_ascii_lowercase();
    lowered.contains("stream disconnected before completion")
        || lowered.contains("disconnected before completion")
}

#[derive(Default)]
struct Utf8ChunkDecoder {
    pending: Vec<u8>,
}

impl Utf8ChunkDecoder {
    fn push(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        let mut output = String::new();

        loop {
            match std::str::from_utf8(&self.pending) {
                Ok(valid) => {
                    output.push_str(valid);
                    self.pending.clear();
                    break;
                }
                Err(error) => {
                    let valid_up_to = error.valid_up_to();
                    if valid_up_to > 0 {
                        output.push_str(
                            std::str::from_utf8(&self.pending[..valid_up_to])
                                .expect("valid utf8 prefix"),
                        );
                    }

                    if let Some(error_len) = error.error_len() {
                        output.push('\u{FFFD}');
                        self.pending.drain(..valid_up_to + error_len);
                        continue;
                    }

                    self.pending.drain(..valid_up_to);
                    break;
                }
            }
        }

        output
    }

    fn finish(&mut self) -> String {
        if self.pending.is_empty() {
            return String::new();
        }

        let output = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::RecvTimeoutError;

    struct ChunkedReader {
        chunks: VecDeque<(Duration, Vec<u8>)>,
    }

    impl ChunkedReader {
        fn new(chunks: Vec<(Duration, Vec<u8>)>) -> Self {
            Self {
                chunks: chunks.into(),
            }
        }
    }

    impl Read for ChunkedReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let Some((delay, chunk)) = self.chunks.pop_front() else {
                return Ok(0);
            };
            if !delay.is_zero() {
                thread::sleep(delay);
            }
            let len = chunk.len().min(buf.len());
            buf[..len].copy_from_slice(&chunk[..len]);
            Ok(len)
        }
    }

    #[test]
    fn stream_reader_emits_partial_chunks_before_newline() {
        let (sender, receiver) = unbounded();
        let reader = ChunkedReader::new(vec![
            (Duration::ZERO, b"hello".to_vec()),
            (Duration::from_millis(250), b" world\n".to_vec()),
        ]);
        let started = Instant::now();
        let join = spawn_stream_reader(reader, 7, LogStream::Stdout, sender);

        let first = receiver.recv_timeout(Duration::from_millis(100));
        match first {
            Ok(RuntimeEvent::OutputChunk { chunk, .. }) => assert_eq!(chunk, "hello"),
            other => panic!("expected immediate output chunk, got {other:?}"),
        }

        let second = receiver.recv_timeout(Duration::from_millis(400));
        match second {
            Ok(RuntimeEvent::OutputChunk { chunk, .. }) => assert_eq!(chunk, " world\n"),
            other => panic!("expected second output chunk, got {other:?}"),
        }

        assert!(started.elapsed() < Duration::from_millis(500));
        join.join().expect("reader join");

        match receiver.recv_timeout(Duration::from_millis(50)) {
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {}
            other => panic!("expected no more chunks, got {other:?}"),
        }
    }

    #[test]
    fn codex_stdout_filter_removes_internal_shell_transcript_block() {
        let mut filter = CodexStdoutFilter::default();
        let chunk = "<user_shell_command>\n<command>\n! exec\n! \"C:\\\\Windows\\\\System32\\\\WindowsPowerShell\\\\v1.0\\\\powershell.exe\" -Command \"rg\"\n</command>\n<result>\n! E:\\project\\run_spider\\target\\debug\\build\\crl.rs:1\n</result>\n</user_shell_command>\n";
        let output = filter.push(chunk);
        let final_chunk = filter.finish();

        assert!(output.is_empty());
        assert!(final_chunk.is_empty());
    }

    #[test]
    fn codex_stdout_filter_keeps_normal_assistant_lines() {
        let mut filter = CodexStdoutFilter::default();
        let output = filter.push("Waited for background terminal\n");
        let final_chunk = filter.finish();

        assert_eq!(output + &final_chunk, "Waited for background terminal\n");
    }

    #[test]
    fn utf8_decoder_keeps_split_multibyte_sequences() {
        let mut decoder = Utf8ChunkDecoder::default();
        let first = decoder.push(&[0xE4, 0xBD]);
        let second = decoder.push(&[0xA0, 0xE5, 0xA5, 0xBD]);

        assert!(first.is_empty());
        assert_eq!(second, "你好");
    }
}
