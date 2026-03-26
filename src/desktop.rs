use crate::codex::{
    DEFAULT_RESUME_ROUNDS, SessionCatalog, default_codex_home, discover_workspace_catalog,
    probe_codex_version,
};
use crate::diagnostics::{append_log, install_panic_hook};
use crate::model::{
    RunStatus, StoredAppState, StoredWorkspace, WorkspaceRunRequest, WorkspaceState,
};
use crate::persistence;
use crate::runtime::{
    RuntimeEvent, TaskHandle, TaskOutcome, spawn_new_session_runner, spawn_workspace_runner,
};
use chrono::{DateTime, Local};
use crossbeam_channel::{Receiver, Sender, unbounded};
use rfd::FileDialog;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[cfg(test)]
use crate::model::{LogEntry, LogStream};

#[cfg(test)]
use std::collections::VecDeque;

slint::include_modules!();

const CODEX_PROBE_REFRESH_INTERVAL: Duration = Duration::from_secs(300);

pub fn run_desktop() -> Result<(), slint::PlatformError> {
    install_panic_hook();
    append_log("desktop app starting");

    let ui = MainWindow::new()?;
    let controller = Rc::new(RefCell::new(DesktopController::new()));

    {
        let mut controller_ref = controller.borrow_mut();
        controller_ref.refresh_all_workspaces();
        controller_ref.probe_codex_environment();
        controller_ref.sync_to_ui(&ui);
    }

    wire_callbacks(&ui, &controller);
    ui.run()
}

