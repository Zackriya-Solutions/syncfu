# Architecture Comparison: Notification & Overlay Systems

> Comparing syncfu's current architecture against 4 real-world Tauri apps to identify
> the optimal cross-platform notification overlay strategy.

---

## Executive Summary

syncfu currently uses a **full-screen transparent WebviewWindow** as a single overlay surface.
This approach has fundamental problems: the overlay doesn't render notifications visually,
click-through requires complex cursor polling, and it's not cross-platform reliable.

Every production Tauri app studied here uses a **different strategy**: small, positioned,
platform-native floating windows. The clear winner is the **NSPanel (macOS) + Win32
HWND_TOPMOST (Windows) + GTK Layer Shell (Linux)** pattern used by both Handy and Cap.

---

## Projects Compared

| Project | Purpose | Stack | Multi-window? | Production? |
|---------|---------|-------|---------------|-------------|
| **syncfu** (current) | Notification overlay for AI agents | Tauri 2 + React + Zustand | 2 windows (overlay + main) | In development |
| **Handy** | Offline speech-to-text | Tauri 2 + React + Zustand | 2 windows (settings + overlay) | Yes, shipped |
| **Cap** | Screen recording (Loom alt) | Tauri 2.5 + SolidJS + specta | 10+ window types | Yes, shipped |
| **Meetily Pro** | Meeting recording & transcription | Tauri 2 + Next.js + React | 3 windows (main + overlay + license) | Yes, shipped |
| **tauri-plugin-liquid-glass** | Native glass effects plugin | Tauri 2 plugin (Rust + TS) | N/A (plugin) | Published |

---

## 1. Overlay Window Strategy

### syncfu (Current) — Full-Screen Transparent Surface

```
┌─────────────────────────────────────────┐
│  Overlay Window (FULLSCREEN)            │
│  transparent=true, always_on_top=true   │
│  pointer-events: none on root           │
│  pointer-events: auto on cards          │
│                                         │
│            ┌──────────┐                 │
│            │ Card 1   │ ← 380px wide    │
│            │ Card 2   │    top-right    │
│            │ Card 3   │                 │
│            └──────────┘                 │
│                                         │
│  Problem: entire screen covered by      │
│  WebviewWindow. Clicks must pass thru.  │
│  CSS pointer-events ≠ OS-level pass.    │
└─────────────────────────────────────────┘
```

**Problems:**
- Full-screen transparent window consumes resources even when empty
- CSS `pointer-events: none` doesn't pass clicks to OS windows beneath on all platforms
- Requires complex `setIgnoreCursorEvents` polling (Phase 3 — not implemented)
- macOS: transparent fullscreen windows have compositing overhead
- Windows/Linux: inconsistent always-on-top behavior for fullscreen windows
- Not reliable for multi-monitor setups

### Handy — Small Positioned Platform-Native Panel

```
┌──────────────────────────┐
│  Regular Desktop         │
│                          │
│   ┌──────────────────┐   │  ← 172×36px NSPanel
│   │ 🎙 |||||| Cancel │   │    centered on cursor's monitor
│   └──────────────────┘   │    non-activating, non-focusing
│                          │
│  No fullscreen overlay.  │
│  Just a tiny floating    │
│  panel above everything. │
└──────────────────────────┘
```

**Platform-specific implementations:**

| Platform | API | Key Properties |
|----------|-----|---------------|
| macOS | `tauri-nspanel` → NSPanel | `PanelLevel::Status`, `can_join_all_spaces()`, `no_activate(true)`, `is_floating_panel: true` |
| Windows | WebviewWindow + Win32 | `always_on_top(true)` + `SetWindowPos(HWND_TOPMOST, ..., SWP_NOACTIVATE)` |
| Linux | GTK Layer Shell (Wayland) | `Layer::Overlay`, `KeyboardMode::None`, anchor to `Edge::Top/Bottom` |
| Linux fallback | Regular always-on-top | If layer-shell unavailable (e.g. KDE Wayland) |

