//! capture-bypass GUI — Rust/egui frontend
//!
//! Requires Administrator privileges (enforced by embedded UAC manifest via build.rs).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod theme;
mod i18n;

use i18n::Language;

use eframe::egui::{self, Color32, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Condvar, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use winreg::{enums::*, RegKey};

// Windows API
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, TRUE},
    Graphics::Gdi::{
        GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    },
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        Threading::{
            IsWow64Process, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
    UI::{
        Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON},
        Input::KeyboardAndMouse::{
            HOT_KEY_MODIFIERS, RegisterHotKey, UnregisterHotKey,
        },
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, EnumWindows, GetIconInfo, GetMessageW,
            GetWindowDisplayAffinity, GetWindowTextW, GetWindowThreadProcessId,
            ICONINFO, IsWindowVisible, MSG, PeekMessageW, PM_REMOVE,
            TranslateMessage, WM_HOTKEY,
        },
    },
};

// Constants

const WDA_NONE: u32 = 0x00000000;
const WDA_MONITOR: u32 = 0x00000001;
const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;

// SetWinEventHook flags — not exported as named constants in windows-rs 0.58
const WINEVENT_OUTOFCONTEXT:   u32 = 0x0000;
const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;

const BROWSER_NAMES: &[&str] = &[
    "chrome.exe",
    "msedge.exe",
    "firefox.exe",
    "brave.exe",
    "opera.exe",
    "vivaldi.exe",
    "thorium.exe",
];

// ── WinEvent hook — instant window detection ───────────────────────────────
//
// A background thread installs a system-wide WINEVENT_OUTOFCONTEXT hook that
// fires whenever any window becomes visible (EVENT_OBJECT_SHOW).  The callback
// signals a global Condvar so both the display scan thread and the auto-inject
// thread wake up immediately instead of waiting for their next poll interval.
//
// Without this, detection latency is 0–500 ms (or 0–800 ms for auto-inject).
// With this, it drops to roughly 0–50 ms worst case.

/// Shared waker: the WinEvent callback notifies this; scan threads wait on it.
/// Initialised lazily on first use; valid for the entire process lifetime.
static SCAN_WAKER: OnceLock<(Mutex<()>, Condvar)> = OnceLock::new();

/// Returns (or creates) the global scan waker.
#[inline]
fn scan_waker() -> &'static (Mutex<()>, Condvar) {
    SCAN_WAKER.get_or_init(|| (Mutex::new(()), Condvar::new()))
}

/// WinEvent callback — fires in the hook thread's message pump whenever a
/// window-level SHOW event occurs in any process (WINEVENT_OUTOFCONTEXT).
unsafe extern "system" fn winevent_proc(
    _hook:         HWINEVENTHOOK,
    _event:        u32,
    hwnd:          HWND,
    id_object:     i32,
    _id_child:     i32,
    _event_thread: u32,
    _event_time:   u32,
) {
    // id_object == 0 is OBJID_WINDOW — the window itself, not a child control.
    // hwnd must be non-null.  Filtering here avoids waking scan threads for
    // tooltip, menu, and other noise events.
    if hwnd.0.is_null() || id_object != 0 { return; }

    if let Some((lock, cvar)) = SCAN_WAKER.get() {
        // We only need to signal — the content of the mutex is unused.
        let _guard = lock.lock().unwrap();
        cvar.notify_all();
    }
}

// Per-process rule types

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
enum ProcessRuleMode {
    #[default]
    AlwaysOneShot,
    AlwaysPersistent,
    Skip,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProcessRule {
    process_name: String,
    #[serde(default)]
    mode: ProcessRuleMode,
}

// Persistent config

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    persistent_mode: bool,
    #[serde(default)]
    auto_inject: bool,
    #[serde(default)]
    protected_only: bool,
    #[serde(default)]
    toast_enabled: bool,
    #[serde(default)]
    hotkey_enabled: bool,
    #[serde(default)]
    show_log: bool,
    #[serde(default)]
    watch_names: Vec<String>,
    #[serde(default = "default_sort_col")]
    sort_col: u8, // 0=pid,1=process,2=title,3=status
    #[serde(default)]
    sort_asc: bool,
    #[serde(default = "default_minimize_to_tray")]
    minimize_to_tray: bool,
    #[serde(default)]
    logging_enabled: bool,
    #[serde(default)]
    discord_rpc_enabled: bool,
    // Hotkey virtual-key code (Windows VK_* value). Default = 'B' (0x42).
    #[serde(default = "default_hotkey_key")]
    hotkey_key: u32,
    // Hotkey modifier mask (MOD_CONTROL=0x2, MOD_SHIFT=0x4, MOD_ALT=0x1).
    // Default = Ctrl+Shift (0x6).
    #[serde(default = "default_hotkey_mods")]
    hotkey_mods: u32,
    #[serde(default)]
    silent_startup: bool,
    #[serde(default)]
    fast_scan: bool,
    #[serde(default)]
    lifetime_strips: u64,
    #[serde(default)]
    strip_on_launch: bool,
    #[serde(default)]
    process_rules: Vec<ProcessRule>,
    #[serde(default)]
    exclusions: Vec<String>,
    #[serde(default)]
    language: Language,
}

fn default_sort_col() -> u8 { 0 }
fn default_minimize_to_tray() -> bool { true }
fn default_hotkey_key()  -> u32 { b'B' as u32 }
fn default_hotkey_mods() -> u32 { 0x02 | 0x04 } // MOD_CONTROL | MOD_SHIFT

impl Default for Config {
    fn default() -> Self {
        Config {
            persistent_mode: false,
            auto_inject: false,
            protected_only: false,
            toast_enabled: false,
            hotkey_enabled: false,
            show_log: false,
            watch_names: Vec::new(),
            sort_col: 0,
            sort_asc: true,
            minimize_to_tray: true,
            logging_enabled: false,
            discord_rpc_enabled: false,
            hotkey_key: default_hotkey_key(),
            hotkey_mods: default_hotkey_mods(),
            silent_startup: false,
            fast_scan: false,
            lifetime_strips: 0,
            strip_on_launch: false,
            process_rules: Vec::new(),
            exclusions: Vec::new(),
            language: Language::English,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs_next::config_dir().map(|d| d.join("capture-bypass").join("config.toml"))
}

fn load_config() -> Config {
    let path = match config_path() {
        Some(p) => p,
        None => return Config::default(),
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Config::default(),
    };
    toml::from_str(&text).unwrap_or_default()
}

fn load_config_from_path(path: &std::path::Path) -> Config {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Config::default(),
    };
    toml::from_str(&text).unwrap_or_default()
}

fn save_config(cfg: &Config) {
    let path = match config_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string_pretty(cfg) {
        let _ = std::fs::write(path, text);
    }
}

// Help content
//
// Structure: &[( tab_label, &[( sub_heading, body_text )] )]
// render_help_window() turns each sub_heading into a bold label + separator,
// so no raw ━━━ dividers needed.

fn help_sections(lang: Language) -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
    let h = i18n::help_heading(lang);
    let b = i18n::help_body(lang);
    vec![
        (i18n::help_tab(lang).overview, vec![
            (h.what_is, b.what_is),
            (h.how_does_it_work, b.how_does_it_work),
            (h.legal_notice, b.legal_notice),
        ]),
        (i18n::help_tab(lang).requirements_build, vec![
            (h.requirements, b.requirements),
            (h.build_x64, b.build_x64),
            (h.build_x86, b.build_x86),
        ]),
        (i18n::help_tab(lang).usage_guide, vec![
            (h.window_list, b.window_list),
            (h.header_buttons, b.header_buttons),
            (h.filter_bar, b.filter_bar),
            (h.status_bar, b.status_bar),
        ]),
        (i18n::help_tab(lang).settings, vec![
            (h.opening_settings, b.opening_settings),
            (h.silent_startup, b.silent_startup),
            (h.strip_on_launch, b.strip_on_launch),
            (h.fast_scan, b.fast_scan),
            (h.desktop_notifications, b.desktop_notifications),
            (h.global_hotkey, b.global_hotkey),
            (h.discord_rich_presence, b.discord_rich_presence),
            (h.export_import, b.export_import),
            (h.injection_log_file, b.injection_log_file),
            (h.windows_defender, b.windows_defender),
        ]),
        (i18n::help_tab(lang).per_process_rules, vec![
            (h.per_process_rules, b.per_process_rules),
            (h.exclusion_list, b.exclusion_list),
        ]),
        (i18n::help_tab(lang).injection_modes, vec![
            (h.one_shot_mode, b.one_shot_mode),
            (h.persistent_mode, b.persistent_mode),
            (h.re_injection, b.re_injection),
        ]),
        (i18n::help_tab(lang).browser_injection, vec![
            (h.why_browsers, b.why_browsers),
            (h.what_cb_does, b.what_cb_does),
            (h.tip, b.tip),
        ]),
        (i18n::help_tab(lang).system_tray, vec![
            (h.system_tray, b.system_tray),
            (h.auto_inject, b.auto_inject),
        ]),
        (i18n::help_tab(lang).auto_update, vec![
            (h.how_updates_work, b.how_updates_work),
            (h.update_wrong, b.update_wrong),
        ]),
        (i18n::help_tab(lang).troubleshooting, vec![
            (h.dlls_not_found, b.dlls_not_found),
            (h.strip_failed, b.strip_failed),
            (h.injection_ok_but_black, b.injection_ok_but_black),
            (h.notifications_missing, b.notifications_missing),
            (h.antivirus_flags, b.antivirus_flags),
            (h.x86_fails, b.x86_fails),
            (h.config_location, b.config_location),
        ]),
    ]
}

// Data model

#[derive(Clone, Debug)]
struct WindowEntry {
    pid: u32,
    process_name: String,
    title: String,
    affinity: u32,
    is_protected: bool,
    is_32bit: bool,
}

// Injection result message

struct InjResult {
    msg: String,
    ok: bool,
}

// Injection log

struct LogEntry {
    time: Instant,
    msg: String,
    ok: bool,
}

// Column sort

#[derive(Clone, Copy, PartialEq)]
enum SortCol { Pid, Process, Title, Status }

impl SortCol {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => SortCol::Process,
            2 => SortCol::Title,
            3 => SortCol::Status,
            _ => SortCol::Pid,
        }
    }
    fn to_u8(self) -> u8 {
        match self {
            SortCol::Pid => 0,
            SortCol::Process => 1,
            SortCol::Title => 2,
            SortCol::Status => 3,
        }
    }
}

// Update check state

enum UpdateState {
    Checking,
    Available(String), // newer tag name
    UpToDate,
    Failed,
}

// Auto-update download state (shown in the header)
enum DownloadState {
    Idle,
    Downloading(f32),               // 0.0 – 1.0
    Verifying,
    Ready(std::path::PathBuf),      // verified, waiting for user to click Restart
    Failed(String),
}

// Messages from the download thread back to the UI thread
enum DownloadMsg {
    Progress(f32),
    Verifying,
    Done(std::path::PathBuf),
    Failed(String),
}

// Discord RPC -- sent from the main thread to the RPC background thread
// whenever something worth showing changes.
struct RpcState {
    auto_inject: bool,
    persistent:  bool,
    strip_count: u32,
}

const DISCORD_APP_ID: &str = "1506230832283648000";

// App

struct App {
    // Shared window list (background refresh thread writes, UI reads)
    shared_windows: Arc<Mutex<Vec<WindowEntry>>>,

    // DLL base dir (next to the exe)
    exe_dir: PathBuf,

    // Injection mode
    persistent_mode: bool,

    // Auto-inject
    auto_inject_enabled: bool,
    auto_inject_running: Arc<AtomicBool>,
    auto_inject_seen: Arc<Mutex<HashSet<u32>>>,

    // UI state
    filter: String,
    protected_only: bool,
    show_help: bool,
    help_section: usize,
    show_settings: bool,

    // Status bar
    status_msg: String,
    status_color: Color32,
    status_time: Option<Instant>,

    // Launch at Windows startup
    startup_enabled: bool,

    // One-shot re-injection tracking
    one_shot_stripped: HashMap<u32, String>,
    reapply_alert: Vec<(u32, String)>,

    // Injection log
    log_entries: Vec<LogEntry>,
    show_log: bool,

    // Column sorting
    sort_col: SortCol,
    sort_asc: bool,

    // Watch mode — inject automatically whenever a process with a watched name appears
    watch_names: Vec<String>,
    watch_input: String,
    watch_seen: HashSet<u32>, // PIDs already handled by watch mode this session

    // Global hotkey
    hotkey_enabled: bool,
    hotkey_id: i32,
    hotkey_key: u32,    // Windows VK_* code for the trigger key
    hotkey_mods: u32,   // MOD_CONTROL | MOD_SHIFT | MOD_ALT bitmask
    hotkey_recording: bool, // true while waiting for user to press a new combo

    // Auto-update
    update_state: Option<UpdateState>,
    update_rx: Option<Receiver<UpdateState>>,
    // Download-and-install flow
    download_state: DownloadState,
    download_rx: Option<Receiver<DownloadMsg>>,
    // true while the "Update to vX.X.X?" confirmation dialog is shown
    show_update_confirm: bool,
    // true once the user has clicked "Update now" — download completion triggers
    // automatic restart without requiring a second button click
    update_confirmed: bool,

    // Toast notifications (auto-inject strips show a desktop notification)
    toast_enabled: bool,
    // Notification grouping
    toast_pending: Vec<String>,
    toast_batch_deadline: Option<Instant>,

    // Process icon cache: process_name → egui TextureHandle
    icon_cache: HashMap<String, Option<egui::TextureHandle>>,

    // Tray (must stay alive for the duration of the app)
    tray_icon: Option<tray_icon::TrayIcon>,
    tray_open_id: Option<tray_icon::menu::MenuId>,
    tray_quit_id: Option<tray_icon::menu::MenuId>,
    // When true, X hides to tray instead of closing the process.
    minimize_to_tray: bool,
    // When true, each injection is appended to %APPDATA%\capture-bypass\injection.log
    logging_enabled: bool,

