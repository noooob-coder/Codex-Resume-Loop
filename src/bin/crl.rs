use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use crl_desktop::codex::{
    DEFAULT_RESUME_ROUNDS, build_resume_prompt, default_codex_home, discover_workspace_sessions,
    resolve_resume_command,
};
use crl_desktop::model::SessionSummary;
use directories::BaseDirs;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[derive(Debug, Parser)]
#[command(name = "crl", about = "Codex Resume Loop CLI")]
struct Cli {
    #[arg(long, alias = "Install")]
    install: bool,
    #[arg(long, alias = "Uninstall")]
    uninstall: bool,
    #[arg(long, alias = "SessionId")]
    session_id: Option<String>,
    #[arg(long, alias = "Latest")]
    latest: bool,
    #[arg(long, alias = "AllowCurrentSession")]
    allow_current_session: bool,
    #[arg(long, alias = "Interactive")]
    interactive: bool,
    #[arg(long, alias = "ListSessions")]
    list_sessions: bool,
    #[arg(long, alias = "MaxSessions", default_value_t = 20)]
    max_sessions: usize,
    #[arg(long, alias = "CodexHome")]
    codex_home: Option<PathBuf>,
    #[arg(long, alias = "DryRun")]
    dry_run: bool,
    times: Option<u32>,
    prompt: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.install {
        install_cli()?;
        return Ok(());
    }
    if cli.uninstall {
        uninstall_cli()?;
        return Ok(());
    }

    run_resume_loop(cli)
}

