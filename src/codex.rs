use crate::model::SessionSummary;
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use directories::BaseDirs;
use dunce::canonicalize;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub const DEFAULT_RESUME_ROUNDS: u32 = 1;

const RESUME_EXECUTION_CONTRACT: &str = "Execution contract:\n\
- Do not ask the user whether to continue, whether to proceed, or whether they want another round.\n\
- Continue from the exact previous stopping point and keep working until the original request is fully satisfied.\n\
- When you think you are done, compare the current result against the original request item by item. If anything is missing, continue and fill the gap instead of stopping.\n\
- Only stop early for a real blocker that cannot be resolved locally, such as a missing dependency, missing permission, or unavailable required input that cannot be inferred.";

#[derive(Debug, Clone)]
pub struct CodexLaunch {
    program: PathBuf,
    prefix_args: Vec<OsString>,
}

impl CodexLaunch {
    pub fn command(&self) -> Command {
        self.command_with_window_hidden(true)
    }

    pub fn interactive_command(&self) -> Command {
        self.command_with_window_hidden(false)
    }

    fn command_with_window_hidden(&self, _hide_window: bool) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.prefix_args);
        #[cfg(target_os = "windows")]
        if _hide_window {
            command.creation_flags(CREATE_NO_WINDOW);
        }
        command
    }

    pub fn describe(&self) -> String {
        let mut parts = Vec::with_capacity(1 + self.prefix_args.len());
        parts.push(self.program.display().to_string());
        parts.extend(
            self.prefix_args
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned()),
        );
        parts.join(" ")
    }
}

#[derive(Debug, Deserialize)]
struct SessionMetaEnvelope {
    #[serde(rename = "type")]
    line_type: String,
    payload: SessionMetaPayload,
}

#[derive(Debug, Deserialize)]
struct SessionMetaPayload {
    id: String,
    timestamp: String,
    cwd: String,
}

#[derive(Debug, Deserialize)]
struct HistoryEntry {
    session_id: String,
    ts: i64,
    text: String,
}

#[derive(Debug, Clone)]
struct RawSession {
    session_id: String,
    started_at: DateTime<Local>,
    file_path: PathBuf,
}

#[derive(Debug, Clone)]
struct IndexedSession {
    workspace_key: String,
    raw_session: RawSession,
}

#[derive(Debug, Default)]
struct HistorySummary {
    first_text: Option<String>,
    last_text: Option<String>,
    last_at: Option<DateTime<Local>>,
    count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SessionCatalog {
    sessions_by_workspace: HashMap<String, Vec<SessionSummary>>,
}

impl SessionCatalog {
    pub fn sessions_for_workspace(&self, workspace_path: &Path) -> Result<Vec<SessionSummary>> {
        let normalized_workspace = normalize_path(workspace_path)?;
        Ok(self
            .sessions_by_workspace
            .get(&normalized_workspace)
            .cloned()
            .unwrap_or_default())
    }

    #[cfg(test)]
    fn workspace_count(&self) -> usize {
        self.sessions_by_workspace.len()
    }
}

pub fn default_codex_home() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        return base_dirs.home_dir().join(".codex");
    }

    PathBuf::from(".codex")
}

pub fn build_resume_prompt(user_prompt: &str) -> String {
    let trimmed = compact_inline_text(user_prompt);
    let execution_contract = compact_inline_text(RESUME_EXECUTION_CONTRACT);
    format!(
        "Continue from the exact previous stopping point and finish the unfinished work. Original request that must be preserved exactly: {trimmed} {execution_contract}"
    )
}

pub fn prepare_resume_command(launch: &CodexLaunch, session_id: &str, prompt: &str) -> Command {
    let mut command = launch.command();
    command
        .arg("exec")
        .arg("resume")
        .arg("--skip-git-repo-check")
        .arg(session_id)
        .arg(prompt);
    command
}

pub fn resolve_resume_command(session_id: &str, prompt: &str) -> Result<Command> {
    let launch = resolve_codex_launch()?;
    Ok(prepare_resume_command(&launch, session_id, prompt))
}