    // Discord Rich Presence
    discord_rpc_enabled: bool,
    discord_rpc_running: Arc<AtomicBool>,
    discord_rpc_tx: Option<std::sync::mpsc::SyncSender<RpcState>>,
    // How many successful strips this session (shown in Discord status)
    session_strip_count: Arc<std::sync::atomic::AtomicU32>,

    // Set by the off-thread tray watcher when the user clicks Open.
    tray_show: Arc<AtomicBool>,

    // Channel: background injection threads → UI
    inject_tx: Sender<InjResult>,
    inject_rx: Receiver<InjResult>,

    // Tray state tracking
    tray_was_protected: bool,

    // Silent startup
    silent_startup: bool,

    // Fast scan
    fast_scan: bool,
    fast_scan_arc: Arc<AtomicBool>,

    // Lifetime stats
    lifetime_strips: u64,

    // Strip on launch
    strip_on_launch: bool,
    strip_on_launch_pending: bool,

    // Per-process rules
    process_rules: Vec<ProcessRule>,
    rule_input: String,
    rule_mode_input: ProcessRuleMode,
    auto_inject_rules: Arc<Mutex<Vec<ProcessRule>>>,

    // Exclusion list
    exclusions: Vec<String>,
    exclusion_input: String,
    auto_inject_exclusions: Arc<Mutex<Vec<String>>>,

