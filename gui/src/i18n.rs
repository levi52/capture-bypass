use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh")]
    Chinese,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "EN",
            Language::Chinese => "中",
        }
    }
}

// ── Header ───────────────────────────────────────────────────────────────

pub struct HeaderStrings {
    pub admin: &'static str,
    pub help: &'static str,
    pub settings: &'static str,
    pub log_on: &'static str,
    pub log_off: &'static str,
    pub stress_test: &'static str,
    pub mode: &'static str,
    pub one_shot: &'static str,
    pub persistent: &'static str,
    pub strip_all_protected: &'static str,
    pub refresh: &'static str,
    pub update_available: &'static str,
    pub restart_to_update: &'static str,
    pub retry_update: &'static str,
    pub downloading: &'static str,
    pub verifying: &'static str,
}

const HEADER_EN: HeaderStrings = HeaderStrings {
    admin: "ADMIN",
    help: "📖 Help",
    settings: "Settings",
    log_on: "Log ON",
    log_off: "Log",
    stress_test: "Stress Test",
    mode: "Mode",
    one_shot: "One-shot",
    persistent: "Persistent",
    strip_all_protected: "Strip All Protected",
    refresh: "⟳  Refresh",
    update_available: "🆕 v{tag} available",
    restart_to_update: "🔄 Restart to update",
    retry_update: "✗ Retry update?",
    downloading: "⬇ {pct}%",
    verifying: "🔍 …",
};

const HEADER_ZH: HeaderStrings = HeaderStrings {
    admin: "管理员",
    help: "📖 帮助",
    settings: "设置",
    log_on: "日志 开启",
    log_off: "日志",
    stress_test: "压力测试",
    mode: "模式",
    one_shot: "一次性",
    persistent: "持久模式",
    strip_all_protected: "剥离全部保护",
    refresh: "⟳  刷新",
    update_available: "🆕 v{tag} 可用",
    restart_to_update: "🔄 重启以更新",
    retry_update: "✗ 重试更新？",
    downloading: "⬇ {pct}%",
    verifying: "🔍 …",
};

pub fn header(lang: Language) -> &'static HeaderStrings {
    match lang {
        Language::English => &HEADER_EN,
        Language::Chinese => &HEADER_ZH,
    }
}

// ── Header hover texts ──────────────────────────────────────────────────

pub struct HeaderHoverStrings {
    pub click_to_update: &'static str,
    pub downloading_bg: &'static str,
    pub verifying_download: &'static str,
    pub ready_to_install: &'static str,
    pub settings_hint: &'static str,
    pub log_hint: &'static str,
    pub stress_test_hint: &'static str,
}

const HEADER_HOVER_EN: HeaderHoverStrings = HeaderHoverStrings {
    click_to_update: "Click to update",
    downloading_bg: "Downloading v{tag} in the background…",
    verifying_download: "Verifying download…",
    ready_to_install: "v{tag} is ready — click to install and relaunch",
    settings_hint: "Startup, notifications, hotkey",
    log_hint: "Toggle the injection log panel",
    stress_test_hint: "Launch stress_tester.exe — a self-protecting window.\n\
        Also tests Scenario A (process scan) and Scenario B\n\
        (module ejection) to verify stealth defences work.",
};

const HEADER_HOVER_ZH: HeaderHoverStrings = HeaderHoverStrings {
    click_to_update: "点击以更新",
    downloading_bg: "正在后台下载 v{tag}…",
    verifying_download: "正在验证下载…",
    ready_to_install: "v{tag} 已就绪 — 点击安装并重启",
    settings_hint: "启动、通知、热键",
    log_hint: "切换注入日志面板",
    stress_test_hint: "启动 stress_tester.exe — 自保护窗口。\n\
        同时测试场景A（进程扫描）和场景B\n\
        （模块弹出）以验证隐身防御功能。",
};

pub fn header_hover(lang: Language) -> &'static HeaderHoverStrings {
    match lang {
        Language::English => &HEADER_HOVER_EN,
        Language::Chinese => &HEADER_HOVER_ZH,
    }
}

// ── Filter bar ──────────────────────────────────────────────────────────

pub struct FilterStrings {
    pub filter: &'static str,
    pub hint: &'static str,
    pub protected_only: &'static str,
    pub auto_inject_on: &'static str,
    pub auto_inject_off: &'static str,
    pub n_protected: &'static str,
    pub zero_protected: &'static str,
    pub watch: &'static str,
    pub add: &'static str,
}

const FILTER_EN: FilterStrings = FilterStrings {
    filter: "Filter:",
    hint: "process, title, or PID",
    protected_only: "Protected only",
    auto_inject_on: "Auto-inject ON",
    auto_inject_off: "Auto-inject OFF",
    n_protected: "● {n} protected",
    zero_protected: "● 0 protected",
    watch: "Watch:",
    add: "+ Add",
};

const FILTER_ZH: FilterStrings = FilterStrings {
    filter: "筛选：",
    hint: "进程名、标题或PID",
    protected_only: "仅受保护的",
    auto_inject_on: "自动注入 开启",
    auto_inject_off: "自动注入 关闭",
    n_protected: "● {n} 个受保护",
    zero_protected: "● 0 个受保护",
    watch: "监视：",
    add: "+ 添加",
};

pub fn filter(lang: Language) -> &'static FilterStrings {
    match lang {
        Language::English => &FILTER_EN,
        Language::Chinese => &FILTER_ZH,
    }
}

// ── Table ───────────────────────────────────────────────────────────────

pub struct TableStrings {
    pub pid: &'static str,
    pub process: &'static str,
    pub arch: &'static str,
    pub window_title: &'static str,
    pub status: &'static str,
    pub action: &'static str,
    pub strip_protection: &'static str,
    pub no_matching_windows: &'static str,
}

const TABLE_EN: TableStrings = TableStrings {
    pid: "PID",
    process: "Process",
    arch: "Arch",
    window_title: "Window Title",
    status: "Status",
    action: "Action",
    strip_protection: "Strip Protection",
    no_matching_windows: "No matching windows.",
};

const TABLE_ZH: TableStrings = TableStrings {
    pid: "PID",
    process: "进程",
    arch: "架构",
    window_title: "窗口标题",
    status: "状态",
    action: "操作",
    strip_protection: "剥离保护",
    no_matching_windows: "没有匹配的窗口。",
};

pub fn table(lang: Language) -> &'static TableStrings {
    match lang {
        Language::English => &TABLE_EN,
        Language::Chinese => &TABLE_ZH,
    }
}

// ── Status badges ───────────────────────────────────────────────────────

pub struct BadgeStrings {
    pub protected: &'static str,
    pub monitor: &'static str,
    pub clear: &'static str,
}

const BADGE_EN: BadgeStrings = BadgeStrings {
    protected: "● PROTECTED",
    monitor: "● MONITOR",
    clear: "● CLEAR",
};

const BADGE_ZH: BadgeStrings = BadgeStrings {
    protected: "● 受保护",
    monitor: "● 监控中",
    clear: "● 正常",
};

pub fn badge(lang: Language) -> &'static BadgeStrings {
    match lang {
        Language::English => &BADGE_EN,
        Language::Chinese => &BADGE_ZH,
    }
}

// ── Injection messages (format templates) ──────────────────────────────

pub struct InjectStrings {
    pub dll_not_found: &'static str,
    pub stripped_pid: &'static str,
    pub pid_error: &'static str,
    pub child_suffix: &'static str,
    pub stripping_n: &'static str,
    pub escalated: &'static str,
    pub auto_stripped: &'static str,
    pub verb_pid: &'static str,
    pub blocked_by_os: &'static str,
    pub warning: &'static str,
}

const INJECT_EN: InjectStrings = InjectStrings {
    dll_not_found: "✗  DLL not found ({arch}): {path}  — build with cargo build --release",
    stripped_pid: "✓  Stripped PID {pid} ({name})",
    pid_error: "✗  PID {pid} ({name}): {e}",
    child_suffix: "{name} (child)",
    stripping_n: "⚡ Stripping {count} protected process(es)…",
    escalated: "🔄 Escalated→persistent",
    auto_stripped: "🤖 Auto-stripped",
    verb_pid: "{verb} {name} (PID {pid})",
    blocked_by_os: "⛔ {name} (PID {pid}) blocked by OS policy: {reason}",
    warning: "⚠️ {name} (PID {pid}): {e}",
};

const INJECT_ZH: InjectStrings = InjectStrings {
    dll_not_found: "✗  DLL 未找到 ({arch})：{path}  — 请执行 cargo build --release 编译",
    stripped_pid: "✓  已剥离 PID {pid} ({name})",
    pid_error: "✗  PID {pid}（{name}）：{e}",
    child_suffix: "{name}（子进程）",
    stripping_n: "⚡ 正在剥离 {count} 个受保护进程…",
    escalated: "🔄 升级→持久模式",
    auto_stripped: "🤖 自动剥离",
    verb_pid: "{verb} {name} (PID {pid})",
    blocked_by_os: "⛔ {name} (PID {pid}) 被系统策略阻止：{reason}",
    warning: "⚠️ {name} (PID {pid})：{e}",
};

