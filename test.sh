#!/bin/bash
# Test script to verify the config file is valid

echo "Testing Zellij configuration..."
if zellij --config=examples/config-snippet.kdl setup --check 2>&1 | grep -q "Config is valid"; then
    echo "✓ Configuration is valid"
    exit 0
else
    echo "Testing if config parses without errors..."
    # The config should at least parse without syntax errors
    zellij --config=examples/config-snippet.kdl setup --check 2>&1
fi
