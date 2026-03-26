# syncfu

**The notification layer your AI agents are missing.**

syncfu is a standalone overlay notification system that sits between your background processes — AI agents, autonomous loops, skills, CI pipelines, cron jobs, anything — and you. It renders always-on-top native notifications that bypass the OS notification center, so nothing gets buried.

One command. One notification on your screen. That's it.

```bash
syncfu send "All 47 tests passing."
```

Need a decision from the user? Add `--wait` and the CLI blocks until they click.

```bash
syncfu send -t "Deploy?" -a "yes:Yes" -a "no:No:danger" \
  --wait "Ship to production?"
# stdout: "yes" or "no", exit 0. Dismissed = exit 1. Timeout = exit 2.
```

Need more? Add flags.

```bash
syncfu send -t "Loop complete" -p high -i circle-check \
  --action "open:Open PR:primary" --action "skip:Skip:secondary" \
  "All 47 tests passing."
```

Built with Tauri v2 + Rust + React. macOS first, Windows + Linux coming.

---

## Why this exists

If you run AI agents, autonomous coding loops, or long-running background tasks — you already know the problem. The work finishes, but you don't notice. The agent wrote 14 files, ran the tests, opened a PR... and you're on Twitter. Or the build broke 20 minutes ago and you've been waiting on nothing.

OS notifications are unreliable for this. They get swallowed by Focus Mode, grouped into oblivion, or silently dropped. You need a notification layer that respects your attention — one that puts information on your screen when it matters, with action buttons so you can respond without context-switching.

syncfu is that layer.

---

## How it works

syncfu runs as a **system tray app** with two faces:

1. **Overlay** — an invisible, always-on-top layer across your screen. When a notification arrives (via HTTP or WebSocket), it slides in from the right with action buttons, progress bars, markdown content — whatever you sent. Clicks pass through to your apps underneath. Only the notification cards are interactive.

2. **Desktop app** — a standard window you can open from the tray icon (or dock on macOS). Shows your full **notification history** — searchable, filterable by sender/priority/date. Click any entry to see the full markdown body, which actions were taken, and callback results. Think of it as your notification inbox.

```
Your App ──HTTP POST──▸ syncfu server ──▸ Overlay Notification
Your App ──WebSocket──▸ syncfu server ──▸ Overlay Notification ──▸ History (persisted)
CLI      ──HTTP POST──▸ syncfu server ──▸ Overlay Notification
                                    │
                            Open syncfu app → browse full history
```

### Ports

| Protocol | Port | Purpose |
|----------|------|---------|
| HTTP REST | `9868` | Send, update, dismiss notifications |
| WebSocket | `9869` | Bidirectional — send notifications, receive action callbacks (planned) |

---

## Install

### macOS
```bash
brew install --cask syncfu
```

### Linux
```bash
# Debian/Ubuntu
curl -fsSL https://syncfu.dev/install.sh | sh

# Or grab the .AppImage from releases
```

### Windows
```bash
winget install syncfu
```

### From source
```bash
git clone https://github.com/nicosujith/syncfu.git
cd syncfu
pnpm install
cargo tauri build
```

### CLI
```bash
cargo install syncfu-cli
```

---

## Quick start

```bash
# Start syncfu (it lives in your system tray)
syncfu

# Send your first notification — just a message
syncfu send "Hello from syncfu!"

# With a title and priority
syncfu send -t "Build Complete" -p high "All 142 tests passing"

# With action buttons
syncfu send -t "PR #42" --action "approve:Approve:primary" --action "skip:Skip:secondary" "Review requested"

# Block until the user responds (--wait)
ACTION=$(syncfu send -t "Approve?" -a "yes:Yes" -a "no:No:danger" --wait "Merge PR #42?")
echo "User chose: $ACTION"  # "yes", "no", "dismissed", or "timeout"

# Or use curl
curl -X POST localhost:9868/notify \
  -H "Content-Type: application/json" \
  -d '{"sender":"test","title":"It works","body":"Your first notification"}'

# List active notifications
syncfu list | jq '.[].title'

# Check server health
syncfu health
```