    // Language
    language: Language,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply the custom visual theme (fonts, colors, spacing, rounding).
        theme::install(&cc.egui_ctx);

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));

        let cfg = load_config();

        let shared_windows: Arc<Mutex<Vec<WindowEntry>>> = Arc::new(Mutex::new(Vec::new()));

        let fast_scan_arc_local: Arc<AtomicBool> = Arc::new(AtomicBool::new(cfg.fast_scan));

        // Kick off background refresh thread.
        // Normal cadence: 500 ms (or 100 ms in fast-scan mode).
        // When the WinEvent hook fires (new window appears), the Condvar is
        // notified and the thread wakes immediately instead of waiting out
        // the full interval — reducing detection latency to ~0 ms.
        {
            let shared = Arc::clone(&shared_windows);
            let ctx = cc.egui_ctx.clone();
            let fast_scan_flag = Arc::clone(&fast_scan_arc_local);
            std::thread::Builder::new()
                .name("window-scan".into())
                .spawn(move || loop {
                    let windows = enumerate_windows();
                    *shared.lock().unwrap() = windows;
                    ctx.request_repaint();
                    let sleep_ms = if fast_scan_flag.load(Ordering::Relaxed) { 100 } else { 500 };
                    let (lock, cvar) = scan_waker();
                    let guard = lock.lock().unwrap();
                    let _ = cvar.wait_timeout(guard, Duration::from_millis(sleep_ms));
                })
                .ok();
        }

        let (inject_tx, inject_rx) = mpsc::channel();

        // Kick off auto-update check in background
        let (update_tx, update_rx) = mpsc::channel::<UpdateState>();
        std::thread::spawn(move || {
            let _ = update_tx.send(UpdateState::Checking);
            match check_for_update() {
                Ok(Some(tag)) => { let _ = update_tx.send(UpdateState::Available(tag)); }
                Ok(None)      => { let _ = update_tx.send(UpdateState::UpToDate); }
                Err(_)        => { let _ = update_tx.send(UpdateState::Failed); }
            }
        });

        // Register global hotkey if saved as enabled
        let hotkey_id = 1_i32;
        if cfg.hotkey_enabled {
            register_hotkey(hotkey_id, cfg.hotkey_mods, cfg.hotkey_key);
        }

        // Set up system tray
        let (tray_icon, tray_open_id, tray_quit_id) = build_tray(cfg.language);

        // Tray event watcher thread
        // We cannot poll tray events inside update() because winit stops
        // dispatching repaints to hidden viewports, so update() never runs
        // while the window is in the tray.  A dedicated blocking thread solves
        // this: it receives events regardless of window visibility.
        // Quit  → exit(0) immediately (OS cleans up the tray icon on process death)
        // Open  → set tray_show flag + request_repaint() so update() restores the window
        let tray_show = Arc::new(AtomicBool::new(false));
        {
            let quit_id    = tray_quit_id.clone();
            let open_id    = tray_open_id.clone();
            let show_flag  = Arc::clone(&tray_show);
            let repaint_ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                loop {
                    if let Ok(event) = tray_icon::menu::MenuEvent::receiver().recv() {
                        let is_quit = quit_id.as_ref().map(|id| id == &event.id).unwrap_or(false);
                        let is_open = open_id.as_ref().map(|id| id == &event.id).unwrap_or(false);
                        if is_quit {
                            std::process::exit(0);
                        } else if is_open {
                            show_flag.store(true, Ordering::Relaxed);
                            repaint_ctx.request_repaint();
                        }
                    }
                }
            });
        }

        // ── WinEvent hook thread ──────────────────────────────────────────
        // Installs a system-wide EVENT_OBJECT_SHOW hook and runs a Win32
        // message pump so the hook callbacks can fire.  The callback wakes
        // the scan/auto-inject threads instantly when a new window appears.
        // We use WINEVENT_OUTOFCONTEXT so no DLL injection is needed — the
        // callback runs in this process's thread, not the target's.
        std::thread::Builder::new()
            .name("winevent-hook".into())
            .spawn(move || unsafe {
                // Initialise the global waker before installing the hook so
                // the callback can safely call SCAN_WAKER.get() at any time.
                scan_waker();

                // EVENT_SYSTEM_FOREGROUND (0x0003): fires when the user switches
                // to any window — catches protection that was already applied to
                // an existing window before auto-inject started watching it.
                // EVENT_OBJECT_SHOW (0x8002): fires when a new window appears.
                // Together these cover both the "new window" and "switched to"
                // cases without polling, keeping detection latency near zero.
                const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
                const EVENT_OBJECT_SHOW:       u32 = 0x8002;

                let hook_fg = SetWinEventHook(
                    EVENT_SYSTEM_FOREGROUND,
                    EVENT_SYSTEM_FOREGROUND,
                    None,
                    Some(winevent_proc),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                );
                let hook = SetWinEventHook(
                    EVENT_OBJECT_SHOW,
                    EVENT_OBJECT_SHOW,
                    None, // no DLL needed with OUTOFCONTEXT
                    Some(winevent_proc),
                    0,    // all processes
                    0,    // all threads
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                );

                // Run a blocking message loop so callbacks can be dispatched.
                let mut msg = MSG::default();
                loop {
                    match GetMessageW(&mut msg, None, 0, 0).0 {
                        0 | -1 => break,
                        _ => {
                            let _ = TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                }

                // Unreachable in normal operation, but clean up properly.
                use windows::Win32::UI::Accessibility::UnhookWinEvent;
                if !hook_fg.is_invalid() { let _ = UnhookWinEvent(hook_fg); }
                if !hook.is_invalid()    { let _ = UnhookWinEvent(hook); }
            })
            .ok(); // non-critical — polling still works if spawn fails

        let auto_inject_rules_arc = Arc::new(Mutex::new(cfg.process_rules.clone()));
        let auto_inject_exclusions_arc = Arc::new(Mutex::new(cfg.exclusions.clone()));

        let mut app = App {
            shared_windows,
            exe_dir,
            persistent_mode: cfg.persistent_mode,
            auto_inject_enabled: false, // started below if saved
            auto_inject_running: Arc::new(AtomicBool::new(false)),
            auto_inject_seen: Arc::new(Mutex::new(HashSet::new())),
            filter: String::new(),
            protected_only: cfg.protected_only,
            show_help: false,
            help_section: 0,
            show_settings: false,
            status_msg: String::from("Ready."),
            status_color: Color32::GRAY,
            status_time: None,
            startup_enabled: read_startup_reg(),
            one_shot_stripped: HashMap::new(),
            reapply_alert: Vec::new(),
            log_entries: Vec::new(),
            show_log: cfg.show_log,
            sort_col: SortCol::from_u8(cfg.sort_col),
            sort_asc: cfg.sort_asc,
            watch_names: cfg.watch_names.clone(),
            watch_input: String::new(),
            watch_seen: HashSet::new(),
            hotkey_enabled: cfg.hotkey_enabled,
            hotkey_id,
            hotkey_key: cfg.hotkey_key,
            hotkey_mods: cfg.hotkey_mods,
            hotkey_recording: false,
            update_state: None,
            update_rx: Some(update_rx),
            download_state: DownloadState::Idle,
            download_rx: None,
            show_update_confirm: false,
            update_confirmed: false,
            toast_enabled: cfg.toast_enabled,
            toast_pending: Vec::new(),
            toast_batch_deadline: None,
            icon_cache: HashMap::new(),
            tray_icon,
            tray_open_id,
            tray_quit_id,
            minimize_to_tray: cfg.minimize_to_tray,
            logging_enabled: cfg.logging_enabled,
            discord_rpc_enabled: cfg.discord_rpc_enabled,
            discord_rpc_running: Arc::new(AtomicBool::new(false)),
            discord_rpc_tx: None,
            session_strip_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            tray_show,
            inject_tx,
            inject_rx,
            tray_was_protected: false,
            silent_startup: cfg.silent_startup,
            fast_scan: cfg.fast_scan,
            fast_scan_arc: fast_scan_arc_local,
            lifetime_strips: cfg.lifetime_strips,
            strip_on_launch: cfg.strip_on_launch,
            strip_on_launch_pending: cfg.strip_on_launch,
            process_rules: cfg.process_rules.clone(),
            rule_input: String::new(),
            rule_mode_input: ProcessRuleMode::AlwaysOneShot,
            auto_inject_rules: auto_inject_rules_arc,
            exclusions: cfg.exclusions.clone(),
            exclusion_input: String::new(),
            auto_inject_exclusions: auto_inject_exclusions_arc,
            language: cfg.language,
        };

        // Restore auto-inject if it was running when the app last closed
        if cfg.auto_inject {
            app.auto_inject_enabled = true;
            app.start_auto_inject();
        }

        // Restore Discord RPC if it was enabled
        if cfg.discord_rpc_enabled {
            app.start_discord_rpc();
        }

        app
    }

    // DLL path resolution

    fn dll_path(&self, is_32bit: bool) -> PathBuf {
        let name = if self.persistent_mode {
            "payload_dll_persistent.dll"
        } else {
            "payload_dll.dll"
        };
        if is_32bit {
            // Prefer {exe_dir}/x86/ — used by the installer and the GUI zip bundle.
            let installed = self.exe_dir.join("x86").join(name);
            if installed.exists() {
                return installed;
            }
            // Fall back to the Cargo build-output layout for local development.
            self.exe_dir
                .join("..")
                .join("i686-pc-windows-msvc")
                .join("release")
                .join(name)
        } else {
            self.exe_dir.join(name)
        }
    }

    // Injection

    fn inject_pid_async(&self, pid: u32, process_name: String, is_32bit: bool) {
        let dll_path = self.dll_path(is_32bit);
        let tx = self.inject_tx.clone();
        let lang = self.language;
        std::thread::spawn(move || {
            let inj = i18n::inject(lang);
            if !dll_path.exists() {
                let arch = if is_32bit { "x86" } else { "x64" };
                let _ = tx.send(InjResult {
                    msg: inj.dll_not_found
                        .replace("{arch}", arch)
                        .replace("{path}", &dll_path.display().to_string()),
                    ok: false,
                });
                return;
            }
            match injector_core::inject_fast(pid, &dll_path) {
                Ok(()) => {
                    let _ = tx.send(InjResult {
                        msg: inj.stripped_pid
                            .replace("{pid}", &pid.to_string())
                            .replace("{name}", &process_name),
                        ok: true,
                    });
                }
                Err(e) => {
                    let _ = tx.send(InjResult {
                        msg: inj.pid_error
                            .replace("{pid}", &pid.to_string())
                            .replace("{name}", &process_name)
                            .replace("{e}", &e.to_string()),
                        ok: false,
                    });
                }
            }
        });
    }

    fn strip_window(&self, entry: &WindowEntry) {
        let is_browser = BROWSER_NAMES
            .iter()
            .any(|b| entry.process_name.eq_ignore_ascii_case(b));

        let mut targets: Vec<(u32, String, bool)> =
            vec![(entry.pid, entry.process_name.clone(), entry.is_32bit)];

        if is_browser {
            for child_pid in get_child_pids(entry.pid) {
                let is_32 = is_process_32bit(child_pid);
                let child_name = i18n::inject(self.language).child_suffix.replace("{name}", &entry.process_name);
                targets.push((child_pid, child_name, is_32));
            }
        }

        for (pid, name, is_32) in targets {
            self.inject_pid_async(pid, name, is_32);
        }
    }

    fn strip_all_protected(&self, windows: &[WindowEntry]) {
        let mut seen: HashSet<u32> = HashSet::new();
        let mut count = 0usize;

        for w in windows.iter().filter(|w| w.is_protected) {
            if seen.insert(w.pid) {
                self.strip_window(w);
                count += 1;
            }
        }

        if count == 0 {
            // will show in status from the inject results; handled by caller
        }
        let _ = self.inject_tx.send(InjResult {
            msg: i18n::inject(self.language).stripping_n.replace("{count}", &count.to_string()),
            ok: true,
        });
    }

    fn set_status(&mut self, msg: impl Into<String>, ok: bool) {
        self.status_msg = msg.into();
        self.status_color = if ok {
            theme::SUCCESS
        } else {
            theme::DANGER
        };
        self.status_time = Some(Instant::now());
    }

    fn set_status_neutral(&mut self, msg: impl Into<String>) {
        self.status_msg = msg.into();
        self.status_color = Color32::GRAY;
        self.status_time = Some(Instant::now());
    }

    // Injection log file helpers

    /// Returns `%APPDATA%\capture-bypass\injection.log`
    fn log_path() -> Option<std::path::PathBuf> {
        dirs_next::config_dir().map(|d| d.join("capture-bypass").join("injection.log"))
    }

    /// Computes a UTC timestamp string (YYYY-MM-DD HH:MM:SS UTC) from SystemTime
    /// without pulling in the chrono crate.
    fn utc_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Gregorian calendar math
        let s   = secs % 60;
        let m   = (secs / 60) % 60;
        let h   = (secs / 3_600) % 24;
        let days = secs / 86_400; // days since 1970-01-01
        // Shift epoch to 2000-03-01 (makes leap-year math simpler)
        let days400 = days + 10_957 + 31 + 28; // offset to 2000-03-01
        let (era, doe) = {
            let era = days400 / 146_097;
            (era, days400 - era * 146_097)
        };
        let yoe = (doe - doe/1_460 + doe/36_524 - doe/146_096) / 365;
        let y   = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp  = (5 * doy + 2) / 153;
        let d   = doy - (153 * mp + 2) / 5 + 1;
        let mo  = if mp < 10 { mp + 3 } else { mp - 9 };
        let yr  = if mo <= 2 { y + 1 } else { y };
        format!("{yr:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02} UTC")
    }

    /// Appends a single log line to the injection log file.
    /// Creates the directory and file if they don't exist.
    fn append_log_entry(&self, line: &str) {
        if let Some(path) = Self::log_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true).append(true).open(&path)
            {
                let ts = Self::utc_timestamp();
                let _ = writeln!(f, "[{ts}] {line}");
            }
        }
    }

    // Persist config

    fn persist(&self) {
        save_config(&Config {
            persistent_mode: self.persistent_mode,
            auto_inject: self.auto_inject_enabled,
            protected_only: self.protected_only,
            toast_enabled: self.toast_enabled,
            hotkey_enabled: self.hotkey_enabled,
            show_log: self.show_log,
            watch_names: self.watch_names.clone(),
            sort_col: self.sort_col.to_u8(),
            sort_asc: self.sort_asc,
            minimize_to_tray: self.minimize_to_tray,
            logging_enabled: self.logging_enabled,
            discord_rpc_enabled: self.discord_rpc_enabled,
            hotkey_key: self.hotkey_key,
            hotkey_mods: self.hotkey_mods,
            silent_startup: self.silent_startup,
            fast_scan: self.fast_scan,
            lifetime_strips: self.lifetime_strips,
            strip_on_launch: self.strip_on_launch,
            process_rules: self.process_rules.clone(),
            exclusions: self.exclusions.clone(),
            language: self.language,
        });
    }

    // Auto-inject

    fn start_auto_inject(&mut self) {
        self.auto_inject_running.store(true, Ordering::Relaxed);
        let running    = Arc::clone(&self.auto_inject_running);
        let exe_dir    = self.exe_dir.clone();
        let persistent = self.persistent_mode;
        let tx         = self.inject_tx.clone();
        let fast_scan  = self.fast_scan;
        let rules      = Arc::clone(&self.auto_inject_rules);
        let exclusions = Arc::clone(&self.auto_inject_exclusions);
        let lang       = self.language;

        std::thread::spawn(move || {
            // Per-PID state, entirely local to this thread — no Arc needed.
            //
            // (process_name, used_persistent, gave_up, last_injected)
            //
            //  gave_up = true   → OS-level block (MitigationPolicy); don't retry.
            //  used_persistent  → persistent DLL was injected.  Give it a 2 s
            //      grace period for the IAT hook to install + initial strip to
            //      run.  After that, if protection is still visible, re-inject
            //      (handles dynamic GetProcAddress callers and hook-install races).
            //  Neither          → one-shot succeeded; re-protection → escalate.
            let mut state: HashMap<u32, (String, bool, bool, std::time::Instant)> =
                HashMap::new();

            while running.load(Ordering::Relaxed) {
                // Wait for the next poll interval OR an immediate WinEvent
                // wake-up, whichever comes first.  This makes auto-inject
                // react to new windows in ~0 ms instead of up to 400 ms.
                let sleep_ms = if fast_scan { 100 } else { 400 };
                let (lock, cvar) = scan_waker();
                let guard = lock.lock().unwrap();
                let _ = cvar.wait_timeout(guard, Duration::from_millis(sleep_ms));

                let windows = enumerate_windows();

                // Remove entries for processes that have exited
                // When a process restarts it gets a fresh PID and is treated as
                // a new arrival, which is exactly what we want.
                let live_pids: HashSet<u32> = windows.iter().map(|w| w.pid).collect();
                state.retain(|pid, _| live_pids.contains(pid));

                for w in windows.iter().filter(|w| w.is_protected) {
                    // Check exclusions
                    {
                        let excl = exclusions.lock().unwrap();
                        if excl.iter().any(|e| e.eq_ignore_ascii_case(&w.process_name)) {
                            drop(excl);
                            continue;
                        }
                        drop(excl);
                    }

                    // Check per-process rules for Skip
                    let rules_snap = rules.lock().unwrap().clone();
                    if let Some(rule) = rules_snap.iter().find(|r| r.process_name.eq_ignore_ascii_case(&w.process_name)) {
                        if rule.mode == ProcessRuleMode::Skip { continue; }
                    }

                    // Decide what action (if any) to take
                    let (use_persistent, is_escalation) =
                        match state.get(&w.pid) {
                            // Never seen this PID → use the user's global setting.
                            None => (persistent, false),

                            // Persistent DLL was injected.
                            // Give it 2 s to install its IAT hook and run the
                            // initial strip (the DLL sleeps 100 ms before that).
                            // After the grace period, if protection is still
                            // visible, re-inject — this catches callers that
                            // resolve SetWindowDisplayAffinity via GetProcAddress
                            // at runtime (bypassing the IAT hook entirely) and
                            // any race during hook installation.
                            Some((_, true, _, last)) => {
                                if last.elapsed() < Duration::from_secs(2) {
                                    continue; // still within grace period
                                }
                                (true, true) // re-inject persistent
                            }

                            // OS policy blocked us — nothing more we can do.
                            Some((_, _, true, _)) => continue,

                            // One-shot injection succeeded earlier but protection
                            // is back → the app re-applied it.  Escalate to the
                            // persistent DLL, which hooks SetWindowDisplayAffinity
                            // so the app can no longer reapply the flag.
                            Some((_, false, false, _)) => (true, true),
                        };

                    // Apply per-process rule override for persistent/oneshot
                    let use_persistent_for_this = rules_snap.iter()
                        .find(|r| r.process_name.eq_ignore_ascii_case(&w.process_name))
                        .map(|r| r.mode == ProcessRuleMode::AlwaysPersistent)
                        .unwrap_or(use_persistent);

                    let dll_name = if use_persistent_for_this {
                        "payload_dll_persistent.dll"
                    } else {
                        "payload_dll.dll"
                    };

                    let resolve_dll = |is32: bool| -> PathBuf {
                        if is32 {
                            let p = exe_dir.join("x86").join(dll_name);
                            if p.exists() { return p; }
                            exe_dir.join("..").join("i686-pc-windows-msvc")
                                   .join("release").join(dll_name)
                        } else {
                            exe_dir.join(dll_name)
                        }
                    };

                    // Expand browsers to child processes (renderer isolation).
                    let mut pids: Vec<(u32, String, bool)> =
                        vec![(w.pid, w.process_name.clone(), w.is_32bit)];
                    if BROWSER_NAMES.iter().any(|b| w.process_name.eq_ignore_ascii_case(b)) {
                        let child_suffix = i18n::inject(lang).child_suffix.to_string();
                        for child in get_child_pids(w.pid) {
                            pids.push((child, child_suffix.replace("{name}", &w.process_name),
                                       is_process_32bit(child)));
                        }
                    }

                    for (pid, name, is32) in pids {
                        let dll_path = resolve_dll(is32);
                        if !dll_path.exists() { continue; }

                        match injector_core::inject_fast(pid, &dll_path) {
                            Ok(()) => {
                                let inj = i18n::inject(lang);
                                let verb = if is_escalation {
                                    inj.escalated
                                } else {
                                    inj.auto_stripped
                                };
                                let msg = inj.verb_pid
                                    .replace("{verb}", verb)
                                    .replace("{name}", &name)
                                    .replace("{pid}", &pid.to_string());
                                let _ = tx.send(InjResult { msg, ok: true });
                                state.insert(pid, (name, use_persistent_for_this, false,
                                                   std::time::Instant::now()));
                            }

                            // OS-enforced block — nothing we can do in user-mode.
                            // Log once and give up on this PID.
                            Err(injector_core::InjectError::MitigationPolicy(reason)) => {
                                let inj = i18n::inject(lang);
                                let msg = inj.blocked_by_os
                                    .replace("{name}", &name)
                                    .replace("{pid}", &pid.to_string())
                                    .replace("{reason}", &reason);
                                let _ = tx.send(InjResult { msg, ok: false });
                                state.insert(pid, (name, false, true,
                                                   std::time::Instant::now()));
                            }

                            // Other errors (privilege race, timing) — log only
                            // if we haven't already logged for this PID.
                            Err(e) => {
                                if !state.contains_key(&pid) {
                                    let inj = i18n::inject(lang);
                                    let msg = inj.warning
                                        .replace("{name}", &name)
                                        .replace("{pid}", &pid.to_string())
                                        .replace("{e}", &e.to_string());
                                    let _ = tx.send(InjResult { msg, ok: false });
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn stop_auto_inject(&mut self) {
        self.auto_inject_running.store(false, Ordering::Relaxed);
    }

    // Discord Rich Presence

    fn start_discord_rpc(&mut self) {
        use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
        use std::sync::mpsc;

        if self.discord_rpc_running.load(Ordering::Relaxed) {
            return;
        }
        self.discord_rpc_running.store(true, Ordering::Relaxed);

        let (tx, rx) = mpsc::sync_channel::<RpcState>(8);
        self.discord_rpc_tx = Some(tx);

        let running = Arc::clone(&self.discord_rpc_running);
        let strip_count = Arc::clone(&self.session_strip_count);
        let lang = self.language;

        std::thread::Builder::new()
            .name("discord-rpc".into())
            .spawn(move || {
                // new() is infallible in discord-rich-presence 1.1 — connect() can fail
                let mut client = DiscordIpcClient::new(DISCORD_APP_ID);
                if client.connect().is_err() {
                    return;
                }

                let d = i18n::discord(lang);

                // Push the initial presence right away
                let count = strip_count.load(Ordering::Relaxed);
                let _ = client.set_activity(
                    activity::Activity::new()
                        .details("capture-bypass")
                        .state(&d.strips_this_session.replace("{count}", &count.to_string()))
                        .buttons(vec![
                            activity::Button::new(
                                d.get_capture_bypass,
                                "https://github.com/levi52/capture-bypass/releases/latest",
                            ),
                            activity::Button::new(
                                "★ GitHub",
                                "https://github.com/levi52/capture-bypass",
                            ),
                        ]),
                );

                // Keep updating whenever the main thread sends a new state
                while running.load(Ordering::Relaxed) {
                    match rx.recv_timeout(std::time::Duration::from_secs(5)) {
                        Ok(state) => {
                            let mode_str = if state.persistent { d.persistent } else { d.one_shot };
                            let auto_str = if state.auto_inject { d.auto_inject_on } else { d.manual };
                            let status = format!("{auto_str} · {mode_str} · {strips} strips",
                                strips = state.strip_count);
                            let _ = client.set_activity(
                                activity::Activity::new()
                                    .details("capture-bypass")
                                    .state(&status)
                                    .buttons(vec![
                                        activity::Button::new(
                                            d.get_capture_bypass,
                                            "https://github.com/levi52/capture-bypass/releases/latest",
                                        ),
                                        activity::Button::new(
                                            "★ GitHub",
                                            "https://github.com/levi52/capture-bypass",
                                        ),
                                    ]),
                            );
                        }
                        // Timeout just means no update — stay connected and loop
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        // Channel closed — main thread dropped the sender
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }

                let _ = client.clear_activity();
                let _ = client.close();
            })
            .ok();
    }

    fn stop_discord_rpc(&mut self) {
        self.discord_rpc_running.store(false, Ordering::Relaxed);
        // Dropping the sender unblocks the RPC thread's recv_timeout immediately
        self.discord_rpc_tx = None;
    }

    // Send the current state to the RPC thread (no-op if RPC is off)
    fn push_rpc_state(&self) {
        if let Some(tx) = &self.discord_rpc_tx {
            let _ = tx.try_send(RpcState {
                auto_inject: self.auto_inject_enabled,
                persistent:  self.persistent_mode,
                strip_count: self.session_strip_count.load(Ordering::Relaxed),
            });
        }
    }
}

// egui rendering

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll injection results
        let mut rpc_needs_update = false;
        while let Ok(result) = self.inject_rx.try_recv() {
            if self.logging_enabled {
                self.append_log_entry(&result.msg);
            }
            if result.ok {
                self.session_strip_count.fetch_add(1, Ordering::Relaxed);
                self.lifetime_strips += 1;
                if self.lifetime_strips % 10 == 0 {
                    self.persist();
                }
                rpc_needs_update = true;
                self.toast_pending.push(result.msg.clone());
                if self.toast_batch_deadline.is_none() {
                    self.toast_batch_deadline = Some(Instant::now() + Duration::from_millis(400));
                }
            }
            self.log_entries.push(LogEntry {
                time: Instant::now(),
                msg: result.msg.clone(),
                ok: result.ok,
            });
            if self.log_entries.len() > 500 {
                self.log_entries.remove(0);
            }
            self.set_status(result.msg, result.ok);
        }
        // Toast batch flush
        if let Some(deadline) = self.toast_batch_deadline {
            if Instant::now() >= deadline {
                if self.toast_enabled && !self.toast_pending.is_empty() {
                    let msgs = std::mem::take(&mut self.toast_pending);
                    let toast = i18n::toast(self.language);
                    if msgs.len() == 1 {
                        send_toast(toast.title, &msgs[0]);
                    } else {
                        send_toast(toast.title, &toast.stripped_n_windows.replace("{n}", &msgs.len().to_string()));
                    }
                } else {
                    self.toast_pending.clear();
                }
                self.toast_batch_deadline = None;
            }
        }
        if rpc_needs_update {
            self.push_rpc_state();
        }

        // Poll auto-update result — just record the state; download only starts after user confirms.
        if let Some(rx) = &self.update_rx {
            while let Ok(state) = rx.try_recv() {
                self.update_state = Some(state);
            }
        }

        // Poll download progress.
        // Drain into a local vec first so the borrow on self.download_rx ends
        // before we need to write self.download_rx = None.
        let download_msgs: Vec<DownloadMsg> = self.download_rx
            .as_ref()
            .map(|rx| std::iter::from_fn(|| rx.try_recv().ok()).collect())
            .unwrap_or_default();
        let mut clear_download_rx = false;
        for msg in download_msgs {
            match msg {
                DownloadMsg::Progress(p) => {
                    self.download_state = DownloadState::Downloading(p);
                }
                DownloadMsg::Verifying => {
                    self.download_state = DownloadState::Verifying;
                }
                DownloadMsg::Done(path) => {
                    self.download_state = DownloadState::Ready(path);
                    clear_download_rx = true;
                }
                DownloadMsg::Failed(e) => {
                    self.download_state = DownloadState::Failed(e);
                    clear_download_rx = true;
                }
            }
        }
        if clear_download_rx {
            self.download_rx = None;
        }

        // If the user already confirmed the update, apply it the moment the
        // download finishes — no second button click required.
        if self.update_confirmed {
            if let DownloadState::Ready(_) = &self.download_state {
                if let DownloadState::Ready(path) =
                    std::mem::replace(&mut self.download_state, DownloadState::Idle)
                {
                    self.update_confirmed = false;
                    let exe_dir = self.exe_dir.clone();
                    apply_update(&path, &exe_dir);
                }
            }
        }

        // Poll global hotkey messages
        if self.hotkey_enabled {
            unsafe {
                let mut msg: MSG = std::mem::zeroed();
                if PeekMessageW(&mut msg, None, WM_HOTKEY, WM_HOTKEY, PM_REMOVE).as_bool() {
                    // WM_HOTKEY — Strip All Protected
                    let all = self.shared_windows.lock().unwrap().clone();
                    if all.iter().any(|w| w.is_protected) {
                        self.strip_all_protected(&all);
                    }
                }
            }
        }

        // Tray "Open" action
        // Quit is handled by the off-thread tray watcher (process::exit).
        // Open sets this flag + requests a repaint so we catch it here.
        if self.tray_show.load(Ordering::Relaxed) {
            self.tray_show.store(false, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Handle window close (X button)
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.minimize_to_tray {
                // Hide to tray — the tray watcher thread handles true quit.
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
            // else: let eframe handle the close normally → process exits.
        }

        // Snapshot current window list
        let all_windows: Vec<WindowEntry> = self.shared_windows.lock().unwrap().clone();

        // Tray icon reacts to protection state
        {
            let any_protected = all_windows.iter().any(|w| w.is_protected);
            if any_protected != self.tray_was_protected {
                self.tray_was_protected = any_protected;
                if let Some(ref tray) = self.tray_icon {
                    if any_protected {
                        if let Some(icon) = make_tray_icon(255, 0, 0) {
                            let _ = tray.set_icon(Some(icon));
                        }
                        let _ = tray.set_tooltip(Some(i18n::tray(self.language).tooltip_protected));
                    } else {
                        if let Some(icon) = make_tray_icon(0x44, 0x88, 0xFF) {
                            let _ = tray.set_icon(Some(icon));
                        }
                        let _ = tray.set_tooltip(Some(i18n::tray(self.language).tooltip));
                    }
                }
            }
        }

        // Strip on launch
        if self.strip_on_launch_pending && !all_windows.is_empty() {
            self.strip_on_launch_pending = false;
            if all_windows.iter().any(|w| w.is_protected) {
                self.strip_all_protected(&all_windows);
                self.set_status(i18n::status_msg(self.language).launch_strip.to_string(), true);
            }
        }

        // Detect re-applied protection on one-shot stripped processes
        // Only fires when NOT in persistent mode and NOT in auto-inject (which
        // would handle it silently).  Moves matching PIDs to reapply_alert so the
        // modal popup can prompt the user.
        if !self.persistent_mode && !self.auto_inject_enabled && !self.one_shot_stripped.is_empty() {
            for w in all_windows.iter().filter(|w| w.is_protected) {
                if let Some(name) = self.one_shot_stripped.remove(&w.pid) {
                    if !self.reapply_alert.iter().any(|(pid, _)| *pid == w.pid) {
                        self.reapply_alert.push((w.pid, name));
                    }
                }
            }
        }

        // Watch mode — inject when a watched process name appears
        if !self.watch_names.is_empty() && !self.auto_inject_enabled {
            for w in all_windows.iter() {
                let matched = self.watch_names.iter().any(|n| {
                    w.process_name.to_lowercase() == n.to_lowercase()
                });
                if matched && !self.watch_seen.contains(&w.pid) {
                    self.watch_seen.insert(w.pid);
                    self.inject_pid_async(w.pid, w.process_name.clone(), w.is_32bit);
                }
            }
            // Prune PIDs that are no longer alive
            let live_pids: HashSet<u32> = all_windows.iter().map(|w| w.pid).collect();
            self.watch_seen.retain(|pid| live_pids.contains(pid));
        }

        // Apply filter
        let filter_lc = self.filter.to_lowercase();
        let mut filtered: Vec<&WindowEntry> = all_windows
            .iter()
            .filter(|w| {
                if self.protected_only && !w.is_protected {
                    return false;
                }
                if filter_lc.is_empty() {
                    return true;
                }
                w.title.to_lowercase().contains(&filter_lc)
                    || w.process_name.to_lowercase().contains(&filter_lc)
                    || w.pid.to_string().contains(&filter_lc)
            })
            .collect();

        // Apply column sort
        let sort_col = self.sort_col;
        let sort_asc = self.sort_asc;
        filtered.sort_by(|a, b| {
            let ord = match sort_col {
                SortCol::Pid     => a.pid.cmp(&b.pid),
                SortCol::Process => a.process_name.to_lowercase().cmp(&b.process_name.to_lowercase()),
                SortCol::Title   => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                SortCol::Status  => a.affinity.cmp(&b.affinity),
            };
            if sort_asc { ord } else { ord.reverse() }
        });

        // Help window
        render_help_window(ctx, &mut self.show_help, &mut self.help_section, self.language);

        // Compute n_prot early (used in filter bar and status bar)
        let n_prot = all_windows.iter().filter(|w| w.is_protected).count();

        // Settings window
        let mut toggle_startup_from_settings          = false;
        let mut toggle_toast_from_settings            = false;
        let mut toggle_hotkey_from_settings           = false;
        let mut toggle_minimize_to_tray_from_settings = false;
        let mut toggle_logging_from_settings          = false;
        let mut open_log_file_from_settings           = false;
        let mut toggle_discord_rpc_from_settings      = false;
        let mut start_hotkey_recording                = false;
        let mut toggle_silent_startup                 = false;
        let mut toggle_fast_scan                      = false;
        let mut toggle_strip_on_launch                = false;
        let mut do_export_config                      = false;
        let mut do_import_config                      = false;
        let mut add_process_rule                      = false;
        let mut remove_process_rule: Option<usize>    = None;
        let mut add_exclusion                         = false;
        let mut remove_exclusion: Option<usize>       = None;
        render_settings_window(
            ctx,
            &mut self.show_settings,
            self.language,
            self.startup_enabled,
            self.toast_enabled,
            self.hotkey_enabled,
            self.minimize_to_tray,
            self.logging_enabled,
            self.discord_rpc_enabled,
            self.hotkey_mods,
            self.hotkey_key,
            self.hotkey_recording,
            self.silent_startup,
            self.fast_scan,
            self.strip_on_launch,
            &self.process_rules,
            &mut self.rule_input,
            &mut self.rule_mode_input,
            &self.exclusions,
            &mut self.exclusion_input,
            &mut toggle_startup_from_settings,
            &mut toggle_toast_from_settings,
            &mut toggle_hotkey_from_settings,
            &mut toggle_minimize_to_tray_from_settings,
            &mut toggle_logging_from_settings,
            &mut open_log_file_from_settings,
            &mut toggle_discord_rpc_from_settings,
            &mut start_hotkey_recording,
            &mut toggle_silent_startup,
            &mut toggle_fast_scan,
            &mut toggle_strip_on_launch,
            &mut do_export_config,
            &mut do_import_config,
            &mut add_process_rule,
            &mut remove_process_rule,
            &mut add_exclusion,
            &mut remove_exclusion,
            &self.exe_dir.to_string_lossy(),
        );

        // Update confirmation dialog
        // Rendered as a floating Window before the header panel so it
        // appears on top.  show_update_confirm is toggled by the header button.
        if self.show_update_confirm {
            if let Some(UpdateState::Available(tag)) = &self.update_state {
                let tag = tag.clone();
                let mut confirmed = false;
                let mut cancelled = false;
                let u = i18n::update(self.language);
                egui::Window::new(u.window_title)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(u.update_to.replace("{tag}", &tag));
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(u.installer_hint)
                                .weak()
                                .small(),
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.add(
                                egui::Button::new(RichText::new(u.update_now.to_string()).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(34, 120, 50))
                            ).clicked() {
                                confirmed = true;
                            }
                            ui.add_space(8.0);
                            if ui.button(u.later).clicked() {
                                cancelled = true;
                            }
                        });
                    });
                if confirmed {
                    self.show_update_confirm = false;
                    self.update_confirmed = true;
                    start_download(tag, &mut self.download_state,
                                   &mut self.download_rx, &self.exe_dir);
                }
                if cancelled {
                    self.show_update_confirm = false;
                }
            } else {
                self.show_update_confirm = false;
            }
        }

        // Top bar
        // Collect actions here to avoid borrow conflicts
        let mut do_strip_all = false;
        let mut toggle_mode = false;
        let mut toggle_auto = false;
        let mut toggle_help = false;
        let mut toggle_settings = false;
        let mut manual_refresh = false;
        let mut do_stress_test = false;
        let mut toggle_log = false;
        let mut toggle_update_confirm = false;
        let mut toggle_language = false;
        let mut new_sort: Option<(SortCol, bool)> = None;
        let mut remove_watch: Option<usize> = None;
        let mut add_watch = false;
        // Modal reapply actions — collected during modal rendering below.
        let mut dismiss_reapply = false;
        let mut switch_and_restrip = false;

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("capture-bypass").color(theme::TEXT).strong());
                ui.label(RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).monospace().size(11.0).color(theme::TEXT_FAINT));
                ui.label(RichText::new(i18n::header(self.language).admin).monospace().size(10.0).color(theme::ACCENT));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Language toggle
                    let lang_label = self.language.label();
                    if ui.button(lang_label)
                        .on_hover_text(if self.language == Language::English { "Switch to Chinese" } else { "切换到英文" })
                        .clicked()
                    {
                        toggle_language = true;
                    }
                    ui.add_space(4.0);

                    if ui.button(i18n::header(self.language).help).clicked() {
                        toggle_help = true;
                    }
                    ui.add_space(4.0);

                    // Update header — click to open confirm dialog, then download, then restart.
                    if let Some(UpdateState::Available(tag)) = &self.update_state {
                        let tag = tag.clone();
                        let h = i18n::header(self.language);
                        let hh = i18n::header_hover(self.language);
                        let do_restart = match &self.download_state {
                            // Not yet downloading — show clickable "available" button
                            DownloadState::Idle => {
                                if ui.add(egui::Button::new(RichText::new(&h.update_available.replace("{tag}", &tag)).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(120, 80, 20)))
                                    .on_hover_text(hh.click_to_update)
                                    .clicked()
                                {
                                    toggle_update_confirm = true;
                                }
                                false
                            }
                            DownloadState::Downloading(pct) => {
                                let pct = *pct;
                                let label = if pct > 0.0 {
                                    h.downloading.replace("{pct}", &format!("{:.0}", pct * 100.0))
                                } else {
                                    "⬇ …".to_string()
                                };
                                ui.add(egui::Button::new(label)
                                    .fill(Color32::from_rgb(25, 55, 15)))
                                    .on_hover_text(hh.downloading_bg.replace("{tag}", &tag));
                                false
                            }
                            DownloadState::Verifying => {
                                ui.add(egui::Button::new(h.verifying)
                                    .fill(Color32::from_rgb(15, 45, 65)))
                                    .on_hover_text(hh.verifying_download);
                                false
                            }
                            // The one click the user ever needs
                            DownloadState::Ready(_) => {
                                ui.add(egui::Button::new(RichText::new(h.restart_to_update.to_string()).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(34, 120, 50)))
                                    .on_hover_text(
                                        hh.ready_to_install.replace("{tag}", &tag)
                                    )
                                    .clicked()
                            }
                            DownloadState::Failed(e) => {
                                let e = e.clone();
                                if ui.add(egui::Button::new(h.retry_update)
                                        .fill(Color32::from_rgb(80, 15, 15)))
                                    .on_hover_text(e)
                                    .clicked()
                                {
                                    start_download(tag, &mut self.download_state,
                                                   &mut self.download_rx, &self.exe_dir);
                                }
                                false
                            }
                        };

                        if do_restart {
                            // Take the path out of the Ready state so we can use it
                            if let DownloadState::Ready(path) =
                                std::mem::replace(&mut self.download_state, DownloadState::Idle)
                            {
                                let exe_dir = self.exe_dir.clone();
                                apply_update(&path, &exe_dir);
                            }
                        }

                        ui.add_space(4.0);
                    }

                    // Settings
                    if theme::ghost(ui, i18n::header(self.language).settings)
                        .on_hover_text(i18n::header_hover(self.language).settings_hint)
                        .clicked()
                    {
                        toggle_settings = true;
                    }
                    ui.add_space(4.0);

                    // Log toggle
                    let log_on = self.show_log;
                    if theme::toggle_button(ui, log_on, if log_on { i18n::header(self.language).log_on } else { i18n::header(self.language).log_off })
                        .on_hover_text(i18n::header_hover(self.language).log_hint)
                        .clicked()
                    {
                        toggle_log = true;
                    }
                    ui.add_space(4.0);

                    if ui
                        .button(i18n::header(self.language).stress_test)
                        .on_hover_text(i18n::header_hover(self.language).stress_test_hint)
                        .clicked()
                    {
                        do_stress_test = true;
                    }
                    ui.add_space(4.0);

                    let h = i18n::header(self.language);
                    let mode_label = if self.persistent_mode { h.persistent } else { h.one_shot };
                    if theme::toggle_button(ui, self.persistent_mode, mode_label).clicked() {
                        toggle_mode = true;
                    }
                    ui.label(theme::caption(h.mode));
                    ui.add_space(8.0);

                    if theme::danger(ui, i18n::header(self.language).strip_all_protected).clicked() {
                        do_strip_all = true;
                    }
                    ui.add_space(4.0);

                    if ui.button(i18n::header(self.language).refresh).clicked() {
                        manual_refresh = true;
                    }
                });
            });
            ui.add_space(4.0);

            // Filter bar + checkboxes
            ui.horizontal(|ui| {
                let f = i18n::filter(self.language);
                ui.label(f.filter);
                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .desired_width(220.0)
                        .hint_text(f.hint),
                );
                if ui.small_button("✕").clicked() {
                    self.filter.clear();
                }
                ui.add_space(12.0);
                ui.checkbox(&mut self.protected_only, f.protected_only);
                ui.add_space(12.0);

                let auto_label = if self.auto_inject_enabled {
                    f.auto_inject_on
                } else {
                    f.auto_inject_off
                };
                if theme::toggle_button(ui, self.auto_inject_enabled, auto_label).clicked() {
                    toggle_auto = true;
                }

                ui.add_space(12.0);
                if n_prot > 0 {
                    ui.label(RichText::new(f.n_protected.replace("{n}", &n_prot.to_string())).monospace().color(theme::DANGER).strong());
                } else {
                    ui.label(RichText::new(f.zero_protected).monospace().color(theme::SUCCESS));
                }
            });

            // Watch mode row
            ui.horizontal(|ui| {
                let f = i18n::filter(self.language);
                ui.label(f.watch);
                ui.add(
                    egui::TextEdit::singleline(&mut self.watch_input)
                        .desired_width(150.0)
                        .hint_text("process.exe"),
                );
                if ui.small_button(f.add).clicked() {
                    add_watch = true;
                }
                ui.add_space(8.0);
                let names: Vec<(usize, String)> = self.watch_names.iter().cloned().enumerate().collect();
                for (i, name) in names {
                    let resp = ui.add(
                        egui::Button::new(
                            RichText::new(format!("{name}  ✕")).color(theme::ACCENT_2).monospace().size(11.5),
                        )
                        .fill(theme::accent_a(22))
                        .stroke(egui::Stroke::new(1.0, theme::accent_a(70))),
                    );
                    if resp.clicked() {
                        remove_watch = Some(i);
                    }
                }
            });
            ui.add_space(4.0);
        });

        // Apply deferred actions
        if toggle_language {
            self.language = match self.language {
                Language::English => Language::Chinese,
                Language::Chinese => Language::English,
            };
            self.persist();
        }
        if toggle_help {
            self.show_help = !self.show_help;
        }
        if toggle_mode {
            self.persistent_mode = !self.persistent_mode;
            let mode = if self.persistent_mode { i18n::status_msg(self.language).mode_persistent } else { i18n::status_msg(self.language).mode_one_shot };
            self.set_status_neutral(mode);
            if self.persistent_mode {
                self.one_shot_stripped.clear();
                self.reapply_alert.clear();
            }
            self.persist();
            self.push_rpc_state();
        }
        if toggle_auto {
            self.auto_inject_enabled = !self.auto_inject_enabled;
            if self.auto_inject_enabled {
                self.start_auto_inject();
                self.set_status(i18n::status_msg(self.language).auto_inject_active, true);
            } else {
                self.stop_auto_inject();
                self.set_status_neutral(i18n::status_msg(self.language).auto_inject_disabled);
            }
            self.persist();
            self.push_rpc_state();
        }
        if manual_refresh {
            // Background thread handles refresh; just show feedback
            self.set_status_neutral(i18n::status_msg(self.language).refreshing);
        }
        if do_stress_test {
            let stress_path = self.exe_dir.join("stress_tester.exe");
            if stress_path.exists() {
                match std::process::Command::new(&stress_path).spawn() {
                    Ok(_) => self.set_status_neutral(i18n::status_msg(self.language).launched_stress_tester),
                    Err(e) => self.set_status(i18n::status_msg(self.language).could_not_launch_stress_tester.replace("{e}", &e.to_string()), false),
                }
            } else {
                self.set_status(
                    i18n::status_msg(self.language).stress_tester_not_found,
                    false,
                );
            }
        }
        if toggle_log {
            self.show_log = !self.show_log;
            self.persist();
        }
        if toggle_settings {
            self.show_settings = !self.show_settings;
        }
        if toggle_update_confirm {
            self.show_update_confirm = !self.show_update_confirm;
        }
        // Actions forwarded from the settings window
        if toggle_startup_from_settings {
            let desired = !self.startup_enabled;
            if write_startup_reg(desired) {
                self.startup_enabled = desired;
                let s = if desired { i18n::status_msg(self.language).added_to_startup } else { i18n::status_msg(self.language).removed_from_startup };
                self.set_status(s, desired);
            } else {
                self.set_status(i18n::status_msg(self.language).could_not_write_startup, false);
            }
        }
        if toggle_toast_from_settings {
            self.toast_enabled = !self.toast_enabled;
            let s = if self.toast_enabled { i18n::status_msg(self.language).toast_on } else { i18n::status_msg(self.language).toast_off };
            self.set_status_neutral(s);
            self.persist();
        }
        if toggle_hotkey_from_settings {
            self.hotkey_enabled = !self.hotkey_enabled;
            if self.hotkey_enabled {
                register_hotkey(self.hotkey_id, self.hotkey_mods, self.hotkey_key);
                let combo = hotkey_display(self.hotkey_mods, self.hotkey_key);
                self.set_status_neutral(i18n::status_msg(self.language).hotkey_registered.replace("{combo}", &combo));
            } else {
                unregister_hotkey(self.hotkey_id);
                self.set_status_neutral(i18n::status_msg(self.language).hotkey_unregistered);
            }
            self.persist();
        }
        if toggle_minimize_to_tray_from_settings {
            self.minimize_to_tray = !self.minimize_to_tray;
            let s = if self.minimize_to_tray {
                i18n::status_msg(self.language).closes_to_tray
            } else {
                i18n::status_msg(self.language).exits_app
            };
            self.set_status_neutral(s);
            self.persist();
        }
        if toggle_logging_from_settings {
            self.logging_enabled = !self.logging_enabled;
            let s = if self.logging_enabled {
                i18n::status_msg(self.language).log_on
            } else {
                i18n::status_msg(self.language).log_off
            };
            self.set_status_neutral(s);
            self.persist();
        }
        if open_log_file_from_settings {
            if let Some(path) = App::log_path() {
                // Create the file if it doesn't exist yet so Explorer can open it.
                if !path.exists() {
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::File::create(&path);
                }
                let _ = std::process::Command::new("explorer").arg(&path).spawn();
            }
        }
        if toggle_discord_rpc_from_settings {
            self.discord_rpc_enabled = !self.discord_rpc_enabled;
            if self.discord_rpc_enabled {
                self.start_discord_rpc();
                self.set_status_neutral(i18n::status_msg(self.language).discord_on);
            } else {
                self.stop_discord_rpc();
                self.set_status_neutral(i18n::status_msg(self.language).discord_off);
            }
            self.persist();
        }
        if start_hotkey_recording {
            self.hotkey_recording = true;
            // Unregister the hotkey while recording so the key presses reach egui
            if self.hotkey_enabled {
                unregister_hotkey(self.hotkey_id);
            }
        }
        if toggle_silent_startup {
            self.silent_startup = !self.silent_startup;
            let s = if self.silent_startup { i18n::status_msg(self.language).silent_startup_on } else { i18n::status_msg(self.language).silent_startup_off };
            self.set_status_neutral(s);
            self.persist();
        }
        if toggle_fast_scan {
            self.fast_scan = !self.fast_scan;
            self.fast_scan_arc.store(self.fast_scan, Ordering::Relaxed);
            let s = if self.fast_scan { i18n::status_msg(self.language).fast_scan_on } else { i18n::status_msg(self.language).fast_scan_off };
            self.set_status_neutral(s);
            self.persist();
        }
        if toggle_strip_on_launch {
            self.strip_on_launch = !self.strip_on_launch;
            let s = if self.strip_on_launch { i18n::status_msg(self.language).strip_on_launch_on } else { i18n::status_msg(self.language).strip_on_launch_off };
            self.set_status_neutral(s);
            self.persist();
        }
        if do_export_config {
            match (config_path(), dirs_next::desktop_dir()) {
                (Some(src), Some(desktop)) => {
                    let dest = desktop.join("capture-bypass-config.toml");
                    match std::fs::copy(&src, &dest) {
                        Ok(_) => self.set_status(i18n::status_msg(self.language).config_exported, true),
                        Err(e) => self.set_status(i18n::status_msg(self.language).export_failed.replace("{e}", &e.to_string()), false),
                    }
                }
                _ => self.set_status(i18n::status_msg(self.language).could_not_determine_path, false),
            }
        }
        if do_import_config {
            match dirs_next::desktop_dir() {
                Some(desktop) => {
                    let src = desktop.join("capture-bypass-config.toml");
                    let imported = load_config_from_path(&src);
                    self.persistent_mode   = imported.persistent_mode;
                    self.protected_only    = imported.protected_only;
                    self.toast_enabled     = imported.toast_enabled;
                    self.hotkey_enabled    = imported.hotkey_enabled;
                    self.show_log          = imported.show_log;
                    self.watch_names       = imported.watch_names.clone();
                    self.sort_col          = SortCol::from_u8(imported.sort_col);
                    self.sort_asc          = imported.sort_asc;
                    self.minimize_to_tray  = imported.minimize_to_tray;
                    self.logging_enabled   = imported.logging_enabled;
                    self.discord_rpc_enabled = imported.discord_rpc_enabled;
                    self.hotkey_key        = imported.hotkey_key;
                    self.hotkey_mods       = imported.hotkey_mods;
                    self.silent_startup    = imported.silent_startup;
                    self.fast_scan         = imported.fast_scan;
                    self.fast_scan_arc.store(imported.fast_scan, Ordering::Relaxed);
                    self.lifetime_strips   = imported.lifetime_strips;
                    self.strip_on_launch   = imported.strip_on_launch;
                    self.process_rules     = imported.process_rules.clone();
                    *self.auto_inject_rules.lock().unwrap() = imported.process_rules.clone();
                    self.exclusions        = imported.exclusions.clone();
                    *self.auto_inject_exclusions.lock().unwrap() = imported.exclusions.clone();
                    self.persist();
                    self.set_status(i18n::status_msg(self.language).config_imported, true);
                }
                None => self.set_status(i18n::status_msg(self.language).could_not_determine_desktop, false),
            }
        }
        if add_process_rule {
            let name = self.rule_input.trim().to_string();
            if !name.is_empty() && !self.process_rules.iter().any(|r| r.process_name.eq_ignore_ascii_case(&name)) {
                let mode = self.rule_mode_input.clone();
                self.process_rules.push(ProcessRule { process_name: name, mode });
                self.rule_input.clear();
                *self.auto_inject_rules.lock().unwrap() = self.process_rules.clone();
                self.persist();
            }
        }
        if let Some(idx) = remove_process_rule {
            if idx < self.process_rules.len() {
                self.process_rules.remove(idx);
                *self.auto_inject_rules.lock().unwrap() = self.process_rules.clone();
                self.persist();
            }
        }
        if add_exclusion {
            let name = self.exclusion_input.trim().to_string();
            if !name.is_empty() && !self.exclusions.iter().any(|e| e.eq_ignore_ascii_case(&name)) {
                self.exclusions.push(name);
                self.exclusion_input.clear();
                *self.auto_inject_exclusions.lock().unwrap() = self.exclusions.clone();
                self.persist();
            }
        }
        if let Some(idx) = remove_exclusion {
            if idx < self.exclusions.len() {
                self.exclusions.remove(idx);
                *self.auto_inject_exclusions.lock().unwrap() = self.exclusions.clone();
                self.persist();
            }
        }

        // Capture new hotkey combo when in recording mode
        if self.hotkey_recording {
            let mut captured: Option<(u32, u32)> = None; // (mods, vk)
            let escape = ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                        if let Some(vk) = egui_key_to_vk(*key) {
                            let mods = egui_mods_to_win(*modifiers);
                            // Require at least one modifier so we don't steal bare letter keys
                            if mods != 0 {
                                captured = Some((mods, vk));
                            }
                        }
                    }
                }
                i.key_pressed(egui::Key::Escape)
            });
            if let Some((mods, vk)) = captured {
                self.hotkey_recording = false;
                self.hotkey_mods = mods;
                self.hotkey_key = vk;
                if self.hotkey_enabled {
                    register_hotkey(self.hotkey_id, mods, vk);
                }
                let combo = hotkey_display(mods, vk);
                self.set_status_neutral(i18n::status_msg(self.language).hotkey_set.replace("{combo}", &combo));
                self.persist();
            } else if escape {
                self.hotkey_recording = false;
                // Re-register the old hotkey since we cancelled
                if self.hotkey_enabled {
                    register_hotkey(self.hotkey_id, self.hotkey_mods, self.hotkey_key);
                }
            }
        }

        if add_watch {
            let name = self.watch_input.trim().to_string();
            if !name.is_empty() && !self.watch_names.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
                self.watch_names.push(name);
                self.watch_input.clear();
                self.persist();
            }
        }
        if let Some(idx) = remove_watch {
            if idx < self.watch_names.len() {
                self.watch_names.remove(idx);
                self.persist();
            }
        }
        if let Some((col, asc)) = new_sort {
            self.sort_col = col;
            self.sort_asc = asc;
            self.persist();
        }
        if do_strip_all {
            if all_windows.iter().any(|w| w.is_protected) {
                // Track stripped PIDs for re-protection detection (one-shot mode only).
                if !self.persistent_mode {
                    let mut seen: HashSet<u32> = HashSet::new();
                    for w in all_windows.iter().filter(|w| w.is_protected) {
                        if seen.insert(w.pid) {
                            self.one_shot_stripped.insert(w.pid, w.process_name.clone());
                            if BROWSER_NAMES.iter().any(|b| w.process_name.eq_ignore_ascii_case(b)) {
                                let child_suffix = i18n::inject(self.language).child_suffix.to_string();
                                for child_pid in get_child_pids(w.pid) {
                                    self.one_shot_stripped.insert(
                                        child_pid,
                                        child_suffix.replace("{name}", &w.process_name),
                                    );
                                }
                            }
                        }
                    }
                }
                self.strip_all_protected(&all_windows);
            } else {
                self.set_status_neutral(i18n::status_msg(self.language).no_protected_found);
            }
        }

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let ts = self
                    .status_time
                    .map(|t| {
                        let secs = t.elapsed().as_secs();
                        if secs < 60 {
                            format!(" [{secs}s ago]")
                        } else {
                            String::new()
                        }
                    })
                    .unwrap_or_default();
                ui.label(
                    RichText::new(format!("{}{ts}", self.status_msg))
                        .color(self.status_color),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let n_32 = all_windows.iter().filter(|w| w.is_32bit).count();
                    ui.label(
                        RichText::new(format!(
                            "v{}  •  {} lifetime  •  {} windows  •  {} protected  •  {} 32-bit",
                            env!("CARGO_PKG_VERSION"),
                            self.lifetime_strips,
                            all_windows.len(),
                            n_prot,
                            n_32
                        ))
                        .weak()
                        .small(),
                    );
                });
            });
            ui.add_space(4.0);
        });

        // Injection log panel
        if self.show_log {
            let l = i18n::log(self.language);
            egui::TopBottomPanel::bottom("log_panel")
                .resizable(true)
                .min_height(80.0)
                .default_height(150.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(l.title);
                        ui.add_space(8.0);
                        if ui.small_button(l.clear).clicked() {
                            self.log_entries.clear();
                        }
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for entry in &self.log_entries {
                                let elapsed = entry.time.elapsed().as_secs();
                                let color = if entry.ok {
                                    theme::SUCCESS
                                } else {
                                    theme::DANGER
                                };
                                ui.label(
                                    RichText::new(format!("[{elapsed}s ago]  {}", entry.msg))
                                        .color(color)
                                        .small()
                                        .monospace(),
                                );
                            }
                        });
                });
        }

        // Main table
        // Collect any row-level injection requests
        let mut inject_target: Option<WindowEntry> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            let t = i18n::table(self.language);
            if filtered.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new(t.no_matching_windows).weak().italics());
                });
                return;
            }

            TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(22.0))   // Icon
                .column(Column::exact(70.0))   // PID
                .column(Column::exact(160.0))  // Process
                .column(Column::exact(40.0))   // Arch
                .column(Column::remainder())   // Title
                .column(Column::exact(115.0))  // Status
                .column(Column::exact(150.0))  // Action
                .header(22.0, |mut header| {
                    header.col(|ui| { ui.strong(""); }); // Icon (no label)
                    let sc = sort_col;
                    let sa = sort_asc;
                    let arrow = |col: SortCol| -> &'static str {
                        if sc == col { if sa { " ▲" } else { " ▼" } } else { "" }
                    };
                    header.col(|ui| {
                        if ui.button(format!("{}{}", t.pid, arrow(SortCol::Pid))).clicked() {
                            new_sort = Some((SortCol::Pid, if sc == SortCol::Pid { !sa } else { true }));
                        }
                    });
                    header.col(|ui| {
                        if ui.button(format!("{}{}", t.process, arrow(SortCol::Process))).clicked() {
                            new_sort = Some((SortCol::Process, if sc == SortCol::Process { !sa } else { true }));
                        }
                    });
                    header.col(|ui| { ui.strong(t.arch); });
                    header.col(|ui| {
                        if ui.button(format!("{}{}", t.window_title, arrow(SortCol::Title))).clicked() {
                            new_sort = Some((SortCol::Title, if sc == SortCol::Title { !sa } else { true }));
                        }
                    });
                    header.col(|ui| {
                        if ui.button(format!("{}{}", t.status, arrow(SortCol::Status))).clicked() {
                            new_sort = Some((SortCol::Status, if sc == SortCol::Status { !sa } else { true }));
                        }
                    });
                    header.col(|ui| { ui.strong(t.action); });
                })
                .body(|mut body| {
                    for entry in &filtered {
                        let row_h = 26.0;
                        body.row(row_h, |mut row| {
                            // Process icon
                            row.col(|ui| {
                                if let Some(tex) = get_process_icon(
                                    &entry.process_name,
                                    &mut self.icon_cache,
                                    ctx,
                                    &self.exe_dir,
                                ) {
                                    ui.add(egui::Image::new(&*tex).max_size([16.0, 16.0].into()).fit_to_exact_size([16.0, 16.0].into()));
                                }
                            });
                            // PID
                            row.col(|ui| {
                                ui.monospace(entry.pid.to_string());
                            });
                            // Process name
                            row.col(|ui| {
                                ui.colored_label(
                                    theme::PROCESS,
                                    truncate(&entry.process_name, 24),
                                );
                            });
                            // Arch badge
                            row.col(|ui| {
                                if entry.is_32bit {
                                    ui.label(
                                        RichText::new("32")
                                            .color(theme::AMBER)
                                            .strong()
                                            .small(),
                                    );
                                } else {
                                    ui.label(
                                        RichText::new("64")
                                            .color(Color32::GRAY)
                                            .small(),
                                    );
                                }
                            });
                            // Title
                            row.col(|ui| {
                                ui.label(truncate(&entry.title, 72));
                            });
                            // Status badge
                            row.col(|ui| {
                                render_status_badge(ui, entry.affinity, self.language);
                            });
                            // Action button
                            row.col(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(i18n::table(self.language).strip_protection)
                                            .min_size([140.0, 20.0].into()),
                                    )
                                    .clicked()
                                {
                                    inject_target = Some((*entry).clone());
                                }
                            });
                        });
                    }
                });
        });

        if let Some(target) = inject_target {
            // Track in one-shot mode so re-protection can be detected.
            if !self.persistent_mode {
                self.one_shot_stripped.insert(target.pid, target.process_name.clone());
                if BROWSER_NAMES.iter().any(|b| target.process_name.eq_ignore_ascii_case(b)) {
                    let child_suffix = i18n::inject(self.language).child_suffix.to_string();
                    for child_pid in get_child_pids(target.pid) {
                        self.one_shot_stripped
                            .insert(child_pid, child_suffix.replace("{name}", &target.process_name));
                    }
                }
            }
            self.strip_window(&target);
        }

        // Re-protection modal
        // Shown when a one-shot-stripped process has re-applied capture protection.
        if !self.reapply_alert.is_empty() {
            let r = i18n::reapply(self.language);
            egui::Window::new(r.window_title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(r.description);
                    ui.add_space(4.0);
                    for (pid, name) in &self.reapply_alert {
                        ui.label(
                            RichText::new(format!("  • {name}  (PID {pid})"))
                                .color(theme::DANGER)
                                .strong(),
                        );
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(r.one_shot_hint);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(r.fix_hint)
                            .strong(),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if theme::primary(ui, r.switch_button).clicked() {
                            switch_and_restrip = true;
                        }
                        ui.add_space(8.0);
                        if ui.button(r.dismiss).clicked() {
                            dismiss_reapply = true;
                        }
                    });
                });
        }

        // Request repaint for pending toast batch
        if self.toast_batch_deadline.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // Apply modal actions
        if dismiss_reapply {
            self.reapply_alert.clear();
        }
        if switch_and_restrip {
            let alerted = std::mem::take(&mut self.reapply_alert);
            self.persistent_mode = true;
            self.one_shot_stripped.clear();
            for (pid, name) in &alerted {
                let is_32bit = all_windows
                    .iter()
                    .find(|w| w.pid == *pid)
                    .map(|w| w.is_32bit)
                    .unwrap_or(false);
                self.inject_pid_async(*pid, name.clone(), is_32bit);
            }
            self.set_status(
                i18n::status_msg(self.language).switched_to_persistent.replace("{count}", &alerted.len().to_string()),
                true,
            );
        }
    }
}