pub fn inject(lang: Language) -> &'static InjectStrings {
    match lang {
        Language::English => &INJECT_EN,
        Language::Chinese => &INJECT_ZH,
    }
}

// ── Status messages ─────────────────────────────────────────────────────

pub struct StatusStrings {
    pub mode_persistent: &'static str,
    pub mode_one_shot: &'static str,
    pub auto_inject_active: &'static str,
    pub auto_inject_disabled: &'static str,
    pub refreshing: &'static str,
    pub launched_stress_tester: &'static str,
    pub could_not_launch_stress_tester: &'static str,
    pub stress_tester_not_found: &'static str,
    pub launch_strip: &'static str,
    pub added_to_startup: &'static str,
    pub removed_from_startup: &'static str,
    pub could_not_write_startup: &'static str,
    pub toast_on: &'static str,
    pub toast_off: &'static str,
    pub hotkey_registered: &'static str,
    pub hotkey_unregistered: &'static str,
    pub hotkey_set: &'static str,
    pub closes_to_tray: &'static str,
    pub exits_app: &'static str,
    pub log_on: &'static str,
    pub log_off: &'static str,
    pub discord_on: &'static str,
    pub discord_off: &'static str,
    pub silent_startup_on: &'static str,
    pub silent_startup_off: &'static str,
    pub fast_scan_on: &'static str,
    pub fast_scan_off: &'static str,
    pub strip_on_launch_on: &'static str,
    pub strip_on_launch_off: &'static str,
    pub config_exported: &'static str,
    pub export_failed: &'static str,
    pub could_not_determine_path: &'static str,
    pub config_imported: &'static str,
    pub could_not_determine_desktop: &'static str,
    pub no_protected_found: &'static str,
    pub switched_to_persistent: &'static str,
}

const STATUS_EN: StatusStrings = StatusStrings {
    mode_persistent: "Mode: Persistent",
    mode_one_shot: "Mode: One-shot",
    auto_inject_active: "🤖 Auto-inject active — minimize to tray.",
    auto_inject_disabled: "Auto-inject disabled.",
    refreshing: "Refreshing…",
    launched_stress_tester: "Launched stress_tester.exe.",
    could_not_launch_stress_tester: "✗ Could not launch stress_tester: {e}",
    stress_tester_not_found: "✗ stress_tester.exe not found — build with: cargo build --release -p stress_tester",
    launch_strip: "🚀 Launch strip: stripping all protected windows.",
    added_to_startup: "🚀 Added to Windows startup.",
    removed_from_startup: "Removed from Windows startup.",
    could_not_write_startup: "✗ Could not write startup registry key.",
    toast_on: "🔔 Toast notifications ON.",
    toast_off: "🔕 Toast notifications OFF.",
    hotkey_registered: "⌨ Hotkey registered: {combo}",
    hotkey_unregistered: "⌨ Hotkey unregistered.",
    hotkey_set: "⌨ Hotkey set to: {combo}",
    closes_to_tray: "✕ closes to tray.",
    exits_app: "✕ exits the app.",
    log_on: "📋 Injection log file ON.",
    log_off: "📋 Injection log file OFF.",
    discord_on: "🎮 Discord Rich Presence ON.",
    discord_off: "🎮 Discord Rich Presence OFF.",
    silent_startup_on: "🤫 Silent startup ON.",
    silent_startup_off: "Silent startup OFF.",
    fast_scan_on: "⚡ Fast scan ON (100ms).",
    fast_scan_off: "Fast scan OFF (500ms).",
    strip_on_launch_on: "🚀 Strip on launch ON.",
    strip_on_launch_off: "Strip on launch OFF.",
    config_exported: "📤 Config exported to Desktop.",
    export_failed: "✗ Export failed: {e}",
    could_not_determine_path: "✗ Could not determine config or desktop path.",
    config_imported: "📥 Config imported from Desktop.",
    could_not_determine_desktop: "✗ Could not determine desktop path.",
    no_protected_found: "No protected windows found — hit Refresh first.",
    switched_to_persistent: "🔁 Switched to Persistent — re-stripping {count} process(es).",
};

const STATUS_ZH: StatusStrings = StatusStrings {
    mode_persistent: "模式：持久模式",
    mode_one_shot: "模式：一次性",
    auto_inject_active: "🤖 自动注入已激活 — 请最小化到托盘。",
    auto_inject_disabled: "自动注入已禁用。",
    refreshing: "正在刷新…",
    launched_stress_tester: "已启动 stress_tester.exe。",
    could_not_launch_stress_tester: "✗ 无法启动 stress_tester：{e}",
    stress_tester_not_found: "✗ stress_tester.exe 未找到 — 请执行：cargo build --release -p stress_tester",
    launch_strip: "🚀 启动剥离：正在剥离所有受保护窗口。",
    added_to_startup: "🚀 已添加到 Windows 启动项。",
    removed_from_startup: "已从 Windows 启动项移除。",
    could_not_write_startup: "✗ 无法写入启动注册表项。",
    toast_on: "🔔 桌面通知 已开启。",
    toast_off: "🔕 桌面通知 已关闭。",
    hotkey_registered: "⌨ 热键已注册：{combo}",
    hotkey_unregistered: "⌨ 热键已取消注册。",
    hotkey_set: "⌨ 热键已设置为：{combo}",
    closes_to_tray: "✕ 关闭时最小化到托盘。",
    exits_app: "✕ 关闭时退出应用。",
    log_on: "📋 注入日志文件 已开启。",
    log_off: "📋 注入日志文件 已关闭。",
    discord_on: "🎮 Discord Rich Presence 已开启。",
    discord_off: "🎮 Discord Rich Presence 已关闭。",
    silent_startup_on: "🤫 静默启动 已开启。",
    silent_startup_off: "静默启动 已关闭。",
    fast_scan_on: "⚡ 快速扫描 已开启 (100ms)。",
    fast_scan_off: "快速扫描 已关闭 (500ms)。",
    strip_on_launch_on: "🚀 启动时剥离 已开启。",
    strip_on_launch_off: "启动时剥离 已关闭。",
    config_exported: "📤 配置已导出到桌面。",
    export_failed: "✗ 导出失败：{e}",
    could_not_determine_path: "✗ 无法确定配置或桌面路径。",
    config_imported: "📥 已从桌面导入配置。",
    could_not_determine_desktop: "✗ 无法确定桌面路径。",
    no_protected_found: "未找到受保护窗口 — 请先点击刷新。",
    switched_to_persistent: "🔁 已切换到持久模式 — 正在重新剥离 {count} 个进程。",
};

pub fn status_msg(lang: Language) -> &'static StatusStrings {
    match lang {
        Language::English => &STATUS_EN,
        Language::Chinese => &STATUS_ZH,
    }
}

// ── Update dialog ───────────────────────────────────────────────────────

pub struct UpdateStrings {
    pub window_title: &'static str,
    pub update_to: &'static str,
    pub installer_hint: &'static str,
    pub update_now: &'static str,
    pub later: &'static str,
}

const UPDATE_EN: UpdateStrings = UpdateStrings {
    window_title: "Update available",
    update_to: "Update to {tag}?",
    installer_hint: "The installer will run silently in the background.\n\
        When it finishes the app will relaunch automatically.",
    update_now: "⬇ Update now",
    later: "Later",
};

const UPDATE_ZH: UpdateStrings = UpdateStrings {
    window_title: "有可用更新",
    update_to: "更新到 {tag}？",
    installer_hint: "安装程序将在后台静默运行。\n\
        完成后应用将自动重新启动。",
    update_now: "⬇ 立即更新",
    later: "稍后",
};

pub fn update(lang: Language) -> &'static UpdateStrings {
    match lang {
        Language::English => &UPDATE_EN,
        Language::Chinese => &UPDATE_ZH,
    }
}

// ── Re-protection modal ────────────────────────────────────────────────

pub struct ReapplyStrings {
    pub window_title: &'static str,
    pub description: &'static str,
    pub one_shot_hint: &'static str,
    pub fix_hint: &'static str,
    pub switch_button: &'static str,
    pub dismiss: &'static str,
}

const REAPPLY_EN: ReapplyStrings = ReapplyStrings {
    window_title: "⚠️  Protection Re-applied",
    description: "The following app(s) re-applied capture protection after being stripped:",
    one_shot_hint: "⚡ One-shot mode injects once and exits — Windows caches the DLL path,\n\
        so re-injecting with the same file is a no-op.",
    fix_hint: "✅  Fix: switch to 🔁 Persistent mode.\n\
        The persistent DLL hooks SetWindowDisplayAffinity at the call site\n\
        (IAT hook) so protection can never be re-applied via a static import.\n\
        A 5 s polling fallback handles dynamic callers as a safety net.",
    switch_button: "Switch to Persistent & Re-strip",
    dismiss: "Dismiss",
};

