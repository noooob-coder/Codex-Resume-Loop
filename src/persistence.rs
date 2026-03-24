use crate::model::StoredAppState;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Storage::FileSystem::{
    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
};

fn config_path() -> Result<PathBuf> {
    let project_dirs =
        ProjectDirs::from("dev", "shcem", "crl-desktop").context("无法定位应用配置目录")?;
    let config_dir = project_dirs.config_dir();
    fs::create_dir_all(config_dir)
        .with_context(|| format!("无法创建配置目录：{}", config_dir.display()))?;
    Ok(config_dir.join("state.json"))
}

pub fn load_state() -> Result<StoredAppState> {
    let path = config_path()?;
    load_state_from(&path)
}

pub fn save_state(state: &StoredAppState) -> Result<()> {
    let path = config_path()?;
    save_state_to(&path, state)
}

pub(crate) fn load_state_from(path: &Path) -> Result<StoredAppState> {
    if !path.exists() {
        return Ok(StoredAppState::default());
    }

    let text = fs::read_to_string(path)
        .with_context(|| format!("无法读取配置文件：{}", path.display()))?;
    let state = serde_json::from_str(&text)
        .with_context(|| format!("无法解析配置文件：{}", path.display()))?;
    Ok(state)
}

pub(crate) fn save_state_to(path: &Path, state: &StoredAppState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建配置目录：{}", parent.display()))?;
    }

    let temp_path = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(state).context("无法序列化应用状态")?;
    fs::write(&temp_path, text)
        .with_context(|| format!("无法写入临时配置文件：{}", temp_path.display()))?;
    replace_file(&temp_path, path)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn replace_file(from: &Path, to: &Path) -> Result<()> {
    let from_wide = to_wide(from);
    let to_wide = to_wide(to);
    let result = unsafe {
        MoveFileExW(
            from_wide.as_ptr(),
            to_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };

    if result == 0 {
        anyhow::bail!(
            "无法替换配置文件：{} -> {} ({})",
            from.display(),
            to.display(),
            std::io::Error::last_os_error()
        );
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn to_wide(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn replace_file(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to)
        .with_context(|| format!("无法替换配置文件：{} -> {}", from.display(), to.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_state() -> StoredAppState {
        StoredAppState {
            codex_home: Some(r"C:\Users\shcem\.codex".to_owned()),
            selected_workspace_id: Some(7),
            next_workspace_id: 8,
            auto_refresh_enabled: true,
            auto_refresh_seconds: 15,
            workspaces: Vec::new(),
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("state.json");
        let state = sample_state();

        save_state_to(&path, &state).expect("save");
        let loaded = load_state_from(&path).expect("load");

        assert_eq!(loaded.selected_workspace_id, Some(7));
        assert_eq!(loaded.next_workspace_id, 8);
        assert_eq!(loaded.auto_refresh_seconds, 15);
    }

    #[test]
    fn save_overwrites_existing_state() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("state.json");

        let mut first = sample_state();
        first.selected_workspace_id = Some(1);
        first.auto_refresh_seconds = 10;
        save_state_to(&path, &first).expect("first save");

        let mut second = sample_state();
        second.selected_workspace_id = Some(9);
        second.auto_refresh_seconds = 45;
        save_state_to(&path, &second).expect("second save");

        let loaded = load_state_from(&path).expect("load");
        assert_eq!(loaded.selected_workspace_id, Some(9));
        assert_eq!(loaded.auto_refresh_seconds, 45);
    }
}