fn run_resume_loop(cli: Cli) -> Result<()> {
    let workspace = env::current_dir().context("Unable to read current working directory")?;
    let codex_home = cli.codex_home.unwrap_or_else(default_codex_home);
    let current_session = env::var("CODEX_THREAD_ID").ok();

    let all_sessions = discover_workspace_sessions(&codex_home, &workspace)?;
    let mut selectable_sessions = all_sessions
        .iter()
        .filter(|session| {
            if cli.allow_current_session {
                return true;
            }
            current_session
                .as_deref()
                .map(|current| session.session_id != current)
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();

    if cli.list_sessions {
        print_sessions(&all_sessions, &workspace, cli.max_sessions);
        return Ok(());
    }

    if selectable_sessions.is_empty() {
        if !all_sessions.is_empty() && current_session.is_some() && !cli.allow_current_session {
            bail!(
                "Only the current Codex session is available in this workspace. Re-run outside Codex or pass -AllowCurrentSession."
            );
        }
        bail!("No resumable Codex sessions were found for the current workspace.");
    }

    selectable_sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    let wizard_mode = cli.interactive
        || (cli.times.is_none() && cli.prompt.is_none() && cli.session_id.is_none() && !cli.latest);

    let session = if let Some(session_id) = cli.session_id.as_deref() {
        find_session(&all_sessions, session_id)?
    } else if cli.latest {
        selectable_sessions[0].clone()
    } else {
        select_session(&selectable_sessions, cli.max_sessions, wizard_mode)?
    };

    if let Some(current) = current_session.as_deref()
        && !cli.allow_current_session
        && session.session_id == current
    {
        bail!(
            "The selected session is the current Codex session. Re-run outside Codex or pass -AllowCurrentSession."
        );
    }

    let times = resolve_rounds(cli.times, wizard_mode)?;
    let prompt = resolve_prompt(cli.prompt, wizard_mode)?;
    let round_prompt = build_resume_prompt(&prompt);

    print_plan(&workspace, &session, times, &prompt, cli.dry_run);

    if cli.dry_run {
        for round in 1..=times {
            println!("== Round {round}/{times} ==");
            println!("[DryRun] Skipped actual execution.");
        }
        println!("Dry run completed.");
        return Ok(());
    }

    let mut failed_rounds = Vec::new();

    for round in 1..=times {
        println!("== Round {round}/{times} ==");
        let mut command = resolve_resume_command(&session.session_id, &round_prompt)
            .with_context(|| format!("Failed to build codex command for round {round}"))?;
        let status = command
            .current_dir(&workspace)
            .status()
            .with_context(|| format!("Failed to start codex for round {round}"))?;

        if !status.success() {
            let code = format_exit_code(status);
            println!("Round {round} failed with exit code: {code}. Continuing.");
            failed_rounds.push((round, code));
        }
    }

    if !failed_rounds.is_empty() {
        println!("All rounds attempted.");
        for (round, code) in &failed_rounds {
            println!("  Round {round} failed with exit code: {code}");
        }
        bail!(
            "Completed {times} rounds, but {} round(s) failed.",
            failed_rounds.len()
        );
    }

    println!("All rounds completed.");
    Ok(())
}

fn install_cli() -> Result<()> {
    let current_exe = env::current_exe().context("Unable to locate current executable")?;
    let base_dirs = BaseDirs::new().context("Unable to locate user directories")?;
    let install_root = base_dirs
        .data_local_dir()
        .join("Programs")
        .join("codex-resume-loop");
    let bin_dir = base_dirs.home_dir().join(".local").join("bin");
    fs::create_dir_all(&install_root).with_context(|| {
        format!(
            "Unable to create install directory: {}",
            install_root.display()
        )
    })?;
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("Unable to create bin directory: {}", bin_dir.display()))?;

    let primary_exe = install_root.join("crl.exe");
    let legacy_exe = install_root.join("codex-resume-loop.exe");
    fs::copy(&current_exe, &primary_exe)
        .with_context(|| format!("Unable to copy CLI to {}", primary_exe.display()))?;
    fs::copy(&current_exe, &legacy_exe)
        .with_context(|| format!("Unable to copy CLI to {}", legacy_exe.display()))?;

    add_to_user_path(&bin_dir)?;
    remove_old_wrappers(&install_root, &bin_dir)?;
    copy_alias(&primary_exe, &bin_dir.join("crl.exe"))?;
    copy_alias(&legacy_exe, &bin_dir.join("codex-resume-loop.exe"))?;

    println!("Installed CLI binaries:");
    println!("  {}", primary_exe.display());
    println!("  {}", legacy_exe.display());
    println!();
    println!("Available commands:");
    println!("  crl");
    println!("  codex-resume-loop");
    Ok(())
}

fn uninstall_cli() -> Result<()> {
    let base_dirs = BaseDirs::new().context("Unable to locate user directories")?;
    let install_root = base_dirs
        .data_local_dir()
        .join("Programs")
        .join("codex-resume-loop");
    if !install_root.exists() {
        bail!(
            "Install directory does not exist: {}",
            install_root.display()
        );
    }

    println!("About to uninstall CRL CLI from:");
    println!("  {}", install_root.display());
    if !confirm("Continue?", false)? {
        println!("Cancelled.");
        return Ok(());
    }

    let bin_dir = base_dirs.home_dir().join(".local").join("bin");
    for alias in [
        "crl.exe",
        "codex-resume-loop.exe",
        "crl.cmd",
        "codex-resume-loop.cmd",
    ] {
        let path = bin_dir.join(alias);
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }

    let current_exe = env::current_exe().context("Unable to locate current executable")?;
    if current_exe.starts_with(&install_root) {
        let script_path = env::temp_dir().join(format!(
            "crl-uninstall-{}.cmd",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let script = format!(
            "@echo off\r\nping 127.0.0.1 -n 2 >nul\r\nrmdir /s /q \"{}\"\r\ndel \"%~f0\"\r\n",
            install_root.display()
        );
        fs::write(&script_path, script).with_context(|| {
            format!(
                "Unable to write temp uninstall script: {}",
                script_path.display()
            )
        })?;
        Command::new(&script_path).spawn().with_context(|| {
            format!(
                "Unable to launch temp uninstall script: {}",
                script_path.display()
            )
        })?;
        println!(
            "Uninstall scheduled. The install directory will be removed after this process exits."
        );
    } else {
        fs::remove_dir_all(&install_root).with_context(|| {
            format!(
                "Unable to remove install directory: {}",
                install_root.display()
            )
        })?;
        println!("Uninstall completed.");
    }

    Ok(())
}

fn add_to_user_path(bin_dir: &Path) -> Result<()> {
    let current_path = env::var("Path").unwrap_or_default();
    if !current_path.split(';').any(|part| {
        part.trim_end_matches('\\')
            .eq_ignore_ascii_case(&bin_dir.to_string_lossy())
    }) {
        let mut new_path = current_path;
        if !new_path.is_empty() && !new_path.ends_with(';') {
            new_path.push(';');
        }
        new_path.push_str(&bin_dir.to_string_lossy());
        unsafe {
            env::set_var("Path", &new_path);
        }
    }

    let user_path = env::var("Path").unwrap_or_default();
    let _ = user_path;
    Ok(())
}

fn copy_alias(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to)
        .with_context(|| format!("Unable to copy {} -> {}", from.display(), to.display()))?;
    Ok(())
}

fn remove_old_wrappers(install_root: &Path, bin_dir: &Path) -> Result<()> {
    for old_file in [
        "crl.cmd",
        "codex-resume-loop.cmd",
        "codex-resume-loop.ps1",
        "codex-resume-loop-launcher.cmd",
        "install.ps1",
        "install.cmd",
        "uninstall.ps1",
        "README.md",
    ] {
        let path = install_root.join(old_file);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Unable to remove old wrapper: {}", path.display()))?;
        }
    }

    for old_file in ["crl.cmd", "codex-resume-loop.cmd"] {
        let path = bin_dir.join(old_file);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Unable to remove old wrapper: {}", path.display()))?;
        }
    }

    Ok(())
}