const REAPPLY_ZH: ReapplyStrings = ReapplyStrings {
    window_title: "⚠️  保护已重新应用",
    description: "以下应用在被剥离后重新应用了捕获保护：",
    one_shot_hint: "⚡ 一次性模式仅注入一次后退出 — Windows 会缓存 DLL 路径，\n\
        因此使用相同文件重新注入将无效。",
    fix_hint: "✅  解决方案：切换到 🔁 持久模式。\n\
        持久 DLL 在调用点拦截 SetWindowDisplayAffinity\n\
        （IAT 钩子），使保护无法通过静态导入重新应用。\n\
        5 秒轮询作为安全网处理动态调用者。",
    switch_button: "切换到持久模式并重新剥离",
    dismiss: "忽略",
};

pub fn reapply(lang: Language) -> &'static ReapplyStrings {
    match lang {
        Language::English => &REAPPLY_EN,
        Language::Chinese => &REAPPLY_ZH,
    }
}

// ── Log panel ──────────────────────────────────────────────────────────

pub struct LogStrings {
    pub title: &'static str,
    pub clear: &'static str,
}

const LOG_EN: LogStrings = LogStrings {
    title: "Injection Log",
    clear: "Clear",
};

const LOG_ZH: LogStrings = LogStrings {
    title: "注入日志",
    clear: "清空",
};

pub fn log(lang: Language) -> &'static LogStrings {
    match lang {
        Language::English => &LOG_EN,
        Language::Chinese => &LOG_ZH,
    }
}

// ── Settings window ────────────────────────────────────────────────────

pub struct SettingsStrings {
    pub window_title: &'static str,
    // Section headers
    pub startup: &'static str,
    pub windows_defender: &'static str,
    pub notifications: &'static str,
    pub hotkey: &'static str,
    pub window: &'static str,
    pub logging: &'static str,
    pub discord: &'static str,
    pub detection: &'static str,
    pub per_process_rules: &'static str,
    pub exclusions: &'static str,
    pub config: &'static str,
    // Startup options
    pub start_with_windows: &'static str,
    pub start_with_windows_desc: &'static str,
    pub silent_startup: &'static str,
    pub silent_startup_desc: &'static str,
    pub strip_on_launch: &'static str,
    pub strip_on_launch_desc: &'static str,
    // Defender
    pub defender_hint: &'static str,
    pub copy_commands: &'static str,
    // Notifications
    pub desktop_notifications: &'static str,
    pub desktop_notifications_desc: &'static str,
    // Hotkey
    pub on: &'static str,
    pub off: &'static str,
    pub toggle_hotkey_hint: &'static str,
    pub listening: &'static str,
    pub listening_hint: &'static str,
    pub change: &'static str,
    pub change_hint: &'static str,
    pub press_key_hint: &'static str,
    pub hotkey_desc: &'static str,
    // Window
    pub minimize_to_tray: &'static str,
    pub minimize_to_tray_desc: &'static str,
    // Logging
    pub injection_log_file: &'static str,
    pub injection_log_file_desc: &'static str,
    pub open_log_file: &'static str,
    // Discord
    pub discord_rich_presence: &'static str,
    pub discord_rich_presence_desc: &'static str,
    // Detection
    pub fast_scan: &'static str,
    pub fast_scan_desc: &'static str,
    // Per-process rules
    pub one_shot: &'static str,
    pub persistent: &'static str,
    pub skip: &'static str,
    pub add_rule: &'static str,
    // Exclusions
    pub add_exclusion: &'static str,
    // Config
    pub export: &'static str,
    pub export_hint: &'static str,
    pub import: &'static str,
    pub import_hint: &'static str,
}

const SETTINGS_EN: SettingsStrings = SettingsStrings {
    window_title: "⚙  Settings",
    startup: "Startup",
    windows_defender: "Windows Defender",
    notifications: "Notifications",
    hotkey: "Hotkey",
    window: "Window",
    logging: "Logging",
    discord: "Discord",
    detection: "Detection",
    per_process_rules: "Per-process Rules",
    exclusions: "Exclusions",
    config: "Config",
    start_with_windows: "Start with Windows",
    start_with_windows_desc: "Launch at login (UAC prompt each time — needs admin).",
    silent_startup: "Silent startup",
    silent_startup_desc: "Start minimized straight to the system tray.",
    strip_on_launch: "Strip all on launch",
    strip_on_launch_desc: "Clear every protected window on the first scan.",
    defender_hint: "If Defender flags payload_dll.dll or payload_dll_persistent.dll as suspicious, add an exclusion manually. Open PowerShell as Administrator and run:",
    copy_commands: "📋  Copy commands",
    desktop_notifications: "Desktop notifications",
    desktop_notifications_desc: "Toast when auto-inject strips a process in the background.",
    on: "(ON)",
    off: "(OFF)",
    toggle_hotkey_hint: "Toggle the global hotkey on or off",
    listening: "● Listening…",
    listening_hint: "Press a key combo (Ctrl/Shift/Alt + key). Esc to cancel.",
    change: "Change…",
    change_hint: "Click then press a new key combination",
    press_key_hint: "  Press any key with Ctrl, Shift, or Alt held. Esc to cancel.",
    hotkey_desc: "  {combo} → Strip All Protected windows, even when minimised to tray.",
    minimize_to_tray: "Minimize to tray on close",
    minimize_to_tray_desc: "✕ hides to the tray instead of quitting (tray Quit still exits).",
    injection_log_file: "Injection log file",
    injection_log_file_desc: "Append a timestamped entry to injection.log on every strip.",
    open_log_file: "📂  Open log file",
    discord_rich_presence: "Discord Rich Presence",
    discord_rich_presence_desc: "Show mode, auto-inject state, and strip count in Discord.",
    fast_scan: "Fast scan — 100 ms",
    fast_scan_desc: "Scan every 100 ms instead of 500 ms (slightly more CPU).",
    one_shot: "One-shot",
    persistent: "Persistent",
    skip: "Skip",
    add_rule: "➕",
    add_exclusion: "➕ Add",
    export: "📤 Export",
    export_hint: "Export config to Desktop as capture-bypass-config.toml",
    import: "📥 Import",
    import_hint: "Import config from Desktop capture-bypass-config.toml",
};

const SETTINGS_ZH: SettingsStrings = SettingsStrings {
    window_title: "⚙  设置",
    startup: "启动",
    windows_defender: "Windows Defender",
    notifications: "通知",
    hotkey: "热键",
    window: "窗口",
    logging: "日志",
    discord: "Discord",
    detection: "检测",
    per_process_rules: "进程规则",
    exclusions: "排除列表",
    config: "配置",
    start_with_windows: "开机自启动",
    start_with_windows_desc: "登录时启动（每次需确认UAC — 需要管理员权限）。",
    silent_startup: "静默启动",
    silent_startup_desc: "启动时直接最小化到系统托盘。",
    strip_on_launch: "启动时全部剥离",
    strip_on_launch_desc: "在首次扫描时清除所有受保护窗口。",
    defender_hint: "如果 Defender 将 payload_dll.dll 或 payload_dll_persistent.dll 标记为可疑，请手动添加排除项。以管理员身份打开 PowerShell 并运行：",
    copy_commands: "📋  复制命令",
    desktop_notifications: "桌面通知",
    desktop_notifications_desc: "自动注入在后台剥离进程时弹出通知。",
    on: "（开启）",
    off: "（关闭）",
    toggle_hotkey_hint: "切换全局热键开关",
    listening: "● 监听中…",
    listening_hint: "按下快捷键组合（Ctrl/Shift/Alt + 按键）。按 Esc 取消。",
    change: "更改…",
    change_hint: "点击后按下新的快捷键组合",
    press_key_hint: "  按住 Ctrl、Shift 或 Alt 后按任意键。按 Esc 取消。",
    hotkey_desc: "  {combo} → 剥离所有受保护窗口，即使最小化到托盘。",
    minimize_to_tray: "关闭时最小化到托盘",
    minimize_to_tray_desc: "✕ 隐藏到托盘而非退出（托盘的退出仍会退出）。",
    injection_log_file: "注入日志文件",
    injection_log_file_desc: "每次剥离时向 injection.log 追加带时间戳的记录。",
    open_log_file: "📂  打开日志文件",
    discord_rich_presence: "Discord 状态显示",
    discord_rich_presence_desc: "在 Discord 中显示模式、自动注入状态和剥离计数。",
    fast_scan: "快速扫描 — 100 毫秒",
    fast_scan_desc: "扫描间隔从 500 毫秒缩短到 100 毫秒（CPU 略有增加）。",
    one_shot: "一次性",
    persistent: "持久模式",
    skip: "跳过",
    add_rule: "➕",
    add_exclusion: "➕ 添加",
    export: "📤 导出",
    export_hint: "将配置导出到桌面 capture-bypass-config.toml",
    import: "📥 导入",
    import_hint: "从桌面 capture-bypass-config.toml 导入配置",
};

pub fn settings(lang: Language) -> &'static SettingsStrings {
    match lang {
        Language::English => &SETTINGS_EN,
        Language::Chinese => &SETTINGS_ZH,
    }
}

// ── System tray ────────────────────────────────────────────────────────

