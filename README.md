# zellij-popup

A Zellij plugin that manages long-lived floating panes (lazygit, btm, etc.) with a single keybind to toggle them on and off. Only one popup is visible at a time — toggling a second one hides the first.

## How it works

The plugin runs silently in the background. You trigger it by sending a `toggle` message to it from a keybind, passing the command to run as the payload. On the first press the pane is created; subsequent presses hide or show it. The pane's process stays alive while hidden.

```
first press  → open floating pane (90% of screen, centred)
second press → hide it (process keeps running)
third press  → show it again
```

If a different popup is already visible when you press a keybind, it is hidden automatically before the new one appears.

### Why `shared_except "locked"` and `Alt` keys

Binds placed in the `normal` keybind block are only intercepted by Zellij when no terminal pane is focused in a modal way. Once a floating pane (e.g. lazygit) has focus, keystrokes go directly to the process — so a `normal`-only bind cannot dismiss it.

Binds in `shared_except "locked"` fire in every input mode, including when a terminal pane has focus. `Alt` key combinations are used because most of the `Ctrl` combinations are already claimed by Zellij's default binds in that shared block.

## Requirements

- [Zellij](https://zellij.dev) ≥ 0.44
- Rust with the `wasm32-wasip1` target

```sh
rustup target add wasm32-wasip1
```

## Build

```sh
cargo build
# wasm artifact: target/wasm32-wasip1/debug/zellij-popup.wasm

cargo build --release
# wasm artifact: target/wasm32-wasip1/release/zellij-popup.wasm
```

## Quick test (without touching your main config)

A self-contained `config.kdl` is included. It inherits all default Zellij keybinds and adds two popup toggles:

| Keybind | Command |
|---------|---------|
| `Alt g` | `lazygit` |
| `Alt b` | `btm` |

```sh
# Build first, then start Zellij with the test config
cargo build
zellij --config config.kdl
```

To also open the dev layout (editor + hot-reload helper):

```sh
zellij --config config.kdl --layout zellij.kdl
```

## Adding to your own config

### 1. Load the plugin on session start

```kdl
load_plugins {
    "file:/path/to/zellij-popup.wasm"
}
```

### 2. Bind keys

Binds must be in `shared_except "locked"` (not just `normal`) so they fire even when the popup pane has focus and can dismiss it.

```kdl
keybinds {
    shared_except "locked" {
        bind "Alt g" {
            MessagePlugin "file:/path/to/zellij-popup.wasm" {
                name "toggle"
                payload "lazygit"
            }
        }
        bind "Alt b" {
            MessagePlugin "file:/path/to/zellij-popup.wasm" {
                name "toggle"
                payload "btm"
            }
        }
    }
}
```

### Passing extra flags to the command

The `payload` is the full command line — the first token is the binary, the rest are its arguments:

```kdl
bind "Alt g" {
    MessagePlugin "file:/path/to/zellij-popup.wasm" {
        name "toggle"
        payload "lazygit -p /path/to/repo"
    }
}
```

## Reload after rebuilding

While inside a running session:

```sh
cargo build
zellij action start-or-reload-plugin file:target/wasm32-wasip1/debug/zellij-popup.wasm
```

## Development

Run the included dev layout for a hot-reload environment:

```sh
zellij --config config.kdl --layout zellij.kdl
```

This opens `src/main.rs` in your `$EDITOR`, a shell pane for running `cargo build`, and a floating pane that watches for the compiled `.wasm` and reloads it automatically.