fn find_session(sessions: &[SessionSummary], session_id: &str) -> Result<SessionSummary> {
    sessions
        .iter()
        .find(|session| session.session_id == session_id)
        .cloned()
        .ok_or_else(|| anyhow!("Session not found in current workspace: {session_id}"))
}

fn select_session(
    sessions: &[SessionSummary],
    max_sessions: usize,
    wizard_mode: bool,
) -> Result<SessionSummary> {
    if sessions.len() == 1 {
        return Ok(sessions[0].clone());
    }

    if !wizard_mode {
        return Ok(sessions[0].clone());
    }

    print_sessions(
        sessions,
        &env::current_dir().unwrap_or_default(),
        max_sessions,
    );
    println!();
    println!("Press Enter to choose the newest session.");

    loop {
        let input = read_line("Select a session number (Enter=1, q=quit): ")?;
        if input.trim().is_empty() {
            return Ok(sessions[0].clone());
        }
        if input.trim().eq_ignore_ascii_case("q") {
            bail!("Cancelled by user");
        }
        let selection = input
            .trim()
            .parse::<usize>()
            .with_context(|| format!("Invalid selection: {}", input.trim()))?;
        if selection >= 1 && selection <= sessions.len().min(max_sessions) {
            return Ok(sessions[selection - 1].clone());
        }
        println!("Please enter a valid number from the list.");
    }
}

fn print_sessions(sessions: &[SessionSummary], workspace: &Path, max_sessions: usize) {
    println!("Resumable sessions for workspace:");
    println!("  {}", workspace.display());
    println!();
    for (index, session) in sessions.iter().take(max_sessions).enumerate() {
        println!(
            "{:>2}. {}  {}  {:>3} msgs  {}",
            index + 1,
            session.last_activity.format("%Y-%m-%d %H:%M"),
            short_id(&session.session_id),
            session.message_count,
            session.title
        );
    }
    if sessions.len() > max_sessions {
        println!();
        println!("Showing {} of {} sessions.", max_sessions, sessions.len());
    }
}

fn print_plan(workspace: &Path, session: &SessionSummary, times: u32, prompt: &str, dry_run: bool) {
    println!();
    println!("Execution plan:");
    println!("  Workspace : {}", workspace.display());
    println!("  Session   : {}", session.session_id);
    println!("  Summary   : {}", session.title);
    println!("  Rounds    : {}", times);
    println!("  Prompt    : {}", prompt);
    println!(
        "  Mode      : {}",
        if dry_run { "DryRun" } else { "Execute" }
    );
    println!();
}

fn ensure_positive_rounds(rounds: u32) -> Result<u32> {
    if rounds < 1 {
        bail!("Rounds must be greater than or equal to 1");
    }
    Ok(rounds)
}

fn ensure_prompt(prompt: String) -> Result<String> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        bail!("Prompt cannot be empty");
    }
    Ok(trimmed.to_owned())
}

fn resolve_rounds(times: Option<u32>, wizard_mode: bool) -> Result<u32> {
    if let Some(times) = times {
        return ensure_positive_rounds(times);
    }

    if wizard_mode {
        return read_positive_int(
            &format!("Enter rounds (default {DEFAULT_RESUME_ROUNDS}): "),
            Some(DEFAULT_RESUME_ROUNDS),
        );
    }

    bail!("Missing rounds. Pass a value like `crl 3 \"prompt\"` or run `crl` for the wizard.");
}

fn resolve_prompt(prompt: Option<String>, wizard_mode: bool) -> Result<String> {
    if let Some(prompt) = prompt {
        return ensure_prompt(prompt);
    }

    if wizard_mode {
        return read_required_string("Enter prompt: ");
    }

    bail!("Missing prompt. Pass a prompt or run `crl` for the wizard.");
}

