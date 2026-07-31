# ⚡ Pure Rust Activity Tracker & Analytics Dashboard

A lightweight, 100% Pure Rust daily application activity monitor and visualization dashboard designed for Linux (Hyprland / Wayland / X11) and Waybar integration.

## ✨ Key Features

- **100% Pure Rust Core:** Zero heavy dependencies or background Python interpreters. Extremely low CPU and memory footprint.
- **Hyprland Active Window Tracking:** Automatically tracks active window class, window title, duration, and hourly distribution.
- **Auto-Filtering Noise:** Automatically filters out transient popups (< 30s) and cleans system desktop IDs into human-readable app names (e.g., `Firefox Browser`, `Satty (Screenshot)`, `Dolphin File Manager`).
- **Percentage & Ratio Visuals:** Renders active percentages (`< 0.1%`, `0.5%`, `28%`) with animated visual ratio progress bars.
- **Embedded Web Server & WebKitGTK GUI:** Built-in HTTP server (`tiny_http`) serving a modern dark-themed dashboard with Doughnut & Bar charts.
- **Markdown Daily Report Generation:** Exports daily activity summaries to Markdown files (`~/.local/share/activity-rust-gui/reports/YYYY-MM-DD.md`) with clean tables and categories.
- **Waybar Shortcut Ready:** Easily trigger the dashboard directly from your Waybar status bar.

## 📁 Repository Structure

```
activity-tracker/
├── Cargo.toml
├── src/
│   └── main.rs
├── index.html
├── chart.umd.js
├── scripts/
│   └── show-activity.sh
└── README.md
```

## 🚀 Building & Installation

### Prerequisites
Ensure you have Rust and the WebKitGTK / GTK3 development libraries installed:

```bash
# CachyOS / Arch Linux
sudo pacman -S rust cargo webkit2gtk-4.1 gtk3
```

### Build Release Binary
```bash
cargo build --release
```
The compiled binary will be at `target/release/activity-gui`.

---

## ⚙️ Systemd Daemon Setup

To run the tracker automatically in the background on startup, create a Systemd user service:

`~/.config/systemd/user/activity-tracker.service`:
```ini
[Unit]
Description=Pure Rust Activity Tracker Daemon
After=graphical-session.target

[Service]
ExecStart=/home/cahya/2026/activity-tracker/target/release/activity-gui --daemon
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
```

Enable and start the service:
```bash
systemctl --user daemon-reload
systemctl --user enable --now activity-tracker.service
```

---

## 📝 Waybar Integration

Add the custom module to your Waybar configuration:

`custom-activity.jsonc`:
```json
{
    "custom/activity": {
        "format": "📝",
        "tooltip": true,
        "tooltip-format": "Klik untuk lihat Rekapan Aktivitas Harian (Activity Tracker)",
        "on-click": "/home/cahya/2026/activity-tracker/scripts/show-activity.sh"
    }
}
```

---

## 📜 License
Licensed under the [MIT License](LICENSE).