### Always running

syncfu is designed to stay alive in the background — your agents depend on it.

- **Starts at login** (configurable) — silently, tray + overlay only
- **Closing the window** hides it to the tray — doesn't quit
- **Ctrl+Q / Cmd+Q** asks: *"Quit syncfu? Agents won't be able to notify you."* — with **Quit** and **Send to Background** options
- **Tray → Quit** also confirms before exiting
- You'll never accidentally kill it

---

## Use cases

### AI agents & autonomous loops

**Claude Code skills and hooks**
Your `/remind` skill fires a cron, but the alert is just a terminal bell you'll never hear. Wire it to syncfu and get an overlay notification with action buttons — snooze, mark done, or open the file.

```bash
# Inside any Claude Code skill or hook
syncfu send -t "$TITLE" -s remind --sound default \
  --action "done:Done:primary" --action "snooze:Snooze 15m:secondary" \
  "$BODY"
```

**Autonomous coding loops**
Running `/loop` or a multi-agent workflow that takes 30 minutes? Get notified when each phase completes, when tests fail, or when the loop needs human input.

```bash
syncfu send -t "Phase 3/5 complete" -s loop-operator \
  --progress 0.6 --progress-label "3 of 5" \
  "Integration tests: 42 passed, 0 failed. Starting E2E phase..."
```

**Agent decision gates**
An agent needs human approval before a destructive action. Use `--wait` to block the agent until the user responds via the overlay notification.

```bash
syncfu send -t "Confirm" -s agent -p high \
  -a "yes:Proceed" -a "no:Cancel:danger" \
  --wait --wait-timeout 120 \
  "Delete 47 stale branches?" && git branch -d $(git branch --merged)
```

**Agent handoff alerts**
When one agent finishes and queues work for another, notify the human so they can review before the next agent picks it up.

**Stalled loop detection**
A watchdog process monitors your autonomous loop and pings syncfu if no progress has been made in N minutes: *"Loop stalled — no commits in 12 minutes. Last action: running tests."*

**Multi-agent orchestration dashboards**
Running 5 parallel agents? Each one reports status to syncfu with its own sender ID and group key. See them stacked as a live progress board on your screen.

---

### CI/CD & DevOps

**Build notifications**
GitHub Actions, GitLab CI, Jenkins — POST to syncfu when builds finish. Include pass/fail status, duration, coverage delta, and a "Open PR" action button.

```bash
syncfu send -t "Build passed" -s github-actions -i circle-check \
  --action "open_pr:Open PR:primary" --sound success --group ci-builds \
  "main built in 3m 42s — 142 tests passed, coverage 87% (+2.1%)"
```

**Deploy progress**
Track multi-stage deployments with live progress bars that update in real-time via WebSocket.

**Infrastructure alerts**
Disk full, memory pressure, certificate expiring, container restart loop — get an overlay notification instead of an email you'll read tomorrow.

**Database migration status**
Long-running migrations report progress: *"Migrating users table — 2.4M of 8.1M rows (30%)"* with a live progress bar.

---

### Development workflow

**Test watcher results**
`cargo watch` or `jest --watch` pipes results to syncfu. Green notification when tests pass. Red critical-priority notification when they fail — even if your terminal is buried under 14 windows.

```bash
# Notify on test pass or fail
cargo test && syncfu send -t "Tests passed" -p low -i circle-check "All green" \
  || syncfu send -t "Tests failed" -p critical -i circle-x "Check terminal"
```

**Long compilation finished**
Rust full rebuild? C++ linking? Go generate? Get notified when it's done instead of checking every 30 seconds.

**PR review requested**
A webhook listener catches GitHub PR review requests and sends a syncfu notification with "Review" and "Skip" action buttons.

**Merge conflict alerts**
Your branch just conflicted with main. A hook detects it and notifies you before you waste time building on a broken base.

