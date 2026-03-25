# Notification Shell — Universal Tauri Overlay Notification System

## Context

Build a standalone Tauri v2 app that acts as a universal notification overlay shell. External processes send notifications via HTTP/WebSocket, and the app renders them as always-on-top overlay windows — bypassing OS notification center entirely. Works on macOS, Windows, and Linux.

**Reference implementation:** `/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend` (Meetily's meeting detection notifications — NSPanel on macOS, transparent overlay windows, notification stacking, action button callbacks).

## Confirmed Decisions

| Decision | Choice |
|----------|--------|
| IPC | HTTP REST (port 9876) + WebSocket (port 9877) |
| Rendering | Single transparent overlay window, notifications as CSS-animated DOM elements |
| App mode | System tray daemon + main app window (notification history) |
| Startup | Launch at login (configurable), starts silently (tray + overlay only) |
| Quit | Ctrl/Cmd+Q and tray Quit → confirmation dialog; window close → hide to background |
| Features | Full: actions, progress bars, grouping, custom themes/CSS, sounds, history, markdown |

---

## Architecture

```
External Process ──HTTP POST──▸ axum server ──▸ NotificationManager ──emit──▸ Frontend Overlay
External Process ──WebSocket──▸ tungstenite  ──▸     (shared Arc)    ──emit──▸ (single window)
                                                         │
                                                    ┌────┴────┐
                                                    │ History  │ (SQLite)
                                                    │ Sound    │ (rodio)
                                                    │ Lifecycle│ (timeout/callbacks)
                                                    └──────────┘
```

---

## Project Structure

```
notification-shell/
├── package.json                        # React + Vite frontend
├── vite.config.ts
├── index.html                          # Multi-window entry point (router decides)
├── src/                                # Frontend (React + TypeScript)
│   ├── main.tsx                        # Reads window label → renders overlay or app
│   ├── App.tsx                         # Router: overlay vs main-app based on window label
│   ├── types/
│   │   └── notification.ts             # Mirrors Rust types
│   ├── hooks/
│   │   ├── useNotifications.ts         # Tauri event subscription
│   │   ├── useClickThrough.ts          # Mouse tracking + setIgnoreCursorEvents toggle
│   │   ├── useHistory.ts              # Fetch + paginate notification history
│   │   └── useSound.ts
│   ├── components/
│   │   ├── overlay/                    # Overlay window components
│   │   │   ├── NotificationOverlay.tsx # Full-screen transparent container
│   │   │   ├── NotificationStack.tsx   # Positioning/stacking logic
│   │   │   ├── NotificationCard.tsx    # Single notification card
│   │   │   ├── NotificationGroup.tsx   # Grouped notifications
│   │   │   ├── ActionButton.tsx
│   │   │   ├── ProgressBar.tsx
│   │   │   └── MarkdownBody.tsx        # react-markdown wrapper
│   │   └── app/                        # Main app window components
│   │       ├── MainApp.tsx             # App shell — sidebar + content
│   │       ├── HistoryView.tsx         # Notification history list (default view)
│   │       ├── HistoryRow.tsx          # Single history entry with sender, time, priority
│   │       ├── HistoryFilters.tsx      # Filter by sender, priority, date range
│   │       ├── HistoryDetail.tsx       # Expanded view of a single notification
│   │       ├── Sidebar.tsx             # Nav: History, (future: Settings, Senders, etc.)
│   │       └── EmptyState.tsx          # "No notifications yet" placeholder
│   ├── stores/
│   │   ├── notificationStore.ts        # Zustand — active overlay notifications
│   │   └── historyStore.ts             # Zustand — history list, filters, pagination
│   └── styles/
│       ├── globals.css
│       ├── overlay.css                 # Overlay-specific (transparent bg, pointer-events)
│       ├── app.css                     # Main app window styles
│       ├── default-theme.css
│       └── animations.css              # slide-in, slide-out, reflow keyframes
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs                      # Tauri setup, plugin registration
│   │   ├── commands.rs                 # All #[tauri::command] functions
│   │   ├── notification/
│   │   │   ├── mod.rs
│   │   │   ├── types.rs               # NotificationPayload, Action, Progress, Priority
│   │   │   ├── manager.rs             # NotificationManager (Arc-shared state)
│   │   │   ├── lifecycle.rs           # Timeout scheduling, callback dispatch
│   │   │   └── history.rs             # SQLite-backed history
│   │   ├── server/
│   │   │   ├── mod.rs
│   │   │   ├── http.rs                # axum (POST /notify, GET /health, etc.)
│   │   │   ├── websocket.rs           # tokio-tungstenite bidirectional
│   │   │   └── protocol.rs            # Shared WS message types
│   │   ├── overlay/
│   │   │   ├── mod.rs
│   │   │   ├── window.rs              # Overlay window creation/management
│   │   │   └── click_through.rs       # Platform-specific mouse polling
│   │   ├── sound/
│   │   │   └── player.rs              # rodio playback
│   │   ├── tray/
│   │   │   └── menu.rs                # System tray icon + dynamic menu
│   │   └── config/
│   │       └── settings.rs            # Persisted app settings
│   └── migrations/
│       └── 001_create_history.sql
├── cli/
│   ├── Cargo.toml                      # Standalone binary
│   └── src/main.rs                     # `notify-shell send/dismiss/list/status`
└── sounds/
    ├── default.wav
    ├── success.wav
    └── error.wav
```

---

## Notification Payload (JSON Schema)

```json
{
  "sender": "ci-pipeline",
  "title": "Build Complete",
  "body": "**main** built in 3m 42s\n- 142 tests passed\n- Coverage: 87%",
  "icon": "https://github.com/favicon.ico",
  "priority": "normal",
  "timeout": { "seconds": 15 },
  "actions": [
    { "id": "open_pr", "label": "Open PR", "style": "primary" },
    { "id": "dismiss", "label": "Dismiss", "style": "secondary" }
  ],
  "progress": { "value": 0.75, "label": "3 of 4", "style": "bar" },
  "group": "ci-builds",
  "theme": "github-dark",
  "sound": "success",
  "callback_url": "http://localhost:8080/callback"
}
```

### Rust Types (`src-tauri/src/notification/types.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub id: String,                     // UUID, auto-generated if not provided
    pub sender: String,                 // e.g. "github-actions", "slack-bot"
    pub title: String,
    pub body: String,                   // Markdown-supported
    pub icon: Option<String>,           // URL, file path, or base64 data URI
    pub priority: Priority,             // low | normal | high | critical
    pub timeout: Timeout,               // never | default | Seconds(u64)
    pub actions: Vec<Action>,           // Up to 3 action buttons
    pub progress: Option<ProgressInfo>, // Optional progress bar
    pub group: Option<String>,          // Group key for stacking
    pub theme: Option<String>,          // CSS class name or inline CSS
    pub sound: Option<String>,          // Sound file name or "none"
    pub callback_url: Option<String>,   // URL to POST when action clicked
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub style: ActionStyle,             // primary | secondary | danger
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub value: f64,                     // 0.0 - 1.0
    pub label: Option<String>,
    pub style: ProgressStyle,           // bar | ring
}