// Help window

/// Render one body block line-by-line so egui doesn't collapse manual
/// whitespace/indentation into a single wrapped paragraph.
/// Lines that look like shell commands are shown in monospace green.
fn render_help_body(ui: &mut egui::Ui, body: &str) {
    for raw_line in body.split('\n') {
        let line = raw_line.trim_end();
        if line.is_empty() {
            ui.add_space(5.0);
        } else if line.trim_start().starts_with("cargo ")
            || line.trim_start().starts_with("rustup ")
            || line.trim_start().starts_with("target\\")
            || line.trim_start().starts_with("target/")
            || line.trim_start().starts_with("git ")
            || line.trim_start().starts_with("python ")
        {
            ui.label(
                RichText::new(line)
                    .font(egui::FontId::monospace(12.5))
                    .color(theme::SUCCESS),
            );
        } else {
            ui.label(line);
        }
    }
}

fn render_help_window(ctx: &egui::Context, show: &mut bool, section: &mut usize, lang: Language) {
    if !*show {
        return;
    }

    let help = help_sections(lang);

    egui::Window::new("📖  Help")
        .open(show)
        .resizable(true)
        .collapsible(false)
        .default_size([720.0, 540.0])
        .min_size([480.0, 340.0])
        .show(ctx, |ui| {
            // Tab bar
            // Wraps automatically on narrow windows — no sidebar, no
            // horizontal_top, so the window never expands sideways.
            ui.horizontal_wrapped(|ui| {
                ui.add_space(2.0);
                for (i, (title, _)) in help.iter().enumerate() {
                    let selected = *section == i;
                    if theme::toggle_button(ui, selected, *title)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        *section = i;
                    }
                }
            });

            ui.separator();

            // Content area — full width, vertically scrollable
            egui::ScrollArea::vertical()
                .id_salt("help_content")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some((tab_title, sub_sections)) = help.get(*section) {
                        ui.add_space(4.0);
                        ui.label(RichText::new(*tab_title).size(18.0).strong());
                        ui.add_space(8.0);

                        for (heading, body) in sub_sections.iter() {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(*heading)
                                    .size(13.5)
                                    .strong()
                                    .color(theme::PROCESS),
                            );
                            ui.add_space(6.0);
                            render_help_body(ui, body);
                            ui.add_space(10.0);
                        }
                    }
                });
        });
}