pub fn prepare_new_session_command(launch: &CodexLaunch, prompt: Option<&str>) -> Command {
    let mut command = launch.interactive_command();
    if let Some(prompt) = prompt.map(str::trim).filter(|prompt| !prompt.is_empty()) {
        command.arg(prompt);
    }
    command
}

pub fn resolve_new_session_command(prompt: Option<&str>) -> Result<Command> {
    let launch = resolve_codex_launch()?;
    Ok(prepare_new_session_command(&launch, prompt))
}

pub fn resolve_codex_launch() -> Result<CodexLaunch> {
    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::<PathBuf>::new();

        let mut where_command = Command::new("where.exe");
        where_command.arg("codex");
        #[cfg(target_os = "windows")]
        where_command.creation_flags(CREATE_NO_WINDOW);
        let output = where_command.output().context("无法在 PATH 中定位 codex")?;

        if output.status.success() {
            candidates.extend(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(PathBuf::from),
            );
        }

        for fallback in windows_codex_fallback_candidates() {
            if fallback.exists() && !candidates.iter().any(|existing| existing == &fallback) {
                candidates.push(fallback);
            }
        }

        if candidates.is_empty() {
            anyhow::bail!("没有找到可用的 codex 入口；请确认 npm 的全局安装目录可访问");
        }

        candidates.sort_by_key(|path| {
            match path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                .as_deref()
            {
                Some("cmd") => 0_u8,
                Some("bat") => 1,
                Some("exe") => 2,
                Some("ps1") => 3,
                _ => 4,
            }
        });

        let target = candidates.remove(0);
        let extension = target
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        if matches!(extension.as_deref(), Some("cmd") | Some("bat"))
            && let Some(npm_root) = target.parent()
        {
            let script = npm_root
                .join("node_modules")
                .join("@openai")
                .join("codex")
                .join("bin")
                .join("codex.js");
            if script.exists() {
                return Ok(CodexLaunch {
                    program: resolve_windows_node_program(npm_root),
                    prefix_args: vec![script.into_os_string()],
                });
            }

            return Ok(CodexLaunch {
                program: PathBuf::from("cmd.exe"),
                prefix_args: vec![
                    OsString::from("/d"),
                    OsString::from("/s"),
                    OsString::from("/c"),
                    target.into_os_string(),
                ],
            });
        }

        if matches!(extension.as_deref(), Some("ps1")) {
            return Ok(CodexLaunch {
                program: PathBuf::from("powershell.exe"),
                prefix_args: vec![
                    OsString::from("-NoProfile"),
                    OsString::from("-ExecutionPolicy"),
                    OsString::from("Bypass"),
                    OsString::from("-File"),
                    target.into_os_string(),
                ],
            });
        }

        Ok(CodexLaunch {
            program: target,
            prefix_args: Vec::new(),
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(CodexLaunch {
            program: PathBuf::from("codex"),
            prefix_args: Vec::new(),
        })
    }
}

#[cfg(target_os = "windows")]
fn windows_codex_fallback_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(appdata) = std::env::var_os("APPDATA") {
        let npm_dir = PathBuf::from(appdata).join("npm");
        candidates.push(npm_dir.join("codex.cmd"));
        candidates.push(npm_dir.join("codex.ps1"));
        candidates.push(npm_dir.join("codex"));
    }

    if let Some(base_dirs) = BaseDirs::new() {
        let npm_dir = base_dirs
            .home_dir()
            .join("AppData")
            .join("Roaming")
            .join("npm");
        candidates.push(npm_dir.join("codex.cmd"));
        candidates.push(npm_dir.join("codex.ps1"));
        candidates.push(npm_dir.join("codex"));
    }

    candidates
}