pub enum Priority { Low, Normal, High, Critical }
pub enum Timeout { Never, Default, Seconds(u64) }
pub enum ActionStyle { Primary, Secondary, Danger }
pub enum ProgressStyle { Bar, Ring }
```

---

## HTTP Server (axum, port 9876)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/notify` | Send notification → returns `{ "id": "uuid" }` |
| `POST` | `/notify/{id}/update` | Update progress/body of existing notification |
| `POST` | `/notify/{id}/dismiss` | Dismiss specific notification |
| `POST` | `/dismiss-all` | Dismiss all |
| `GET` | `/health` | `{ "status": "ok", "active_count": N }` |
| `GET` | `/history` | Query with `?sender=X&limit=50` |

---

## WebSocket Server (tokio-tungstenite, port 9877)

**Inbound (client → shell):**
```json
{ "type": "notify", "payload": { ... } }
{ "type": "update", "id": "uuid", "payload": { "progress": { "value": 0.75 } } }
{ "type": "dismiss", "id": "uuid" }
{ "type": "dismiss_all" }
{ "type": "subscribe", "sender": "my-app" }
```

**Outbound (shell → client):**
```json
{ "type": "action", "notification_id": "uuid", "action_id": "approve" }
{ "type": "dismissed", "notification_id": "uuid", "reason": "timeout|user|replaced" }
```

