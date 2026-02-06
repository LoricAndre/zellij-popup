# Usage Guide

## Quick Start

1. **Build and install**:
   ```bash
   make install
   ```

2. **Try it out** (in a terminal inside Zellij):
   ```bash
   zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'
   ```

3. **Set up keybindings**:
   ```bash
   zellij --config examples/complete-config.kdl
   ```

   Then press `Alt+h` to toggle htop!

## Three Ways to Use This Plugin

### 1. Command Line (zellij pipe)

Works immediately after installing the plugin - no config needed.

```bash
# Toggle a pane
zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'

# Open a pane (won't close if already open)
zellij pipe --plugin-configuration --name open --payload '{"name":"logs","command":"tail -f /var/log/syslog"}'

# Close a pane
zellij pipe --plugin-configuration --name close --payload '{"name":"htop"}'
```

### 2. Shell Script

Use the included helper script:

```bash
# Make it available in your PATH
sudo cp examples/toggle-pane.sh /usr/local/bin/zj-popup
chmod +x /usr/local/bin/zj-popup

# Use it
zj-popup htop
zj-popup git
zj-popup myserver "cd ~/project && npm start"
```

Or create shell aliases:

```bash
# Add to ~/.zshrc or ~/.bashrc
alias zh='zellij pipe --plugin-configuration --name toggle --payload '"'"'{"name":"htop","command":"htop"}'"'"''
alias zg='zellij pipe --plugin-configuration --name toggle --payload '"'"'{"name":"git","command":"lazygit"}'"'"''
```

### 3. Keybindings (Recommended)

Use the complete config for the best experience:

```bash
cp examples/complete-config.kdl ~/.config/zellij/config.kdl
zellij kill-all-sessions
zellij
```

Keybindings:
- `Alt+h` - Toggle htop
- `Alt+g` - Toggle lazygit
- `Alt+t` - Toggle terminal (zsh)
- `Alt+s` - Toggle development server
- `Alt+l` - Toggle logs

## Customizing Commands

Edit `examples/complete-config.kdl` and change the payload:

```kdl
bind "Alt m" {
    MessagePlugin "popup" {
        name "toggle"
        payload "{\"name\":\"monitor\",\"command\":\"docker stats\"}"
    }
}
```

## How It Works

1. **Named panes**: Each pane has a unique name (e.g., "htop", "git")
2. **Toggle behavior**:
   - First call: Opens the pane
   - Second call: Closes the pane
3. **Floating panes**: Opens as 80% width/height, positioned at 10% from top-left
4. **Interactive zsh**: Commands run in `zsh -c "command"`

## Tips

- Use short command names for the "name" field
- Commands can be complex: `"cd ~/project && npm run dev"`
- Each named pane maintains its own state
- Closing the pane manually allows it to be reopened

## Troubleshooting

**Plugin not found**:
```bash
ls -lh ~/.config/zellij/plugins/zellij-popup.wasm
# If missing, run: make install
```

**Permission errors**:
The plugin requests these permissions automatically:
- ReadApplicationState
- ChangeApplicationState
- RunCommands
- OpenFiles

**Keybindings not working**:
1. Make sure the plugin is loaded in your layout
2. Check that you're not in "locked" mode (press `Ctrl+g` to exit)
3. Restart Zellij: `zellij kill-all-sessions && zellij`

**Command not running**:
- Verify zsh is installed: `which zsh`
- Test the command directly: `zsh -c "your-command"`
- Check Zellij logs: `tail -f /tmp/zellij-*/zellij-log/*`

## Examples

### System Monitoring
```bash
zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'
zellij pipe --plugin-configuration --name toggle --payload '{"name":"btop","command":"btop"}'
```

### Git Tools
```bash
zellij pipe --plugin-configuration --name toggle --payload '{"name":"lazygit","command":"lazygit"}'
zellij pipe --plugin-configuration --name toggle --payload '{"name":"tig","command":"tig"}'
```

### File Managers
```bash
zellij pipe --plugin-configuration --name toggle --payload '{"name":"ranger","command":"ranger"}'
zellij pipe --plugin-configuration --name toggle --payload '{"name":"lf","command":"lf"}'
```

### Development Servers
```bash
zellij pipe --plugin-configuration --name toggle --payload '{"name":"server","command":"cd ~/myapp && npm run dev"}'
zellij pipe --plugin-configuration --name toggle --payload '{"name":"rails","command":"cd ~/myapp && rails server"}'
```

### Logs and Monitoring
```bash
zellij pipe --plugin-configuration --name toggle --payload '{"name":"syslog","command":"tail -f /var/log/syslog"}'
zellij pipe --plugin-configuration --name toggle --payload '{"name":"docker","command":"docker logs -f mycontainer"}'
```