**Key insight:** Handy creates the overlay window ONCE at startup, then shows/hides it. Position
is recalculated each time based on the monitor containing the mouse cursor.

### Cap — Per-Display Multi-Window Overlays

```
Monitor 1                    Monitor 2
┌──────────────────────┐     ┌──────────────────────┐
│                      │     │                      │
│  ┌────────────────┐  │     │  ┌────────────────┐  │
│  │ Target Select  │  │     │  │ Target Select  │  │
│  │ Overlay (M1)   │  │     │  │ Overlay (M2)   │  │
│  └────────────────┘  │     │  └────────────────┘  │
│                      │     │                      │
│  ┌──────────┐        │     │                      │
│  │Recording │        │     │                      │
│  │Controls  │ 320×150│     │                      │
│  └──────────┘        │     │                      │
└──────────────────────┘     └──────────────────────┘
```

**Window types for overlay purposes:**
- `RecordingControls` — NSPanel (macOS) or WebviewWindow, bottom-center, 320×150px
- `RecordingsOverlay` — Transparent floating, bottom-left, shows recent recordings
- `TargetSelectOverlay` — **One per connected display**, fullscreen on each monitor
- `WindowCaptureOccluder` — Per-display semi-transparent darkening

**Cap's approach:**
- Uses `tauri-nspanel` for macOS recording controls (PanelManager state machine)
- Creates overlay windows dynamically per display
- Each window type has explicit `is_transparent()` flag
- Frontend sets `data-transparent-window` attribute + `body.style.background = "transparent"`

### Meetily Pro — Single Positioned Overlay Button

```
┌──────────────────────────────┐
│                              │
│        ┌─────────────┐       │
│        │ ⏸ 🔴 ||||| │ ← 180×40px, always-on-top
│        └─────────────┘       │
│                              │
│  Position calculated from    │
│  monitor work_area + scale   │
│  factor. 9 position slots.   │
└──────────────────────────────┘
```

**Position calculation:**
```
work_area = monitor.work_area()  (excludes menu bar, taskbar)
scale = monitor.scale_factor()   (handles Retina/HiDPI)

(Left|Center|Right, Top|Center|Bottom) → 9 positions
  e.g. (Center, Top) → x = work_area.x + (width - 180)/2, y = work_area.y + 40
```

**Properties:**
- `decorations(false), transparent(true), always_on_top(true), skip_taskbar(true), focused(false)`
- Window is created on recording start, hidden (not destroyed) on stop
- Sends commands back to backend (stop/pause/resume via `invoke()`)
- Receives `audio-visualization` events for live level bars

---

## 2. Event & IPC Architecture

### syncfu (Current)

```
HTTP POST /notify
    → axum handler
    → NotificationManager.add()
    → app.emit("notification:add", payload)
    → tauriEvent.listen() in useNotifications hook
    → Zustand store update
    → React re-render
```

**Limitation:** Single global emit — all windows receive all events. No window-targeted
events. The overlay and main window both subscribe to the same events.

### Handy — Coordinator + Typed Events (specta)

```
Shortcut press
    → TranscriptionCoordinator (mpsc channel, single-threaded)
    → State machine: Idle → Recording → Processing → Idle
    → app.emit("show-overlay", "recording")
    → app.emit("mic-level", Vec<f32>)
    → Frontend overlay listens and renders

Commands: tauri-specta auto-generates TypeScript bindings from #[tauri::command]
```

**Key pattern:** `TranscriptionCoordinator` serializes ALL events through a single mpsc
channel. This prevents race conditions between shortcuts, CLI signals, and async work.
30ms debounce prevents duplicate triggers.

### Cap — Typed Events + Watch Channels

```
Recording start
    → RecordingEvent::Started.emit(&app)    // tauri_specta::Event
    → All frontends receive typed event
    → Camera frames via WebSocket (watch::channel)
    → Progress via TanStack Query + Channel<UploadProgress>
```

