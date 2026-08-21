#!/usr/bin/env bash

# Ensure 100% Pure Rust Activity Tracker Daemon is active
if ! pgrep -f "activity-gui --daemon" > /dev/null; then
    nohup ~/.local/share/activity-rust-gui/target/release/activity-gui --daemon > /dev/null 2>&1 &
    sleep 0.3
fi

PROFILE_DIR="/tmp/activity-dashboard-chrome-profile"
mkdir -p "$PROFILE_DIR"
touch "$PROFILE_DIR/First Run"

# Launch App Dashboard Window with flags to skip ToS/Fre and enable floating window
TS=$(date +%s)
chromium \
    --app="http://127.0.0.1:9877/index.html?t=${TS}" \
    --class="ActivityDashboard" \
    --user-data-dir="$PROFILE_DIR" \
    --no-first-run \
    --no-default-browser-check \
    --disable-session-crashed-bubble \
    --disable-infobars \
    --password-store=basic \
    --disable-features=Translate,OptimizationHints \
    > /dev/null 2>&1 &