pub struct TrayStrings {
    pub open: &'static str,
    pub quit: &'static str,
    pub tooltip: &'static str,
    pub tooltip_protected: &'static str,
}

const TRAY_EN: TrayStrings = TrayStrings {
    open: "Open",
    quit: "Quit",
    tooltip: "capture-bypass",
    tooltip_protected: "capture-bypass — protected windows detected!",
};

const TRAY_ZH: TrayStrings = TrayStrings {
    open: "打开",
    quit: "退出",
    tooltip: "capture-bypass",
    tooltip_protected: "capture-bypass — 检测到受保护窗口！",
};

pub fn tray(lang: Language) -> &'static TrayStrings {
    match lang {
        Language::English => &TRAY_EN,
        Language::Chinese => &TRAY_ZH,
    }
}

// ── Toast notification ─────────────────────────────────────────────────

pub struct ToastStrings {
    pub title: &'static str,
    pub stripped_n_windows: &'static str,
}

const TOAST_EN: ToastStrings = ToastStrings {
    title: "capture-bypass",
    stripped_n_windows: "Stripped {n} windows",
};

const TOAST_ZH: ToastStrings = ToastStrings {
    title: "capture-bypass",
    stripped_n_windows: "已剥离 {n} 个窗口",
};

pub fn toast(lang: Language) -> &'static ToastStrings {
    match lang {
        Language::English => &TOAST_EN,
        Language::Chinese => &TOAST_ZH,
    }
}

// ── Discord Rich Presence ──────────────────────────────────────────────

pub struct DiscordStrings {
    pub strips_this_session: &'static str,
    pub get_capture_bypass: &'static str,
    pub persistent: &'static str,
    pub one_shot: &'static str,
    pub auto_inject_on: &'static str,
    pub manual: &'static str,
}

const DISCORD_EN: DiscordStrings = DiscordStrings {
    strips_this_session: "Strips this session: {count}",
    get_capture_bypass: "Get capture-bypass",
    persistent: "persistent",
    one_shot: "one-shot",
    auto_inject_on: "auto-inject on",
    manual: "manual",
};

const DISCORD_ZH: DiscordStrings = DiscordStrings {
    strips_this_session: "本次会话已剥离：{count}",
    get_capture_bypass: "获取 capture-bypass",
    persistent: "持久模式",
    one_shot: "一次性",
    auto_inject_on: "自动注入开启",
    manual: "手动",
};

pub fn discord(lang: Language) -> &'static DiscordStrings {
    match lang {
        Language::English => &DISCORD_EN,
        Language::Chinese => &DISCORD_ZH,
    }
}

// ── Help sections ──────────────────────────────────────────────────────

pub struct HelpTabStrings {
    pub overview: &'static str,
    pub requirements_build: &'static str,
    pub usage_guide: &'static str,
    pub settings: &'static str,
    pub per_process_rules: &'static str,
    pub injection_modes: &'static str,
    pub browser_injection: &'static str,
    pub system_tray: &'static str,
    pub auto_update: &'static str,
    pub troubleshooting: &'static str,
}

const HELP_TAB_EN: HelpTabStrings = HelpTabStrings {
    overview: "Overview",
    requirements_build: "Requirements & Build",
    usage_guide: "Usage Guide",
    settings: "Settings",
    per_process_rules: "Per-Process Rules & Exclusions",
    injection_modes: "Injection Modes",
    browser_injection: "Browser Injection",
    system_tray: "System Tray & Auto-inject",
    auto_update: "Auto-Update",
    troubleshooting: "Troubleshooting",
};

const HELP_TAB_ZH: HelpTabStrings = HelpTabStrings {
    overview: "概述",
    requirements_build: "环境要求与编译",
    usage_guide: "使用指南",
    settings: "设置",
    per_process_rules: "进程规则与排除列表",
    injection_modes: "注入模式",
    browser_injection: "浏览器注入",
    system_tray: "系统托盘与自动注入",
    auto_update: "自动更新",
    troubleshooting: "故障排除",
};

pub fn help_tab(lang: Language) -> &'static HelpTabStrings {
    match lang {
        Language::English => &HELP_TAB_EN,
        Language::Chinese => &HELP_TAB_ZH,
    }
}

// ── Help section headings ──────────────────────────────────────────────

pub struct HelpHeadingStrings {
    // Overview
    pub what_is: &'static str,
    pub how_does_it_work: &'static str,
    pub legal_notice: &'static str,
    // Requirements & Build
    pub requirements: &'static str,
    pub build_x64: &'static str,
    pub build_x86: &'static str,
    // Usage Guide
    pub window_list: &'static str,
    pub header_buttons: &'static str,
    pub filter_bar: &'static str,
    pub status_bar: &'static str,
    // Settings
    pub opening_settings: &'static str,
    pub silent_startup: &'static str,
    pub strip_on_launch: &'static str,
    pub fast_scan: &'static str,
    pub desktop_notifications: &'static str,
    pub global_hotkey: &'static str,
    pub discord_rich_presence: &'static str,
    pub export_import: &'static str,
    pub injection_log_file: &'static str,
    pub windows_defender: &'static str,
    // Per-process rules
    pub per_process_rules: &'static str,
    pub exclusion_list: &'static str,
    // Injection modes
    pub one_shot_mode: &'static str,
    pub persistent_mode: &'static str,
    pub re_injection: &'static str,
    // Browser injection
    pub why_browsers: &'static str,
    pub what_cb_does: &'static str,
    pub tip: &'static str,
    // System tray
    pub system_tray: &'static str,
    pub auto_inject: &'static str,
    // Auto-update
    pub how_updates_work: &'static str,
    pub update_wrong: &'static str,
    // Troubleshooting
    pub dlls_not_found: &'static str,
    pub strip_failed: &'static str,
    pub injection_ok_but_black: &'static str,
    pub notifications_missing: &'static str,
    pub antivirus_flags: &'static str,
    pub x86_fails: &'static str,
    pub config_location: &'static str,
}

const HELP_HEADING_EN: HelpHeadingStrings = HelpHeadingStrings {
    what_is: "What is Capture Bypass?",
    how_does_it_work: "How does it work?",
    legal_notice: "Legal notice",
    requirements: "Requirements",
    build_x64: "Build — x64 (required)",
    build_x86: "Build — x86 (optional, for 32-bit targets)",
    window_list: "Window list",
    header_buttons: "Header buttons",
    filter_bar: "Filter bar",
    status_bar: "Status bar",
    opening_settings: "Opening Settings",
    silent_startup: "Silent startup",
    strip_on_launch: "Strip on launch",
    fast_scan: "Fast scan",
    desktop_notifications: "Desktop notifications",
    global_hotkey: "Global hotkey",
    discord_rich_presence: "Discord Rich Presence",
    export_import: "Export / Import config",
    injection_log_file: "Injection log file",
    windows_defender: "Windows Defender",
    per_process_rules: "Per-process rules",
    exclusion_list: "Exclusion list",
    one_shot_mode: "⚡ One-shot mode (default)",
    persistent_mode: "🔁 Persistent mode",
    re_injection: "Re-injection & re-protection",
    why_browsers: "Why browsers need special handling",
    what_cb_does: "What Capture Bypass does",
    tip: "Tip",
    system_tray: "System tray",
    auto_inject: "Auto-inject",
    how_updates_work: "How updates work",
    update_wrong: "Update installs wrong / nothing happens",
    dlls_not_found: "DLLs not found",
    strip_failed: "\"Strip failed\" / access denied",
    injection_ok_but_black: "Injection succeeds but window is still black in OBS",
    notifications_missing: "Notifications don't appear",
    antivirus_flags: "Antivirus flags the DLL",
    x86_fails: "x86 injection fails even with x86 binaries present",
    config_location: "Config file location",
};

const HELP_HEADING_ZH: HelpHeadingStrings = HelpHeadingStrings {
    what_is: "什么是 Capture Bypass？",
    how_does_it_work: "它如何工作？",
    legal_notice: "法律声明",
    requirements: "环境要求",
    build_x64: "编译 — x64（必需）",
    build_x86: "编译 — x86（可选，用于32位目标）",
    window_list: "窗口列表",
    header_buttons: "头部按钮",
    filter_bar: "过滤栏",
    status_bar: "状态栏",
    opening_settings: "打开设置",
    silent_startup: "静默启动",
    strip_on_launch: "启动时剥离",
    fast_scan: "快速扫描",
    desktop_notifications: "桌面通知",
    global_hotkey: "全局热键",
    discord_rich_presence: "Discord Rich Presence",
    export_import: "导入/导出配置",
    injection_log_file: "注入日志文件",
    windows_defender: "Windows Defender",
    per_process_rules: "进程规则",
    exclusion_list: "排除列表",
    one_shot_mode: "⚡ 一次性模式（默认）",
    persistent_mode: "🔁 持久模式",
    re_injection: "重新注入与重新保护",
    why_browsers: "为什么浏览器需要特殊处理",
    what_cb_does: "Capture Bypass 的处理方式",
    tip: "提示",
    system_tray: "系统托盘",
    auto_inject: "自动注入",
    how_updates_work: "更新工作原理",
    update_wrong: "更新安装异常 / 无反应",
    dlls_not_found: "DLL 未找到",
    strip_failed: "\"剥离失败\" / 访问被拒绝",
    injection_ok_but_black: "注入成功但 OBS 中窗口仍为黑屏",
    notifications_missing: "通知不出现",
    antivirus_flags: "杀毒软件将 DLL 标记为威胁",
    x86_fails: "即使存在 x86 二进制文件，x86 注入仍然失败",
    config_location: "配置文件位置",
};