// Settings window

#[allow(clippy::too_many_arguments)]
fn render_settings_window(
    ctx: &egui::Context,
    show: &mut bool,
    lang: Language,
    startup_enabled: bool,
    toast_enabled: bool,
    hotkey_enabled: bool,
    minimize_to_tray: bool,
    logging_enabled: bool,
    discord_rpc_enabled: bool,
    hotkey_mods: u32,
    hotkey_key: u32,
    hotkey_recording: bool,
    silent_startup: bool,
    fast_scan: bool,
    strip_on_launch: bool,
    process_rules: &[ProcessRule],
    rule_input: &mut String,
    rule_mode_input: &mut ProcessRuleMode,
    exclusions: &[String],
    exclusion_input: &mut String,
    toggle_startup: &mut bool,
    toggle_toast: &mut bool,
    toggle_hotkey: &mut bool,
    toggle_minimize_to_tray: &mut bool,
    toggle_logging: &mut bool,
    open_log_file: &mut bool,
    toggle_discord_rpc: &mut bool,
    start_hotkey_recording: &mut bool,
    toggle_silent_startup: &mut bool,
    toggle_fast_scan: &mut bool,
    toggle_strip_on_launch: &mut bool,
    do_export_config: &mut bool,
    do_import_config: &mut bool,
    add_process_rule: &mut bool,
    remove_process_rule: &mut Option<usize>,
    add_exclusion: &mut bool,
    remove_exclusion: &mut Option<usize>,
    exe_dir: &str,
) {
    if !*show {
        return;
    }

    let s = i18n::settings(lang);

    egui::Window::new(s.window_title)
        .open(show)
        .resizable(true)
        .collapsible(false)
        .default_size([400.0, 600.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);

            // Startup
            ui.label(RichText::new(s.startup).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            if theme::setting_row(ui, startup_enabled, s.start_with_windows,
                s.start_with_windows_desc) {
                *toggle_startup = true;
            }
            if theme::setting_row(ui, silent_startup, s.silent_startup,
                s.silent_startup_desc) {
                *toggle_silent_startup = true;
            }
            if theme::setting_row(ui, strip_on_launch, s.strip_on_launch,
                s.strip_on_launch_desc) {
                *toggle_strip_on_launch = true;
            }
            ui.add_space(12.0);

            // Windows Defender
            ui.label(RichText::new(s.windows_defender).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            ui.label(
                RichText::new(s.defender_hint)
                    .weak()
                    .small(),
            );
            ui.add_space(4.0);
            let mut ps_cmd = format!(
                "Add-MpPreference -ExclusionPath '{exe_dir}'\nAdd-MpPreference -ExclusionProcess 'capture_bypass_gui.exe'"
            );
            ui.add(
                egui::TextEdit::multiline(&mut ps_cmd)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(2)
                    .interactive(false),
            );
            ui.add_space(4.0);
            if ui.button(s.copy_commands).clicked() {
                ui.output_mut(|o| o.copied_text = ps_cmd.clone());
            }
            ui.add_space(12.0);

            // Notifications
            ui.label(RichText::new(s.notifications).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            if theme::setting_row(ui, toast_enabled, s.desktop_notifications,
                s.desktop_notifications_desc) {
                *toggle_toast = true;
            }
            ui.add_space(12.0);

            // Hotkey
            ui.label(RichText::new(s.hotkey).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);

            let combo = hotkey_display(hotkey_mods, hotkey_key);
            let hk_state = if hotkey_enabled { s.on } else { s.off };
            ui.horizontal(|ui| {
                let hk_label = format!("{}  {}", combo, hk_state);
                if theme::toggle_button(ui, hotkey_enabled, &hk_label)
                    .on_hover_text(s.toggle_hotkey_hint)
                    .clicked()
                {
                    *toggle_hotkey = true;
                }

                // Change-hotkey button / recording indicator
                if hotkey_recording {
                    theme::danger(ui, s.listening)
                        .on_hover_text(s.listening_hint);
                } else {
                    if theme::ghost(ui, s.change)
                        .on_hover_text(s.change_hint)
                        .clicked()
                    {
                        *start_hotkey_recording = true;
                    }
                }
            });
            ui.add_space(4.0);

            // Description
            let desc = if hotkey_recording {
                RichText::new(s.press_key_hint)
                    .size(10.5).color(theme::DANGER)
            } else {
                RichText::new(s.hotkey_desc.replace("{combo}", &combo))
                .size(10.5).color(theme::TEXT_DIM)
            };
            ui.label(desc);
            ui.add_space(12.0);

            // Window
            ui.label(RichText::new(s.window).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            if theme::setting_row(ui, minimize_to_tray, s.minimize_to_tray,
                s.minimize_to_tray_desc) {
                *toggle_minimize_to_tray = true;
            }
            ui.add_space(12.0);

            // Logging
            ui.label(RichText::new(s.logging).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            if theme::setting_row(ui, logging_enabled, s.injection_log_file,
                s.injection_log_file_desc) {
                *toggle_logging = true;
            }
            if logging_enabled {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    if ui
                        .button(s.open_log_file)
                        .on_hover_text("%APPDATA%\\capture-bypass\\injection.log")
                        .clicked()
                    {
                        *open_log_file = true;
                    }
                });
                ui.add_space(2.0);
                ui.label(
                    RichText::new("  %APPDATA%\\capture-bypass\\injection.log")
                        .size(10.5)
                        .color(theme::TEXT_DIM),
                );
            }
            ui.add_space(8.0);

            // Discord Rich Presence
            ui.label(RichText::new(s.discord).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            if theme::setting_row(ui, discord_rpc_enabled, s.discord_rich_presence,
                s.discord_rich_presence_desc) {
                *toggle_discord_rpc = true;
            }
            ui.add_space(8.0);

            // Detection
            ui.label(RichText::new(s.detection).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            if theme::setting_row(ui, fast_scan, s.fast_scan,
                s.fast_scan_desc) {
                *toggle_fast_scan = true;
            }
            ui.add_space(12.0);

            // Per-process Rules
            ui.label(RichText::new(s.per_process_rules).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(rule_input).desired_width(130.0).hint_text("process.exe"));
                let mode_label = match rule_mode_input {
                    ProcessRuleMode::AlwaysOneShot    => s.one_shot,
                    ProcessRuleMode::AlwaysPersistent => s.persistent,
                    ProcessRuleMode::Skip             => s.skip,
                };
                if ui.button(mode_label).clicked() {
                    *rule_mode_input = match rule_mode_input {
                        ProcessRuleMode::AlwaysOneShot    => ProcessRuleMode::AlwaysPersistent,
                        ProcessRuleMode::AlwaysPersistent => ProcessRuleMode::Skip,
                        ProcessRuleMode::Skip             => ProcessRuleMode::AlwaysOneShot,
                    };
                }
                if ui.button(s.add_rule).clicked() {
                    *add_process_rule = true;
                }
            });
            ui.add_space(4.0);
            let rules_snapshot: Vec<(usize, String, String)> = process_rules.iter().enumerate()
                .map(|(i, r)| {
                    let mode_str = match r.mode {
                        ProcessRuleMode::AlwaysOneShot    => s.one_shot,
                        ProcessRuleMode::AlwaysPersistent => s.persistent,
                        ProcessRuleMode::Skip             => s.skip,
                    };
                    (i, r.process_name.clone(), mode_str.to_string())
                })
                .collect();
            for (i, name, mode_str) in rules_snapshot {
                ui.horizontal(|ui| {
                    ui.label(format!("[{name}]  [{mode_str}]"));
                    if ui.small_button("✕").clicked() {
                        *remove_process_rule = Some(i);
                    }
                });
            }
            ui.add_space(12.0);

            // Exclusions
            ui.label(RichText::new(s.exclusions).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(exclusion_input).desired_width(150.0).hint_text("process.exe"));
                if ui.button(s.add_exclusion).clicked() {
                    *add_exclusion = true;
                }
            });
            ui.add_space(4.0);
            let excl_snapshot: Vec<(usize, String)> = exclusions.iter().enumerate()
                .map(|(i, e)| (i, e.clone()))
                .collect();
            for (i, name) in excl_snapshot {
                ui.horizontal(|ui| {
                    ui.label(format!("[{name}]"));
                    if ui.small_button("✕").clicked() {
                        *remove_exclusion = Some(i);
                    }
                });
            }
            ui.add_space(12.0);

            // Config export/import
            ui.label(RichText::new(s.config).strong().color(theme::ACCENT));
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(s.export).on_hover_text(s.export_hint).clicked() {
                    *do_export_config = true;
                }
                ui.add_space(8.0);
                if ui.button(s.import).on_hover_text(s.import_hint).clicked() {
                    *do_import_config = true;
                }
            });
            ui.add_space(4.0);
            }); // end ScrollArea
        });
}

