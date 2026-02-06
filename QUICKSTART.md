# Quick Start Guide

## 1. Install Rust and WASM target

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-wasip1
```

## 2. Build and Install

```bash
# Build the plugin
make build

# Install to Zellij plugins directory
make install
```

## 3. Configure Zellij

Use the complete example config:

```bash
# Test it first
zellij --config examples/complete-config.kdl

# If you like it, use it as your main config
cp examples/complete-config.kdl ~/.config/zellij/config.kdl
```

Or manually add to your existing config by merging `examples/config-snippet.kdl` and `examples/layout.kdl`

## 4. Restart Zellij

```bash
# Kill existing sessions
zellij kill-all-sessions

# Start new session
zellij
```

## 5. Test It Out

### Using Keybindings (if you added the config snippet)

1. Press `Alt+h` to toggle htop
2. Press `Alt+h` again to close it
3. Press `Alt+g` to toggle lazygit

### Using Command Line

```bash
# Toggle htop in a floating pane
zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'

# Toggle it again to close
zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'
```

### Using the Example Script

```bash
# Make it available in your PATH (optional)
sudo cp examples/toggle-pane.sh /usr/local/bin/zj-toggle

# Use it
zj-toggle htop
zj-toggle git
zj-toggle myserver "cd ~/project && npm start"
```

## Common Commands

```bash
# Toggle htop
zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'

# Toggle lazygit
zellij pipe --plugin-configuration --name toggle --payload '{"name":"git","command":"lazygit"}'

# Toggle ranger (file manager)
zellij pipe --plugin-configuration --name toggle --payload '{"name":"files","command":"ranger"}'

# Open a development server
zellij pipe --plugin-configuration --name toggle --payload '{"name":"server","command":"cd ~/myproject && npm run dev"}'
```

## Troubleshooting

### Plugin not loading

1. Check that the plugin file exists:
   ```bash
   ls -lh ~/.config/zellij/plugins/zellij-popup.wasm
   ```

2. Check Zellij logs:
   ```bash
   tail -f /tmp/zellij-*/zellij-log/*
   ```

### Permission errors

Make sure the plugin has the required permissions in your config. The plugin requests:
- ReadApplicationState
- ChangeApplicationState
- RunCommands
- OpenFiles

### Commands not running in zsh

The plugin runs commands using: `zsh -c '<your-command>'`

Make sure zsh is installed:
```bash
which zsh
```

## Next Steps

- Customize the keybindings in `~/.config/zellij/config.kdl`
- Add your own named panes with custom commands
- Create shell aliases for frequently used toggles
