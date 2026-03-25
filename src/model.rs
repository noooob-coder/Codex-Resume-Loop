use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

pub const MAX_LOG_LINES: usize = 600;
pub const MAX_TERMINAL_CHARS: usize = 200_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAppState {
    pub codex_home: Option<String>,
    pub selected_workspace_id: Option<u64>,
    pub next_workspace_id: u64,
    pub auto_refresh_enabled: bool,
    pub auto_refresh_seconds: u32,
    pub workspaces: Vec<StoredWorkspace>,
}

impl Default for StoredAppState {
    fn default() -> Self {
        Self {
            codex_home: None,
            selected_workspace_id: None,
            next_workspace_id: 0,
            auto_refresh_enabled: true,
            auto_refresh_seconds: 30,
            workspaces: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredWorkspace {
    pub id: u64,
    pub label: String,
    pub path: String,
    pub prompt: String,
    pub rounds: u32,
    pub selected_session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub id: u64,
    pub label: String,
    pub path: String,
    pub prompt: String,
    pub rounds: u32,
    pub selected_session_id: Option<String>,
    pub sessions: Vec<SessionSummary>,
    pub logs: VecDeque<LogEntry>,
    pub terminal_output: String,
    pub status: RunStatus,
    pub last_refresh: Option<DateTime<Local>>,
    pub last_error: Option<String>,
    pub auto_scroll_logs: bool,
    terminal_at_line_start: bool,
    terminal_stream: Option<LogStream>,
}

impl WorkspaceState {
    pub fn from_stored(stored: StoredWorkspace) -> Self {
        Self {
            id: stored.id,
            label: stored.label,
            path: stored.path,
            prompt: stored.prompt,
            rounds: stored.rounds.max(1),
            selected_session_id: stored.selected_session_id,
            sessions: Vec::new(),
            logs: VecDeque::new(),
            terminal_output: String::new(),
            status: RunStatus::Idle,
            last_refresh: None,
            last_error: None,
            auto_scroll_logs: true,
            terminal_at_line_start: true,
            terminal_stream: None,
        }
    }

    pub fn to_stored(&self) -> StoredWorkspace {
        StoredWorkspace {
            id: self.id,
            label: self.label.clone(),
            path: self.path.clone(),
            prompt: self.prompt.clone(),
            rounds: self.rounds.max(1),
            selected_session_id: self.selected_session_id.clone(),
        }
    }

    pub fn display_name(&self) -> String {
        if !self.label.trim().is_empty() {
            return self.label.trim().to_owned();
        }

        let path = Path::new(&self.path);
        path.file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| self.path.clone())
    }

    pub fn push_log(&mut self, log: LogEntry) {
        let stream = log.stream;
        self.logs.push_back(log);
        if !matches!(stream, LogStream::System) {
            let rendered = format_terminal_entry(self.logs.back().expect("just pushed log"));
            if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                self.terminal_output.push('\n');
            }
            self.terminal_output.push_str(&rendered);
            self.terminal_at_line_start = false;
            self.terminal_stream = Some(stream);
            self.trim_terminal_output();
        }
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }

    pub fn append_output_chunk(&mut self, stream: LogStream, chunk: &str) {
        if chunk.is_empty() {
            return;
        }

        for ch in chunk.chars() {
            let ch = if ch == '\r' { '\n' } else { ch };
            if ch == '\n' {
                self.terminal_output.push('\n');
                self.terminal_at_line_start = true;
                self.terminal_stream = None;
                continue;
            }

            if self.terminal_at_line_start {
                self.terminal_output.push_str(stream_prefix(stream));
                self.terminal_output.push(' ');
                self.terminal_at_line_start = false;
                self.terminal_stream = Some(stream);
            } else if self.terminal_stream != Some(stream) {
                self.terminal_output.push('\n');
                self.terminal_output.push_str(stream_prefix(stream));
                self.terminal_output.push(' ');
                self.terminal_stream = Some(stream);
            }

            self.terminal_output.push(ch);
        }

        self.trim_terminal_output();
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
        self.terminal_output.clear();
        self.terminal_at_line_start = true;
        self.terminal_stream = None;
    }

    pub fn selected_session(&self) -> Option<&SessionSummary> {
        let selected = self.selected_session_id.as_deref()?;
        self.sessions
            .iter()
            .find(|session| session.session_id == selected)
    }

    pub fn ensure_selected_session(&mut self) {
        if self.sessions.is_empty() {
            self.selected_session_id = None;
            return;
        }

        let selected_exists = self
            .selected_session_id
            .as_deref()
            .map(|selected| {
                self.sessions
                    .iter()
                    .any(|session| session.session_id == selected)
            })
            .unwrap_or(false);

        if !selected_exists {
            self.selected_session_id = Some(self.sessions[0].session_id.clone());
        }
    }

    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(self.path.clone())
    }

