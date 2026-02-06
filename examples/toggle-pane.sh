#!/bin/bash
# Example script showing how to toggle named floating panes

# Toggle htop
toggle_htop() {
    zellij pipe --plugin-configuration --name toggle --payload '{"name":"htop","command":"htop"}'
}

# Toggle lazygit
toggle_git() {
    zellij pipe --plugin-configuration --name toggle --payload '{"name":"git","command":"lazygit"}'
}

# Toggle custom command
toggle_custom() {
    local name="$1"
    local command="$2"
    zellij pipe --plugin-configuration --name toggle --payload "{\"name\":\"$name\",\"command\":\"$command\"}"
}

# Main
case "$1" in
    htop)
        toggle_htop
        ;;
    git)
        toggle_git
        ;;
    *)
        if [ -z "$1" ] || [ -z "$2" ]; then
            echo "Usage: $0 {htop|git|<name> <command>}"
            echo "Examples:"
            echo "  $0 htop"
            echo "  $0 git"
            echo "  $0 myapp 'cd ~/project && npm run dev'"
            exit 1
        fi
        toggle_custom "$1" "$2"
        ;;
esac