**Lint / type-check results**
Run `eslint` or `tsc --noEmit` in the background and get a clean/dirty notification overlay.

---

### Personal productivity & ADHD support

**Reminders that actually reach you**
If you have ADHD, you know: setting a reminder is useless if the reminder is a silent badge on an app you don't check. syncfu puts the reminder ON YOUR SCREEN as an unmissable overlay.

```bash
syncfu send -t "Stand-up in 5 minutes" -p high --sound default --timeout 300 \
  "Prepare: yesterday's PR review, today's auth refactor"
```

**Time-boxed focus sessions**
Start a 25-minute pomodoro. syncfu shows a progress bar that updates every minute. When time's up, a critical-priority notification appears: *"Pomodoro complete. Take a break."*

**Context switching prompts**
Scheduled notifications that interrupt with context: *"You've been on this bug for 45 minutes. Current approach: checking race condition in manager.rs. Consider: stepping back and reading the test output again."*

**Meeting prep alerts**
5 minutes before a meeting: *"Standup in 5m — you committed to reviewing the auth PR yesterday. It's still open."*

**End-of-day review**
A nightly cron fires at 6pm: *"EOD check: 3 reminders still open, 2 PRs need review, tomorrow's first meeting is at 9am."*

**Medication reminders**
For ADHD medication timing — a critical-priority notification with no auto-dismiss that stays on screen until you acknowledge it. Use `--wait` so the reminding system knows you actually took it.

```bash
syncfu send -t "Medication" -p critical -i pill --timeout never \
  -a "taken:Taken" -a "skip:Skip:danger" \
  --wait "Time to take your medication"
# Blocks until user confirms — returns "taken" or "skip"
```

**Hydration / posture / break nudges**
Recurring gentle notifications every 30-60 minutes. Low priority, auto-dismiss after 10 seconds, but enough to break the hyperfocus tunnel.

---

### Server & infrastructure monitoring

**Health check dashboard**
Poll your services every 60 seconds. Show a stacked notification group: all green, or highlight the failing service in red.

**SSL certificate expiry**
*"api.example.com certificate expires in 7 days"* — with a "Renew" action button that triggers your renewal script.

**Disk space warnings**
*"/dev/sda1 is 92% full — 14GB remaining"* — grouped with other infra alerts.

**Container restart loops**
*"api-server container has restarted 4 times in the last hour"* — critical priority.

**Cron job completion**
Your nightly backup, database vacuum, or log rotation finished (or failed). Know immediately.

---

### Data & ML pipelines

**Training run progress**
Long-running ML training jobs report epoch progress, loss curves (as text), and ETA via WebSocket. Get notified at milestones or when training completes.

```bash
syncfu send -t "Epoch 45/100" -s training --progress 0.45 --group training-run-7 \
  "Loss: 0.0234 (↓12%) — Val accuracy: 94.2% — ETA: 2h 15m"
```

**Data pipeline stage completion**
ETL pipeline: extract done → transform done → load done. Each stage fires a notification. Failure at any stage fires a critical alert.

**Model evaluation results**
*"Model v2.3 eval complete: accuracy 94.2% (+1.8%), latency p99 45ms (-12ms). Ready to deploy?"* with Deploy/Reject action buttons.

**Dataset processing**
Processing 10M records? Get a progress bar notification that updates every 1000 records.

---

### Team & collaboration

**Slack/Discord message highlights**
A bot watches specific channels and forwards high-priority messages to syncfu. Never miss an @mention because Slack was in a background tab.

**Email triage alerts**
Filter emails server-side and forward urgent ones as syncfu notifications: *"Email from CTO: 'Need the post-mortem by EOD'"* — with "Open Email" and "Snooze 1h" actions.

**Shared incident response**
During an incident, a shared syncfu channel pushes updates to everyone on the team. Status changes, new findings, and action items appear as overlay notifications.

---

### Home automation & IoT

**Smart home events**
Home Assistant, Node-RED, or any IoT platform POSTs events to syncfu. Front door opened, dryer finished, garage left open.

