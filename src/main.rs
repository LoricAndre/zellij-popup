use zellij_tile::prelude::*;

use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct State {
    // Latest pane manifest: the single source of truth for which popups exist
    // and whether they are visible.
    manifest: Option<PaneManifest>,
    // Command lines of the popups this plugin spawned.
    spawned: BTreeSet<String>,
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
        subscribe(&[EventType::PaneUpdate, EventType::PermissionRequestResult]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.ready = true;
            }
            Event::PaneUpdate(manifest) => {
                self.manifest = Some(manifest);
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

            let args: Vec<String> = parts.map(String::from).collect();
            self.toggle_popup(command, args);
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
    fn panes(&self) -> Vec<&PaneInfo> {
        self.manifest
            .iter()
            .flat_map(|m| m.panes.values().flatten())
            .filter(|p| !p.is_plugin)
            .collect()
    }

    fn toggle_popup(&mut self, command: String, args: Vec<String>) {
        let line = std::iter::once(command.as_str())
            .chain(args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ");

        let panes = self.panes();
        let existing = panes
            .iter()
            .find(|p| p.terminal_command.as_deref() == Some(line.as_str()))
            .map(|p| (p.id, p.is_suppressed, p.exited));

        // Visible popups other than the target get hidden first, so only one shows.
        let others: Vec<u32> = panes
            .iter()
            .filter(|p| {
                p.is_floating
                    && !p.is_suppressed
                    && p.terminal_command
                        .as_ref()
                        .is_some_and(|c| c != &line && self.spawned.contains(c))
            })
            .map(|p| p.id)
            .collect();

        match existing {
            Some((id, false, false)) => hide_pane_with_id(PaneId::Terminal(id)),
            Some((id, true, false)) => {
                for other in others {
                    hide_pane_with_id(PaneId::Terminal(other));
                }
                show_pane_with_id(PaneId::Terminal(id), true, true);
                // Zellij resets floating pane size to its default when
                // suppressed panes are shown.
                change_floating_panes_coordinates(vec![(PaneId::Terminal(id), popup_coords())]);
            }
            other => {
                // An exited command pane is replaced rather than re-run in place.
                if let Some((id, _, true)) = other {
                    close_pane_with_id(PaneId::Terminal(id));
                }
                for other in others {
                    hide_pane_with_id(PaneId::Terminal(other));
                }

                let mut cmd = CommandToRun::new(command);
                cmd.args = args;
                cmd.cwd = get_focused_pane_info()
                    .and_then(|(_, id)| get_pane_cwd(id))
                    .ok()
                    .or_else(|| std::env::current_dir().ok());
                open_command_pane_floating(cmd, Some(popup_coords()), BTreeMap::new());
                self.spawned.insert(line);
            }
        }
    }
}