// Status badge

fn render_status_badge(ui: &mut Ui, affinity: u32, lang: Language) {
    let b = i18n::badge(lang);
    match affinity {
        WDA_EXCLUDEFROMCAPTURE => {
            ui.label(RichText::new(b.protected).monospace().size(11.0).color(theme::DANGER).strong());
        }
        WDA_MONITOR => {
            ui.label(RichText::new(b.monitor).monospace().size(11.0).color(theme::AMBER).strong());
        }
        WDA_NONE => {
            ui.label(RichText::new(b.clear).monospace().size(11.0).color(theme::SUCCESS).strong());
        }
        _ => {
            ui.label(RichText::new("● ?").monospace().size(11.0).color(theme::TEXT_FAINT));
        }
    }
}

// Tray setup

fn make_tray_icon(r: u8, g: u8, b: u8) -> Option<tray_icon::Icon> {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for i in 0..(size * size) as usize {
        rgba[i * 4]     = r;
        rgba[i * 4 + 1] = g;
        rgba[i * 4 + 2] = b;
        rgba[i * 4 + 3] = 0xFF;
    }
    tray_icon::Icon::from_rgba(rgba, size, size).ok()
}

fn build_tray(lang: Language) -> (
    Option<tray_icon::TrayIcon>,
    Option<tray_icon::menu::MenuId>,
    Option<tray_icon::menu::MenuId>,
) {
    use tray_icon::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        Icon, TrayIconBuilder,
    };

    let t = i18n::tray(lang);

    // Generate a 32×32 solid blue square as the tray icon
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for i in 0..(size * size) as usize {
        rgba[i * 4] = 0x44; // R
        rgba[i * 4 + 1] = 0x88; // G
        rgba[i * 4 + 2] = 0xFF; // B
        rgba[i * 4 + 3] = 0xFF; // A
    }

    let icon = match Icon::from_rgba(rgba, size, size) {
        Ok(i) => i,
        Err(_) => return (None, None, None),
    };

    let open_item = MenuItem::new(t.open, true, None);
    let quit_item = MenuItem::new(t.quit, true, None);
    let open_id = open_item.id().clone();
    let quit_id = quit_item.id().clone();

    let menu = Menu::new();
    let sep = PredefinedMenuItem::separator();
    if menu
        .append_items(&[
            &open_item as &dyn tray_icon::menu::IsMenuItem,
            &sep,
            &quit_item,
        ])
        .is_err()
    {
        return (None, None, None);
    }

    match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .with_tooltip("capture-bypass")
        .build()
    {
        Ok(tray) => (Some(tray), Some(open_id), Some(quit_id)),
        Err(_) => (None, None, None),
    }
}