**Key patterns:**
- `tauri_specta::Event` for type-safe Rust↔TS events (auto-generated)
- Watch channels for high-frequency data (camera frames)
- Actor model (Kameo) for microphone/camera feeds
- Central `App` struct with `Arc<RwLock<T>>` for shared state

### Meetily Pro — Events + Polling Hybrid

```
Backend: app.emit("recording-started")
         app.emit("audio-visualization", levels)

Frontend: listen("recording-started") → setState
          setInterval(500ms) → invoke("get_recording_state") → setState
```

**Key insight:** Uses BOTH events and polling. Events for real-time triggers,
polling every 500ms for state synchronization. Belt-and-suspenders approach.

---

## 3. Cross-Platform Window Behavior

This is the most critical comparison for syncfu.

### macOS

| App | API | Level | Focus Behavior | Spaces |
|-----|-----|-------|---------------|--------|
| syncfu | WebviewWindow | `always_on_top(true)` | `focused(false)` | Current space only |
| Handy | `tauri-nspanel` → NSPanel | `PanelLevel::Status` | `no_activate(true)`, `can_become_key: false` | All spaces (`can_join_all_spaces`) |
| Cap | `tauri-nspanel` → NSPanel | PanelManager state machine | Non-activating | Per-space |
| Meetily | WebviewWindow | `always_on_top(true)` | `focused(false)` | Current space only |