#[cfg(target_os = "windows")]
fn resolve_windows_node_program(npm_root: &Path) -> PathBuf {
    let bundled = npm_root.join("node.exe");
    if bundled.exists() {
        return bundled;
    }

    if let Ok(appdata) = std::env::var("ProgramFiles") {
        let candidate = PathBuf::from(appdata).join("nodejs").join("node.exe");
        if candidate.exists() {
            return candidate;
        }
    }

    if let Ok(appdata_x86) = std::env::var("ProgramFiles(x86)") {
        let candidate = PathBuf::from(appdata_x86).join("nodejs").join("node.exe");
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from("node.exe")
}

pub fn probe_codex_version() -> Result<String> {
    let mut command = resolve_codex_launch()?.command();
    let output = command
        .arg("--version")
        .output()
        .context("无法运行 `codex --version`")?;

    if !output.status.success() {
        anyhow::bail!(
            "`codex --version` 执行失败，退出码 {:?}",
            output.status.code()
        );
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if version.is_empty() {
        anyhow::bail!("`codex --version` 返回了空结果");
    }

    Ok(version)
}

pub fn discover_workspace_sessions(
    codex_home: &Path,
    workspace_path: &Path,
) -> Result<Vec<SessionSummary>> {
    let sessions_root = codex_home.join("sessions");
    if !sessions_root.exists() {
        return Ok(Vec::new());
    }

    let normalized_workspace = normalize_path(workspace_path)?;
    let sessions = collect_indexed_sessions(&sessions_root)?
        .into_iter()
        .filter(|session| session.workspace_key == normalized_workspace)
        .map(|session| session.raw_session)
        .collect::<Vec<_>>();

    summarize_sessions(codex_home, sessions)
}

pub fn discover_workspace_catalog(codex_home: &Path) -> Result<SessionCatalog> {
    let sessions_root = codex_home.join("sessions");
    if !sessions_root.exists() {
        return Ok(SessionCatalog::default());
    }

    let indexed_sessions = collect_indexed_sessions(&sessions_root)?;
    if indexed_sessions.is_empty() {
        return Ok(SessionCatalog::default());
    }

    let raw_sessions = indexed_sessions
        .iter()
        .map(|session| session.raw_session.clone())
        .collect::<Vec<_>>();
    let history_summaries =
        read_history_summaries(&codex_home.join("history.jsonl"), &raw_sessions)?;
    let mut sessions_by_workspace = HashMap::<String, Vec<SessionSummary>>::new();

    for indexed_session in indexed_sessions {
        let summary = history_summaries.get(&indexed_session.raw_session.session_id);
        sessions_by_workspace
            .entry(indexed_session.workspace_key)
            .or_default()
            .push(build_session_summary(indexed_session.raw_session, summary));
    }

    for sessions in sessions_by_workspace.values_mut() {
        sessions.sort_by(|left, right| right.last_activity.cmp(&left.last_activity));
    }

    Ok(SessionCatalog {
        sessions_by_workspace,
    })
}

fn collect_indexed_sessions(sessions_root: &Path) -> Result<Vec<IndexedSession>> {
    let mut sessions = Vec::new();

    for entry in WalkDir::new(sessions_root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }

        let Some((workspace_key, raw_session)) = read_indexed_session(entry.path())? else {
            continue;
        };
        sessions.push(IndexedSession {
            workspace_key,
            raw_session,
        });
    }

    Ok(sessions)
}

fn read_indexed_session(path: &Path) -> Result<Option<(String, RawSession)>> {
    let file = File::open(path).with_context(|| format!("无法打开会话文件：{}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    if reader.read_line(&mut first_line)? == 0 {
        return Ok(None);
    }

    let envelope: SessionMetaEnvelope = match serde_json::from_str(&first_line) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    if envelope.line_type != "session_meta" {
        return Ok(None);
    }

    let workspace_key = match normalize_path(Path::new(&envelope.payload.cwd)) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };
    let started_at = DateTime::parse_from_rfc3339(&envelope.payload.timestamp)
        .map(|value| value.with_timezone(&Local))
        .unwrap_or_else(|_| {
            path.metadata()
                .ok()
                .and_then(|meta| meta.modified().ok())
                .map(DateTime::<Local>::from)
                .unwrap_or_else(Local::now)
        });

    Ok(Some((
        workspace_key,
        RawSession {
            session_id: envelope.payload.id,
            started_at,
            file_path: path.to_path_buf(),
        },
    )))
}

fn summarize_sessions(codex_home: &Path, sessions: Vec<RawSession>) -> Result<Vec<SessionSummary>> {
    if sessions.is_empty() {
        return Ok(Vec::new());
    }

    let history_summaries = read_history_summaries(&codex_home.join("history.jsonl"), &sessions)?;
    let mut results = Vec::with_capacity(sessions.len());
    for raw_session in sessions {
        let summary = history_summaries.get(&raw_session.session_id);
        results.push(build_session_summary(raw_session, summary));
    }
    results.sort_by(|left, right| right.last_activity.cmp(&left.last_activity));
    Ok(results)
}

fn build_session_summary(
    raw_session: RawSession,
    summary: Option<&HistorySummary>,
) -> SessionSummary {
    let title = summary
        .and_then(|summary| summary.first_text.clone())
        .unwrap_or_else(|| "（没有记录到用户提示词）".to_owned());
    let last_text = summary
        .and_then(|summary| summary.last_text.clone())
        .unwrap_or_else(|| title.clone());
    let last_activity = summary
        .and_then(|summary| summary.last_at)
        .unwrap_or(raw_session.started_at);
    let message_count = summary.map(|summary| summary.count).unwrap_or(0);

    SessionSummary {
        session_id: raw_session.session_id,
        title,
        last_text,
        last_activity,
        file_path: raw_session.file_path,
        message_count,
    }
}

fn read_history_summaries(
    history_path: &Path,
    sessions: &[RawSession],
) -> Result<HashMap<String, HistorySummary>> {
    if !history_path.exists() {
        return Ok(HashMap::new());
    }

    let session_ids = sessions
        .iter()
        .map(|session| session.session_id.clone())
        .collect::<HashSet<_>>();

    let file = File::open(history_path)
        .with_context(|| format!("无法打开历史文件：{}", history_path.display()))?;
    let reader = BufReader::new(file);
    let mut summaries = HashMap::<String, HistorySummary>::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let entry: HistoryEntry = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if !session_ids.contains(&entry.session_id) {
            continue;
        }

        let summary = summaries.entry(entry.session_id).or_default();
        let preview = single_line_preview(&entry.text, 72);
        if summary.first_text.is_none() {
            summary.first_text = Some(preview.clone());
        }
        summary.last_text = Some(preview);
        summary.last_at =
            DateTime::from_timestamp(entry.ts, 0).map(|value| value.with_timezone(&Local));
        summary.count += 1;
    }

    Ok(summaries)
}

