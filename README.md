# syncfu

**The notification layer your AI agents are missing.**

syncfu is a standalone overlay notification system that sits between your background processes — AI agents, autonomous loops, skills, CI pipelines, cron jobs, anything — and you. It renders always-on-top native notifications that bypass the OS notification center, so nothing gets buried.

One HTTP call. One notification on your screen. That's it.

```bash
curl -X POST localhost:9868/notify \
  -H "Content-Type: application/json" \
  -d '{"sender":"claude","title":"Loop complete","body":"All 47 tests passing.","icon":"check-circle","font":"Space Grotesk"}'
```

Built with Tauri v2 + Rust + React. Cross-platform: macOS, Windows, Linux.

---

## Why this exists

If you run AI agents, autonomous coding loops, or long-running background tasks — you already know the problem. The work finishes, but you don't notice. The agent wrote 14 files, ran the tests, opened a PR... and you're on Twitter. Or the build broke 20 minutes ago and you've been waiting on nothing.

OS notifications are unreliable for this. They get swallowed by Focus Mode, grouped into oblivion, or silently dropped. You need a notification layer that respects your attention — one that puts information on your screen when it matters, with action buttons so you can respond without context-switching.

syncfu is that layer.

> *"I have ADHD. I don't go back and read stuff. If the information doesn't come to me, it doesn't exist."*
> — the reason this project exists

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
| HTTP REST | `9876` | Send, update, dismiss notifications |
| WebSocket | `9877` | Bidirectional — send notifications, receive action callbacks |

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
# On first launch, it asks to start automatically on login
syncfu

# Send your first notification
syncfu-cli send --title "Hello" --body "syncfu is running"

# Or use curl
curl -X POST localhost:9876/notify \
  -d '{"sender":"test","title":"It works","body":"Your first notification"}'

# Open the desktop app to browse notification history
# (click "Open syncfu" in the tray menu, or click the dock icon on macOS)
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
curl -s -X POST localhost:9876/notify \
  -d "{\"sender\":\"remind\",\"title\":\"$TITLE\",\"body\":\"$BODY\",\"sound\":\"default\",\"actions\":[{\"id\":\"done\",\"label\":\"Done\",\"style\":\"primary\"},{\"id\":\"snooze\",\"label\":\"Snooze 15m\",\"style\":\"secondary\"}]}"
```

**Autonomous coding loops**
Running `/loop` or a multi-agent workflow that takes 30 minutes? Get notified when each phase completes, when tests fail, or when the loop needs human input.

```json
{"sender":"loop-operator","title":"Phase 3/5 complete","body":"Integration tests: **42 passed**, 0 failed\nStarting E2E phase...","progress":{"value":0.6,"label":"3 of 5","style":"bar"}}
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

```json
{"sender":"github-actions","title":"Build passed","body":"**main** built in 3m 42s\n- 142 tests passed\n- Coverage: 87% (+2.1%)","priority":"normal","actions":[{"id":"open_pr","label":"Open PR","style":"primary"}],"sound":"success","group":"ci-builds"}
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
# Watch tests and notify on failure
cargo test 2>&1 || syncfu-cli send --title "Tests failed" --body "$(cargo test 2>&1 | tail -20)" --priority critical --sound error
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
# From the /remind skill
syncfu-cli send \
  --title "Stand-up in 5 minutes" \
  --body "Prepare: yesterday's PR review, today's auth refactor" \
  --priority high \
  --sound default \
  --timeout 300
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
For ADHD medication timing — a critical-priority notification with no auto-dismiss that stays on screen until you acknowledge it.

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

```json
{"sender":"training","title":"Epoch 45/100","body":"Loss: 0.0234 (↓12%)\nVal accuracy: 94.2%\nETA: 2h 15m","progress":{"value":0.45,"style":"bar"},"group":"training-run-7"}
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
| `POST` | `/dismiss-all` | Dismiss all active notifications |
| `GET` | `/health` | Server status and active notification count |
| `GET` | `/history` | Query history (`?sender=X&limit=50`) |

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

### WebSocket protocol

Connect to `ws://localhost:9877` for bidirectional communication.

**Send notifications:**
```json
{"type": "notify", "payload": { "sender": "my-app", "title": "Hello", "body": "World" }}
```

