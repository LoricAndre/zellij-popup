use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    panes: HashMap<String, PaneState>,
    userspace_configuration: BTreeMap<String, String>,
    pane_manifest: Option<PaneManifest>,
}

#[derive(Clone)]
struct PaneState {
    pane_id: Option<u32>,
    command: String,
    coordinates: Option<FloatingPaneCoordinates>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            PermissionType::OpenFiles,
        ]);
        subscribe(&[EventType::PaneUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(result) => {
                if result != PermissionStatus::Granted {
                    eprintln!("Permission denied");
                }
            }
            Event::PaneUpdate(manifest) => {
                eprintln!("PaneUpdate received, {} panes", manifest.panes.len());
                self.update_pane_tracking(&manifest);
                self.pane_manifest = Some(manifest);
            }
            Event::PaneClosed(pane_id) => {
                eprintln!("PaneClosed: pane_id={:?}", pane_id);
                // PaneId might be complex, so just clear all closed panes on next update
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let action = pipe_message.name.as_str();
        let payload = pipe_message.payload.unwrap_or_default();

        match action {
            "toggle" => {
                if let Ok(config) = serde_json::from_str::<ToggleConfig>(&payload) {
                    self.toggle_pane(&config.name, &config.command, config.cwd);
                }
            }
            "open" => {
                if let Ok(config) = serde_json::from_str::<ToggleConfig>(&payload) {
                    self.open_pane(&config.name, &config.command, config.cwd);
                }
            }
            "close" => {
                if let Ok(config) = serde_json::from_str::<CloseConfig>(&payload) {
                    self.close_pane(&config.name);
                }
            }
            _ => {}
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        // This plugin runs in headless mode, no UI needed
    }
}

impl State {
    fn update_pane_tracking(&mut self, manifest: &PaneManifest) {
        // Match panes in the manifest with our tracked panes based on command
        // First, clear any tracked panes that no longer exist
        let mut existing_pane_ids = std::collections::HashSet::new();
        for (_tab_pos, panes) in &manifest.panes {
            for pane_info in panes {
                if !pane_info.is_plugin {
                    existing_pane_ids.insert(pane_info.id);
                }
            }
        }

        for pane_state in self.panes.values_mut() {
            if let Some(pane_id) = pane_state.pane_id {
                if !existing_pane_ids.contains(&pane_id) {
                    eprintln!("Pane {} no longer exists, clearing", pane_id);
                    pane_state.pane_id = None;
                }
            }
        }

        // Now match untracked popups to panes
        for (name, pane_state) in self.panes.iter_mut() {
            if pane_state.pane_id.is_none() {
                // Look for a pane with matching title or command
                for (_tab_pos, panes) in &manifest.panes {
                    for pane_info in panes {
                        if pane_info.is_plugin {
                            continue;
                        }
                        let pane_id = pane_info.id;

                        // First try matching by title (most reliable)
                        if pane_info.title == *name {
                            eprintln!("Matched pane {} to popup {} by title", pane_id, name);
                            pane_state.pane_id = Some(pane_id);
                            break;
                        }

                        // Otherwise try matching by command and then set the title
                        let pane_command = pane_info.terminal_command.clone().unwrap_or_default();
                        eprintln!(
                            "Checking pane {}: title={:?}, command={:?}",
                            pane_id, pane_info.title, pane_command
                        );
                        if pane_command.contains(&pane_state.command) {
                            eprintln!(
                                "Matched pane {} to popup {} by command, setting title",
                                pane_id, name
                            );
                            pane_state.pane_id = Some(pane_id);
                            rename_terminal_pane(pane_id, name);
                            break;
                        }
                    }
                    if pane_state.pane_id.is_some() {
                        break;
                    }
                }
            }
        }
    }

    fn toggle_pane(&mut self, name: &str, command: &str, cwd: Option<PathBuf>) {
        eprintln!("toggle_pane: name={}, command={}", name, command);
        if let Some(pane_state) = self.panes.get(name) {
            eprintln!(
                "Found existing pane state, pane_id={:?}",
                pane_state.pane_id
            );
            if let Some(pane_id) = pane_state.pane_id {
                // Check if pane is currently visible by looking at manifest
                if let Some(ref manifest) = self.pane_manifest {
                    for (_tab_pos, panes) in &manifest.panes {
                        for pane_info in panes {
                            if pane_info.id == pane_id {
                                if pane_info.is_suppressed {
                                    eprintln!("Pane is hidden, showing it");
                                    show_pane_with_id(PaneId::Terminal(pane_id), true);
                                    // Restore the original coordinates
                                    if let Some(coords) = pane_state.coordinates.clone() {
                                        eprintln!("Restoring coordinates for pane {}", pane_id);
                                        change_floating_panes_coordinates(vec![(
                                            PaneId::Terminal(pane_id),
                                            coords,
                                        )]);
                                    }
                                } else {
                                    eprintln!("Pane is visible, hiding it");
                                    hide_pane_with_id(PaneId::Terminal(pane_id));
                                }
                                return;
                            }
                        }
                    }
                }
                // Pane not found in manifest, might have been closed manually
                eprintln!("Pane not found in manifest, opening new one");
            }
        }
        eprintln!("Pane doesn't exist, opening it");
        // Pane doesn't exist, open it
        self.open_pane(name, command, cwd);
    }

    fn open_pane(&mut self, name: &str, command: &str, cwd: Option<PathBuf>) {
        // Close all other panes
        for pane in &self.panes {
            if let Some(id) = pane.1.pane_id {
                close_terminal_pane(id);
            }
        }
        // Check if pane already exists and is open
        if let Some(pane_state) = self.panes.get(name) {
            if let Some(pane_id) = pane_state.pane_id {
                // Pane exists, just focus it
                eprintln!("Pane {} already exists, focusing", pane_id);
                focus_terminal_pane(pane_id, true);
                return;
            }
        }

        // Create context to identify this pane
        let mut context = BTreeMap::new();
        context.insert("popup_name".to_string(), name.to_string());

        // Define the floating pane coordinates
        let coordinates = FloatingPaneCoordinates::new(
            Some("10%".to_string()),
            Some("10%".to_string()),
            Some("80%".to_string()),
            Some("80%".to_string()),
            None, // skip_plugin_ids parameter
        );

        // Open a new floating pane
        open_command_pane_floating(
            CommandToRun {
                path: "zsh".into(),
                args: vec!["-i".into(), "-c".into(), command.into()],
                cwd,
            },
            coordinates.clone(),
            context,
        );

        // Store the pane state - pane_id will be set when PaneUpdate event fires
        self.panes.insert(
            name.to_string(),
            PaneState {
                pane_id: None, // Will be updated by PaneUpdate event
                command: command.to_string(),
                coordinates,
            },
        );

        eprintln!(
            "Opened pane for popup {}, will be renamed when tracked",
            name
        );
    }

    fn close_pane(&mut self, name: &str) {
        if let Some(pane_state) = self.panes.get_mut(name) {
            if let Some(pane_id) = pane_state.pane_id {
                eprintln!("Closing pane {} for popup {}", pane_id, name);
                close_terminal_pane(pane_id);
                // Clear the pane_id so it can be reopened later
                // PaneUpdate will handle cleanup of the tracking
                pane_state.pane_id = None;
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct ToggleConfig {
    name: String,
    command: String,
    cwd: Option<PathBuf>,
}

#[derive(serde::Deserialize)]
struct CloseConfig {
    name: String,
}
