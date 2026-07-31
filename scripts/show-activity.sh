#!/usr/bin/env bash

# Ensure 100% Pure Rust Activity Tracker Daemon is active
if ! pgrep -f "activity-gui --daemon" > /dev/null; then
    nohup ~/.local/share/activity-rust-gui/target/release/activity-gui --daemon > /dev/null 2>&1 &
    sleep 0.3
fi

# Launch App Dashboard Window with timestamp to force fresh fetch and eliminate cached 0% items
TS=$(date +%s)
chromium --app="http://127.0.0.1:9877/index.html?t=${TS}" --user-data-dir="/tmp/activity-dashboard-chrome-profile" > /dev/null 2>&1 &