**Receive action callbacks:**
```json
{"type": "action", "notification_id": "uuid", "action_id": "approve"}
{"type": "dismissed", "notification_id": "uuid", "reason": "timeout"}
```

**Subscribe to a specific sender's events:**
```json
{"type": "subscribe", "sender": "my-app"}
```

---

## CLI

```bash
syncfu-cli send --title "Done" --body "All tests passed" --sound success
syncfu-cli send --title "Deploy" --body "Deploying..." --progress 0.5 --group deploys
syncfu-cli dismiss <id>
syncfu-cli dismiss-all
syncfu-cli list
syncfu-cli history --sender ci --limit 20
syncfu-cli status
```

### Pipe-friendly

```bash
# Notify when a long command finishes
cargo build --release && syncfu-cli send --title "Build done" --sound success \
  || syncfu-cli send --title "Build failed" --priority critical --sound error

# Pipe output as notification body
kubectl get pods --no-headers | syncfu-cli send --title "Pod Status" --body -

# Watch a log and notify on errors
tail -f app.log | grep --line-buffered "ERROR" | while read line; do
  syncfu-cli send --title "Error detected" --body "$line" --priority high
done
```

---

## Integrations

### Claude Code hooks

Add to your `.claude/hooks.json` to get notified on every agent completion:

```json
{
  "hooks": {
    "Stop": [{
      "command": "syncfu-cli send --title 'Agent complete' --body \"$(git diff --stat HEAD~1 2>/dev/null || echo 'No changes')\" --sound default"
    }]
  }
}
```

### GitHub Actions

```yaml
- name: Notify syncfu
  if: always()
  run: |
    STATUS=${{ job.status }}
    curl -s -X POST http://your-machine:9876/notify \
      -H "Content-Type: application/json" \
      -d "{\"sender\":\"github-actions\",\"title\":\"${{ github.workflow }} — $STATUS\",\"body\":\"${{ github.repository }}@${{ github.ref_name }}\",\"priority\":\"$([ $STATUS = 'success' ] && echo normal || echo critical)\",\"sound\":\"$([ $STATUS = 'success' ] && echo success || echo error)\"}"
```

### Shell aliases

```bash
# Add to .zshrc / .bashrc
alias notify='syncfu-cli send --title'
alias notify-done='syncfu-cli send --title "Done" --body'
alias notify-fail='syncfu-cli send --title "Failed" --priority critical --sound error --body'

# Usage
long-running-command; notify-done "Finished long-running-command"
```

### Node.js / Python / Any language

```javascript
// Node.js
await fetch('http://localhost:9876/notify', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ sender: 'my-app', title: 'Done', body: 'Task complete' })
});
```

```python
# Python
import requests
requests.post('http://localhost:9876/notify', json={
    'sender': 'my-app', 'title': 'Done', 'body': 'Task complete'
})
```

---

## Architecture

```
                           ┌─────────────────────────────────┐
                           │         syncfu (Tauri v2)        │
                           │                                  │
 HTTP POST :9876 ─────────▸│  axum server ──▸ Notification    │
                           │                  Manager         │──emit──▸ React Overlay
 WebSocket :9877 ─────────▸│  tungstenite ──▸ (Arc shared)   │         (transparent window)
                           │                      │           │
 CLI ─────────────────────▸│                 ┌────┴────┐      │
                           │                 │ History │      │
                           │                 │ Sound   │      │
                           │                 │ Tray    │      │
                           │                 └─────────┘      │
                           └─────────────────────────────────┘
```

- **Rust backend**: axum HTTP + tokio-tungstenite WS + SQLite history + rodio sound
- **React frontend**: Zustand store, CSS animations, markdown rendering
- **Overlay**: Single transparent always-on-top window, click-through except on notification cards
- **System tray**: Pause, clear, history, settings, server status

---

## Configuration

syncfu stores settings in `{app_data_dir}/syncfu/settings.json`:

```json
{
  "http_port": 9876,
  "ws_port": 9877,
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
- [x] 119 tests (68 frontend + 51 Rust)
- [x] Webhook callbacks (action buttons POST to `callbackUrl`)
- [ ] Click-through mechanism
- [ ] WebSocket server (port 9869)
- [ ] CLI tool
- [ ] Markdown body rendering
- [ ] Notification grouping
- [ ] Sound playback
- [ ] SQLite history
- [ ] Multi-monitor support
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