**Package delivery**
Tracking webhook fires when a package is out for delivery or delivered.

**Weather alerts**
Severe weather warnings pushed as high-priority notifications.

---

### Financial & monitoring

**Price alerts**
Stock hits a target, crypto crosses a threshold, or a SaaS bill exceeds budget — instant overlay notification.

**Billing threshold warnings**
*"AWS spend this month: $847 (85% of $1000 budget)"* — with a progress bar.

**API rate limit warnings**
*"OpenAI API: 892/1000 requests used this minute"* — so you can throttle before you hit 429s.

---

### Creative & content workflows

**Render complete**
Video render, 3D scene, image generation batch — done. *"Blender render complete: 240 frames in 47 minutes."*

**Content publishing confirmations**
Blog post deployed, social media post published, newsletter sent — confirmed via notification.

**Transcription/processing complete**
Audio file finished transcribing, podcast episode processed, video subtitles generated.

---

## Desktop app

Open syncfu from the system tray or dock to see your notification history — every notification that's ever been sent, searchable and filterable.

```
┌─────────────────────────────────────────────────────┐
│  syncfu                                      ─ □ ✕  │
├──────────┬──────────────────────────────────────────┤
│          │  [Search...]  [Sender ▾]  [Priority ▾]   │
│ History  │  [Today] [7d] [30d] [All]                │
│          │                                          │
│ ──────── │  🟢 ci-pipeline    Build passed    2m ago│
│          │  🔴 test-watcher   3 tests fail   14m ago│
│ Settings │  🟡 remind         Stand-up in 5m  1h ago│
│ (soon)   │  ⚪ deploy          v2.3.1 live    3h ago│
│          │                                          │
│          │  ── Detail ──────────────────────────     │
│          │  Build passed — ci-pipeline               │
│          │  main built in 3m 42s                     │
│          │  - 142 tests passed                       │
│          │  - Coverage: 87%                          │
│          │  Action taken: "Open PR" at 14:32         │
└──────────┴──────────────────────────────────────────┘
```

**Features:**
- Full history of all notifications, persisted in SQLite
- Filter by sender, priority level, date range
- Full-text search across title, body, and sender
- Click any row to see full markdown body and action/callback details
- Real-time: new notifications appear at the top while the app is open
- Closing the window hides it — syncfu keeps running in the tray

---

## API reference

### HTTP endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/notify` | Send a notification |
| `POST` | `/notify/{id}/update` | Update an existing notification (progress, body) |
| `POST` | `/notify/{id}/action` | Trigger an action (fires webhook to `callbackUrl`, dismisses) |
| `POST` | `/notify/{id}/dismiss` | Dismiss a specific notification |
| `GET` | `/notify/{id}/wait` | SSE stream — blocks until action/dismiss (powers `--wait`) |
| `POST` | `/dismiss-all` | Dismiss all active notifications |
| `GET` | `/health` | Server status and active notification count |
| `GET` | `/active` | List all active notifications (JSON array) |

### Notification payload

```json
{
  "sender": "my-app",
  "title": "Build Complete",
  "body": "**main** built in 3m 42s\n- 142 tests passed",
  "icon": "https://github.com/favicon.ico",
  "priority": "normal",
  "timeout": { "seconds": 15 },
  "actions": [
    { "id": "open", "label": "Open PR", "style": "primary" },
    { "id": "dismiss", "label": "Dismiss", "style": "secondary" }
  ],
  "progress": { "value": 0.75, "label": "3 of 4", "style": "bar" },
  "group": "ci-builds",
  "theme": "github-dark",
  "sound": "success",
  "callback_url": "http://localhost:8080/callback"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sender` | string | yes | Identifier for the sending process |