**Winner:** NSPanel. It's the correct macOS primitive for floating utilities:
- Doesn't appear in Mission Control
- Doesn't steal focus from the active app
- Can join all Spaces (critical for always-visible notifications)
- Proper z-ordering above windows but below system UI
- Non-activating (clicking it doesn't deactivate the frontmost app)

### Windows

| App | API | Z-Order | Focus |
|-----|-----|---------|-------|
| syncfu | WebviewWindow | `always_on_top(true)` | `focused(false)` |
| Handy | WebviewWindow + Win32 | `SetWindowPos(HWND_TOPMOST)` after show | `SWP_NOACTIVATE` |
| Meetily | WebviewWindow | `always_on_top(true)` | `focused(false)` |

**Winner:** Handy's approach — Tauri's `always_on_top` alone isn't reliable on Windows.
Other windows can steal the topmost position. Re-asserting with `SetWindowPos(HWND_TOPMOST, ...,
SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE)` after show ensures consistent z-order.

### Linux (Wayland/X11)

| App | API | Compositor Support |
|-----|-----|--------------------|
| syncfu | WebviewWindow | `always_on_top(true)` only |
| Handy | GTK Layer Shell | Wayland-native overlay protocol |

**Winner:** Handy. GTK Layer Shell is THE correct way to do overlays on Wayland:
- `Layer::Overlay` for z-order above all windows
- `KeyboardMode::None` to not intercept keyboard
- Anchors to screen edges for positioning
- Falls back to always-on-top if layer-shell unavailable
- Special detection for KDE Wayland (skips layer-shell due to protocol instability)

---

## 4. Click-Through & Interaction

### syncfu (Current)

```css
.overlay-root { pointer-events: none; }     /* CSS-level pass-through */
.notification-card { pointer-events: auto; } /* Re-enable on cards */
```

**Problem:** CSS `pointer-events` only works within the web rendering engine. The OS
still sees the fullscreen window. Requires Tauri's `setIgnoreCursorEvents(true)` with
mouse position polling to detect when cursor is over a card vs. empty space.
This is Phase 3 in PLAN.md and NOT implemented.

### Handy / Cap / Meetily — No Click-Through Needed

```
Small window (172×36px or 180×40px)
    → Entire window is interactive
    → No click-through required
    → Desktop beneath is naturally clickable
    → No polling, no cursor detection
```

**This eliminates the entire click-through problem.** Small positioned windows don't
cover the desktop, so there's nothing to click through.

---

## 5. State Management

| App | Backend State | Frontend State | Sync Mechanism |
|-----|--------------|----------------|---------------|
| syncfu | `RwLock<IndexMap>` in NotificationManager | Zustand store (immutable) | Tauri events only |
| Handy | Managers (Audio, Model, Transcription, History) | Zustand + immer | Events + specta commands |
| Cap | Central `App` struct with `Arc<RwLock>` | SolidJS signals | specta events + watch channels |
| Meetily | `AtomicBool` + `Mutex` globals | React Context (8 contexts) | Events + 500ms polling |

**Observations:**
- syncfu's state management is actually well-designed (Zustand + immutable patterns)
- But missing: persistence (SQLite for history), typed bindings (specta), polling fallback
- Cap's `tauri_specta` approach provides compile-time type safety for all IPC

---

## 6. Architecture Patterns Worth Adopting

### From tauri-plugin-liquid-glass

| Pattern | What It Does | How to Apply |
|---------|-------------|-------------|
| **Registry Pattern** | `Mutex<HashMap<String, ViewEntry>>` tracks all active views | Track active notification windows by ID |
| **Main Thread Dispatch** | `run_on_main_sync()` ensures all UI ops on main thread | Wrap all window creation/manipulation |
| **Extension Trait** | `LiquidGlassExt` extends AppHandle with custom methods | `NotificationExt` on AppHandle for `.show_notification()` |
| **Strategy Pattern** | Different backends per platform version | Platform-specific overlay implementations |

### From Handy

| Pattern | What It Does | How to Apply |
|---------|-------------|-------------|
| **Platform overlay abstraction** | NSPanel/Win32/GTK layer-shell with unified API | Three platform impls behind one trait |
| **Coordinator pattern** | Single-threaded mpsc serializes all state transitions | Route all notification ops through coordinator |
| **Monitor-aware positioning** | Calculate position from cursor's monitor | Position notification stack on active monitor |
| **Create once, show/hide** | Overlay created at startup, toggled visibility | Don't recreate windows per notification |

### From Cap

| Pattern | What It Does | How to Apply |
|---------|-------------|-------------|
| **Per-display windows** | Create overlay windows for each connected monitor | Support multi-monitor notification routing |
| **tauri_specta typed events** | Auto-generate TS types from Rust events | Eliminate serialization mismatches |
| **PanelManager state machine** | Creating → Ready → Destroying lifecycle | Manage notification window lifecycle safely |
| **Window type enum** | Centralized window type definitions with properties | `OverlayWindow` enum with `is_transparent()`, `size()`, etc. |

### From Meetily Pro

| Pattern | What It Does | How to Apply |
|---------|-------------|-------------|
| **Work area calculation** | Position from `monitor.work_area()` (excludes bars) | Avoid placing notifications behind menu bar/taskbar |
| **Scale factor handling** | Divide by `monitor.scale_factor()` for logical coords | Support Retina/HiDPI displays correctly |
| **Events + polling hybrid** | Events for triggers, polling for state sync | Add periodic state reconciliation |
| **Window hide (not close)** | Close button hides, tray reopens | Keep overlay allocated, toggle visibility |

---

## 7. Recommended Architecture Change

### Current: Full-Screen Transparent Overlay

```
┌─────────────────── FULLSCREEN WEBVIEW ────────────────────┐
│  transparent, always_on_top                                │
│  CSS pointer-events: none                                  │
│                         ┌──────────┐                       │
│                         │ Card     │                       │
│                         │ Card     │                       │
│                         └──────────┘                       │
│  PROBLEM: Needs cursor polling for click-through           │
│  PROBLEM: Single monitor only                              │
│  PROBLEM: Compositing overhead                             │
└────────────────────────────────────────────────────────────┘
```

### Proposed: Small Positioned Notification Window (Per-Monitor)

```
Desktop (no fullscreen overlay)
│
├── Monitor 1
│   └── NotificationPanel (380×600px max, top-right)
│       ├── macOS: NSPanel (PanelLevel::Status, non-activating)
│       ├── Windows: WebviewWindow + SetWindowPos(HWND_TOPMOST)
│       └── Linux: GTK Layer Shell (Layer::Overlay) or fallback
│
├── Monitor 2 (optional, for multi-monitor routing)
│   └── NotificationPanel (same)
│
└── Main Window (on-demand, from tray)
    └── History, settings, test UI
```

### Key Changes

| Aspect | Current | Proposed |
|--------|---------|----------|
| **Overlay size** | Full-screen (1920×1080+) | Notification stack only (~380×600px) |
| **Click-through** | Needs cursor polling + `setIgnoreCursorEvents` | Not needed — small window doesn't cover desktop |
| **macOS** | `WebviewWindow.always_on_top(true)` | `tauri-nspanel` → NSPanel with `PanelLevel::Status` |
| **Windows** | `always_on_top(true)` only | + `SetWindowPos(HWND_TOPMOST, SWP_NOACTIVATE)` |
| **Linux** | `always_on_top(true)` only | GTK Layer Shell with fallback |
| **Multi-monitor** | Single monitor | Per-monitor notification panels |
| **Spaces (macOS)** | Current space only | `can_join_all_spaces()` via NSPanel |
| **Resource usage** | Always compositing fullscreen | Only compositing when notifications visible |
| **Window creation** | Created at startup, always visible | Created at startup, hidden when no notifications |
| **IPC types** | Manual JSON serialization | `tauri_specta` for type-safe bindings |
| **State coordination** | Direct function calls | Coordinator pattern (mpsc channel) |

### What We Keep (Already Good)

- HTTP server on port 9868 (well-designed, tested)
- NotificationManager with RwLock + IndexMap (solid)
- Zustand store with immutable patterns (correct approach)
- NotificationCard component design (good UI)
- System tray integration (working)
- tauri-plugin-log integration (proper)
- Test infrastructure (103 tests, 97.56% coverage)

### What We Change

1. **Replace fullscreen overlay with positioned panel window**
   - Delete current overlay window creation in `lib.rs`
   - Add platform-specific overlay module (`overlay/macos.rs`, `overlay/windows.rs`, `overlay/linux.rs`)
   - Use `tauri-nspanel` for macOS
   - Add Win32 z-order enforcement for Windows
   - Add GTK layer-shell for Linux

2. **Add notification coordinator**
   - mpsc channel for all notification operations
   - Serializes add/dismiss/update/timeout events
   - Prevents race conditions from concurrent HTTP requests

3. **Add `tauri_specta` for typed IPC**
   - Auto-generate TypeScript event types from Rust
   - Eliminate manual type duplication between `types.rs` and `notification.ts`
   - Compile-time guarantee of IPC contract

4. **Multi-monitor awareness**
   - Detect active monitor from cursor position
   - Route notifications to correct monitor's panel
   - Support per-monitor notification preferences

5. **Dynamic window sizing**
   - Panel height grows/shrinks with notification count
   - Empty → hidden (no window visible)
   - 1 card → ~100px height
   - 5 cards → ~600px height
   - Position: top-right of active monitor's work area

---

## 8. Dependency Additions

| Crate | Purpose | Used By |
|-------|---------|---------|
| `tauri-nspanel` | macOS NSPanel support | Handy, Cap |
| `tauri-specta` + `specta` | Type-safe Rust↔TS bindings | Handy, Cap |
| `gtk-layer-shell` (optional) | Wayland overlay protocol | Handy |
| `windows-sys` (optional) | Win32 API for SetWindowPos | Handy |

---

## 9. Risk Assessment

| Risk | Mitigation |
|------|-----------|
| `tauri-nspanel` is third-party | Widely used (Handy + Cap both ship with it), actively maintained |
| GTK layer-shell adds Linux complexity | Make it optional, fall back to always-on-top |
| Breaking existing tests | Overlay component tests stay the same — only window container changes |
| Multi-monitor edge cases | Start with single-monitor, add multi-monitor as enhancement |
| Win32 API usage | Well-documented pattern, same as Handy's production code |

---

## 10. Implementation Priority

1. **Replace fullscreen overlay → positioned panel** (highest impact, fixes rendering bug)
2. **Add tauri-nspanel for macOS** (correct native behavior)
3. **Add Win32 z-order enforcement** (Windows reliability)
4. **Add tauri_specta for typed IPC** (developer experience, prevents bugs)
5. **Add notification coordinator** (robustness under load)
6. **Multi-monitor support** (enhancement)
7. **GTK layer-shell for Linux** (Wayland correctness)

---

## 11. Visual & UX Deep Dive: What Users Actually See

### Meetily Pro — Meeting Detection Notifications (macOS)

Meetily uses a **custom Swift NSPanel notification system** (not Tauri's notification plugin)
for rich, actionable, native-feeling notifications.

#### How Meeting Detection Works

```
Background task polls microphone every 5s
    ↓
Zoom/Teams/Meet/Slack/Discord starts using mic
    ↓
Platform detector identifies app by bundle ID (e.g. us.zoom.xos)
    ↓
Emits mic-detection event: { type: 'mic_started', app: 'Zoom' }
    ↓
Creates NSPanel notification with "Start Recording" button
    ↓
User clicks button → auto-starts recording with timestamp name
```

#### Notification Visual Design

```
┌────────────────────────────────────────────┐
│  [icon]  Zoom - Meeting Detected     [×]   │  ← 380pt × 64pt
│          Start recording?    [Start Rec]   │  ← Action button
└────────────────────────────────────────────┘
   ↑ top-right, 15pt margin from screen edge
   ↑ rounded corners (11pt radius)
   ↑ glassmorphism (NSVisualEffectView, popover material)
   ↑ white border accent
```

**Visual specs:**
- **Size:** 380pt width × 64pt height (auto-expanding for content)
- **Position:** Top-right corner, 15pt margin from screen edges
- **Background:** Semi-transparent with `NSVisualEffectView` (popover material) — true glassmorphism
- **Border:** White border accent for definition against any background
- **Corner radius:** 11pt
- **Shadow:** Native NSPanel shadow

**Content layout:**
- **App icon:** 32×32 with shadow, left-aligned
- **Title:** 14pt semibold, truncates with ellipsis
- **Body:** 11pt regular, secondary text
- **Action button:** White, rounded (10pt radius), semibold 14pt font
- **Close button:** 15×15 circular (×), hidden until hover, black at 50% alpha

**Stacking behavior:**
- Max 5 notifications visible simultaneously
- Stack downward from top-right
- 10pt spacing between notifications
- Auto-reposition when one is dismissed

**Animations:**
- **Slide-in:** From right, 0.3s easeOut
- **Dismiss:** Fade out, 0.2s easeIn
- **Hover:** Button brightness change + shadow
- **Close button:** Fades in on notification hover

**Action button flow:**
```
User clicks "Start Recording"
    → Swift calls rustOnNotificationConfirm(id, deepLink)
    → Rust: show/focus main window
    → Auto-generate meeting name: "Meeting 2026-03-26_14-30-45"
    → Start recording immediately
    → Dismiss notification with animation
    → Emit recording-started-from-notification event
```

**Key implementation detail:** Meetily uses `swift-rs` to bridge Rust ↔ Swift. The notification
UI is entirely native Swift (NSPanel + NSVisualEffectView + NSStackView), not web-rendered.
This gives it native look, feel, and animation behavior that a WebView can't match.

---

### Handy — Floating Recording Overlay

Handy's overlay is a **tiny pill-shaped floating panel** that appears during recording
and shows real-time audio visualization.

#### Visual Design

```
┌─────────────────────────────────────┐
│  🎙  |||||||||||  [×]              │  ← 172×36px pill
└─────────────────────────────────────┘
   ↑ centered horizontally on active monitor
   ↑ top or bottom position (user setting)
   ↑ #000000cc background (black, 80% opacity)
   ↑ 18px border-radius (pill shape)
```

**Window specs:**
- **Size:** 172×36px (fixed, not resizable)
- **Shape:** Pill (border-radius: 18px)
- **Background:** `#000000cc` (black at 80% opacity)
- **No blur/backdrop-filter** — just semi-transparent black
- **No window decorations, no shadow**

**Position calculation:**
```
monitor = cursor's current monitor
work_area = monitor.work_area()  (excludes menu bar/dock)

Horizontal: centered → x = work_area.x + (width - 172) / 2
Vertical:
  Top:    y = work_area.y + 46px (macOS) or 4px (others)
  Bottom: y = work_area.y + height - 36 - 15px (macOS) or 40px (others)
```

**Color palette:**
- Background: `#000000cc` (black 80%)
- Accent: `#FAA2CA` (soft pink — icons, cancel button)
- Bars: `#FFE5EE` (pale pink, almost white)
- Cancel hover: `#FAA2CA33` (pink at 20%)
- Text: white

#### CSS Grid Layout

```css
display: grid;
grid-template-columns: auto 1fr auto;
/*  [Mic Icon]  [Audio Bars / Status Text]  [Cancel Button]  */
```

#### Audio Level Bars (Recording State)

**9 vertical bars** with real-time audio visualization:

- **Dimensions:** 6px wide, 4px min height, 20px max height, 2px border-radius
- **Color:** `#FFE5EE` with opacity tied to amplitude (min 20%)
- **Spacing:** 3px gap

**Smoothing algorithm (exponential):**
```typescript
smoothed = previousValue * 0.7 + newValue * 0.3
// 70% previous value retained → smooth, responsive feel
```

**Height calculation (non-linear):**
```typescript
height = Math.min(20, 4 + Math.pow(amplitude, 0.7) * 16)
// Power curve makes quiet sounds more visible
// Loud sounds compress toward max
```

**CSS transitions:**
- Height: `60ms ease-out`
- Opacity: `120ms ease-out`

**Data flow:**
```
Rust audio processor → Vec<f32> (16 frequency bands)
    → emit("mic-level", levels)
    → JS receives, slices to 9 bars
    → Exponential smoothing applied per frame
    → React state update → CSS transition animates
```

#### State Machine

```
[Hidden]
    ↓ emit("show-overlay", "recording")
[Recording] — mic icon + 9 animated bars + cancel button
    ↓ emit("show-overlay", "transcribing")
[Transcribing] — "Transcribing..." text with pulse animation
    ↓ emit("hide-overlay")
[Fade Out] — 300ms opacity transition
    ↓ thread::sleep(300ms) + window.hide()
[Hidden]
```

**Transcribing pulse animation:**
```css
@keyframes transcribing-pulse {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
}
/* 1.5s cycle, infinite, ease-in-out */
```

#### Cancel Button

- **Size:** 24×24px circle
- **Icon:** Custom SVG in `#FAA2CA` pink
- **Hover:** Background → `#FAA2CA33`, scale → 1.05x
- **Active (click):** Scale → 0.95x (press-down effect)
- **Transitions:** bg 150ms, transform 100ms

**On click:**
```
invoke("cancel_operation")
    → Rust: reset all toggle states
    → Stop audio recording
    → Cancel transcription
    → Change tray icon to Idle
    → Overlay fades out
```

#### Non-Interference Design

| Property | How It Avoids Disrupting User |
|----------|------------------------------|
| `focused(false)` | Never steals keyboard focus |
| `skip_taskbar(true)` | Doesn't clutter taskbar |
| `accept_first_mouse(true)` | Cancel button works without first activating window |
| 172×36px size | Tiny footprint, minimal screen obstruction |
| Center-top position | Predictable, out of the way of most work |
| No keyboard capture | All keyboard input goes to background app |
| No window activation | Clicking cancel doesn't activate the overlay window |

---

### Cap — Recording Controls & Recordings Overlay

Cap uses **two distinct overlay approaches** for different purposes:

#### Recording Controls (NSPanel, bottom-center)
```
                  ┌──────────────────────────┐
                  │  ⏸  🔴  ⏹  00:03:42    │  ← 320×150px
                  └──────────────────────────┘
                        ↑ bottom-center, 120px from bottom
                        ↑ NSPanel (macOS) or WebviewWindow
                        ↑ PanelManager state machine
```

**PanelManager lifecycle:**
```
Creating → (panel allocated) → Ready → (recording stops) → Destroying → (cleaned up)
```

#### Recordings Overlay (transparent, bottom-left)
```
┌──────────────────┐
│  Recent:         │  ← Transparent WebviewWindow
│  [Recording 1]   │     bottom-left corner
│  [Recording 2]   │     auto-fades after timeout
│  [Recording 3]   │
└──────────────────┘
```

- Frontend sets `data-transparent-window` attribute
- `body.style.background = "transparent"` on mount
- Shows recent recordings with one-click sharing

---

### Comparison: What syncfu Should Learn

| Aspect | Meetily Notifications | Handy Overlay | syncfu Should Do |
|--------|----------------------|---------------|------------------|
| **Rendering** | Native Swift NSPanel | WebView in small window | WebView in small window (pragmatic) |
| **Glassmorphism** | NSVisualEffectView (native) | None (solid black 80%) | CSS backdrop-filter + platform glass where available |
| **Action buttons** | Native Swift buttons | Single cancel button | Web-rendered action buttons (already have this) |
| **Stacking** | Max 5, top-right, 10pt gap | Single overlay, no stacking | Max 5 cards in panel, top-right stack |
| **Animations** | Slide-in 0.3s, fade-out 0.2s | Fade in/out 300ms | Keep current slide-in-right 300ms |
| **Audio viz** | 9-element array to overlay | 9 bars, 60ms transitions | Could support progress bars (already in types) |
| **Position** | Top-right, 15pt margin | Center top/bottom, monitor-aware | Top-right, work-area-aware, monitor-aware |
| **Dismiss** | Click ×, auto-timeout | No auto-dismiss (manual cancel) | Click × + configurable auto-timeout |
| **Focus** | Non-activating NSPanel | focused(false) | NSPanel (macOS) + focused(false) (others) |

### Key UX Takeaways for syncfu

1. **Meetily's approach to actionable notifications is exactly what syncfu needs** — the "Start Recording" button pattern maps directly to syncfu's action buttons. Consider native Swift panels for macOS.

2. **Handy's non-interference design is the gold standard** — `focused(false)` + `accept_first_mouse(true)` + `skip_taskbar(true)` + tiny window = zero disruption.

3. **Both apps create overlay windows ONCE and toggle visibility** — never recreate per notification. syncfu should follow this pattern.

4. **Meetily's glassmorphism via NSVisualEffectView** gives native material design on macOS. syncfu can approximate this with CSS `backdrop-filter: blur()` in the WebView, which works on macOS and modern Windows.

5. **Handy's audio smoothing algorithm** (`prev * 0.7 + new * 0.3`) is directly applicable to syncfu's progress bar animations.

6. **Neither app uses a fullscreen transparent overlay** — both prove that small positioned windows are the correct approach.

---

*Generated 2026-03-26 from analysis of: Handy (cjpais/Handy), Cap (CapSoftware/Cap),
Meetily Pro (meeting-minutes-safvan/frontend), tauri-plugin-liquid-glass (hkandala/tauri-plugin-liquid-glass)*