fn wire_callbacks(ui: &MainWindow, controller: &Rc<RefCell<DesktopController>>) {
    let weak = ui.as_weak();

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_tick(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                if controller.tick() {
                    controller.sync_to_ui(ui);
                }
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_browse_codex_home(move || {
            with_ui(&weak, |ui| {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    let mut controller = controller.borrow_mut();
                    controller.codex_home_input = folder.to_string_lossy().to_string();
                    controller.schedule_codex_home_refresh();
                    controller.sync_to_ui(ui);
                }
            });
        });
    }

    {
        let controller = controller.clone();
        ui.on_codex_home_edited(move |value| {
            let mut controller = controller.borrow_mut();
            controller.codex_home_input = value.to_string();
            controller.schedule_codex_home_refresh();
        });
    }

    {
        let controller = controller.clone();
        ui.on_add_path_edited(move |value| {
            controller.borrow_mut().add_path_input = value.to_string();
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_add_workspace(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.add_workspace_from_input();
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_browse_workspace(move || {
            with_ui(&weak, |ui| {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    let mut controller = controller.borrow_mut();
                    controller.add_workspace(folder);
                    controller.add_path_input.clear();
                    controller.sync_to_ui(ui);
                }
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_filter_edited(move |value| {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.update_workspace_filter(value.to_string());
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        ui.on_auto_refresh_toggled(move |value| {
            let mut controller = controller.borrow_mut();
            controller.auto_refresh_enabled = value;
            controller.dirty = true;
        });
    }

    {
        let controller = controller.clone();
        ui.on_auto_refresh_seconds_edited(move |value| {
            if let Ok(seconds) = value.trim().parse::<u32>() {
                let mut controller = controller.borrow_mut();
                controller.auto_refresh_seconds = seconds.max(5);
                controller.dirty = true;
            }
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_select_workspace(move |index| {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.select_workspace(index);
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        ui.on_selected_label_edited(move |value| {
            controller
                .borrow_mut()
                .update_selected_label(value.to_string());
        });
    }

    {
        let controller = controller.clone();
        ui.on_selected_rounds_edited(move |value| {
            controller
                .borrow_mut()
                .update_selected_rounds(value.to_string());
        });
    }

    {
        let controller = controller.clone();
        ui.on_selected_prompt_edited(move |value| {
            controller
                .borrow_mut()
                .update_selected_prompt(value.to_string());
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_select_session(move |index| {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.select_session(index);
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_start_selected(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.start_selected_workspace();
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_stop_selected(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.stop_selected_workspace();
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_refresh_selected(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.refresh_selected_workspace();
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_open_selected_folder(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.open_selected_workspace_folder();
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_create_new_session(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.create_new_session_for_selected_workspace();
                controller.sync_to_ui(ui);
            });
        });
    }

    {
        let controller = controller.clone();
        let weak = weak.clone();
        ui.on_clear_logs(move || {
            with_ui(&weak, |ui| {
                let mut controller = controller.borrow_mut();
                controller.clear_selected_logs();
                controller.sync_to_ui(ui);
            });
        });
    }
}

fn with_ui<F>(weak: &Weak<MainWindow>, f: F)
where
    F: FnOnce(&MainWindow),
{
    if let Some(ui) = weak.upgrade() {
        f(&ui);
    }
}

struct DesktopController {
    codex_home_input: String,
    add_path_input: String,
    workspace_filter: String,
    workspaces: Vec<WorkspaceState>,
    selected_workspace_id: Option<u64>,
    next_workspace_id: u64,
    tasks: HashMap<u64, TaskHandle>,
    event_tx: Sender<RuntimeEvent>,
    event_rx: Receiver<RuntimeEvent>,
    codex_version: Option<String>,
    codex_error: Option<String>,
    last_probe_at: Option<DateTime<Local>>,
    last_probe_instant: Option<Instant>,
    auto_refresh_enabled: bool,
    auto_refresh_seconds: u32,
    last_auto_refresh: Instant,
    dirty: bool,
    ui_dirty: bool,
    notice: Option<String>,
    autostart_probe_pending: bool,
    codex_home_refresh_due: Option<Instant>,
}

impl DesktopController {
    fn new() -> Self {
        let (event_tx, event_rx) = unbounded();
        let stored = persistence::load_state().unwrap_or_default();
        let codex_home = stored
            .codex_home
            .unwrap_or_else(|| default_codex_home().to_string_lossy().to_string());

        let mut controller = Self {
            codex_home_input: codex_home,
            add_path_input: String::new(),
            workspace_filter: String::new(),
            workspaces: stored
                .workspaces
                .into_iter()
                .map(WorkspaceState::from_stored)
                .collect(),
            selected_workspace_id: stored.selected_workspace_id,
            next_workspace_id: stored.next_workspace_id.max(1),
            tasks: HashMap::new(),
            event_tx,
            event_rx,
            codex_version: None,
            codex_error: None,
            last_probe_at: None,
            last_probe_instant: None,
            auto_refresh_enabled: stored.auto_refresh_enabled,
            auto_refresh_seconds: stored.auto_refresh_seconds.max(5),
            last_auto_refresh: Instant::now(),
            dirty: false,
            ui_dirty: true,
            notice: None,
            autostart_probe_pending: std::env::var("CRL_AUTOSTART_FIRST")
                .map(|value| value == "1")
                .unwrap_or(false),
            codex_home_refresh_due: None,
        };

        if controller.workspaces.is_empty() {
            controller.next_workspace_id = 1;
        } else if controller.selected_workspace_id.is_none() {
            controller.selected_workspace_id =
                controller.workspaces.first().map(|workspace| workspace.id);
        }

        controller
    }

    fn tick(&mut self) -> bool {
        let mut changed = false;
        self.process_runtime_events();
        changed |= self.take_ui_dirty();

        if let Some(due) = self.codex_home_refresh_due
            && Instant::now() >= due
        {
            self.codex_home_refresh_due = None;
            self.refresh_all_workspaces();
            self.probe_codex_environment();
            changed = true;
        }

        if self.auto_refresh_enabled
            && self.last_auto_refresh.elapsed()
                >= Duration::from_secs(self.auto_refresh_seconds.max(5) as u64)
        {
            self.refresh_all_workspaces();
            if self.should_probe_codex() {
                self.probe_codex_environment();
            }
            changed = true;
        }

        if self.autostart_probe_pending
            && self.tasks.is_empty()
            && let Some(workspace_id) = self.selected_workspace_id
        {
            self.autostart_probe_pending = false;
            append_log(&format!(
                "autostart probe firing for workspace_id={workspace_id}"
            ));
            self.start_workspace(workspace_id);
            changed = true;
        }

        self.save_if_dirty();
        changed | self.take_ui_dirty()
    }

    fn mark_ui_dirty(&mut self) {
        self.ui_dirty = true;
    }

    fn take_ui_dirty(&mut self) -> bool {
        let value = self.ui_dirty;
        self.ui_dirty = false;
        value
    }

    fn sync_to_ui(&self, ui: &MainWindow) {
        ui.set_codex_home(self.codex_home_input.clone().into());
        ui.set_add_path(self.add_path_input.clone().into());
        ui.set_filter_text(self.workspace_filter.clone().into());
        ui.set_auto_refresh_enabled(self.auto_refresh_enabled);
        ui.set_auto_refresh_seconds(self.auto_refresh_seconds.to_string().into());
        ui.set_tick_enabled(self.needs_tick());
        ui.set_tick_interval_ms(self.recommended_tick_interval_ms());
        ui.set_codex_status(self.codex_status_text().into());
        ui.set_codex_is_ready(self.codex_version.is_some());
        ui.set_notice(self.notice.clone().unwrap_or_default().into());

        let (visible_ids, workspace_rows, workspace_options) = self.project_visible_workspaces();
        let selected_index = self
            .selected_workspace_id
            .and_then(|id| visible_ids.iter().position(|candidate| *candidate == id))
            .map(|index| index as i32)
            .unwrap_or(-1);
        ui.set_workspace_rows(ModelRc::from(Rc::new(VecModel::from(workspace_rows))));
        ui.set_workspace_options(ModelRc::from(Rc::new(VecModel::from(workspace_options))));
        ui.set_selected_workspace_index(selected_index);

        if let Some(selected) = self.selected_workspace() {
            ui.set_has_selected_workspace(true);
            ui.set_selected_running(selected.status.is_running());
            ui.set_selected_workspace_name(selected.display_name().into());
            ui.set_selected_workspace_path(selected.path.clone().into());
            ui.set_selected_status(selected.status.label().into());
            ui.set_selected_detail(selected.status.detail().unwrap_or_default().into());
            ui.set_selected_label(selected.label.clone().into());
            ui.set_selected_rounds(selected.rounds.to_string().into());
            ui.set_selected_prompt(selected.prompt.clone().into());

            let session_rows = selected
                .sessions
                .iter()
                .map(|session| SessionRow {
                    id: session.session_id.clone().into(),
                    title: format!("{} · {}", short_id(&session.session_id), session.title).into(),
                    subtitle: format!(
                        "{} · {}",
                        session.last_activity.format("%m-%d %H:%M"),
                        session.last_text
                    )
                    .into(),
                })
                .collect::<Vec<_>>();
            let session_options = selected
                .sessions
                .iter()
                .map(|session| {
                    format!("{} · {}", short_id(&session.session_id), session.title).into()
                })
                .collect::<Vec<SharedString>>();
            let session_index = selected
                .selected_session_id
                .as_deref()
                .and_then(|session_id| {
                    selected
                        .sessions
                        .iter()
                        .position(|session| session.session_id == session_id)
                })
                .map(|index| index as i32)
                .unwrap_or(-1);
            let (selected_session_title, selected_session_subtitle) =
                selected_session_panel_text(selected);
            ui.set_selected_session_index(-1);
            ui.set_session_rows(ModelRc::from(Rc::new(VecModel::from(session_rows))));
            ui.set_session_options(ModelRc::from(Rc::new(VecModel::from(session_options))));
            ui.set_selected_session_title(selected_session_title);
            ui.set_selected_session_subtitle(selected_session_subtitle);
            ui.set_selected_session_index(session_index);

            ui.set_terminal_output(selected.terminal_output.clone().into());
        } else {
            ui.set_has_selected_workspace(false);
            ui.set_selected_running(false);
            ui.set_selected_workspace_name(SharedString::default());
            ui.set_selected_workspace_path(SharedString::default());
            ui.set_selected_status(SharedString::default());
            ui.set_selected_detail(SharedString::default());
            ui.set_selected_label(SharedString::default());
            ui.set_selected_rounds("1".into());
            ui.set_selected_prompt(SharedString::default());
            ui.set_session_rows(ModelRc::from(Rc::new(VecModel::from(
                Vec::<SessionRow>::new(),
            ))));
            ui.set_session_options(ModelRc::from(Rc::new(VecModel::from(
                Vec::<SharedString>::new(),
            ))));
            ui.set_selected_session_title(SharedString::default());
            ui.set_selected_session_subtitle(SharedString::default());
            ui.set_selected_session_index(-1);
            ui.set_terminal_output(SharedString::default());
        }
    }

    fn schedule_codex_home_refresh(&mut self) {
        self.dirty = true;
        self.codex_home_refresh_due = Some(Instant::now() + Duration::from_millis(500));
        self.mark_ui_dirty();
    }

    fn recommended_tick_interval_ms(&self) -> i32 {
        if !self.tasks.is_empty() {
            return 120;
        }
        if self.codex_home_refresh_due.is_some() || self.autostart_probe_pending {
            return 120;
        }
        if self.auto_refresh_enabled {
            return 500;
        }
        1200
    }

    fn codex_home_path(&self) -> PathBuf {
        PathBuf::from(self.codex_home_input.trim())
    }

    fn workspace(&self, workspace_id: u64) -> Option<&WorkspaceState> {
        self.workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
    }

    fn workspace_mut(&mut self, workspace_id: u64) -> Option<&mut WorkspaceState> {
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.id == workspace_id)
    }

    fn selected_workspace(&self) -> Option<&WorkspaceState> {
        let id = self.selected_workspace_id?;
        self.workspace(id)
    }

    fn selected_workspace_mut(&mut self) -> Option<&mut WorkspaceState> {
        let id = self.selected_workspace_id?;
        self.workspace_mut(id)
    }

    fn visible_workspace_ids(&self) -> Vec<u64> {
        let filter = self.workspace_filter.trim().to_lowercase();
        self.workspaces
            .iter()
            .filter(|workspace| self.workspace_matches_filter(workspace, &filter))
            .map(|workspace| workspace.id)
            .collect()
    }

    fn update_workspace_filter(&mut self, value: String) {
        self.workspace_filter = value;
        let visible_ids = self.visible_workspace_ids();

        if !self
            .selected_workspace_id
            .map(|id| visible_ids.contains(&id))
            .unwrap_or(false)
        {
            self.selected_workspace_id = visible_ids.first().copied();
        }

        self.mark_ui_dirty();
    }

    fn select_workspace(&mut self, visible_index: i32) {
        if visible_index < 0 {
            return;
        }
        let visible_ids = self.visible_workspace_ids();
        let Some(workspace_id) = visible_ids.get(visible_index as usize).copied() else {
            return;
        };
        self.selected_workspace_id = Some(workspace_id);
        if let Some(workspace) = self.selected_workspace_mut() {
            workspace.ensure_selected_session();
        }
        self.dirty = true;
        self.mark_ui_dirty();
    }

    fn update_selected_label(&mut self, value: String) {
        if let Some(workspace) = self.selected_workspace_mut() {
            workspace.label = value;
            self.dirty = true;
            self.mark_ui_dirty();
        }
    }

    fn update_selected_rounds(&mut self, value: String) {
        if let Ok(rounds) = value.trim().parse::<u32>()
            && let Some(workspace) = self.selected_workspace_mut()
        {
            workspace.rounds = rounds.max(1);
            self.dirty = true;
            self.mark_ui_dirty();
        }
    }

    fn update_selected_prompt(&mut self, value: String) {
        if let Some(workspace) = self.selected_workspace_mut() {
            workspace.prompt = value;
            self.dirty = true;
            self.mark_ui_dirty();
        }
    }

    fn select_session(&mut self, session_index: i32) {
        if session_index < 0 {
            return;
        }
        if let Some(workspace) = self.selected_workspace_mut()
            && let Some(session) = workspace.sessions.get(session_index as usize)
        {
            workspace.selected_session_id = Some(session.session_id.clone());
            self.dirty = true;
            self.mark_ui_dirty();
        }
    }

    fn add_workspace_from_input(&mut self) {
        let value = self.add_path_input.trim().to_owned();
        if value.is_empty() {
            return;
        }
        self.add_workspace(PathBuf::from(value));
        self.add_path_input.clear();
    }

    fn add_workspace(&mut self, path: PathBuf) {
        let normalized = path.to_string_lossy().to_string();
        if let Some(existing) = self
            .workspaces
            .iter()
            .find(|workspace| workspace.path.eq_ignore_ascii_case(&normalized))
        {
            self.selected_workspace_id = Some(existing.id);
            self.notice = Some("该工作区已存在，已自动选中。".to_owned());
            self.mark_ui_dirty();
            return;
        }

        let workspace = WorkspaceState::from_stored(StoredWorkspace {
            id: self.next_workspace_id,
            label: String::new(),
            path: normalized,
            prompt: "继续上一次结束的位置，完成未完成的工作。".to_owned(),
            rounds: DEFAULT_RESUME_ROUNDS,
            selected_session_id: None,
        });
        self.selected_workspace_id = Some(workspace.id);
        self.next_workspace_id += 1;
        self.workspaces.push(workspace);
        self.dirty = true;
        self.mark_ui_dirty();
        if let Some(workspace_id) = self.selected_workspace_id {
            self.refresh_workspace_sessions(workspace_id);
        }
    }

    fn start_selected_workspace(&mut self) {
        if let Some(workspace_id) = self.selected_workspace_id {
            self.start_workspace(workspace_id);
        }
    }

    fn stop_selected_workspace(&mut self) {
        if let Some(workspace_id) = self.selected_workspace_id {
            self.stop_workspace(workspace_id);
        }
    }

    fn refresh_selected_workspace(&mut self) {
        if let Some(workspace_id) = self.selected_workspace_id {
            self.refresh_workspace_sessions(workspace_id);
        }
    }

    fn open_selected_workspace_folder(&mut self) {
        if let Some(workspace_id) = self.selected_workspace_id {
            self.open_workspace_folder(workspace_id);
        }
    }

    fn create_new_session_for_selected_workspace(&mut self) {
        if let Some(workspace_id) = self.selected_workspace_id {
            self.create_new_session_for_workspace(workspace_id);
        }
    }

    fn clear_selected_logs(&mut self) {
        if let Some(workspace) = self.selected_workspace_mut() {
            workspace.clear_logs();
            self.mark_ui_dirty();
        }
    }

    fn refresh_workspace_sessions(&mut self, workspace_id: u64) {
        let codex_home = self.codex_home_path();
        match discover_workspace_catalog(&codex_home) {
            Ok(catalog) => self.refresh_workspace_sessions_from_catalog(workspace_id, &catalog),
            Err(error) => self.apply_workspace_refresh_error(workspace_id, error.to_string()),
        }
    }

    fn refresh_all_workspaces(&mut self) {
        let workspace_ids = self
            .workspaces
            .iter()
            .map(|workspace| workspace.id)
            .collect::<Vec<_>>();
        let codex_home = self.codex_home_path();
        match discover_workspace_catalog(&codex_home) {
            Ok(catalog) => {
                for workspace_id in workspace_ids {
                    self.refresh_workspace_sessions_from_catalog(workspace_id, &catalog);
                }
            }
            Err(error) => {
                let message = error.to_string();
                for workspace_id in workspace_ids {
                    self.apply_workspace_refresh_error(workspace_id, message.clone());
                }
            }
        }
        self.last_auto_refresh = Instant::now();
    }

    fn probe_codex_environment(&mut self) {
        self.last_probe_at = Some(Local::now());
        self.last_probe_instant = Some(Instant::now());
        match probe_codex_version() {
            Ok(version) => {
                self.codex_version = Some(version);
                self.codex_error = None;
                self.mark_ui_dirty();
            }
            Err(error) => {
                self.codex_version = None;
                self.codex_error = Some(error.to_string());
                self.mark_ui_dirty();
            }
        }
    }
    fn start_workspace(&mut self, workspace_id: u64) {
        append_log(&format!(
            "start_workspace called for workspace_id={workspace_id}"
        ));
        if self.codex_error.is_some() {
            self.notice = Some("Codex is unavailable, so the resume task cannot be started.".to_owned());
            append_log("start_workspace aborted: codex unavailable");
            self.mark_ui_dirty();
            return;
        }

        let Some(workspace) = self.workspace(workspace_id) else {
            return;
        };
        if workspace.status.is_running() || self.tasks.contains_key(&workspace_id) {
            return;
        }

        let Some(session_id) = workspace.selected_session_id.clone() else {
            self.notice = Some(
                "Select a session before starting a resume run, or create a new conversation first.".to_owned(),
            );
            append_log("start_workspace aborted: no session selected");
            self.mark_ui_dirty();
            return;
        };

        if workspace.prompt.trim().is_empty() {
            self.notice = Some("Prompt cannot be empty.".to_owned());
            append_log("start_workspace aborted: prompt empty");
            self.mark_ui_dirty();
            return;
        }

        append_log(&format!(
            "spawning workspace runner workspace_id={} rounds={} session_id={}",
            workspace_id,
            workspace.rounds.max(1),
            session_id
        ));

        let request = WorkspaceRunRequest {
            workspace_id,
            path: workspace.path_buf(),
            session_id,
            prompt: workspace.prompt.clone(),
            rounds: workspace.rounds.max(1),
        };

        let handle = spawn_workspace_runner(request, self.event_tx.clone());
        self.tasks.insert(workspace_id, handle);
        self.notice = Some("Resume task submitted to the background runtime.".to_owned());
        self.mark_ui_dirty();
    }

    fn stop_workspace(&mut self, workspace_id: u64) {
        if let Some(handle) = self.tasks.get(&workspace_id) {
            handle.stop();
            self.mark_ui_dirty();
        }
    }

    fn open_workspace_folder(&mut self, workspace_id: u64) {
        let Some(workspace) = self.workspace(workspace_id) else {
            return;
        };
        if let Err(error) = Command::new("explorer").arg(&workspace.path).spawn() {
            self.notice = Some(format!("无法打开目录：{error}"));
            self.mark_ui_dirty();
        }
    }
    fn create_new_session_for_workspace(&mut self, workspace_id: u64) {
        if self.codex_error.is_some() {
            self.notice = Some("Codex is unavailable, so a new conversation cannot be started.".to_owned());
            self.mark_ui_dirty();
            return;
        }

        let Some(workspace) = self.workspace(workspace_id) else {
            return;
        };
        let workspace_path = workspace.path_buf();
        if !workspace_path.exists() {
            self.notice = Some("The workspace directory does not exist.".to_owned());
            self.mark_ui_dirty();
            return;
        }

        if workspace.status.is_running() || self.tasks.contains_key(&workspace_id) {
            self.notice = Some("A task is already running for this workspace.".to_owned());
            self.mark_ui_dirty();
            return;
        }

        if let Some(workspace) = self.workspace_mut(workspace_id) {
            workspace.clear_logs();
        }
        let handle = spawn_new_session_runner(
            workspace_id,
            workspace_path,
            self.event_tx.clone(),
        );
        self.tasks.insert(workspace_id, handle);
        self.notice = Some("Creating a new Codex conversation in the background.".to_owned());
        self.mark_ui_dirty();
    }

    fn process_runtime_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                RuntimeEvent::Log {
                    workspace_id,
                    entry,
                } => {
                    if let Some(workspace) = self.workspace_mut(workspace_id) {
                        workspace.push_log(entry);
                        self.mark_ui_dirty();
                    }
                }
                RuntimeEvent::OutputChunk {
                    workspace_id,
                    stream,
                    chunk,
                } => {
                    if let Some(workspace) = self.workspace_mut(workspace_id) {
                        workspace.append_output_chunk(stream, &chunk);
                        self.mark_ui_dirty();
                    }
                }
                RuntimeEvent::RoundStarted {
                    workspace_id,
                    current_round,
                    total_rounds,
                } => {
                    if let Some(workspace) = self.workspace_mut(workspace_id) {
                        workspace.status = RunStatus::Running {
                            current_round,
                            total_rounds,
                        };
                        self.mark_ui_dirty();
                    }
                }
                RuntimeEvent::Finished {
                    workspace_id,
                    outcome,
                } => {
                    let should_refresh_sessions = matches!(&outcome, TaskOutcome::Completed);
                    append_log(&format!(
                        "runtime finished for workspace_id={workspace_id}: {:?}",
                        outcome
                    ));
                    self.tasks.remove(&workspace_id);
                    if let Some(workspace) = self.workspace_mut(workspace_id) {
                        workspace.status = match outcome {
                            TaskOutcome::Completed => RunStatus::Completed {
                                finished_at: Local::now(),
                            },
                            TaskOutcome::Stopped => RunStatus::Stopped {
                                finished_at: Local::now(),
                            },
                            TaskOutcome::Error(message) => RunStatus::Error(message),
                        };
                        self.mark_ui_dirty();
                    }
                    if should_refresh_sessions {
                        self.refresh_workspace_sessions(workspace_id);
                    }
                }
            }
        }
    }

    fn save_if_dirty(&mut self) {
        if !self.dirty {
            return;
        }

        let state = StoredAppState {
            codex_home: Some(self.codex_home_input.trim().to_owned()),
            selected_workspace_id: self.selected_workspace_id,
            next_workspace_id: self.next_workspace_id,
            auto_refresh_enabled: self.auto_refresh_enabled,
            auto_refresh_seconds: self.auto_refresh_seconds.max(5),
            workspaces: self
                .workspaces
                .iter()
                .map(WorkspaceState::to_stored)
                .collect(),
        };

        if let Err(error) = persistence::save_state(&state) {
            self.notice = Some(format!("无法保存应用状态：{error}"));
        } else {
            self.dirty = false;
        }
    }

    fn codex_status_text(&self) -> String {
        if let Some(version) = &self.codex_version {
            format!("Codex 已就绪 {version}")
        } else if let Some(error) = &self.codex_error {
            format!("Codex 未就绪：{error}")
        } else {
            "Codex 未就绪".to_owned()
        }
    }

    fn project_visible_workspaces(&self) -> (Vec<u64>, Vec<WorkspaceRow>, Vec<SharedString>) {
        let filter = self.workspace_filter.trim().to_lowercase();
        let mut visible_ids = Vec::new();
        let mut workspace_rows = Vec::new();
        let mut workspace_options = Vec::new();

        for workspace in &self.workspaces {
            if !self.workspace_matches_filter(workspace, &filter) {
                continue;
            }

            let display_name = workspace.display_name();
            visible_ids.push(workspace.id);
            workspace_rows.push(WorkspaceRow {
                id: workspace.id as i32,
                name: display_name.clone().into(),
                path: workspace.path.clone().into(),
                status: workspace.status.label().into(),
                sessions: workspace.sessions.len().to_string().into(),
                rounds: workspace.rounds.to_string().into(),
            });
            workspace_options.push(display_name.into());
        }

        (visible_ids, workspace_rows, workspace_options)
    }

    fn workspace_matches_filter(&self, workspace: &WorkspaceState, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }

        workspace.display_name().to_lowercase().contains(filter)
            || workspace.path.to_lowercase().contains(filter)
            || workspace
                .selected_session_id
                .as_deref()
                .map(|value| value.to_lowercase().contains(filter))
                .unwrap_or(false)
    }

    fn refresh_workspace_sessions_from_catalog(
        &mut self,
        workspace_id: u64,
        catalog: &SessionCatalog,
    ) {
        let Some(workspace) = self.workspace_mut(workspace_id) else {
            return;
        };

        let workspace_path = PathBuf::from(workspace.path.clone());
        if !workspace_path.exists() {
            workspace.sessions.clear();
            workspace.selected_session_id = None;
            workspace.status = RunStatus::Error("工作区目录不存在".to_owned());
            workspace.last_error = Some("工作区目录不存在".to_owned());
            self.mark_ui_dirty();
            return;
        }

        match catalog.sessions_for_workspace(&workspace_path) {
            Ok(sessions) => {
                apply_session_refresh_success(workspace, sessions);
                self.mark_ui_dirty();
            }
            Err(error) => self.apply_workspace_refresh_error(workspace_id, error.to_string()),
        }
    }

    fn apply_workspace_refresh_error(&mut self, workspace_id: u64, error: String) {
        if let Some(workspace) = self.workspace_mut(workspace_id) {
            workspace.sessions.clear();
            workspace.selected_session_id = None;
            workspace.last_error = Some(error.clone());
            workspace.status = RunStatus::Error(error);
            self.mark_ui_dirty();
        }
    }

    fn should_probe_codex(&self) -> bool {
        self.codex_version.is_none()
            || self.codex_error.is_some()
            || self
                .last_probe_instant
                .map(|instant| instant.elapsed() >= CODEX_PROBE_REFRESH_INTERVAL)
                .unwrap_or(true)
    }

    fn needs_tick(&self) -> bool {
        self.auto_refresh_enabled
            || !self.tasks.is_empty()
            || self.autostart_probe_pending
            || self.codex_home_refresh_due.is_some()
            || self.dirty
            || self.ui_dirty
    }
}

fn apply_session_refresh_success(
    workspace: &mut WorkspaceState,
    sessions: Vec<crate::model::SessionSummary>,
) {
    workspace.sessions = sessions;
    workspace.last_refresh = Some(Local::now());
    workspace.last_error = None;
    workspace.ensure_selected_session();
    if !workspace.status.is_running() && !workspace.status.is_terminal() {
        workspace.status = if workspace.sessions.is_empty() {
            RunStatus::NoSessions
        } else {
            RunStatus::Idle
        };
    }
}

fn selected_session_panel_text(workspace: &WorkspaceState) -> (SharedString, SharedString) {
    workspace
        .selected_session()
        .map(|session| {
            (
                format!("{} · {}", short_id(&session.session_id), session.title).into(),
                format!(
                    "{} · {}",
                    session.last_activity.format("%m-%d %H:%M"),
                    session.last_text
                )
                .into(),
            )
        })
        .unwrap_or_else(|| (SharedString::default(), SharedString::default()))
}

fn short_id(session_id: &str) -> String {
    session_id.chars().take(8).collect()
}

#[cfg(test)]
fn format_terminal_output(logs: &VecDeque<LogEntry>) -> String {
    logs.iter()
        .map(|entry| {
            let prefix = match entry.stream {
                LogStream::Stdout => ">",
                LogStream::Stderr => "!",
                LogStream::System => "#",
            };
            format!("{prefix} {}", entry.text)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SessionSummary;
    use std::sync::{LazyLock, Mutex};
    use tempfile::tempdir;

    static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct TestUiModels {
        workspace_rows: Rc<VecModel<WorkspaceRow>>,
        workspace_options: Rc<VecModel<SharedString>>,
        session_rows: Rc<VecModel<SessionRow>>,
        session_options: Rc<VecModel<SharedString>>,
    }

    impl TestUiModels {
        fn new() -> Self {
            Self {
                workspace_rows: Rc::new(VecModel::default()),
                workspace_options: Rc::new(VecModel::default()),
                session_rows: Rc::new(VecModel::default()),
                session_options: Rc::new(VecModel::default()),
            }
        }

        fn publish(
            &self,
            workspace_rows: &[WorkspaceRow],
            workspace_options: &[SharedString],
            session_rows: &[SessionRow],
            session_options: &[SharedString],
        ) {
            self.workspace_rows.set_vec(workspace_rows.to_vec());
            self.workspace_options.set_vec(workspace_options.to_vec());
            self.session_rows.set_vec(session_rows.to_vec());
            self.session_options.set_vec(session_options.to_vec());
        }
    }

    fn sample_workspace(id: u64, path: &str, label: &str) -> WorkspaceState {
        WorkspaceState::from_stored(StoredWorkspace {
            id,
            label: label.to_owned(),
            path: path.to_owned(),
            prompt: "继续".to_owned(),
            rounds: DEFAULT_RESUME_ROUNDS,
            selected_session_id: None,
        })
    }

    fn sample_controller() -> DesktopController {
        let (event_tx, event_rx) = unbounded();
        DesktopController {
            codex_home_input: default_codex_home().to_string_lossy().to_string(),
            add_path_input: String::new(),
            workspace_filter: String::new(),
            workspaces: vec![
                sample_workspace(1, r"E:\one", "one"),
                sample_workspace(2, r"E:\two", "two"),
            ],
            selected_workspace_id: Some(1),
            next_workspace_id: 3,
            tasks: HashMap::new(),
            event_tx,
            event_rx,
            codex_version: None,
            codex_error: None,
            last_probe_at: None,
            last_probe_instant: None,
            auto_refresh_enabled: false,
            auto_refresh_seconds: 30,
            last_auto_refresh: Instant::now(),
            dirty: false,
            ui_dirty: false,
            notice: None,
            autostart_probe_pending: false,
            codex_home_refresh_due: None,
        }
    }

    fn sample_controller_with_workspaces(count: usize) -> DesktopController {
        let mut controller = sample_controller();
        controller.workspaces = (0..count)
            .map(|index| {
                sample_workspace(
                    (index + 1) as u64,
                    &format!(r"E:\workspace-{index}"),
                    &format!("workspace-{index}"),
                )
            })
            .collect();
        controller.selected_workspace_id =
            controller.workspaces.first().map(|workspace| workspace.id);
        controller
    }

    fn sample_workspace_rows(count: usize) -> Vec<WorkspaceRow> {
        (0..count)
            .map(|index| WorkspaceRow {
                id: index as i32,
                name: format!("workspace-{index}").into(),
                path: format!(r"E:\project\workspace-{index}").into(),
                status: "待命".into(),
                sessions: "14".into(),
                rounds: DEFAULT_RESUME_ROUNDS.to_string().into(),
            })
            .collect()
    }

    fn sample_workspace_options(count: usize) -> Vec<SharedString> {
        (0..count)
            .map(|index| format!("workspace-{index}").into())
            .collect()
    }

    fn sample_session_rows(count: usize) -> Vec<SessionRow> {
        (0..count)
            .map(|index| SessionRow {
                id: format!("session-{index}").into(),
                title: format!("session-{index} · title").into(),
                subtitle: "03-24 20:00 · keep going".into(),
            })
            .collect()
    }

    fn sample_session_options(count: usize) -> Vec<SharedString> {
        (0..count)
            .map(|index| format!("session-{index} · title").into())
            .collect()
    }

    fn publish_models_fresh(
        workspace_rows: &[WorkspaceRow],
        workspace_options: &[SharedString],
        session_rows: &[SessionRow],
        session_options: &[SharedString],
    ) {
        let _workspace_rows = ModelRc::from(Rc::new(VecModel::from(workspace_rows.to_vec())));
        let _workspace_options = ModelRc::from(Rc::new(VecModel::from(workspace_options.to_vec())));
        let _session_rows = ModelRc::from(Rc::new(VecModel::from(session_rows.to_vec())));
        let _session_options = ModelRc::from(Rc::new(VecModel::from(session_options.to_vec())));
    }

    fn build_workspace_lists_current(
        controller: &DesktopController,
    ) -> (Vec<WorkspaceRow>, Vec<SharedString>) {
        let visible_ids = controller.visible_workspace_ids();
        let workspace_rows = visible_ids
            .iter()
            .filter_map(|workspace_id| controller.workspace(*workspace_id))
            .map(|workspace| WorkspaceRow {
                id: workspace.id as i32,
                name: workspace.display_name().into(),
                path: workspace.path.clone().into(),
                status: workspace.status.label().into(),
                sessions: workspace.sessions.len().to_string().into(),
                rounds: workspace.rounds.to_string().into(),
            })
            .collect::<Vec<_>>();
        let workspace_options = visible_ids
            .iter()
            .filter_map(|workspace_id| controller.workspace(*workspace_id))
            .map(|workspace| workspace.display_name().into())
            .collect::<Vec<SharedString>>();
        (workspace_rows, workspace_options)
    }

    fn build_workspace_lists_optimized(
        controller: &DesktopController,
    ) -> (Vec<WorkspaceRow>, Vec<SharedString>) {
        let filter = controller.workspace_filter.trim().to_lowercase();
        let visible_workspaces = controller
            .workspaces
            .iter()
            .filter(|workspace| {
                if filter.is_empty() {
                    return true;
                }

                workspace.display_name().to_lowercase().contains(&filter)
                    || workspace.path.to_lowercase().contains(&filter)
                    || workspace
                        .selected_session_id
                        .as_deref()
                        .map(|value| value.to_lowercase().contains(&filter))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        let workspace_rows = visible_workspaces
            .iter()
            .map(|workspace| WorkspaceRow {
                id: workspace.id as i32,
                name: workspace.display_name().into(),
                path: workspace.path.clone().into(),
                status: workspace.status.label().into(),
                sessions: workspace.sessions.len().to_string().into(),
                rounds: workspace.rounds.to_string().into(),
            })
            .collect::<Vec<_>>();
        let workspace_options = visible_workspaces
            .iter()
            .map(|workspace| workspace.display_name().into())
            .collect::<Vec<SharedString>>();
        (workspace_rows, workspace_options)
    }

    #[test]
    fn select_workspace_changes_on_single_call() {
        let mut controller = sample_controller();
        controller.select_workspace(1);
        assert_eq!(controller.selected_workspace_id, Some(2));
    }

    #[test]
    fn select_workspace_ignores_negative_index() {
        let mut controller = sample_controller();
        controller.select_workspace(-1);
        assert_eq!(controller.selected_workspace_id, Some(1));
    }

    #[test]
    fn select_workspace_uses_target_workspaces_own_selected_session() {
        let mut controller = sample_controller();
        controller.workspaces[0].sessions = vec![SessionSummary {
            session_id: "session-a".into(),
            title: "alpha".into(),
            last_text: "alpha".into(),
            last_activity: Local::now(),
            file_path: PathBuf::from("a"),
            message_count: 1,
        }];
        controller.workspaces[0].selected_session_id = Some("session-a".into());

        controller.workspaces[1].sessions = vec![
            SessionSummary {
                session_id: "session-b".into(),
                title: "beta".into(),
                last_text: "beta".into(),
                last_activity: Local::now(),
                file_path: PathBuf::from("b"),
                message_count: 1,
            },
            SessionSummary {
                session_id: "session-c".into(),
                title: "gamma".into(),
                last_text: "gamma".into(),
                last_activity: Local::now(),
                file_path: PathBuf::from("c"),
                message_count: 1,
            },
        ];
        controller.workspaces[1].selected_session_id = None;

        controller.select_workspace(1);

        assert_eq!(controller.selected_workspace_id, Some(2));
        assert_eq!(
            controller.workspaces[1].selected_session_id.as_deref(),
            Some("session-b")
        );
        assert_eq!(
            controller.workspaces[0].selected_session_id.as_deref(),
            Some("session-a")
        );
    }

    #[test]
    fn selected_session_panel_text_uses_current_workspaces_session() {
        let mut first = sample_workspace(1, r"E:\one", "one");
        first.sessions = vec![SessionSummary {
            session_id: "session-a".into(),
            title: "alpha".into(),
            last_text: "first detail".into(),
            last_activity: Local::now(),
            file_path: PathBuf::from("a"),
            message_count: 1,
        }];
        first.selected_session_id = Some("session-a".into());

        let mut second = sample_workspace(2, r"E:\two", "two");
        second.sessions = vec![SessionSummary {
            session_id: "session-b".into(),
            title: "beta".into(),
            last_text: "second detail".into(),
            last_activity: Local::now(),
            file_path: PathBuf::from("b"),
            message_count: 1,
        }];
        second.selected_session_id = Some("session-b".into());

        let (first_title, first_subtitle) = selected_session_panel_text(&first);
        let (second_title, second_subtitle) = selected_session_panel_text(&second);

        assert!(first_title.contains("alpha"));
        assert!(first_subtitle.contains("first detail"));
        assert!(second_title.contains("beta"));
        assert!(second_subtitle.contains("second detail"));
        assert_ne!(first_title, second_title);
        assert_ne!(first_subtitle, second_subtitle);
    }

    #[test]
    fn select_session_changes_on_single_call() {
        let mut controller = sample_controller();
        let workspace = controller.selected_workspace_mut().unwrap();
        workspace.sessions = vec![
            SessionSummary {
                session_id: "a".into(),
                title: "first".into(),
                last_text: "alpha".into(),
                last_activity: Local::now(),
                file_path: PathBuf::from("a"),
                message_count: 1,
            },
            SessionSummary {
                session_id: "b".into(),
                title: "second".into(),
                last_text: "beta".into(),
                last_activity: Local::now(),
                file_path: PathBuf::from("b"),
                message_count: 1,
            },
        ];
        controller.select_session(1);
        assert_eq!(
            controller
                .selected_workspace()
                .unwrap()
                .selected_session_id
                .as_deref(),
            Some("b")
        );
    }

    #[test]
    fn select_session_ignores_negative_index() {
        let mut controller = sample_controller();
        let workspace = controller.selected_workspace_mut().unwrap();
        workspace.sessions = vec![SessionSummary {
            session_id: "a".into(),
            title: "first".into(),
            last_text: "alpha".into(),
            last_activity: Local::now(),
            file_path: PathBuf::from("a"),
            message_count: 1,
        }];
        controller.select_session(-1);
        assert_eq!(
            controller
                .selected_workspace()
                .unwrap()
                .selected_session_id
                .as_deref(),
            None
        );
    }

    #[test]
    fn filter_selects_first_visible_workspace() {
        let mut controller = sample_controller();
        controller.selected_workspace_id = Some(1);
        controller.update_workspace_filter("two".into());
        assert_eq!(controller.selected_workspace_id, Some(2));
        assert!(controller.take_ui_dirty());
    }

    #[test]
    fn filter_clears_selection_when_no_workspace_matches() {
        let mut controller = sample_controller();
        controller.update_workspace_filter("missing".into());
        assert_eq!(controller.selected_workspace_id, None);
    }

    #[test]
    fn refresh_does_not_clear_terminal_status() {
        let mut workspace = sample_workspace(1, r"E:\one", "one");
        workspace.status = RunStatus::Completed {
            finished_at: Local::now(),
        };
        let sessions = vec![SessionSummary {
            session_id: "a".into(),
            title: "first".into(),
            last_text: "alpha".into(),
            last_activity: Local::now(),
            file_path: PathBuf::from("a"),
            message_count: 1,
        }];
        apply_session_refresh_success(&mut workspace, sessions);

        assert_eq!(workspace.status.label(), "已完成");
        assert!(workspace.status.is_terminal());
        assert_eq!(workspace.selected_session_id.as_deref(), Some("a"));
    }

    #[test]
    fn terminal_output_is_stream_like_without_timestamps() {
        let mut logs = VecDeque::new();
        logs.push_back(LogEntry {
            timestamp: Local::now(),
            stream: LogStream::System,
            text: "准备开始".into(),
        });
        logs.push_back(LogEntry {
            timestamp: Local::now(),
            stream: LogStream::Stdout,
            text: "doing work".into(),
        });
        logs.push_back(LogEntry {
            timestamp: Local::now(),
            stream: LogStream::Stderr,
            text: "warning".into(),
        });

        let rendered = format_terminal_output(&logs);
        assert!(rendered.contains("# 准备开始"));
        assert!(rendered.contains("> doing work"));
        assert!(rendered.contains("! warning"));
        assert!(!rendered.contains("["));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn create_new_session_runs_in_background_without_terminal_launcher() {
        let _guard = ENV_MUTEX.lock().expect("env mutex");
        let temp = tempdir().expect("tempdir");
        let workspace_dir = temp.path().join("workspace");
        let codex_home = temp.path().join("codex-home");
        let bin_dir = temp.path().join("bin");
        std::fs::create_dir_all(&workspace_dir).expect("workspace");
        std::fs::create_dir_all(&bin_dir).expect("bin");

        std::fs::write(
            bin_dir.join("codex.cmd"),
            "@echo off\r\npowershell.exe -NoProfile -ExecutionPolicy Bypass -File \"%~dp0mock-codex.ps1\" %*\r\nexit /b %ERRORLEVEL%\r\n",
        )
        .expect("write codex cmd");
        std::fs::write(
            bin_dir.join("mock-codex.ps1"),
            "$argsPath = Join-Path $PSScriptRoot 'args.txt'\r\n[System.IO.File]::WriteAllLines($argsPath, $args)\r\nWrite-Output 'created'\r\nexit 0\r\n",
        )
        .expect("write mock codex");

        let original_path = std::env::var("PATH").unwrap_or_default();
        unsafe {
            std::env::set_var("PATH", format!("{};{}", bin_dir.display(), original_path));
        }

        let mut controller = sample_controller();
        controller.codex_home_input = codex_home.to_string_lossy().to_string();
        controller.workspaces = vec![sample_workspace(
            1,
            &workspace_dir.to_string_lossy(),
            "workspace",
        )];
        controller.workspaces[0].prompt = "this should be ignored".to_owned();
        controller.selected_workspace_id = Some(1);

        controller.create_new_session_for_selected_workspace();

        for _ in 0..60 {
            controller.process_runtime_events();
            if controller.tasks.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        unsafe {
            std::env::set_var("PATH", original_path);
        }

        let args = std::fs::read_to_string(bin_dir.join("args.txt"))
            .expect("args file")
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        assert!(controller.tasks.is_empty());
        assert_eq!(
            args,
            vec![
                "exec".to_owned(),
                "--skip-git-repo-check".to_owned(),
                crate::codex::NEW_SESSION_BOOTSTRAP_PROMPT.to_owned(),
            ]
        );
        assert!(controller.workspaces[0].terminal_output.contains("> created"));
    }

    #[test]
    fn idle_tick_interval_is_slow() {
        let controller = sample_controller();
        assert_eq!(controller.recommended_tick_interval_ms(), 1200);
    }

    #[test]
    fn running_tick_interval_is_fast() {
        let mut controller = sample_controller();
        controller.codex_home_refresh_due = Some(Instant::now());
        assert_eq!(controller.recommended_tick_interval_ms(), 120);
    }

    #[test]
    fn idle_without_auto_refresh_disables_tick() {
        let controller = sample_controller();
        assert!(!controller.needs_tick());
    }

    #[test]
    fn auto_refresh_enables_tick() {
        let mut controller = sample_controller();
        controller.auto_refresh_enabled = true;
        assert!(controller.needs_tick());
    }

    #[test]
    fn pending_refresh_enables_tick() {
        let mut controller = sample_controller();
        controller.codex_home_refresh_due = Some(Instant::now());
        assert!(controller.needs_tick());
    }

    #[test]
    fn recent_probe_is_not_repeated_during_auto_refresh() {
        let mut controller = sample_controller();
        controller.codex_version = Some("0.1.0".into());
        controller.last_probe_instant = Some(Instant::now());

        assert!(!controller.should_probe_codex());
    }

    #[test]
    fn stale_probe_is_retried() {
        let mut controller = sample_controller();
        controller.codex_version = Some("0.1.0".into());
        controller.last_probe_instant = Some(Instant::now() - CODEX_PROBE_REFRESH_INTERVAL);

        assert!(controller.should_probe_codex());
    }

    #[test]
    #[ignore = "manual performance probe"]
    fn benchmark_reused_models_reduce_ui_sync_overhead() {
        let workspace_rows = sample_workspace_rows(180);
        let workspace_options = sample_workspace_options(180);
        let session_rows = sample_session_rows(60);
        let session_options = sample_session_options(60);
        let iterations = 2_000;

        let baseline_start = Instant::now();
        for _ in 0..iterations {
            publish_models_fresh(
                &workspace_rows,
                &workspace_options,
                &session_rows,
                &session_options,
            );
        }
        let baseline = baseline_start.elapsed();

        let models = TestUiModels::new();
        let optimized_start = Instant::now();
        for _ in 0..iterations {
            models.publish(
                &workspace_rows,
                &workspace_options,
                &session_rows,
                &session_options,
            );
        }
        let optimized = optimized_start.elapsed();

        println!(
            "fresh_models={:?} reused_models={:?} speedup={:.2}x",
            baseline,
            optimized,
            baseline.as_secs_f64() / optimized.as_secs_f64()
        );
        assert!(optimized < baseline);
    }

    #[test]
    #[ignore = "manual performance probe"]
    fn benchmark_single_pass_workspace_projection() {
        let controller = sample_controller_with_workspaces(1_200);
        let iterations = 300;

        let baseline_start = Instant::now();
        for _ in 0..iterations {
            let _ = build_workspace_lists_current(&controller);
        }
        let baseline = baseline_start.elapsed();

        let optimized_start = Instant::now();
        for _ in 0..iterations {
            let _ = build_workspace_lists_optimized(&controller);
        }
        let optimized = optimized_start.elapsed();

        println!(
            "current_projection={:?} optimized_projection={:?} speedup={:.2}x",
            baseline,
            optimized,
            baseline.as_secs_f64() / optimized.as_secs_f64()
        );
        assert!(optimized < baseline);
    }
}
