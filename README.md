# Zellij Popup Plugin

A Zellij plugin that allows you to open and toggle named floating panes, each running a command in an interactive zsh session.

## Features

- Open floating panes by name
- Toggle panes (open if closed, close if open)
- Run commands in interactive zsh sessions
- Control via pipe commands or keybindings

## Building

```bash
cargo build --release
```

The compiled plugin will be at `target/wasm32-wasip1/release/zellij-popup.wasm`

## Installation

### Option 1: Use GitHub Release URL (Recommended)

No download needed! Just reference the plugin directly from GitHub in your config:

```kdl
https://github.com/LoricAndre/zellij-popup/releases/latest/download/zellij-popup.wasm
```

See the Configuration section below for examples.

### Option 2: Download Pre-built WASM

```bash
mkdir -p ~/.config/zellij/plugins
wget https://github.com/LoricAndre/zellij-popup/releases/latest/download/zellij-popup.wasm \
  -O ~/.config/zellij/plugins/zellij-popup.wasm
```

### Option 3: Build from Source

1. Build the plugin (requires Rust and the wasm32-wasip1 target):
   ```bash
   rustup target add wasm32-wasip1
   cargo build --release
   ```

2. Copy the plugin to your Zellij plugins directory:
   ```bash
   mkdir -p ~/.config/zellij/plugins
   cp target/wasm32-wasip1/release/zellij-popup.wasm ~/.config/zellij/plugins/
   ```

3. Load the plugin in your Zellij layout or configuration (see Configuration section)

## Configuration

### Option 1: Use the Complete Example Config

```bash
# Test with the complete example config
zellij --config examples/complete-config.kdl

# Or copy it to use as your main config
cp examples/complete-config.kdl ~/.config/zellij/config.kdl
```

### Option 2: Add to Existing Config

You can define a plugin alias in `~/.config/zellij/config.kdl`:

```kdl
plugins {
    popup {
        path "https://github.com/LoricAndre/zellij-popup/releases/latest/download/zellij-popup.wasm"
    }
}
```

Then use the alias in keybindings:

```kdl
keybinds {
    shared_except "locked" {
        bind "Alt h" {
            MessagePlugin "popup" {
                name "toggle"
                payload "{\"name\":\"htop\",\"command\":\"htop\"}"
            }
        }
    }
}
```

And add to your layout file (e.g., `~/.config/zellij/layouts/default.kdl`):

```kdl
layout {
    pane

    // Load the popup plugin (headless - no visible UI)
    pane size=1 borderless=true {
        plugin location="https://github.com/LoricAndre/zellij-popup/releases/latest/download/zellij-popup.wasm"
    }
}
```

**Note:** If you downloaded the plugin locally, use `file:~/.config/zellij/plugins/zellij-popup.wasm` instead.

## Usage

### Toggle a Pane

Use the pipe command to toggle a named pane:

```bash
zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'
```

### Open a Pane

```bash
zellij pipe --plugin-configuration --name open --payload '{"name":"logs","command":"tail -f /var/log/syslog"}'
```

### Close a Pane

```bash
zellij pipe --plugin-configuration --name close --payload '{"name":"htop"}'
```

## Keybinding Examples

Add to your `~/.config/zellij/config.kdl`:

```kdl
keybinds {
    shared_except "locked" {
        // Toggle htop with Alt-h
        bind "Alt h" {
            MessagePlugin "https://github.com/LoricAndre/zellij-popup/releases/latest/download/zellij-popup.wasm" {
                name "toggle"
                payload "{\"name\":\"htop\",\"command\":\"htop\"}"
            }
        }

        // Toggle lazygit with Alt-g
        bind "Alt g" {
            MessagePlugin "https://github.com/LoricAndre/zellij-popup/releases/latest/download/zellij-popup.wasm" {
                name "toggle"
                payload "{\"name\":\"git\",\"command\":\"lazygit\"}"
            }
        }

        // Toggle a development server with Alt-s
        bind "Alt s" {
            MessagePlugin "https://github.com/LoricAndre/zellij-popup/releases/latest/download/zellij-popup.wasm" {
                name "toggle"
                payload "{\"name\":\"server\",\"command\":\"npm run dev\"}"
            }
        }
    }
}
```

**Note:** If you downloaded the plugin locally, use `file:~/.config/zellij/plugins/zellij-popup.wasm` instead of the URL.

## Examples

### Common Use Cases

1. **System Monitor**: Toggle htop
   ```bash
   zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'
   ```

2. **Git Interface**: Toggle lazygit
   ```bash
   zellij pipe --plugin-configuration --name toggle --payload '{"name":"git","command":"lazygit"}'
   ```

3. **File Manager**: Toggle ranger/lf
   ```bash
   zellij pipe --plugin-configuration --name toggle --payload '{"name":"files","command":"ranger"}'
   ```

4. **Development Server**: Toggle a dev server
   ```bash
   zellij pipe --plugin-configuration --name toggle --payload '{"name":"server","command":"cd ~/project && npm run dev"}'
   ```

## Notes

- Each named pane can only have one instance open at a time
- Commands are run in zsh by default
- Panes are floating by default with 80% width and height
- Closing a pane manually will allow it to be reopened with the same name

## License

MIT
