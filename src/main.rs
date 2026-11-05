use zellij_tile::prelude::*;

use std::collections::BTreeMap;

// Each managed popup entry tracks its pane id and whether it is currently visible.
#[derive(Default, Debug)]
struct PopupEntry {
    pane_id: u32,
    visible: bool,
}

#[derive(Default)]
struct State {
    // Map from session name → command name → managed floating pane state.
    popups: BTreeMap<String, BTreeMap<String, PopupEntry>>,
    // The name of the currently visible popup in the current session, if any.
    active: Option<String>,
    // The name of the current session, learned from SessionUpdate events.
    current_session: Option<String>,
    // Set to true once permissions have been granted.
    ready: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::RunCommands,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            EventType::PaneUpdate,
            EventType::PermissionRequestResult,
            EventType::SessionUpdate,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.ready = true;
            }
            Event::PaneUpdate(manifest) => {
                self.sync_pane_state(&manifest);
            }
            Event::SessionUpdate(sessions, _) => {
                if let Some(current) = sessions.iter().find(|s| s.is_current_session) {
                    self.current_session = Some(current.name.clone());
                }
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if !self.ready {
            return false;
        }

        // Expected pipe message name: "toggle"
        // The payload is the full command line: the first whitespace-delimited
        // token is the binary to run, the rest are its arguments.
        // e.g. payload = "lazygit" or "lazygit -p /path/to/repo"
        if pipe_message.name == "toggle" {
            let payload = pipe_message.payload.unwrap_or_default();
            let mut parts = payload.split_whitespace();

            let command = match parts.next() {
                Some(c) => c.to_string(),
                None => return false,
            };

            let extra_args: Vec<String> = parts.map(String::from).collect();

            self.toggle_popup(&command, extra_args);
        }

        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        // This plugin runs in the background — it manages other floating panes
        // and has no visible UI of its own.
    }
}

fn popup_coords() -> FloatingPaneCoordinates {
    FloatingPaneCoordinates::default()
        .with_x_percent(5)
        .with_y_percent(5)
        .with_width_percent(90)
        .with_height_percent(90)
}

impl State {
    fn session_popups(&mut self) -> &mut BTreeMap<String, PopupEntry> {
        let session = self.current_session.get_or_insert_with(|| "default".to_string()).clone();
        self.popups.entry(session).or_default()
    }

    fn toggle_popup(&mut self, command: &str, extra_args: Vec<String>) {
        // Read current state without holding a mutable borrow.
        let existing = self
            .session_popups()
            .get(command)
            .map(|e| (e.pane_id, e.visible));

        match existing {
            Some((pane_id, true)) => {
                // Pane exists and is visible — hide it (toggle off).
                if pane_id != 0 {
                    hide_pane_with_id(PaneId::Terminal(pane_id));
                }
                if let Some(entry) = self.session_popups().get_mut(command) {
                    entry.visible = false;
                }
                self.active = None;
            }
            Some((pane_id, false)) => {
                // Pane exists but is hidden — hide any other visible popup, then show this one.
                self.hide_active();
                if pane_id != 0 {
                    show_pane_with_id(PaneId::Terminal(pane_id), true, true);
                    // Restore the original coordinates — Zellij resets floating
                    // pane size to its default when suppressed panes are shown.
                    change_floating_panes_coordinates(vec![(
                        PaneId::Terminal(pane_id),
                        popup_coords(),
                    )]);
                }
                if let Some(entry) = self.session_popups().get_mut(command) {
                    entry.visible = true;
                }
                self.active = Some(command.to_string());
            }
            None => {
                // Pane doesn't exist yet — open it as a new floating command pane.
                self.hide_active();

                let mut cmd = CommandToRun::new(command);
                cmd.args = extra_args;
                cmd.cwd = get_focused_pane_info()
                    .and_then(|(_, id)| get_pane_cwd(id))
                    .ok()
                    .or_else(|| std::env::current_dir().ok());

                // open_command_pane_floating may return the PaneId synchronously.
                let returned_id =
                    open_command_pane_floating(cmd, Some(popup_coords()), BTreeMap::new());

                // Use the returned PaneId if available; otherwise use 0 as a
                // sentinel and let sync_pane_state() fill it in on the next
                // PaneUpdate event.
                let pane_id = match returned_id {
                    Some(PaneId::Terminal(id)) => id,
                    _ => 0,
                };
                self.session_popups().insert(
                    command.to_string(),
                    PopupEntry {
                        pane_id,
                        visible: true,
                    },
                );
                self.active = Some(command.to_string());
            }
        }
    }

    fn hide_active(&mut self) {
        if let Some(active_name) = self.active.take() {
            if let Some(entry) = self.session_popups().get_mut(&active_name) {
                if entry.visible && entry.pane_id != 0 {
                    hide_pane_with_id(PaneId::Terminal(entry.pane_id));
                    entry.visible = false;
                }
            }
        }
    }

    /// Reconcile internal state against the pane manifest received from Zellij.
    ///
    /// We are the authoritative source for `visible` — we never overwrite it
    /// from the pane manifest, because PaneUpdate events race with our own
    /// hide/show calls and would corrupt the toggle state.
    ///
    /// This function only does two things:
    ///   1. Resolve a pending pane id (pane_id == 0) by matching on title.
    ///   2. Detect panes that have been closed by the user and clean them up.
    fn sync_pane_state(&mut self, manifest: &PaneManifest) {
        let all_panes: Vec<&PaneInfo> = manifest.panes.values().flatten().collect();

        let mut to_remove: Vec<String> = Vec::new();

        for (name, entry) in self.session_popups().iter_mut() {
            if entry.pane_id == 0 {
                // Pending launch: look for a floating command pane whose title
                // contains the command name (Zellij sets the command as the title).
                if let Some(pane) = all_panes.iter().find(|p| {
                    p.is_floating
                        && !p.is_plugin
                        && p.title.to_lowercase().contains(&name.to_lowercase())
                }) {
                    entry.pane_id = pane.id;
                    // Leave entry.visible as-is — we set it when we launched.
                }
            } else {
                // Detect user-closed panes.
                let still_exists = all_panes.iter().any(|p| p.id == entry.pane_id);
                if !still_exists {
                    to_remove.push(name.clone());
                }
            }
        }

        for name in to_remove {
            self.session_popups().remove(&name);
            if self.active.as_deref() == Some(&name) {
                self.active = None;
            }
        }
    }
}
