use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct AppData {
    total_seconds: u64,
    titles: BTreeMap<String, u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct DailyLog {
    date: String,
    total_seconds: u64,
    apps: BTreeMap<String, AppData>,
    hourly: BTreeMap<String, u64>,
}

#[derive(Deserialize, Debug)]
struct HyprWindow {
    class: Option<String>,
    title: Option<String>,
}

fn get_base_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".local/share/activity-tracker")
}

fn get_logs_dir() -> PathBuf {
    let dir = get_base_dir().join("logs");
    fs::create_dir_all(&dir).ok();
    dir
}

fn get_md_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let dir = PathBuf::from(home).join("Documents/Activity-Logs");
    fs::create_dir_all(&dir).ok();
    dir
}

fn get_uid() -> String {
    if let Ok(output) = Command::new("id").arg("-u").output() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return s;
        }
    }
    "1000".to_string()
}

fn ensure_hyprland_env() {
    if std::env::var("XDG_RUNTIME_DIR").is_err() {
        let uid = get_uid();
        std::env::set_var("XDG_RUNTIME_DIR", format!("/run/user/{}", uid));
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        let hypr_dir = PathBuf::from(&runtime_dir).join("hypr");
        let hypr_dir = if hypr_dir.exists() {
            hypr_dir
        } else {
            PathBuf::from("/tmp/hypr")
        };
        if hypr_dir.exists() {
            if let Ok(entries) = fs::read_dir(hypr_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.contains('_') {
                            std::env::set_var("HYPRLAND_INSTANCE_SIGNATURE", &name);
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn get_active_window() -> (String, String) {
    ensure_hyprland_env();
    if let Ok(output) = Command::new("hyprctl").args(["activewindow", "-j"]).output() {
        if output.status.success() {
            if let Ok(window) = serde_json::from_slice::<HyprWindow>(&output.stdout) {
                if let Some(cls) = window.class {
                    if !cls.trim().is_empty() {
                        let title = window.title.unwrap_or_else(|| cls.clone());
                        let clean_title = if title.len() > 120 {
                            format!("{}...", &title[..117])
                        } else {
                            title
                        };
                        return (cls.trim().to_string(), clean_title.trim().to_string());
                    }
                }
            }
        }
    }
    ("Desktop / Idle".to_string(), "Desktop".to_string())
}

fn load_daily_log(date_str: &str) -> DailyLog {
    let path = get_logs_dir().join(format!("{}.json", date_str));
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(log) = serde_json::from_str::<DailyLog>(&content) {
                return log;
            }
        }
    }
    DailyLog {
        date: date_str.to_string(),
        total_seconds: 0,
        apps: BTreeMap::new(),
        hourly: BTreeMap::new(),
    }
}

fn save_daily_log(path: &PathBuf, log: &DailyLog) {
    let tmp_path = path.with_extension("tmp");
    if let Ok(json) = serde_json::to_string_pretty(log) {
        if fs::write(&tmp_path, json).is_ok() {
            fs::rename(tmp_path, path).ok();
        }
    }
}

fn run_daemon() {
    println!("Pure Rust Activity Tracker Daemon running...");
    let interval_secs = 5u64;
    loop {
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let hour_str = now.format("%H").to_string();
        let log_path = get_logs_dir().join(format!("{}.json", date_str));

        let (app_cls, title) = get_active_window();

        let mut log = load_daily_log(&date_str);
        log.total_seconds += interval_secs;

        *log.hourly.entry(hour_str).or_insert(0) += interval_secs;

        let app_entry = log.apps.entry(app_cls).or_default();
        app_entry.total_seconds += interval_secs;
        *app_entry.titles.entry(title).or_insert(0) += interval_secs;

        save_daily_log(&log_path, &log);
        thread::sleep(Duration::from_secs(interval_secs));
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{}d", seconds);
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{}m {}d", minutes, seconds % 60);
    }
    let hours = minutes / 60;
    let rem_min = minutes % 60;
    format!("{}j {}m", hours, rem_min)
}

fn make_progress_bar(pct: f64, len: usize) -> String {
    let filled = ((len as f64) * pct / 100.0).round() as usize;
    let filled = filled.min(len);
    format!("{}{}", "█".repeat(filled), "░".repeat(len - filled))
}

fn categorize_app(app_name: &str) -> &'static str {
    let name = app_name.to_lowercase();
    if ["steam", "gamescope", "lutris", "heroic", "dota", "cs2", "csgo", "minecraft", "roblox", "proton", "wine", "emulator", "ryujinx", "yuzu", "retroarch", "game"].iter().any(|k| name.contains(k)) {
        "🎮 Gaming"
    } else if ["code", "nvim", "neovim", "sublime", "antigravity", "idea", "studio"].iter().any(|k| name.contains(k)) {
        "💻 Coding & Development"
    } else if ["kitty", "foot", "alacritty", "wezterm", "terminal", "bash", "zsh"].iter().any(|k| name.contains(k)) {
        "⚡ Terminal & CLI"
    } else if ["chrome", "chromium", "firefox", "brave", "zen", "vivaldi", "browser"].iter().any(|k| name.contains(k)) {
        "🌐 Browser & Web"
    } else if ["spotify", "vlc", "mpv", "gimp", "inkscape", "obs", "discord", "telegram"].iter().any(|k| name.contains(k)) {
        "🎨 Media & Productivity"
    } else if name.contains("desktop") || name.contains("idle") {
        "💤 Idle / Rest"
    } else {
        "📦 Aplikasi Lainnya"
    }
}

fn generate_markdown(date_str: &str) -> PathBuf {
    let log = load_daily_log(date_str);
    let md_path = get_md_dir().join(format!("{}.md", date_str));

    let mut lines = Vec::new();
    lines.push(format!("# 📅 Rekapan Aktivitas PC ({})\n", date_str));
    lines.push(format!("> **Tanggal Laporan:** `{}`  ", date_str));
    lines.push(format!("> **Total Waktu Aktif:** `{}`  ", format_duration(log.total_seconds)));

    let mut sorted_apps: Vec<(&String, &AppData)> = log.apps.iter().collect();
    sorted_apps.sort_by(|a, b| b.1.total_seconds.cmp(&a.1.total_seconds));

    let most_used = if let Some(first) = sorted_apps.first() {
        format!("`{}` (`{}`)", first.0, format_duration(first.1.total_seconds))
    } else {
        "`Tidak Ada`".to_string()
    };
    lines.push(format!("> **Aplikasi Utama:** {}\n", most_used));

    lines.push("---\n## 📊 Ringkasan Kategori Aktivitas\n".to_string());
    lines.push("| Kategori | Durasi | Persentase | Visual Ratio |".to_string());
    lines.push("| :--- | :---: | :---: | :--- |".to_string());

    let mut cat_map: BTreeMap<&'static str, u64> = BTreeMap::new();
    for (app, data) in &log.apps {
        let cat = categorize_app(app);
        *cat_map.entry(cat).or_default() += data.total_seconds;
    }

    let mut sorted_cats: Vec<(&'static str, u64)> = cat_map.into_iter().collect();
    sorted_cats.sort_by(|a, b| b.1.cmp(&a.1));

    let tot_sec = log.total_seconds.max(1) as f64;
    for (cat, sec) in sorted_cats {
        let pct = (sec as f64 / tot_sec) * 100.0;
        let pbar = make_progress_bar(pct, 15);
        lines.push(format!("| **{}** | `{}` | `{:.1}%` | `{}` |", cat, format_duration(sec), pct, pbar));
    }

    lines.push("\n---\n## 🏆 Peringkat Penggunaan Aplikasi\n".to_string());
    lines.push("| No | Aplikasi | Durasi | Proporsi | Status |".to_string());
    lines.push("| :-: | :--- | :---: | :---: | :--- |".to_string());

    for (idx, (app, data)) in sorted_apps.iter().enumerate() {
        let pct = (data.total_seconds as f64 / tot_sec) * 100.0;
        let pbar = make_progress_bar(pct, 10);
        lines.push(format!("| {} | **{}** | `{}` | `{:.1}%` | `{}` |", idx + 1, app, format_duration(data.total_seconds), pct, pbar));
    }

    lines.push("\n---\n## 📑 Rincian Judul & Aktivitas Spesifik\n".to_string());
    for (app, data) in &sorted_apps {
        lines.push(format!("### 🔹 {} (`{}`)\n", app, format_duration(data.total_seconds)));
        lines.push("| Aktivitas / Window Title | Durasi |".to_string());
        lines.push("| :--- | :---: |".to_string());

        let mut sorted_titles: Vec<(&String, &u64)> = data.titles.iter().collect();
        sorted_titles.sort_by(|a, b| b.1.cmp(a.1));

        for (title, t_sec) in sorted_titles.into_iter().take(15) {
            let clean_title = title.replace('|', "\\|").trim().to_string();
            lines.push(format!("| `{}` | `{}` |", clean_title, format_duration(*t_sec)));
        }
        lines.push("".to_string());
    }

    lines.push("\n---\n*Laporan dibuat secara otomatis oleh Pure Rust Activity Tracker.*".to_string());

    fs::write(&md_path, lines.join("\n")).ok();
    println!("✅ Laporan Markdown dibuat di: {:?}", md_path);
    md_path
}

fn start_embedded_http_server() {
    thread::spawn(|| {
        let server = match tiny_http::Server::http("127.0.0.1:9877") {
            Ok(s) => s,
            Err(_) => return,
        };

        for request in server.incoming_requests() {
            let url = request.url().to_string();
            if url.starts_with("/api/activity") {
                let date_str = if let Some(idx) = url.find("date=") {
                    url[idx + 5..].split('&').next().unwrap_or("")
                } else {
                    ""
                };
                let target_date = if date_str.is_empty() {
                    Local::now().format("%Y-%m-%d").to_string()
                } else {
                    date_str.to_string()
                };

                let log = load_daily_log(&target_date);
                let json = serde_json::to_string(&log).unwrap_or_else(|_| "{}".to_string());

                let response = tiny_http::Response::from_string(json)
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .with_header("Cache-Control: no-cache, no-store, must-revalidate".parse::<tiny_http::Header>().unwrap())
                    .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap());
                let _ = request.respond(response);
            } else if url.starts_with("/api/open_md") {
                let date_str = if let Some(idx) = url.find("date=") {
                    url[idx + 5..].split('&').next().unwrap_or("")
                } else {
                    ""
                };
                let target_date = if date_str.is_empty() {
                    Local::now().format("%Y-%m-%d").to_string()
                } else {
                    date_str.to_string()
                };

                let md_path = generate_markdown(&target_date);
                Command::new("xdg-open").arg(md_path).spawn().ok();

                let response = tiny_http::Response::from_string("{\"status\":\"ok\"}")
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .with_header("Cache-Control: no-cache, no-store, must-revalidate".parse::<tiny_http::Header>().unwrap());
                let _ = request.respond(response);
            } else if url == "/chart.umd.js" {
                let path = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/activity-rust-gui/chart.umd.js");
                if let Ok(bytes) = fs::read(path) {
                    let response = tiny_http::Response::from_data(bytes)
                        .with_header("Content-Type: text/javascript".parse::<tiny_http::Header>().unwrap());
                    let _ = request.respond(response);
                } else {
                    let _ = request.respond(tiny_http::Response::from_string("404").with_status_code(404));
                }
            } else {
                let path = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/activity-rust-gui/index.html");
                if let Ok(bytes) = fs::read(path) {
                    let response = tiny_http::Response::from_data(bytes)
                        .with_header("Content-Type: text/html; charset=utf-8".parse::<tiny_http::Header>().unwrap())
                        .with_header("Cache-Control: no-cache, no-store, must-revalidate".parse::<tiny_http::Header>().unwrap());
                    let _ = request.respond(response);
                } else {
                    let _ = request.respond(tiny_http::Response::from_string("404").with_status_code(404));
                }
            }
        }
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("GDK_BACKEND", "x11");

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--daemon" {
        start_embedded_http_server();
        run_daemon();
        return Ok(());
    }

    if args.len() > 1 && args[1] == "--generate-md" {
        let date_str = args.get(2).cloned().unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
        generate_markdown(&date_str);
        return Ok(());
    }

    // Start embedded Rust HTTP server
    start_embedded_http_server();
    thread::sleep(Duration::from_millis(150));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Activity Tracker Analytics")
        .with_inner_size(tao::dpi::LogicalSize::new(980.0, 700.0))
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new()
        .with_url("http://127.0.0.1:9877/index.html")
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