fn single_line_preview(text: &str, max_len: usize) -> String {
    let compact = compact_inline_text(text);
    if compact.is_empty() {
        return "（没有记录到用户提示词）".to_owned();
    }

    let chars = compact.chars().collect::<Vec<_>>();
    if chars.len() <= max_len {
        return compact;
    }

    chars[..max_len.saturating_sub(3)]
        .iter()
        .collect::<String>()
        + "..."
}

fn compact_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_path(path: &Path) -> Result<String> {
    let absolute = if path.exists() {
        canonicalize(path).with_context(|| format!("无法规范化路径：{}", path.display()))?
    } else if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("无法读取当前工作目录")?
            .join(path)
    };

    let mut text = absolute.to_string_lossy().replace('/', "\\");
    while text.ends_with('\\') {
        text.pop();
    }

    Ok(text.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::Instant;
    use tempfile::tempdir;

    fn write_session_file(path: &Path, session_id: &str, timestamp: &str, workspace: &Path) {
        fs::write(
            path,
            format!(
                "{{\"timestamp\":\"{timestamp}\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"{session_id}\",\"timestamp\":\"{timestamp}\",\"cwd\":\"{}\"}}}}\n",
                workspace.display().to_string().replace('\\', "\\\\")
            ),
        )
        .expect("write session file");
    }

    #[test]
    fn discovers_only_matching_workspace_sessions() {
        let dir = tempdir().expect("tempdir");
        let codex_home = dir.path().join(".codex");
        let sessions_root = codex_home
            .join("sessions")
            .join("2026")
            .join("03")
            .join("23");
        fs::create_dir_all(&sessions_root).expect("create sessions");

        let workspace = dir.path().join("workspace-a");
        let other_workspace = dir.path().join("workspace-b");
        fs::create_dir_all(&workspace).expect("create workspace");
        fs::create_dir_all(&other_workspace).expect("create other workspace");

        fs::write(
            sessions_root.join("a.jsonl"),
            format!(
                "{{\"timestamp\":\"2026-03-23T08:00:00.000Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"session-a\",\"timestamp\":\"2026-03-23T08:00:00.000Z\",\"cwd\":\"{}\"}}}}\n",
                workspace.display().to_string().replace('\\', "\\\\")
            ),
        )
        .expect("write session a");

        fs::write(
            sessions_root.join("b.jsonl"),
            format!(
                "{{\"timestamp\":\"2026-03-23T09:00:00.000Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"session-b\",\"timestamp\":\"2026-03-23T09:00:00.000Z\",\"cwd\":\"{}\"}}}}\n",
                other_workspace.display().to_string().replace('\\', "\\\\")
            ),
        )
        .expect("write session b");

        fs::write(
            codex_home.join("history.jsonl"),
            "{\"session_id\":\"session-a\",\"ts\":1774252800,\"text\":\"hello workspace a\"}\n\
             {\"session_id\":\"session-b\",\"ts\":1774256400,\"text\":\"hello workspace b\"}\n",
        )
        .expect("write history");

        let sessions = discover_workspace_sessions(&codex_home, &workspace).expect("discover");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "session-a");
        assert_eq!(sessions[0].message_count, 1);
        assert!(sessions[0].title.contains("hello workspace a"));
    }

    #[test]
    fn build_resume_prompt_preserves_original_request_and_contract() {
        let prompt = build_resume_prompt("restore exactly");

        assert!(prompt.contains("restore exactly"));
        assert!(prompt.contains("Do not ask the user whether to continue"));
        assert!(prompt.contains("compare the current result against the original request"));
        assert!(!prompt.contains('\n'));
    }

    #[test]
    fn prepare_resume_command_keeps_launcher_prefix_args() {
        let launch = CodexLaunch {
            program: PathBuf::from("node.exe"),
            prefix_args: vec![OsString::from(r"C:\mock\codex.js")],
        };

        let command = prepare_resume_command(&launch, "session-1", "restore exactly");
        let args = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(command.get_program(), Path::new("node.exe"));
        assert_eq!(
            args,
            vec![
                r"C:\mock\codex.js".to_owned(),
                "exec".to_owned(),
                "resume".to_owned(),
                "--skip-git-repo-check".to_owned(),
                "session-1".to_owned(),
                "restore exactly".to_owned(),
            ]
        );
    }

    #[test]
    fn prepare_new_session_command_keeps_launcher_prefix_args() {
        let launch = CodexLaunch {
            program: PathBuf::from("node.exe"),
            prefix_args: vec![OsString::from(r"C:\mock\codex.js")],
        };

        let command = prepare_new_session_command(&launch, Some("start fresh"));
        let args = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(command.get_program(), Path::new("node.exe"));
        assert_eq!(
            args,
            vec![r"C:\mock\codex.js".to_owned(), "start fresh".to_owned()]
        );
    }

    #[test]
    fn catalog_matches_single_workspace_discovery() {
        let dir = tempdir().expect("tempdir");
        let codex_home = dir.path().join(".codex");
        let sessions_root = codex_home
            .join("sessions")
            .join("2026")
            .join("03")
            .join("24");
        fs::create_dir_all(&sessions_root).expect("create sessions");

        let workspaces = (0..3)
            .map(|index| {
                let workspace = dir.path().join(format!("workspace-{index}"));
                fs::create_dir_all(&workspace).expect("create workspace");
                workspace
            })
            .collect::<Vec<_>>();

        let mut history_lines = Vec::new();
        for (workspace_index, workspace) in workspaces.iter().enumerate() {
            for session_index in 0..4 {
                let session_id = format!("session-{workspace_index}-{session_index}");
                let timestamp = format!("2026-03-24T0{}:00:00.000Z", session_index);
                write_session_file(
                    &sessions_root.join(format!("{session_id}.jsonl")),
                    &session_id,
                    &timestamp,
                    workspace,
                );
                history_lines.push(format!(
                    "{{\"session_id\":\"{session_id}\",\"ts\":1774252800,\"text\":\"workspace {workspace_index} session {session_index}\"}}"
                ));
            }
        }
        fs::write(
            codex_home.join("history.jsonl"),
            history_lines.join("\n") + "\n",
        )
        .expect("write history");

        let catalog = discover_workspace_catalog(&codex_home).expect("catalog");
        assert_eq!(catalog.workspace_count(), workspaces.len());

        for workspace in &workspaces {
            let catalog_sessions = catalog
                .sessions_for_workspace(workspace)
                .expect("catalog sessions");
            let single_workspace_sessions =
                discover_workspace_sessions(&codex_home, workspace).expect("single workspace");
            assert_eq!(catalog_sessions.len(), single_workspace_sessions.len());
            assert_eq!(
                catalog_sessions
                    .iter()
                    .map(|session| session.session_id.clone())
                    .collect::<Vec<_>>(),
                single_workspace_sessions
                    .iter()
                    .map(|session| session.session_id.clone())
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    #[ignore = "manual performance probe"]
    fn benchmark_catalog_reuses_single_scan() {
        let dir = tempdir().expect("tempdir");
        let codex_home = dir.path().join(".codex");
        let sessions_root = codex_home
            .join("sessions")
            .join("2026")
            .join("03")
            .join("24");
        fs::create_dir_all(&sessions_root).expect("create sessions");

        let workspace_count = 24;
        let sessions_per_workspace = 14;
        let workspaces = (0..workspace_count)
            .map(|index| {
                let workspace = dir.path().join(format!("workspace-{index}"));
                fs::create_dir_all(&workspace).expect("create workspace");
                workspace
            })
            .collect::<Vec<_>>();

        let mut history_lines = Vec::new();
        for (workspace_index, workspace) in workspaces.iter().enumerate() {
            for session_index in 0..sessions_per_workspace {
                let session_id = format!("session-{workspace_index}-{session_index}");
                let timestamp = format!("2026-03-24T{:02}:00:00.000Z", (session_index % 23) + 1);
                write_session_file(
                    &sessions_root.join(format!("{session_id}.jsonl")),
                    &session_id,
                    &timestamp,
                    workspace,
                );
                history_lines.push(format!(
                    "{{\"session_id\":\"{session_id}\",\"ts\":1774252800,\"text\":\"workspace {workspace_index} session {session_index}\"}}"
                ));
            }
        }
        fs::write(
            codex_home.join("history.jsonl"),
            history_lines.join("\n") + "\n",
        )
        .expect("write history");

        let baseline_start = Instant::now();
        for workspace in &workspaces {
            let sessions = discover_workspace_sessions(&codex_home, workspace)
                .expect("baseline workspace discovery");
            assert_eq!(sessions.len(), sessions_per_workspace);
        }
        let baseline = baseline_start.elapsed();

        let optimized_start = Instant::now();
        let catalog = discover_workspace_catalog(&codex_home).expect("catalog");
        for workspace in &workspaces {
            let sessions = catalog
                .sessions_for_workspace(workspace)
                .expect("catalog workspace discovery");
            assert_eq!(sessions.len(), sessions_per_workspace);
        }
        let optimized = optimized_start.elapsed();

        println!(
            "baseline={:?} optimized={:?} speedup={:.2}x",
            baseline,
            optimized,
            baseline.as_secs_f64() / optimized.as_secs_f64()
        );
    }
}