pub fn help_heading(lang: Language) -> &'static HelpHeadingStrings {
    match lang {
        Language::English => &HELP_HEADING_EN,
        Language::Chinese => &HELP_HEADING_ZH,
    }
}

// ── Help section bodies ────────────────────────────────────────────────

pub struct HelpBodyStrings {
    pub what_is: &'static str,
    pub how_does_it_work: &'static str,
    pub legal_notice: &'static str,
    pub requirements: &'static str,
    pub build_x64: &'static str,
    pub build_x86: &'static str,
    pub window_list: &'static str,
    pub header_buttons: &'static str,
    pub filter_bar: &'static str,
    pub status_bar: &'static str,
    pub opening_settings: &'static str,
    pub silent_startup: &'static str,
    pub strip_on_launch: &'static str,
    pub fast_scan: &'static str,
    pub desktop_notifications: &'static str,
    pub global_hotkey: &'static str,
    pub discord_rich_presence: &'static str,
    pub export_import: &'static str,
    pub injection_log_file: &'static str,
    pub windows_defender: &'static str,
    pub per_process_rules: &'static str,
    pub exclusion_list: &'static str,
    pub one_shot_mode: &'static str,
    pub persistent_mode: &'static str,
    pub re_injection: &'static str,
    pub why_browsers: &'static str,
    pub what_cb_does: &'static str,
    pub tip: &'static str,
    pub system_tray: &'static str,
    pub auto_inject: &'static str,
    pub how_updates_work: &'static str,
    pub update_wrong: &'static str,
    pub dlls_not_found: &'static str,
    pub strip_failed: &'static str,
    pub injection_ok_but_black: &'static str,
    pub notifications_missing: &'static str,
    pub antivirus_flags: &'static str,
    pub x86_fails: &'static str,
    pub config_location: &'static str,
}

