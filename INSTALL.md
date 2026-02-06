# Installation Guide

## Quick Install (Recommended)

```bash
# Build and install the plugin
make install

# Install the layout file
make install-config

# Add keybindings to your config
cat examples/config-snippet.kdl >> ~/.config/zellij/config.kdl
```

Then restart Zellij and press `Alt+h` to test!

## Manual Installation

### 1. Build the plugin

```bash
cargo build --release
```

### 2. Copy the plugin file

```bash
mkdir -p ~/.config/zellij/plugins
cp target/wasm32-wasip1/release/zellij_popup.wasm ~/.config/zellij/plugins/zellij-popup.wasm
```

### 3. Add to your layout

Edit your layout file (e.g., `~/.config/zellij/layouts/default.kdl`):

```kdl
layout {
    pane

    // Load the popup plugin
    pane size=1 borderless=true {
        plugin location="file:~/.config/zellij/plugins/zellij-popup.wasm"
    }
}
```

Or copy the example layout:

```bash
cp examples/popup.kdl ~/.config/zellij/layouts/popup.kdl
```

### 4. Add keybindings

Add these to your `~/.config/zellij/config.kdl`:

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

Or merge the snippet:

```bash
cat examples/config-snippet.kdl >> ~/.config/zellij/config.kdl
```

## Testing Without Config Changes

You can test the plugin without modifying your config:

```bash
# Make sure plugin is installed
make install

# Run with test layout
zellij --layout examples/popup.kdl

# In another pane, test the plugin:
zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'
```

## Minimal Standalone Config

For a completely self-contained test:

```bash
make install
zellij --config examples/minimal-config.kdl --layout examples/popup.kdl
```

Then press `Alt+h` to toggle htop!