    fn trim_terminal_output(&mut self) {
        if self.terminal_output.len() <= MAX_TERMINAL_CHARS {
            return;
        }

        let overflow = self.terminal_output.len() - MAX_TERMINAL_CHARS;
        let trim_start = self
            .terminal_output
            .char_indices()
            .find(|(index, _)| *index >= overflow)
            .map(|(index, _)| index)
            .unwrap_or(self.terminal_output.len());
        let trim_end = self.terminal_output[trim_start..]
            .find('\n')
            .map(|offset| trim_start + offset + 1)
            .unwrap_or(trim_start);
        self.terminal_output.drain(..trim_end);
        self.terminal_at_line_start =
            self.terminal_output.is_empty() || self.terminal_output.ends_with('\n');
        self.terminal_stream = None;
    }
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub session_id: String,
    pub title: String,
    pub last_text: String,
    pub last_activity: DateTime<Local>,
    #[allow(dead_code)]
    pub file_path: PathBuf,
    pub message_count: usize,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub stream: LogStream,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStream {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Default)]
pub enum RunStatus {
    #[default]
    Idle,
    NoSessions,
    Running {
        current_round: u32,
        total_rounds: u32,
    },
    Completed {
        finished_at: DateTime<Local>,
    },
    Stopped {
        finished_at: DateTime<Local>,
    },
    Error(String),
}

impl RunStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Stopped { .. } | Self::Error(_)
        )
    }

    pub fn label(&self) -> String {
        match self {
            Self::Idle => "待命".to_owned(),
            Self::NoSessions => "无会话".to_owned(),
            Self::Running {
                current_round,
                total_rounds,
            } => format!("运行中 {}/{}", current_round, total_rounds),
            Self::Completed { .. } => "已完成".to_owned(),
            Self::Stopped { .. } => "已停止".to_owned(),
            Self::Error(_) => "异常".to_owned(),
        }
    }

    pub fn detail(&self) -> Option<String> {
        match self {
            Self::Completed { finished_at } => Some(format!(
                "任务完成于 {}",
                finished_at.format("%Y-%m-%d %H:%M:%S")
            )),
            Self::Stopped { finished_at } => Some(format!(
                "任务停止于 {}",
                finished_at.format("%Y-%m-%d %H:%M:%S")
            )),
            Self::Error(message) => Some(message.clone()),
            Self::NoSessions => Some("当前工作区未发现可用的 Codex 会话。".to_owned()),
            _ => None,
        }
    }
}

fn format_terminal_entry(entry: &LogEntry) -> String {
    format!("{} {}", stream_prefix(entry.stream), entry.text)
}

fn stream_prefix(stream: LogStream) -> &'static str {
    match stream {
        LogStream::Stdout => ">",
        LogStream::Stderr => "!",
        LogStream::System => "#",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_workspace() -> WorkspaceState {
        WorkspaceState::from_stored(StoredWorkspace {
            id: 1,
            label: "demo".into(),
            path: r"E:\demo".into(),
            prompt: "继续".into(),
            rounds: 1,
            selected_session_id: None,
        })
    }

    #[test]
    fn push_log_updates_terminal_output_cache() {
        let mut workspace = sample_workspace();
        workspace.push_log(LogEntry {
            timestamp: Local::now(),
            stream: LogStream::System,
            text: "准备开始".into(),
        });
        workspace.push_log(LogEntry {
            timestamp: Local::now(),
            stream: LogStream::Stdout,
            text: "working".into(),
        });

        assert!(workspace.terminal_output.contains("> working"));
        assert!(!workspace.terminal_output.contains("# 准备开始"));
    }

    #[test]
    fn append_output_chunk_is_realtime_without_waiting_for_newline() {
        let mut workspace = sample_workspace();

        workspace.append_output_chunk(LogStream::Stdout, "hello");
        assert_eq!(workspace.terminal_output, "> hello");

        workspace.append_output_chunk(LogStream::Stdout, " world\nnext");
        assert_eq!(workspace.terminal_output, "> hello world\n> next");
    }

    #[test]
    fn append_output_chunk_starts_new_line_when_stream_changes() {
        let mut workspace = sample_workspace();

        workspace.append_output_chunk(LogStream::Stdout, "hello");
        workspace.append_output_chunk(LogStream::Stderr, "oops");

        assert_eq!(workspace.terminal_output, "> hello\n! oops");
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceRunRequest {
    pub workspace_id: u64,
    pub path: PathBuf,
    pub session_id: String,
    pub prompt: String,
    pub rounds: u32,
}