const HELP_BODY_EN: HelpBodyStrings = HelpBodyStrings {
    what_is: "Capture Bypass removes the WDA_EXCLUDEFROMCAPTURE screen-capture \
        protection from Windows application windows, letting OBS, the \
        Snipping Tool, and any other screen-capture software record them \
        normally.\n\
        \n\
        Typical use-cases: streaming or recording DRM video players, \
        conference call windows, and any other app that explicitly hides \
        itself from capture software.",
    how_does_it_work: "Windows provides SetWindowDisplayAffinity(), which lets a process \
        protect its own windows from capture.  Because the API only works \
        on a process's own windows, bypassing it requires running code \
        INSIDE the target process.\n\
        \n\
        Capture Bypass does this via DLL injection:\n\
        \n\
        1. OpenProcess        — open a handle to the target process\n\
        2. VirtualAllocEx     — allocate memory inside the target\n\
        3. WriteProcessMemory — write the payload DLL path into that memory\n\
        4. CreateRemoteThread — start a thread inside the target that calls\n\
                                LoadLibraryA, loading the payload DLL\n\
        5. The DLL's DllMain  — calls SetWindowDisplayAffinity(hwnd, WDA_NONE)\n\
                                for every window owned by that process",
    legal_notice: "Only use this tool on windows and processes you own or have \
        explicit permission to capture.  See DISCLAIMER.md in the \
        repository for the full legal disclaimer.",
    requirements: "• Windows 10 build 19041+  (WDA_EXCLUDEFROMCAPTURE requires 2004+)\n\
        • Administrator privileges  (OpenProcess on other processes requires admin)\n\
        • Rust + Cargo  (https://rustup.rs)",
    build_x64: "cargo build --release -p payload_dll -p payload_dll_persistent -p gui\n\
        \n\
        Binaries land in:  target\\release\\",
    build_x86: "rustup target add i686-pc-windows-msvc\n\
        cargo build --release --target i686-pc-windows-msvc \\\n\
              -p payload_dll -p payload_dll_persistent\n\
        \n\
        Binaries land in:  target\\i686-pc-windows-msvc\\release\\\n\
        \n\
        32-bit processes are shown with an orange \"32\" badge in the table.",
    window_list: "The table shows every visible, titled window with:\n\
        \n\
        PID      — Process ID\n\
        Process  — Executable name  (orange \"32\" badge = 32-bit process)\n\
        Title    — Window title\n\
        Status   — Live protection state, refreshed every 500 ms\n\
                     (or ~100 ms with Fast Scan enabled):\n\
                     PROTECTED = WDA_EXCLUDEFROMCAPTURE\n\
                     MONITOR   = WDA_MONITOR\n\
                     OK        = WDA_NONE (capturable)\n\
        Action   — \"Strip Protection\" button for that row",
    header_buttons: "⟳ Refresh             Re-enumerate all windows immediately.\n\
        \n\
        ⚡ Strip All Protected Inject into every currently-protected PID at once.\n\
                               Each process is injected only once even if it owns\n\
                               multiple protected windows.\n\
        \n\
        Mode                  Toggle between One-shot and Persistent injection.\n\
                               One-shot is fast; Persistent fights re-protection.\n\
        \n\
        🔨 Stress Test        Launch stress_tester.exe — a self-protecting window\n\
                               with Fight Mode, Scenario A, and Scenario B.\n\
        \n\
        ⚙ Settings            Opens the full Settings panel.\n\
        \n\
        📋 Log                Toggle the injection history side-panel.\n\
        \n\
        📖 Help               Opens this window.\n\
        \n\
        🆕 vX.Y.Z available   Appears when a new release is detected on GitHub.\n\
                               Click it to open the update confirmation dialog.\n\
                               After you confirm, the installer downloads in the\n\
                               background (progress shown in the header).  When\n\
                               the download finishes the installer runs silently\n\
                               and the app restarts automatically — no second\n\
                               click required.",
    filter_bar: "Type to search live by window title, process name, or PID.  Click ✕ to clear.\n\
        \n\
        \"Protected only\" checkbox hides all unprotected windows.\n\
        \n\
        Protected-window indicator (right side of filter bar):\n\
          🔴 N protected — one or more windows currently have WDA protection.\n\
          🟢 0 protected — no protected windows are visible right now.\n\
        \n\
        🤖 Auto-inject — background thread strips newly-protected windows \
        automatically.  Continues running while the app is minimised to tray.\n\
        \n\
        👁 Watch mode — shows ALL windows including unprotected ones in the list \
        so you can monitor a specific process before protection is applied.",
    status_bar: "The bottom bar shows, left to right:\n\
        \n\
        v{version}       — current app version (e.g. v3.5.9)\n\
        {N} lifetime     — total injections performed across all sessions\n\
        {N} windows      — visible windows currently enumerated\n\
        {N} protected    — windows with WDA_EXCLUDEFROMCAPTURE right now\n\
        {N} 32-bit       — 32-bit processes in the current list\n\
        \n\
        Below that: the most recent action message with a timestamp.  \
        Green = success, red = error, gray = informational.",
    opening_settings: "Click ⚙ Settings in the header to open the Settings window.  \
        All changes are saved to config.toml immediately.",
    silent_startup: "When enabled the main window is hidden on launch — the app starts \
        directly in the system tray.  Re-open it by clicking the tray icon \
        or right-clicking and choosing Open.\n\
        \n\
        Useful for streamers who want capture-bypass running in the \
        background without an extra window appearing at startup.",
    strip_on_launch: "When enabled the app automatically calls Strip All Protected on \
        startup, once the initial window scan completes.  Any protected \
        windows present at launch are stripped without any user action.\n\
        \n\
        Combine with Silent Startup for fully hands-off protection removal \
        every time Windows boots.",
    fast_scan: "Increases the background window-scan interval from ~500 ms to \
        ~100 ms.  Protected windows are detected and (if auto-inject is on) \
        stripped up to 5× faster.\n\
        \n\
        Trade-off: slightly higher CPU usage.  Recommended for apps that \
        apply protection very briefly or mid-render.",
    desktop_notifications: "Toggle Windows balloon-tip notifications for injection events.  \
        When multiple windows are stripped within 400 ms they are grouped \
        into a single \"Stripped N windows\" notification instead of \
        producing a burst of individual toasts.\n\
        \n\
        Note: notifications are sent via Win32 Shell_NotifyIcon so they \
        work correctly even when the app is running as Administrator.",
    global_hotkey: "Assign a keyboard shortcut to trigger Strip All Protected from \
        anywhere — even when the capture-bypass window is not focused or \
        is minimised to the tray.\n\
        \n\
        Click inside the hotkey field and press your desired key combo \
        (e.g. Ctrl+Shift+S), then save.  Leave it blank to disable.",
    discord_rich_presence: "When enabled, the app reports its current activity to Discord so \
        your status shows that you are using capture-bypass.  The presence \
        updates with the number of protected windows detected and the \
        injection count.\n\
        \n\
        Disable this if you stream your Discord status and don't want \
        capture-bypass mentioned publicly.",
    export_import: "Export config — writes the current settings (all toggles, rules, \
        exclusions, hotkey, etc.) to a capture_bypass_config.toml file \
        you choose.  Use this to back up your configuration or move it \
        to another machine.\n\
        \n\
        Import config — loads settings from a previously exported file, \
        replacing the current configuration.  The app applies the imported \
        settings immediately without restarting.",
    injection_log_file: "Enable logging to write every injection attempt (timestamp, PID, \
        process name, result) to a persistent log file alongside the \
        executable.  Useful for debugging or keeping an audit trail.",
    windows_defender: "If Windows Defender flags payload_dll.dll or \
        payload_dll_persistent.dll as suspicious, this is a false \
        positive caused by the DLL injection technique — not malware.\n\
        \n\
        To add an exclusion, open PowerShell as Administrator and run \
        the two commands shown in the Windows Defender section of \
        Settings.  Use the 📋 Copy commands button to copy them to \
        your clipboard, then paste into PowerShell and press Enter.\n\
        \n\
        The commands are:\n\
          Add-MpPreference -ExclusionPath '<install folder>'\n\
          Add-MpPreference -ExclusionProcess 'capture_bypass_gui.exe'\n\
        \n\
        Safe to run multiple times — Defender ignores duplicate entries.",
    per_process_rules: "The Rules table in Settings lets you override the global injection \
        mode for specific processes.\n\
        \n\
        Three modes are available per rule:\n\
        \n\
        Always One-shot   — always use the fast one-shot DLL for this\n\
                            process, regardless of the global Mode toggle.\n\
        \n\
        Always Persistent — always use the persistent (re-applying) DLL\n\
                            for this process.  Good for known fighters.\n\
        \n\
        Skip              — never inject into this process, even if auto-\n\
                            inject or Strip All Protected is triggered.\n\
        \n\
        Add a rule by typing the executable name (e.g. chrome.exe) into \
        the rule input field, choosing a mode, and clicking Add.  \
        Rules are matched case-insensitively against the process name.",
    exclusion_list: "Processes in the exclusion list are completely ignored by all \
        injection operations — manual Strip, Strip All Protected, and \
        auto-inject will all skip them.\n\
        \n\
        Add an entry by typing the executable name (e.g. SecurityAgent.exe) \
        and clicking Add.  Remove entries with the ✕ button.\n\
        \n\
        Use this to prevent accidental injection into system processes, \
        AV software, or any process you want left alone.",
    one_shot_mode: "The payload DLL strips WDA protection once and exits.  \
        Fast and lightweight.\n\
        \n\
        Use when: the target app sets protection only once at startup \
        and never re-applies it.  Also suitable for most browsers and \
        standard media players.",
    persistent_mode: "The payload DLL stays alive inside the target process and uses \
        two layers to keep protection cleared:\n\
        \n\
        1. IAT hook — patches SetWindowDisplayAffinity in the host \
        module's Import Address Table so the call is intercepted at \
        the source.  Every subsequent call via a static import lands in \
        our detour, which forces WDA_NONE instantly with zero latency.\n\
        \n\
        2. Polling fallback — a background thread still calls \
        SetWindowDisplayAffinity(WDA_NONE) every 5 s as a safety net for \
        windows that were already protected at inject time and for apps \
        that resolve the function dynamically via GetProcAddress.\n\
        \n\
        Use when: the target app calls SetWindowDisplayAffinity on a \
        timer to fight back against one-shot injection (e.g. DRM video \
        players, apps with anti-capture enforcement).",
    re_injection: "Windows caches loaded DLLs by file path — if the same DLL path \
        is already loaded in a process, LoadLibraryA silently no-ops.\n\
        \n\
        If a one-shot strip appears to succeed but the status badge \
        returns to PROTECTED shortly after, the app is re-applying \
        protection on a timer.  Switch to 🔁 Persistent mode — a popup \
        will also appear automatically when re-protection is detected.\n\
        \n\
        Per-process rules let you pin specific executables to Persistent \
        mode so you never have to switch manually for known fighters.",
    why_browsers: "Chrome, Edge, Firefox, Brave, Opera, Vivaldi, and Thorium use a \
        multi-process architecture.  DRM-protected video is rendered in a \
        separate child (renderer) process that owns its own windows.  \
        Injecting only into the main PID won't strip the video window.",
    what_cb_does: "When you click \"Strip Protection\" on a browser row, the app \
        automatically:\n\
        \n\
        1. Injects the payload into the main (browser) PID\n\
        2. Enumerates all child processes via CreateToolhelp32Snapshot\n\
        3. Injects into every child PID as well\n\
        \n\
        Auto-inject also performs this recursive child-process scan when \
        it detects a browser process has become protected.",
    tip: "If a browser re-applies protection after navigating to a new \
        video, click ⚡ Strip All Protected again, or enable \
        🤖 Auto-inject so it's handled automatically.  For persistently \
        fighting browsers, add a per-process rule pinned to Persistent mode.",
    system_tray: "Clicking the window's ✕ close button hides the app to the system \
        tray instead of quitting — the icon remains in the notification area.\n\
        \n\
        Tray icon right-click menu:\n\
          Open  — restore the main window\n\
          Quit  — fully exit the application\n\
        \n\
        The tray icon changes colour based on the current state:\n\
          Blue  — no protected windows detected\n\
          Red   — one or more protected windows are currently detected\n\
        \n\
        The tooltip updates to show how many protected windows are present \
        so you can monitor state without opening the app.",
    auto_inject: "Enable the 🤖 Auto-inject toggle in the toolbar.\n\
        \n\
        A background thread watches for newly-protected windows using a \
        system-wide WinEvent hook (EVENT_OBJECT_SHOW).  When the hook fires, \
        the thread wakes immediately — typical detection latency is < 50 ms. \
        The normal 500 ms (or 100 ms Fast Scan) poll interval acts as a \
        safety net for edge cases the hook might miss.\n\
        \n\
        Per-process rules and the exclusion list are respected — processes \
        set to Skip or in the exclusion list are never auto-injected.\n\
        \n\
        Designed for streamers: enable auto-inject, minimise to tray, and \
        any app that tries to block capture is handled silently in the \
        background.  Desktop notifications (if enabled) confirm each strip.",
    how_updates_work: "The app checks GitHub Releases for a newer version tag when it \
        starts and periodically while running.\n\
        \n\
        When a new version is found:\n\
        \n\
        1. A \"🆕 vX.Y.Z available\" button appears in the header.\n\
        2. Clicking it opens a confirmation dialog — no download starts \
           until you explicitly confirm.\n\
        3. After you click \"Update now\", the installer downloads in the \
           background.  A progress indicator appears in the header.\n\
        4. When the download finishes the installer runs silently (/SILENT)\n\
           and the app restarts automatically.  You do not need to click\n\
           anything else after confirming.\n\
        \n\
        The update replaces all app binaries (GUI, payload DLLs, stress \
        tester) in one step.  Your config.toml is preserved.",
    update_wrong: "The installer is downloaded to a temp file next to the executable. \
        If the download fails the status bar will show an error.  Check \
        your internet connection and try again.\n\
        \n\
        The installer requires the same Administrator token the GUI is \
        already running under, so UAC should not prompt again.",
    dlls_not_found: "The payload DLLs haven't been built yet.  Run:\n\
        \n\
        cargo build --release -p payload_dll -p payload_dll_persistent\n\
        \n\
        (For 32-bit support also build the i686 target — see Requirements & Build.)",
    strip_failed: "OpenProcess requires elevated privileges for processes not owned \
        by your session.  Make sure capture_bypass_gui.exe is running as \
        Administrator (the UAC prompt appears on launch).",
    injection_ok_but_black: "1. Wait for the next scan refresh — if the status badge shows OK, \
        OBS may need its capture source refreshed (remove and re-add it).\n\
        2. For browsers, click Strip Protection again — it re-injects \
        child processes too.\n\
        3. If the badge keeps flipping back to PROTECTED, the app is \
        fighting back on a timer.  Switch to 🔁 Persistent mode, or add \
        a per-process rule for that executable.",
    notifications_missing: "Windows balloon tips are suppressed by Focus Assist / Do Not \
        Disturb.  Check Action Center settings and ensure \"Alarms only\" \
        mode is not active.\n\
        \n\
        capture-bypass sends notifications via Win32 Shell_NotifyIcon \
        (not WinRT toast), so they work correctly from an Administrator \
        process — but OS-level suppression still applies.",
    antivirus_flags: "DLL injection is used by both legitimate tools and malware, so \
        heuristic scanners may flag the payload.  Inspect \
        payload_dll/src/lib.rs — it only calls SetWindowDisplayAffinity.\n\
        \n\
        Add an exclusion for the target\\ directory in your AV settings.  \
        You can also use the per-process exclusion list to make the GUI \
        itself ignore specific processes the AV is watching.",
    x86_fails: "A 64-bit process cannot inject into a 32-bit process and \
        vice-versa.  Verify you built the x86 target:\n\
        \n\
        rustup target add i686-pc-windows-msvc\n\
        cargo build --release --target i686-pc-windows-msvc -p payload_dll\n\
        \n\
        32-bit processes are shown with an orange \"32\" badge.  If the \
        badge is orange and injection fails, the x86 DLL is likely missing.",
    config_location: "The config is stored as config.toml in the same directory as \
        capture_bypass_gui.exe.  If settings don't save, check that the \
        directory is writable.  You can also use Export config in Settings \
        to save a copy to any location.",
};