// Window enumeration

/// Callback state threaded through EnumWindows via lparam.
struct EnumState {
    entries: Vec<WindowEntry>,
}

unsafe extern "system" fn enum_windows_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);

    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    let mut title_buf = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title_buf);
    if len == 0 {
        return TRUE;
    }
    let title = String::from_utf16_lossy(&title_buf[..len as usize]);

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    let process_name = get_process_name(pid).unwrap_or_else(|| "<unknown>".into());

    let mut affinity: u32 = 0;
    let _ = GetWindowDisplayAffinity(hwnd, &mut affinity);
    let is_protected = affinity == WDA_EXCLUDEFROMCAPTURE || affinity == WDA_MONITOR;
    let is_32bit = is_process_32bit(pid);

    state.entries.push(WindowEntry {
        pid,
        process_name,
        title,
        affinity,
        is_protected,
        is_32bit,
    });

    TRUE
}

fn enumerate_windows() -> Vec<WindowEntry> {
    let mut state = EnumState {
        entries: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_cb),
            LPARAM(&mut state as *mut EnumState as isize),
        );
    }
    state.entries
}

// Windows helpers

fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .ok()?;
        let full = String::from_utf16_lossy(&buf[..size as usize]);
        std::path::Path::new(&full)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }
}

fn is_process_32bit(pid: u32) -> bool {
    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let mut wow64 = BOOL(0);
        let _ = IsWow64Process(handle, &mut wow64);
        wow64.as_bool()
    }
}

fn get_child_pids(parent_pid: u32) -> Vec<u32> {
    unsafe {
        let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut children = Vec::new();
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                if entry.th32ParentProcessID == parent_pid {
                    children.push(entry.th32ProcessID);
                }
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        children
    }
}

// Utilities

fn truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        format!("{}…", chars[..max_chars].iter().collect::<String>())
    } else {
        s.to_string()
    }
}

// Windows startup registry helpers

const STARTUP_RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const STARTUP_VALUE_NAME: &str = "capture-bypass";

/// Returns true if the startup registry entry currently exists.
fn read_startup_reg() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run) = hkcu.open_subkey(STARTUP_RUN_KEY) {
        run.get_value::<String, _>(STARTUP_VALUE_NAME).is_ok()
    } else {
        false
    }
}

/// Writes (enable=true) or deletes (enable=false) the startup entry.
/// Returns true on success.
fn write_startup_reg(enable: bool) -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(run) = hkcu.open_subkey_with_flags(STARTUP_RUN_KEY, KEY_WRITE) else {
        return false;
    };
    if enable {
        // Quote the path in case it contains spaces (e.g. Program Files)
        let exe = std::env::current_exe().unwrap_or_default();
        let value = format!("\"{}\"", exe.display());
        run.set_value(STARTUP_VALUE_NAME, &value).is_ok()
    } else {
        match run.delete_value(STARTUP_VALUE_NAME) {
            Ok(()) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true, // already gone
            Err(_) => false,
        }
    }
}

// Global hotkey helpers

// Convert an egui Key to a Windows virtual-key code.
// Only letter and digit keys are supported for hotkey binding.
fn egui_key_to_vk(key: egui::Key) -> Option<u32> {
    use egui::Key::*;
    match key {
        A => Some(b'A' as u32), B => Some(b'B' as u32), C => Some(b'C' as u32),
        D => Some(b'D' as u32), E => Some(b'E' as u32), F => Some(b'F' as u32),
        G => Some(b'G' as u32), H => Some(b'H' as u32), I => Some(b'I' as u32),
        J => Some(b'J' as u32), K => Some(b'K' as u32), L => Some(b'L' as u32),
        M => Some(b'M' as u32), N => Some(b'N' as u32), O => Some(b'O' as u32),
        P => Some(b'P' as u32), Q => Some(b'Q' as u32), R => Some(b'R' as u32),
        S => Some(b'S' as u32), T => Some(b'T' as u32), U => Some(b'U' as u32),
        V => Some(b'V' as u32), W => Some(b'W' as u32), X => Some(b'X' as u32),
        Y => Some(b'Y' as u32), Z => Some(b'Z' as u32),
        Num0 => Some(b'0' as u32), Num1 => Some(b'1' as u32),
        Num2 => Some(b'2' as u32), Num3 => Some(b'3' as u32),
        Num4 => Some(b'4' as u32), Num5 => Some(b'5' as u32),
        Num6 => Some(b'6' as u32), Num7 => Some(b'7' as u32),
        Num8 => Some(b'8' as u32), Num9 => Some(b'9' as u32),
        F1  => Some(0x70), F2  => Some(0x71), F3  => Some(0x72), F4  => Some(0x73),
        F5  => Some(0x74), F6  => Some(0x75), F7  => Some(0x76), F8  => Some(0x77),
        F9  => Some(0x78), F10 => Some(0x79), F11 => Some(0x7A), F12 => Some(0x7B),
        _ => None,
    }
}

// Convert egui modifier flags to Windows MOD_* bitmask.
fn egui_mods_to_win(mods: egui::Modifiers) -> u32 {
    let mut flags = 0u32;
    if mods.ctrl  { flags |= 0x02; } // MOD_CONTROL
    if mods.shift { flags |= 0x04; } // MOD_SHIFT
    if mods.alt   { flags |= 0x01; } // MOD_ALT
    flags
}

// Format a (mods, vk) pair as a human-readable string like "Ctrl+Shift+B".
fn hotkey_display(mods: u32, vk: u32) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if mods & 0x02 != 0 { parts.push("Ctrl"); }
    if mods & 0x04 != 0 { parts.push("Shift"); }
    if mods & 0x01 != 0 { parts.push("Alt"); }
    let key_name: String = match vk {
        0x70..=0x7B => format!("F{}", vk - 0x6F), // F1–F12
        k if k >= b'A' as u32 && k <= b'Z' as u32 => {
            char::from(k as u8).to_string()
        }
        k if k >= b'0' as u32 && k <= b'9' as u32 => {
            char::from(k as u8).to_string()
        }
        _ => format!("0x{vk:02X}"),
    };
    parts.push(&key_name);
    // Can't join borrowed Strings directly, so do it manually
    parts.join("+")
}

fn register_hotkey(id: i32, mods: u32, key: u32) {
    unsafe {
        let _ = RegisterHotKey(None, id, HOT_KEY_MODIFIERS(mods), key);
    }
}

fn unregister_hotkey(id: i32) {
    unsafe {
        let _ = UnregisterHotKey(None, id);
    }
}

// Toast notification helper
// Uses a simple PowerShell one-liner so we don't need the WinRT COM machinery.