fn read_positive_int(message: &str, default_value: Option<u32>) -> Result<u32> {
    loop {
        let input = read_line(message)?;
        if input.trim().is_empty()
            && let Some(default_value) = default_value
        {
            return Ok(default_value);
        }
        if let Ok(value) = input.trim().parse::<u32>()
            && value >= 1
        {
            return Ok(value);
        }
        println!("Please enter an integer greater than or equal to 1.");
    }
}

fn read_required_string(message: &str) -> Result<String> {
    loop {
        let input = read_line(message)?;
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_owned());
        }
        println!("Input cannot be empty.");
    }
}

fn confirm(message: &str, default_value: bool) -> Result<bool> {
    let hint = if default_value { "[Y/n]" } else { "[y/N]" };
    loop {
        let input = read_line(&format!("{message} {hint} "))?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(default_value);
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter y or n."),
        }
    }
}

fn read_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush().context("Unable to flush stdout")?;
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .context("Unable to read from stdin")?;
    Ok(buffer)
}

fn short_id(session_id: &str) -> String {
    session_id.chars().take(8).collect()
}

fn format_exit_code(status: ExitStatus) -> String {
    status
        .code()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};
    use tempfile::tempdir;

    static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn wizard_defaults_to_multi_round_execution() {
        let rounds = resolve_rounds(None, true).expect("wizard mode default rounds");
        assert_eq!(rounds, 1);
    }

    #[test]
    fn non_wizard_requires_explicit_rounds() {
        let error = resolve_rounds(None, false).expect_err("missing rounds should fail");
        assert!(error.to_string().contains("Missing rounds"));
    }

    #[test]
    fn continues_attempting_rounds_after_non_zero_exit() {
        let _guard = ENV_MUTEX.lock().expect("env mutex");
        let temp = tempdir().expect("tempdir");
        let workspace = temp.path().join("workspace");
        let codex_home = temp.path().join("codex-home");
        let sessions_dir = codex_home
            .join("sessions")
            .join("2026")
            .join("03")
            .join("24");
        let bin_dir = temp.path().join("bin");
        std::fs::create_dir_all(&workspace).expect("workspace");
        std::fs::create_dir_all(&sessions_dir).expect("sessions");
        std::fs::create_dir_all(&bin_dir).expect("bin");

        std::fs::write(
            sessions_dir.join("session.jsonl"),
            format!(
                "{{\"timestamp\":\"2026-03-24T10:00:00.000Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"session-1\",\"timestamp\":\"2026-03-24T10:00:00.000Z\",\"cwd\":\"{}\"}}}}\n",
                workspace.display().to_string().replace('\\', "\\\\")
            ),
        )
        .expect("write session");
        std::fs::write(
            codex_home.join("history.jsonl"),
            "{\"session_id\":\"session-1\",\"ts\":1774346400,\"text\":\"initial prompt\"}\n",
        )
        .expect("write history");

        std::fs::write(
            bin_dir.join("codex.cmd"),
            "@echo off\r\npowershell.exe -NoProfile -ExecutionPolicy Bypass -File \"%~dp0mock-codex.ps1\" %*\r\nexit /b %ERRORLEVEL%\r\n",
        )
        .expect("write codex cmd");
        std::fs::write(
            bin_dir.join("mock-codex.ps1"),
            "$counterPath = Join-Path $PSScriptRoot 'count.txt'\r\n$count = 0\r\nif (Test-Path $counterPath) { $count = [int](Get-Content $counterPath) }\r\n$count++\r\nSet-Content -Path $counterPath -Value $count\r\nif ($count -eq 1) { exit 1 }\r\nexit 0\r\n",
        )
        .expect("write mock codex");

        let original_dir = env::current_dir().expect("current dir");
        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", format!("{};{}", bin_dir.display(), original_path));
        }
        env::set_current_dir(&workspace).expect("set current dir");

        let result = run_resume_loop(Cli {
            install: false,
            uninstall: false,
            session_id: Some("session-1".to_owned()),
            latest: false,
            allow_current_session: false,
            interactive: false,
            list_sessions: false,
            max_sessions: 20,
            codex_home: Some(codex_home.clone()),
            dry_run: false,
            times: Some(2),
            prompt: Some("restore exactly".to_owned()),
        });

        env::set_current_dir(original_dir).expect("restore current dir");
        unsafe {
            env::set_var("PATH", original_path);
        }

        let count = std::fs::read_to_string(bin_dir.join("count.txt"))
            .expect("count file")
            .trim()
            .parse::<u32>()
            .expect("count parse");

        assert_eq!(count, 2);
        let error = result.expect_err("non-zero round summary should error");
        assert!(error.to_string().contains("Completed 2 rounds"));
    }
}