const HELP_BODY_ZH: HelpBodyStrings = HelpBodyStrings {
    what_is: "Capture Bypass 移除 Windows 应用程序窗口的 WDA_EXCLUDEFROMCAPTURE \
        屏幕捕获保护，让 OBS、截图工具和任何其他屏幕捕获软件能够正常录制。\n\
        \n\
        典型用途：录制 DRM 视频播放器、视频通话窗口，以及任何主动对捕获软件隐藏自身的应用。",
    how_does_it_work: "Windows 提供了 SetWindowDisplayAffinity()，允许进程保护自己的窗口不被捕获。\
        由于该 API 仅对进程自身的窗口有效，绕过它需要在目标进程内部运行代码。\n\
        \n\
        Capture Bypass 通过 DLL 注入实现：\n\
        \n\
        1. OpenProcess        — 打开目标进程的句柄\n\
        2. VirtualAllocEx     — 在目标进程内分配内存\n\
        3. WriteProcessMemory — 将载荷 DLL 路径写入该内存\n\
        4. CreateRemoteThread — 在目标进程中启动线程调用\n\
                                LoadLibraryA 加载载荷 DLL\n\
        5. DLL 的 DllMain     — 对该进程拥有的每个窗口调用\n\
                                SetWindowDisplayAffinity(hwnd, WDA_NONE)",
    legal_notice: "仅在您拥有或获得明确许可的窗口和进程上使用本工具。\
        请参阅仓库中的 DISCLAIMER.md 获取完整法律声明。",
    requirements: "• Windows 10 build 19041+（WDA_EXCLUDEFROMCAPTURE 需要 2004+）\n\
        • 管理员权限（OpenProcess 操作其他进程需要管理员权限）\n\
        • Rust + Cargo（https://rustup.rs）",
    build_x64: "cargo build --release -p payload_dll -p payload_dll_persistent -p gui\n\
        \n\
        二进制文件位于：target\\release\\",
    build_x86: "rustup target add i686-pc-windows-msvc\n\
        cargo build --release --target i686-pc-windows-msvc \\\n\
              -p payload_dll -p payload_dll_persistent\n\
        \n\
        二进制文件位于：target\\i686-pc-windows-msvc\\release\\\n\
        \n\
        32位进程在表格中显示为橙色 \"32\" 徽章。",
    window_list: "表格显示所有可见的、有标题的窗口：\n\
        \n\
        PID      — 进程 ID\n\
        进程     — 可执行文件名（橙色 \"32\" 徽章 = 32位进程）\n\
        标题     — 窗口标题\n\
        状态     — 实时保护状态，每 500 毫秒刷新\n\
                    （启用快速扫描时约 100 毫秒）：\n\
                    受保护 = WDA_EXCLUDEFROMCAPTURE\n\
                    监控中 = WDA_MONITOR\n\
                    正常   = WDA_NONE（可捕获）\n\
        操作     — 该行的 \"剥离保护\" 按钮",
    header_buttons: "⟳ 刷新              立即重新枚举所有窗口。\n\
        \n\
        ⚡ 剥离全部保护    向当前所有受保护的 PID 注入。\n\
                           即使拥有多个受保护窗口，每个进程也只注入一次。\n\
        \n\
        模式                在一次性注入和持久模式之间切换。\n\
                           一次性快速；持久模式对抗重新保护。\n\
        \n\
        🔨 压力测试        启动 stress_tester.exe — 自保护窗口，\n\
                           支持战斗模式、场景A和场景B。\n\
        \n\
        ⚙ 设置             打开完整的设置面板。\n\
        \n\
        📋 日志             切换注入历史侧边栏。\n\
        \n\
        📖 帮助             打开本窗口。\n\
        \n\
        🆕 vX.Y.Z 可用     在 GitHub 上检测到新版本时显示。\n\
                           点击打开更新确认对话框。\n\
                           确认后安装程序在后台下载（进度显示在标题栏）。\n\
                           下载完成后安装程序静默运行并自动重启 —\n\
                           确认后无需再次点击。",
    filter_bar: "输入内容可按窗口标题、进程名或 PID 实时搜索。点击 ✕ 清除。\n\
        \n\
        \"仅受保护的\" 复选框隐藏所有未受保护的窗口。\n\
        \n\
        受保护窗口指示器（过滤栏右侧）：\n\
          🔴 N 个受保护 — 当前有一个或多个窗口具有 WDA 保护。\n\
          🟢 0 个受保护 — 当前没有可见的受保护窗口。\n\
        \n\
        🤖 自动注入 — 后台线程自动剥离新受保护的窗口。\n\
        应用最小化到托盘时继续运行。\n\
        \n\
        👁 监视模式 — 在列表中显示所有窗口（包括未受保护的），\n\
        以便在保护应用之前监视特定进程。",
    status_bar: "底部栏从左到右显示：\n\
        \n\
        v{version}       — 当前应用版本（如 v3.5.9）\n\
        {N} 累计         — 所有会话中执行的总注入次数\n\
        {N} 窗口         — 当前枚举的可见窗口数\n\
        {N} 受保护       — 当前具有 WDA_EXCLUDEFROMCAPTURE 的窗口数\n\
        {N} 32位         — 当前列表中的 32位进程数\n\
        \n\
        下方：最近的操作消息及时间戳。\n\
        绿色 = 成功，红色 = 错误，灰色 = 信息。",
    opening_settings: "点击标题栏的 ⚙ 设置 打开设置窗口。所有更改会立即保存到 config.toml。",
    silent_startup: "启用后主窗口在启动时隐藏 — 应用直接在系统托盘中启动。\
        点击托盘图标或右键选择打开可重新显示。\n\
        \n\
        适合希望 capture-bypass 在后台运行而不额外显示窗口的主播。",
    strip_on_launch: "启用后应用在启动时自动调用剥离全部保护，\
        在初始窗口扫描完成后执行。启动时存在的任何受保护窗口会在无需用户操作的情况下被剥离。\n\
        \n\
        与静默启动结合使用可实现每次 Windows 启动时完全自动的保护移除。",
    fast_scan: "将后台窗口扫描间隔从约 500 毫秒增加到约 100 毫秒。\
        受保护窗口的检测和（如果自动注入开启）剥离速度提高最多 5 倍。\n\
        \n\
        权衡：CPU 使用率略有增加。推荐用于保护持续时间很短或在渲染过程中应用保护的应用。",
    desktop_notifications: "切换注入事件的 Windows 气球提示通知。\
        当多个窗口在 400 毫秒内被剥离时，它们会合并为一个 \"已剥离 N 个窗口\" 通知，\
        而不是产生一连串单独的通知。\n\
        \n\
        注意：通知通过 Win32 Shell_NotifyIcon 发送，因此即使应用以管理员身份运行\
        也能正常工作。",
    global_hotkey: "分配键盘快捷键以在任何地方触发剥离全部保护 —\
        即使 capture-bypass 窗口未获得焦点或已最小化到托盘。\n\
        \n\
        点击热键字段内并按下所需的快捷键组合（如 Ctrl+Shift+S），\
        然后保存。留空以禁用。",
    discord_rich_presence: "启用后，应用会向 Discord 报告当前活动，\
        使您的状态显示您正在使用 capture-bypass。状态会更新检测到的受保护窗口数量和注入计数。\n\
        \n\
        如果您直播 Discord 状态且不希望公开 capture-bypass，请禁用此功能。",
    export_import: "导出配置 — 将当前设置（所有开关、规则、排除项、热键等）\
        写入您选择的 capture_bypass_config.toml 文件。\
        用于备份配置或迁移到其他机器。\n\
        \n\
        导入配置 — 从之前导出的文件加载设置，替换当前配置。\
        应用会立即应用导入的设置，无需重启。",
    injection_log_file: "启用日志记录以将每次注入尝试（时间戳、PID、进程名、结果）\
        写入可执行文件旁边的持久日志文件。用于调试或保留审计跟踪。",
    windows_defender: "如果 Windows Defender 将 payload_dll.dll 或 \
        payload_dll_persistent.dll 标记为可疑，这是 DLL 注入技术导致的误报 — 非恶意软件。\n\
        \n\
        要添加排除项，以管理员身份打开 PowerShell 并运行设置中 \
        Windows Defender 部分显示的两个命令。使用 📋 复制命令 按钮将其\
        复制到剪贴板，然后粘贴到 PowerShell 中并按 Enter。\n\
        \n\
        命令如下：\n\
          Add-MpPreference -ExclusionPath '<安装文件夹>'\n\
          Add-MpPreference -ExclusionProcess 'capture_bypass_gui.exe'\n\
        \n\
        可安全多次运行 — Defender 会忽略重复条目。",
    per_process_rules: "设置中的规则表允许您为特定进程覆盖全局注入模式。\n\
        \n\
        每个规则有三种模式：\n\
        \n\
        始终一次性   — 始终为此进程使用快速一次性 DLL，\n\
                       无论全局模式开关如何。\n\
        \n\
        始终持久模式 — 始终为此进程使用持久（重新应用）DLL。\n\
                       适合已知的对抗者。\n\
        \n\
        跳过         — 即使触发自动注入或剥离全部保护，\n\
                       也永不注入此进程。\n\
        \n\
        在规则输入框中输入可执行文件名（如 chrome.exe），\
        选择模式，然后点击添加。规则对进程名的匹配不区分大小写。",
    exclusion_list: "排除列表中的进程会被所有注入操作完全忽略 —\
        手动剥离、剥离全部保护和自动注入都会跳过它们。\n\
        \n\
        输入可执行文件名（如 SecurityAgent.exe）并点击添加来添加条目。\
        使用 ✕ 按钮删除条目。\n\
        \n\
        用于防止意外注入系统进程、杀毒软件或任何您不想处理的进程。",
    one_shot_mode: "载荷 DLL 剥离 WDA 保护一次后退出。快速且轻量。\n\
        \n\
        适用场景：目标应用仅在启动时设置保护且永不重新应用。\
        也适用于大多数浏览器和标准媒体播放器。",
    persistent_mode: "载荷 DLL 在目标进程内保持活跃，使用两层机制保持保护清除：\n\
        \n\
        1. IAT 钩子 — 在宿主模块的导入地址表中修补 \
        SetWindowDisplayAffinity，使调用在源头被拦截。\
        后续通过静态导入的每次调用都进入我们的跳转函数，\
        以零延迟立即强制 WDA_NONE。\n\
        \n\
        2. 轮询回退 — 后台线程仍每 5 秒调用 \
        SetWindowDisplayAffinity(WDA_NONE) 作为安全网，\
        处理注入时已受保护的窗口和通过 GetProcAddress 动态解析函数的应用。\n\
        \n\
        适用场景：目标应用通过定时器调用 SetWindowDisplayAffinity \
        来对抗一次性注入（如 DRM 视频播放器、具有反捕获机制的应用）。",
    re_injection: "Windows 按文件路径缓存已加载的 DLL — 如果进程中已加载相同的 DLL 路径，\
        LoadLibraryA 会静默无操作。\n\
        \n\
        如果一次性剥离似乎成功但状态徽章很快恢复为受保护，\
        说明应用正在定时器上重新应用保护。切换到 🔁 持久模式 — \
        检测到重新保护时也会自动弹出提示。\n\
        \n\
        进程规则允许您将特定可执行文件固定到持久模式，\
        从而无需为已知对抗者手动切换。",
    why_browsers: "Chrome、Edge、Firefox、Brave、Opera、Vivaldi 和 Thorium 使用多进程架构。\
        DRM 保护的视频在单独的子进程（渲染器）中渲染，该进程拥有自己的窗口。\
        仅注入主 PID 不会剥离视频窗口。",
    what_cb_does: "当您在浏览器行上点击 \"剥离保护\" 时，应用会自动：\n\
        \n\
        1. 将载荷注入主（浏览器）PID\n\
        2. 通过 CreateToolhelp32Snapshot 枚举所有子进程\n\
        3. 同时注入每个子 PID\n\
        \n\
        自动注入在检测到浏览器进程受保护时也会执行此递归子进程扫描。",
    tip: "如果浏览器在导航到新视频后重新应用保护，\
        再次点击 ⚡ 剥离全部保护，或启用 🤖 自动注入使其自动处理。\
        对于持续对抗的浏览器，添加固定到持久模式的进程规则。",
    system_tray: "点击窗口的 ✕ 关闭按钮会将应用隐藏到系统托盘而非退出 —\
        图标保留在通知区域。\n\
        \n\
        托盘图标右键菜单：\n\
          打开 — 恢复主窗口\n\
          退出 — 完全退出应用\n\
        \n\
        托盘图标根据当前状态改变颜色：\n\
          蓝色 — 未检测到受保护窗口\n\
          红色 — 当前检测到一个或多个受保护窗口\n\
        \n\
        工具提示更新显示受保护窗口数量，\
        使您无需打开应用即可监控状态。",
    auto_inject: "在工具栏中启用 🤖 自动注入开关。\n\
        \n\
        后台线程使用系统级 WinEvent 钩子 (EVENT_OBJECT_SHOW) \
        监视新受保护的窗口。当钩子触发时线程立即唤醒 —\
        典型检测延迟 < 50 毫秒。正常的 500 毫秒（或 100 毫秒快速扫描）\
        轮询间隔作为钩子可能遗漏的边缘情况的安全网。\n\
        \n\
        进程规则和排除列表会被尊重 — 设置为跳过或在排除列表中的进程\
        永远不会被自动注入。\n\
        \n\
        为主播设计：启用自动注入，最小化到托盘，\
        任何试图阻止捕获的应用都会在后台静默处理。\
        桌面通知（如果启用）确认每次剥离。",
    how_updates_work: "应用在启动时和运行期间定期检查 GitHub Releases 的较新版本标签。\n\
        \n\
        发现新版本时：\n\
        \n\
        1. 标题栏显示 \"🆕 vX.Y.Z 可用\" 按钮。\n\
        2. 点击打开确认对话框 — 在您明确确认之前不会开始下载。\n\
        3. 点击 \"立即更新\" 后，安装程序在后台下载。\n\
           标题栏显示进度指示器。\n\
        4. 下载完成后安装程序静默运行 (/SILENT)，\n\
           应用自动重启。确认后无需再次点击。\n\
        \n\
        更新一步替换所有应用二进制文件（GUI、载荷 DLL、压力测试器）。\
        您的 config.toml 会保留。",
    update_wrong: "安装程序下载到可执行文件旁边的临时文件。\
        如果下载失败，状态栏将显示错误。检查网络连接并重试。\n\
        \n\
        安装程序需要 GUI 已经运行的相同管理员令牌，因此 UAC 不应再次提示。",
    dlls_not_found: "载荷 DLL 尚未编译。运行：\n\
        \n\
        cargo build --release -p payload_dll -p payload_dll_persistent\n\
        \n\
        （如需 32 位支持，还需编译 i686 目标 — 参见环境要求与编译。）",
    strip_failed: "OpenProcess 对不属于您会话的进程需要提升权限。\
        确保 capture_bypass_gui.exe 以管理员身份运行（启动时出现 UAC 提示）。",
    injection_ok_but_black: "1. 等待下一次扫描刷新 — 如果状态徽章显示正常，\
        OBS 可能需要刷新捕获源（移除并重新添加）。\n\
        2. 对于浏览器，再次点击剥离保护 — 它也会重新注入子进程。\n\
        3. 如果徽章不断恢复为受保护，说明应用正在定时器上对抗。\
        切换到 🔁 持久模式，或为该可执行文件添加进程规则。",
    notifications_missing: "Windows 气球提示被专注助手 / 免打扰模式抑制。\
        检查操作中心设置并确保 \"仅限闹钟\" 模式未激活。\n\
        \n\
        capture-bypass 通过 Win32 Shell_NotifyIcon 发送通知\
        （非 WinRT toast），因此即使应用以管理员身份运行也能正常工作 —\
        但操作系统级别的抑制仍然适用。",
    antivirus_flags: "DLL 注入同时被合法工具和恶意软件使用，\
        因此启发式扫描器可能会标记载荷。检查 payload_dll/src/lib.rs — \
        它仅调用 SetWindowDisplayAffinity。\n\
        \n\
        在杀毒软件设置中为 target\\ 目录添加排除项。\
        还可以使用进程排除列表使 GUI 自身忽略杀毒软件监视的特定进程。",
    x86_fails: "64 位进程无法注入 32 位进程，反之亦然。\
        验证您已编译 x86 目标：\n\
        \n\
        rustup target add i686-pc-windows-msvc\n\
        cargo build --release --target i686-pc-windows-msvc -p payload_dll\n\
        \n\
        32 位进程显示为橙色 \"32\" 徽章。如果徽章为橙色且注入失败，\
        可能缺少 x86 DLL。",
    config_location: "配置以 config.toml 形式存储在 capture_bypass_gui.exe 所在目录中。\
        如果设置未保存，请检查目录是否可写。\
        还可以使用设置中的导出配置将副本保存到任何位置。",
};

pub fn help_body(lang: Language) -> &'static HelpBodyStrings {
    match lang {
        Language::English => &HELP_BODY_EN,
        Language::Chinese => &HELP_BODY_ZH,
    }
}