| `title` | string | yes | Notification title |
| `body` | string | yes | Body text (supports markdown) |
| `icon` | string | no | Lucide icon name (e.g. `phone`, `git-pull-request`, `bell`) |
| `font` | string | no | Google Font name (e.g. `Space Grotesk`, `JetBrains Mono`) — loaded on demand |
| `priority` | string | no | `low`, `normal` (default), `high`, `critical` |
| `timeout` | object | no | `{"seconds": N}` or `"never"` or `"default"` (auto by priority) |
| `actions` | array | no | Up to 3 action buttons |
| `progress` | object | no | Progress bar (`bar` or `ring` style) |
| `group` | string | no | Group key — notifications with same group stack together |
| `theme` | string | no | `light` or `dark` (auto-follows system by default) |
| `sound` | string | no | `default`, `success`, `error`, or `none` |
| `callback_url` | string | no | URL to POST when an action button is clicked |
| `style` | object | no | Per-notification style overrides (see below) |

### Style overrides

The `style` object lets you customize every visual property per notification. All fields optional:

```json
{
  "style": {
    "accentColor": "#22c55e",
    "cardBg": "rgba(10, 40, 20, 0.96)",
    "iconColor": "#4ade80",
    "iconBg": "rgba(34, 197, 94, 0.15)",
    "titleColor": "#bbf7d0",
    "bodyColor": "#86efac",
    "senderColor": "#67e8f9",
    "btnBg": "#7c3aed",
    "btnColor": "#ffffff",
    "btn2Color": "#c084fc",
    "dangerBg": "#dc2626",
    "progressColor": "#22c55e",
    "countdownColor": "#ef4444"
  }
}
```

Full list: `accentColor`, `cardBg`, `cardBorderRadius`, `iconColor`, `iconBg`, `iconBorderColor`, `titleColor`, `titleFontSize`, `bodyColor`, `bodyFontSize`, `senderColor`, `timeColor`, `btnBg`, `btnColor`, `btnBorderColor`, `btn2Bg`, `btn2Color`, `btn2BorderColor`, `dangerBg`, `dangerColor`, `dangerBorderColor`, `progressColor`, `progressTrackColor`, `countdownColor`, `closeBg`, `closeColor`, `closeBorderColor`.

### Per-action button styling

Each action button can override its own colors:

```json
{
  "actions": [
    {
      "id": "deploy",
      "label": "Deploy",
      "style": "primary",
      "icon": "rocket",
      "bg": "#22c55e",
      "color": "#ffffff",
      "borderColor": "#16a34a"
    }
  ]
}

### WebSocket protocol (planned)

Bidirectional communication on port `9869`. Not yet implemented.

---

## Integrations

### Claude Code hooks

Add to your `.claude/hooks.json` to get notified on every agent completion:

```json
{
  "hooks": {
    "Stop": [{
      "command": "syncfu send -t 'Agent done' \"$(git diff --stat HEAD~1 2>/dev/null || echo 'No changes')\""
    }]
  }
}
```

### GitHub Actions

```yaml
- name: Notify syncfu
  if: always()
  run: |
    syncfu send -t "${{ github.workflow }} — ${{ job.status }}" \
      -s github-actions \
      -p ${{ job.status == 'success' && 'normal' || 'critical' }} \
      "${{ github.repository }}@${{ github.ref_name }}"
  env:
    SYNCFU_SERVER: http://your-machine:9868
```

### Shell aliases

```bash
# Add to .zshrc / .bashrc
alias notify='syncfu send -t'
alias notify-done='syncfu send -t "Done"'
alias notify-fail='syncfu send -t "Failed" -p critical --sound error'