---

## Click-Through Strategy (hardest part)

**Problem:** Single overlay window must pass clicks through to apps behind it, EXCEPT on notification cards.

**Solution — two-phase toggle:**

1. **Default:** `setIgnoreCursorEvents(true)` — all clicks pass through
2. **Rust-side mouse poller** (60fps `tokio::time::interval`):
   - **macOS:** `CGEvent::mouseLocation()` via `core-graphics` crate
   - **Windows:** `GetCursorPos` via `windows` crate
   - **Linux X11:** `XQueryPointer` via `x11` crate
3. **Frontend** reports card bounding rects to Rust via `invoke('update_card_rects', { rects })`
4. When Rust detects mouse over a card rect → `set_ignore_cursor_events(false)` → window becomes interactive
5. Frontend `mouseleave` on card → `set_ignore_cursor_events(true)` → back to pass-through

**Key insight:** The `true → false` toggle happens from Rust (which always knows mouse position). The `false → true` toggle happens from frontend (which receives `mouseleave` while window is interactive).

**Linux Wayland fallback:** Use `wlr-layer-shell` for overlay if available. If not (GNOME), fall back to individual small Tauri windows per notification.

---

## Window Configuration

### Overlay Window (always running, invisible chrome)

```rust
// src-tauri/src/overlay/window.rs
WebviewWindowBuilder::new(&app, "overlay", WebviewUrl::App("index.html".into()))
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(false)
    .focused(false)
    .resizable(false)
    .visible(true)
    .inner_size(screen_width, screen_height)
    .position(0.0, 0.0)
```

Frontend CSS: `pointer-events: none` on body, `pointer-events: auto` on `.notification-card`.

### Main App Window (on-demand, opened from tray or dock)

```rust
// src-tauri/src/app_window.rs
WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
    .title("syncfu")
    .inner_size(900.0, 640.0)
    .min_inner_size(600.0, 400.0)
    .decorations(true)
    .resizable(true)
    .center()
    .visible(true)
    .focused(true)
```

**Multi-window routing:** Both windows load `index.html`. The frontend reads the Tauri window label (`getCurrent().label`) in `main.tsx` and renders:
- `"overlay"` → `<NotificationOverlay />` (transparent, pointer-events logic)
- `"main"` → `<MainApp />` (standard app shell with history view)

**Launch at startup:**
- Register as login item on all platforms:
  - **macOS:** `tauri-plugin-autostart` (uses `SMAppService` / Launch Agent)
  - **Windows:** Registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
  - **Linux:** `.desktop` file in `~/.config/autostart/`
- On first launch after install, prompt user: "Start syncfu automatically on login?" → save preference in settings
- App starts silently (tray + overlay only, main window hidden)

**Window lifecycle:**
- Overlay window: created at startup, never closed (hidden when paused)
- Main app window: created on first open (tray menu "Open syncfu" or dock click), hidden on close (not destroyed), re-shown on subsequent opens
- On macOS: clicking the dock icon re-shows the main window via `reopen` event

**Quit behavior:**
- **Cmd+Q / Ctrl+Q (keyboard shortcut):** Intercept via Tauri's `on_window_event`. Show confirmation dialog: *"syncfu is still listening for notifications. Quit anyway?"* with **Quit** and **Send to Background** buttons. "Send to Background" hides the main window and keeps the tray daemon running.
- **Window close button (✕):** Always hides the main window — never quits the app. The overlay and tray remain active.
- **Tray menu → Quit:** Show the same confirmation dialog. This is the only way to fully quit.
- **Rationale:** syncfu should be hard to accidentally kill since background agents depend on it being alive. Closing the window should feel like minimizing, not exiting.

---

## Main App Window — History UI

The main app window is a standard desktop window opened from the system tray or dock. Default view: **Notification History**.

### Layout