fn send_toast(title: &str, body: &str) {
    // WinRT ToastNotificationManager silently fails when called from an
    // elevated (Administrator) process — Windows blocks the notification
    // queue for admin tokens in many Windows 10/11 configurations.
    //
    // System.Windows.Forms.NotifyIcon.ShowBalloonTip() takes the Win32
    // Shell_NotifyIcon path and reliably works from elevated processes.
    // It briefly shows a secondary tray icon while the balloon is visible,
    // then cleans itself up.  System.Windows.Forms is always available via
    // .NET Framework on Windows 10/11 (PowerShell 5 runs on .NET 4.x).
    //
    // Single-quote the values so PowerShell treats them literally — no
    // variable expansion, no escaping headaches.  The only character that
    // needs escaping inside PS single-quoted strings is ' itself → ''.
    let title_ps = title.replace('\'', "''");
    let body_ps  = body.replace('\'',  "''");
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; \
         Add-Type -AssemblyName System.Drawing; \
         $n = New-Object System.Windows.Forms.NotifyIcon; \
         $n.Icon = [System.Drawing.SystemIcons]::Information; \
         $n.BalloonTipIcon  = 'Info'; \
         $n.BalloonTipTitle = '{title_ps}'; \
         $n.BalloonTipText  = '{body_ps}'; \
         $n.Visible = $true; \
         $n.ShowBalloonTip(5000); \
         Start-Sleep -Seconds 6; \
         $n.Dispose()"
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden",
               "-Command", &script])
        .spawn();
}

// Auto-update check

/// Parse "X.Y.Z" (with or without a leading 'v') into a comparable tuple.
fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_start_matches('v');
    let mut parts = s.splitn(4, '.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()
        // strip any pre-release suffix like "-alpha.1"
        .map(|p| p.split('-').next().unwrap_or(p))
        .and_then(|p| p.parse().ok())?;
    Some((major, minor, patch))
}

// Which release asset does this install need?
//   installed (unins000.exe present) → the Inno Setup installer
//   portable (zip extraction)        → the portable zip
// Returns (asset file name, is_portable).
fn update_asset(tag: &str, exe_dir: &std::path::Path) -> (String, bool) {
    // Detect architecture — ARM64 native build gets the arm64 asset
    #[cfg(target_arch = "aarch64")]
    let arch_suffix = "arm64";
    #[cfg(not(target_arch = "aarch64"))]
    let arch_suffix = "x64";

    let is_portable = !exe_dir.join("unins000.exe").exists();
    let name = if is_portable {
        format!("capture-bypass-{tag}-portable-{arch_suffix}.zip")
    } else {
        format!("capture-bypass-setup-{tag}-{arch_suffix}.exe")
    };
    (name, is_portable)
}

// Kick off the background update download.
// Portable installs download the portable zip; installed ones get the installer.
fn start_download(
    tag: String,
    download_state: &mut DownloadState,
    download_rx: &mut Option<Receiver<DownloadMsg>>,
    exe_dir: &std::path::Path,
) {
    let (asset, _is_portable) = update_asset(&tag, exe_dir);

    *download_state = DownloadState::Downloading(0.0);
    let (tx, rx) = mpsc::channel::<DownloadMsg>();
    *download_rx = Some(rx);

    std::thread::Builder::new()
        .name("update-download".into())
        .spawn(move || {
            let url = format!(
                "https://github.com/levi52/capture-bypass/releases/download/v{tag}/{asset}"
            );
            let dest = std::env::temp_dir().join(&asset);

            // Download with progress
            let resp = match ureq::get(&url)
                .set("User-Agent", &format!("capture-bypass/{}", env!("CARGO_PKG_VERSION")))
                .call()
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(DownloadMsg::Failed(format!(
                        "Download failed ({asset}): {e}"
                    )));
                    return;
                }
            };

            let total = resp.header("content-length")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);

            let mut reader = resp.into_reader();
            let mut file = match std::fs::File::create(&dest) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(DownloadMsg::Failed(format!("Can't create temp file: {e}")));
                    return;
                }
            };

            let mut downloaded: u64 = 0;
            let mut buf = vec![0u8; 65_536];
            loop {
                let n = match std::io::Read::read(&mut reader, &mut buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        let _ = tx.send(DownloadMsg::Failed(format!("Read error: {e}")));
                        return;
                    }
                };
                if let Err(e) = std::io::Write::write_all(&mut file, &buf[..n]) {
                    let _ = tx.send(DownloadMsg::Failed(format!("Write error: {e}")));
                    return;
                }
                downloaded += n as u64;
                if total > 0 {
                    let _ = tx.send(DownloadMsg::Progress(downloaded as f32 / total as f32));
                }
            }
            drop(file);

            // Verify SHA256 against the release's SHA256SUMS.txt
            let _ = tx.send(DownloadMsg::Verifying);
            let sums_url = format!(
                "https://github.com/levi52/capture-bypass/releases/download/v{tag}/SHA256SUMS.txt"
            );
            if let Ok(sums_resp) = ureq::get(&sums_url)
                .set("User-Agent", &format!("capture-bypass/{}", env!("CARGO_PKG_VERSION")))
                .call()
            {
                if let Ok(body) = sums_resp.into_string() {
                    // Each line: "<hash>  <filename>"
                    let expected = body.lines()
                        .find(|l| l.ends_with(&asset))
                        .and_then(|l| l.split_whitespace().next())
                        .map(|s| s.to_lowercase());

                    if let Some(expected_hash) = expected {
                        match sha256_of_file(&dest) {
                            Ok(actual) if actual == expected_hash => {}
                            Ok(actual) => {
                                let _ = std::fs::remove_file(&dest);
                                let _ = tx.send(DownloadMsg::Failed(format!(
                                    "SHA256 mismatch — expected {expected_hash}, got {actual}"
                                )));
                                return;
                            }
                            Err(e) => {
                                let _ = tx.send(DownloadMsg::Failed(
                                    format!("SHA256 check failed: {e}")
                                ));
                                return;
                            }
                        }
                    }
                    // If we couldn't find the file in SHA256SUMS.txt we proceed anyway
                    // (older releases may not have had it).
                }
            }

            let _ = tx.send(DownloadMsg::Done(dest));
        })
        .ok();
}

// Apply a verified update and relaunch.
//
//   installer (.exe) → run it silently, wait, relaunch
//   portable   (.zip) → extract to a staging dir, copy over the exe dir, relaunch
//
// A hidden PowerShell does the work *after* this process exits, because the
// running capture_bypass_gui.exe holds a lock on its own file.  The app calls
// std::process::exit(0) immediately after spawning, so the 2 s sleep is only a
// safety margin for the OS to release the handle.
fn apply_update(artifact: &std::path::Path, exe_dir: &std::path::Path) {
    let our_exe = std::env::current_exe()
        .unwrap_or_else(|_| exe_dir.join("capture_bypass_gui.exe"));
    let gui_str = our_exe.display().to_string().replace('\'', "''");

    let script = if artifact.extension().map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false) {
        let zip_str = artifact.display().to_string().replace('\'', "''");
        let stage = std::env::temp_dir().join("capture-bypass-update-stage");
        let stage_str = stage.display().to_string().replace('\'', "''");
        let dir_str = exe_dir.display().to_string().replace('\'', "''");
        format!(
            // 1. Wait for this process to fully exit and release the file lock.
            // 2. Extract the portable zip into a clean staging directory.
            // 3. Copy every file (including the x86\ subfolder) over the
            //    install directory, overwriting the old build.
            // 4. Relaunch with -Verb RunAs so the new exe keeps admin rights
            //    (inherits the already-elevated token from this session).
            "Start-Sleep -Seconds 2; \
             if (Test-Path -LiteralPath '{stage_str}') {{ \
               Remove-Item -LiteralPath '{stage_str}' -Recurse -Force }}; \
             Expand-Archive -LiteralPath '{zip_str}' -DestinationPath '{stage_str}' -Force; \
             Copy-Item -Path (Join-Path '{stage_str}' '*') \
                       -Destination '{dir_str}' -Recurse -Force; \
             Remove-Item -LiteralPath '{stage_str}' -Recurse -Force; \
             Start-Process '{gui_str}' -Verb RunAs"
        )
    } else {
        let installer_str = artifact.display().to_string().replace('\'', "''");
        format!(
            // 1. Wait for this process to fully exit and release the file lock
            //    on capture_bypass_gui.exe before the installer replaces it.
            // 2. Run the installer silently, suppressing all dialogs.
            // 3. Relaunch with -Verb RunAs so the new exe gets admin rights
            //    without a visible UAC prompt.
            "Start-Sleep -Seconds 2; \
             Start-Process '{installer_str}' \
               -ArgumentList '/SILENT /SUPPRESSMSGBOXES /NORESTART' \
               -Wait; \
             Start-Process '{gui_str}' -Verb RunAs"
        )
    };

    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden",
               "-Command", &script])
        .spawn();
    // Exit immediately — ViewportCommand::Close is async and keeps the process
    // alive long enough to lock the exe file, causing the replacement to fail.
    std::process::exit(0);
}

fn sha256_of_file(path: &std::path::Path)
    -> Result<String, Box<dyn std::error::Error + Send + Sync>>
{
    // Shell out to PowerShell — it always has Get-FileHash on Windows.
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-Command",
            &format!(
                "(Get-FileHash '{}' -Algorithm SHA256).Hash.ToLower()",
                path.display()
            ),
        ])
        .output()?;
    let hash = String::from_utf8(out.stdout)?.trim().to_lowercase();
    if hash.len() == 64 {
        Ok(hash)
    } else {
        Err(format!("Unexpected hash output: {hash}").into())
    }
}

fn check_for_update() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let current = env!("CARGO_PKG_VERSION");
    let resp: serde_json::Value = ureq::get(
        "https://api.github.com/repos/levi52/capture-bypass/releases/latest",
    )
    .set("User-Agent", &format!("capture-bypass/{current}"))
    .call()?
    .into_json()?;

    let raw_tag = resp["tag_name"].as_str().unwrap_or("").trim().to_string();
    if raw_tag.is_empty() {
        return Ok(None);
    }

    // Only show the banner when the remote version is strictly newer.
    // Semver comparison avoids false positives when Cargo.toml and the
    // GitHub tag differ in format (e.g. "1.0.0" vs "v1.0.0").
    let remote_ver = parse_semver(&raw_tag);
    let current_ver = parse_semver(current);
    match (current_ver, remote_ver) {
        (Some(c), Some(r)) if r > c => Ok(Some(raw_tag.trim_start_matches('v').to_string())),
        _ => Ok(None),
    }
}

// Process icon helpers

/// Resolve the full executable path from process name by scanning
/// the window list process names — used to locate the exe for SHGetFileInfoW.
fn find_exe_path(process_name: &str) -> Option<PathBuf> {
    // Walk all running processes and return the first full path that matches the name
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry: windows::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W =
            std::mem::zeroed();
        entry.dwSize =
            std::mem::size_of::<windows::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W>()
                as u32;
        if windows::Win32::System::Diagnostics::ToolHelp::Process32FirstW(snap, &mut entry).is_err() {
            return None;
        }
        loop {
            let name_raw: Vec<u16> = entry.szExeFile.iter().copied().take_while(|&c| c != 0).collect();
            let name = String::from_utf16_lossy(&name_raw);
            if name.eq_ignore_ascii_case(process_name) {
                let pid = entry.th32ProcessID;
                if let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                    let mut buf = [0u16; 1024];
                    let mut sz = buf.len() as u32;
                    if QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32,
                        windows::core::PWSTR(buf.as_mut_ptr()), &mut sz).is_ok()
                    {
                        return Some(PathBuf::from(String::from_utf16_lossy(&buf[..sz as usize]).as_str()));
                    }
                }
            }
            if windows::Win32::System::Diagnostics::ToolHelp::Process32NextW(snap, &mut entry).is_err() {
                break;
            }
        }
    }
    None
}

fn load_icon_for_process(
    process_name: &str,
    ctx: &egui::Context,
) -> Option<egui::TextureHandle> {
    let exe_path = find_exe_path(process_name)?;
    let path_wide: Vec<u16> = exe_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut sfi: SHFILEINFOW = std::mem::zeroed();
        let flags = SHGFI_ICON | SHGFI_SMALLICON;
        let res = SHGetFileInfoW(
            windows::core::PCWSTR(path_wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut sfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if res == 0 {
            return None;
        }
        let hicon = sfi.hIcon;

        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            return None;
        }

        let hbmp = icon_info.hbmColor;
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;

        let hdc = GetDC(None);
        // First call: populate width/height
        GetDIBits(hdc, hbmp, 0, 0, None, &mut bmi, DIB_RGB_COLORS);
        let w = bmi.bmiHeader.biWidth.unsigned_abs() as usize;
        let h = bmi.bmiHeader.biHeight.unsigned_abs() as usize;

        if w == 0 || h == 0 {
            ReleaseDC(None, hdc);
            return None;
        }

        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0;
        bmi.bmiHeader.biHeight = -(h as i32); // top-down DIB

        let mut pixels = vec![0u8; w * h * 4];
        GetDIBits(hdc, hbmp, 0, h as u32, Some(pixels.as_mut_ptr().cast()), &mut bmi, DIB_RGB_COLORS);
        ReleaseDC(None, hdc);

        // Windows returns BGRA — swap B and R to get RGBA
        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }

        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyIcon(hicon);

        let image = egui::ColorImage::from_rgba_unmultiplied([w, h], &pixels);
        Some(ctx.load_texture(
            format!("icon_{process_name}"),
            image,
            egui::TextureOptions::LINEAR,
        ))
    }
}

/// Returns cached icon, loading it on first access.
fn get_process_icon<'a>(
    process_name: &str,
    cache: &'a mut HashMap<String, Option<egui::TextureHandle>>,
    ctx: &egui::Context,
    _exe_dir: &Path,
) -> Option<&'a egui::TextureHandle> {
    cache
        .entry(process_name.to_string())
        .or_insert_with(|| load_icon_for_process(process_name, ctx))
        .as_ref()
}

// Entry point

fn main() -> eframe::Result<()> {
    let cfg = load_config();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("capture-bypass")
        .with_inner_size([1100.0, 650.0])
        .with_min_inner_size([900.0, 550.0]);

    if cfg.silent_startup {
        viewport = viewport.with_visible(false);
    }

    let options = eframe::NativeOptions {
        viewport,
        // Don't restore the saved window position — let the OS place it on
        // the primary monitor each launch.  We manage our own config persistence
        // via config.toml for everything that matters.
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        "capture-bypass",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