# Usage
long-running-command; notify-done "Finished long-running-command"
```

### Node.js / Python / Any language

```javascript
// Node.js
await fetch('http://localhost:9868/notify', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ sender: 'my-app', title: 'Done', body: 'Task complete' })
});
```

```python
# Python
import requests
requests.post('http://localhost:9868/notify', json={
    'sender': 'my-app', 'title': 'Done', 'body': 'Task complete'
})
```

---

## Architecture

```
                           ┌──────────────────────────────────┐
                           │          syncfu (Tauri v2)        │
                           │                                   │
 HTTP POST :9868 ─────────▸│  axum server ──▸ Notification     │
                           │       │          Manager          │──emit──▸ React Overlay
 CLI (fire & forget) ─────▸│       │          (Arc shared)     │         (follows cursor monitor)
                           │       │              │            │
 CLI (--wait) ─────────────▸│  SSE stream ◂── Waiter Registry │
                           │       │         (broadcast ch)    │
                           │       │              │            │
                           │       │         ┌────┴────┐       │
                           │       │         │ Tray    │       │
                           │       │         │ Webhook │       │
                           │       │         └─────────┘       │
                           └──────────────────────────────────┘
```

- **Rust backend**: axum HTTP server + SSE wait streams + waiter registry (tokio broadcast)
- **React frontend**: Zustand store, CSS animations, Lucide icons, Google Fonts
- **Overlay**: NSPanel (macOS) — non-activating, follows mouse cursor across monitors
- **CLI**: fire-and-forget by default, `--wait` opens SSE stream for blocking responses
- **System tray**: Pause, clear, settings, server status

---

## Configuration

syncfu stores settings in `{app_data_dir}/syncfu/settings.json`:

```json
{
  "http_port": 9868,
  "ws_port": 9869,
  "position": "top-right",
  "max_visible": 5,
  "default_timeout_seconds": 8,
  "sounds_enabled": true,
  "do_not_disturb": false,
  "history_max_rows": 10000
}
```

---

## Custom themes

Send a `theme` field to apply custom styling per sender:

```css
/* Place in ~/.config/syncfu/themes/github-dark.css */
.notification-card.github-dark {
  background: #161b22;
  border: 1px solid #30363d;
  color: #e6edf3;
}
.notification-card.github-dark .action-button.primary {
  background: #238636;
}
```

Then use `"theme": "github-dark"` in your notification payload.

---

## Roadmap

- [x] Architecture & plan
- [x] Core overlay window + system tray
- [x] NSPanel on macOS (non-activating, joins all Spaces)
- [x] Notification rendering + Liquid Glass design
- [x] HTTP REST server (port 9868)
- [x] Light/dark theme (auto + per-notification override)
- [x] Lucide icons (programmable via `icon` field)
- [x] Google Fonts (programmable via `font` field)
- [x] Slide-in/slide-out animations
- [x] Auto-dismiss with countdown bar (pauses on hover)
- [x] Critical pulsing glow (Siri-style)
- [x] Relative timestamps ("just now", "5m ago")
- [x] Priority-tinted icon containers
- [x] Dynamic panel resize (no click-blocking)
- [x] 154 tests (72 frontend + 56 Rust + 26 CLI)
- [x] Webhook callbacks (action buttons POST to `callbackUrl`)
- [x] Per-notification style overrides (27 customizable properties)
- [x] Per-action button styling (bg, color, borderColor, icon)
- [x] CLI binary (`syncfu send/dismiss/list/health`)
- [x] CLI `--wait` flag (SSE-based blocking until action/dismiss/timeout)
- [x] Multi-monitor support (notification follows mouse cursor)
- [x] 181 tests (72 frontend + 70 Rust server + 29 CLI unit + 10 CLI integration)
- [ ] Click-through mechanism
- [ ] WebSocket server (port 9869)
- [ ] Markdown body rendering
- [ ] Notification grouping
- [ ] Sound playback
- [ ] SQLite history
- [ ] Linux Wayland support
- [ ] Plugin SDK (custom notification templates)
- [ ] Encrypted transport (mTLS / API keys)
- [ ] Remote notifications (tunnel / cloud relay)

---

## Contributing

syncfu is open source under the MIT license. Contributions welcome.

```bash
git clone https://github.com/nicosujith/syncfu.git
cd syncfu
pnpm install
cargo tauri dev
```

---

## License

MIT

---

<p align="center">
  <strong>syncfu.dev</strong> — because your agents shouldn't have to wait for you to check the terminal.
</p>