```
┌─────────────────────────────────────────────────────────┐
│  syncfu                                          ─ □ ✕  │
├──────────┬──────────────────────────────────────────────┤
│          │  ┌─ Filters ──────────────────────────────┐  │
│ History  │  │ [Search...]  [Sender ▾]  [Priority ▾]  │  │
│          │  │ [Today] [7d] [30d] [All]               │  │
│ ──────── │  └────────────────────────────────────────┘  │
│          │                                              │
│ Settings │  ┌─ History ──────────────────────────────┐  │
│ (future) │  │ 🟢 ci-pipeline    Build passed   2m ago│  │
│          │  │ 🔴 test-watcher   3 tests fail  14m ago│  │
│          │  │ 🟡 remind         Stand-up in 5m 1h ago│  │
│          │  │ ⚪ deploy          v2.3.1 live   3h ago│  │
│          │  │ ...                                    │  │
│          │  └────────────────────────────────────────┘  │
│          │                                              │
│          │  ┌─ Detail (click a row) ─────────────────┐  │
│          │  │ **Build passed** — ci-pipeline          │  │
│          │  │ main built in 3m 42s                    │  │
│          │  │ - 142 tests passed                      │  │
│          │  │ - Coverage: 87%                         │  │
│          │  │                                         │  │
│          │  │ Action taken: "Open PR" at 14:32        │  │
│          │  │ Callback: 200 OK                        │  │
│          │  └─────────────────────────────────────────┘  │
└──────────┴──────────────────────────────────────────────┘
```

### History list columns

| Column | Source | Display |
|--------|--------|---------|
| Priority indicator | `priority` | Colored dot (green=low, blue=normal, yellow=high, red=critical) |
| Sender | `sender` | Truncated, monospace |
| Title | `title` | Main text |
| Time | `created_at` | Relative ("2m ago", "1h ago", "yesterday") |
| Status | `dismissed_at`, `action_taken` | Icon: ✓ dismissed, ⚡ action taken, ⏱ timed out |

### Filters

- **Search**: full-text across title + body + sender
- **Sender**: dropdown populated from distinct senders in history
- **Priority**: multi-select (low, normal, high, critical)
- **Date range**: quick picks (Today, 7d, 30d, All) + custom range

### Detail panel

Clicking a history row expands the detail panel showing:
- Full body rendered as markdown
- All actions that were available
- Which action was taken (if any) and when
- Callback URL and response status
- Timestamps: created, dismissed

### Tauri commands for history

```rust
#[tauri::command]
async fn get_history(
    manager: State<'_, Arc<NotificationManager>>,
    sender: Option<String>,
    priority: Option<String>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<HistoryEntry>, String>

#[tauri::command]
async fn get_history_entry(
    manager: State<'_, Arc<NotificationManager>>,
    id: String,
) -> Result<HistoryEntry, String>

#[tauri::command]
async fn get_senders(
    manager: State<'_, Arc<NotificationManager>>,
) -> Result<Vec<String>, String>
```

---

## NotificationManager (Rust)

```rust
// src-tauri/src/notification/manager.rs
pub struct NotificationManager {
    active: RwLock<IndexMap<String, NotificationPayload>>,
    groups: RwLock<HashMap<String, Vec<String>>>,
    history: Arc<NotificationHistory>,
    app_handle: AppHandle,
    settings: RwLock<AppSettings>,
}
```

Shared via `Arc<NotificationManager>` between Tauri commands, HTTP server, and WebSocket server.

**Key operations:**
- `add(payload) → String` — assigns ID, stores, emits `notification:add` to frontend, schedules timeout
- `dismiss(id)` — removes from active, fires `notification:dismiss`, triggers callback
- `update(id, partial)` — updates fields (e.g. progress), emits `notification:update`
- `action_clicked(id, action_id)` — POSTs to `callback_url` / sends WS callback, dismisses

---

## Sound Playback

`rodio` crate. Pre-load `.wav` files from bundled resources at startup. Managed via `app.manage(Arc<SoundPlayer>)`.

Mapping: `"default"` → `default.wav`, `"success"` → `success.wav`, `"error"` → `error.wav`, `"none"` → silent.

---

## History (SQLite)

```sql
-- src-tauri/migrations/001_create_history.sql
CREATE TABLE notification_history (
    id TEXT PRIMARY KEY,
    sender TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    priority TEXT NOT NULL,
    group_key TEXT,
    actions_json TEXT,
    created_at TEXT NOT NULL,
    dismissed_at TEXT,
    action_taken TEXT,
    callback_result TEXT
);
```

Max 10k rows, auto-pruned on startup. Stored in `{app_data_dir}/notification-shell/history.db`.

---

## System Tray

Dynamic menu with state-dependent items:
- **Open syncfu** (shows/creates main app window)
- Active Notifications count
- Pause All / Resume
- Clear All
- Settings (opens settings in main app window)
- Start at Login ✓ / Start at Login (toggle)
- Server status (port, running/stopped)
- Quit (shows confirmation dialog)

Icon states: bell (normal), bell+dot (has notifications), bell+slash (paused).

**Dock behavior (macOS):** App shows in Dock when main window is visible. Clicking dock icon re-opens main window. When main window is closed/hidden, app remains as tray-only daemon.

---

## CLI Wrapper (`cli/`)

Standalone Rust binary (`notify-shell-cli`) using `clap` + `reqwest`:

```bash
notify-shell send --title "Done" --body "All tests passed" --sound success --priority normal
notify-shell send --title "Deploy" --body "Deploying..." --progress 0.5 --group deploys
notify-shell dismiss <id>
notify-shell dismiss-all
notify-shell list
notify-shell history --sender ci --limit 20
notify-shell status
```

---

## Key Crates

```toml
# src-tauri/Cargo.toml
tauri = { version = "2", features = ["macos-private-api", "tray-icon"] }
tauri-plugin-single-instance = "2"
tauri-plugin-autostart = "2"
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
tokio-tungstenite = "0.26"
futures-util = "0.3"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "chrono"] }
rodio = "0.19"
reqwest = { version = "0.12", features = ["json"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dashmap = "6"
indexmap = { version = "2", features = ["serde"] }
anyhow = "1"
dirs = "5"

[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.23"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = ["Win32_UI_WindowsAndMessaging"] }
```

```json
// Frontend dependencies
{
  "react": "^18",
  "react-dom": "^18",
  "@tauri-apps/api": "^2",
  "zustand": "^5",
  "react-markdown": "^9",
  "remark-gfm": "^4"
}
```

---

## Reference Files (from Meetily)

These files contain patterns to reuse/adapt:

| File | What to reuse |
|------|---------------|
| [`src-tauri/src/overlay.rs`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src-tauri/src/overlay.rs) | Overlay window creation, monitor-aware positioning, `WebviewWindowBuilder` config |
| [`swift-notifications/src/lib.swift`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src-tauri/swift-notifications/src/lib.swift) | Notification stacking math (10px spacing, top-right positioning), animation curves, hover tracking with global mouse monitors |
| [`src-tauri/src/notifications/manager.rs`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src-tauri/src/notifications/manager.rs) | `NotificationManager` architecture — `Arc<RwLock>` pattern, `AppHandle` integration, settings coordination |
| [`src-tauri/src/notifications/types.rs`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src-tauri/src/notifications/types.rs) | Notification types, priority enum, timeout enum — adapt for our richer payload |
| [`src-tauri/src/lib.rs`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src-tauri/src/lib.rs) | Tauri v2 setup pattern: `.manage()` for shared state, `.setup()` for async init, plugin registration |
| [`src-tauri/src/tray.rs`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src-tauri/src/tray.rs) | Dynamic tray menu construction, async state queries in menu handlers |
| [`src/components/FloatingRecordingButton.tsx`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src/components/FloatingRecordingButton.tsx) | Transparent overlay component patterns, fade-in animation, Tauri command invocation from React |
| [`src/hooks/useMeetingDetection.ts`](/Users/sujith/work/2025/tests/meeting-minutes-safvan/frontend/src/hooks/useMeetingDetection.ts) | Tauri event listening pattern (`listen<T>('event-name', callback)`) |

---

## Implementation Phases

### Phase 1: Scaffold + Overlay + Main App Window
- Create Tauri v2 app with React+Vite template
- Configure `tauri.conf.json`: no default window, tray enabled, `macOSPrivateApi: true`
- Minimal system tray ("Open syncfu" + Quit)
- Create overlay window (transparent, always-on-top, fullscreen, no decorations)
- Create main app window (standard decorated window, 900x640, hidden by default)
- Multi-window routing in frontend: read window label → render overlay or app shell
- Main app shows notification history placeholder (empty state)
- **Verify:** tray icon visible, "Open syncfu" shows main app window, overlay renders transparently

### Phase 2: Notification Types + Manager
- Define all Rust types in `notification/types.rs`
- Implement `NotificationManager` with `add()` / `dismiss()` / `update()`
- Wire Tauri events (`notification:add`, `notification:dismiss`, `notification:update`)
- Zustand store + `NotificationCard` component + `NotificationStack` positioning
- CSS animations (slide-in-right, slide-out-right, reflow)
- **Verify:** `test_notify` Tauri command shows animated notification in overlay

### Phase 3: Click-Through
- CSS `pointer-events: none` on body, `auto` on cards
- Rust-side mouse position polling (per-platform: CoreGraphics / Win32 / X11)
- `update_card_rects` command from frontend → Rust
- Toggle `setIgnoreCursorEvents` based on mouse-over-card detection
- **Verify:** clicks pass through empty overlay space, notification cards are interactive

### Phase 4: HTTP + WebSocket Servers
- axum HTTP server with all endpoints
- tokio-tungstenite WebSocket server with bidirectional protocol
- Share `Arc<NotificationManager>` between servers and Tauri
- Build CLI wrapper binary
- **Verify:** `curl POST /notify` shows notification, `websocat` receives action callbacks

### Phase 5: Rich Features
- Action buttons + callback dispatch (HTTP POST to `callback_url` + WS outbound)
- Progress bars (update via `POST /notify/{id}/update` or WS `update` message)
- Markdown body rendering (`react-markdown` + `remark-gfm`)
- Notification grouping (collapse/expand by `group` key)
- Sound playback (`rodio` — pre-loaded `.wav` files)
- Custom themes/CSS per sender

### Phase 6: History + Settings + Polish
- SQLite history via `sqlx`
- Wire history into main app window:
  - `HistoryView`: paginated list of past notifications (sender icon, title, time, priority badge)
  - `HistoryFilters`: filter by sender, priority, date range, search text
  - `HistoryDetail`: click a row to see full body (rendered markdown), actions taken, callback results
  - Real-time updates: new notifications appear at top of history while app is open
- Settings view in main app (port, position, default timeout, DND, sounds toggle)
- Sidebar navigation: History (default), Settings, (future expansion slots)
- Multi-monitor support (position on primary monitor)
- Max 5 visible notifications, queue overflow
- Cross-platform testing: macOS, Windows, Linux X11, Linux Wayland

---

## Verification Plan

| Test | Command / Action | Expected Result |
|------|-----------------|-----------------|
| Overlay renders | Launch app | Transparent overlay visible, tray icon shows |
| Main app opens | Click "Open syncfu" in tray | Main app window opens with history view, centered |
| Main app hides | Close main app window | Window hides (not destroyed), tray remains, overlay still active |
| Dock reopen | Click dock icon (macOS) | Main app window re-appears |
| Click-through | Click on desktop through overlay | Click reaches app behind overlay |
| Card interactive | Hover notification card | Card responds to hover, buttons clickable |
| HTTP notify | `curl -X POST localhost:9876/notify -d '{"sender":"test","title":"Hello","body":"World"}'` | Notification slides in from right |
| WebSocket | `websocat ws://localhost:9877` → send notify JSON | Notification appears; click action → receive callback |
| CLI | `notify-shell send --title "Test" --body "Works"` | Notification appears |
| Action callback | Click action button on notification | POST to `callback_url` / WS message sent to sender |
| Progress update | `POST /notify/{id}/update` with new progress value | Progress bar animates |
| Sound | Notification with `"sound": "success"` | Audio plays |
| History | Send + dismiss notifications → `GET /history` | Returns dismissed notifications |
| Multi-platform | Test on macOS, Windows, Linux | All features work (Wayland with known limitations) |
