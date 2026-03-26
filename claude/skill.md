# syncfu — Overlay Notification CLI

Send overlay notifications to the user's screen via the syncfu desktop app (HTTP server on port 9868).

## Commands

```bash
syncfu send "message"                                    # Simple notification
syncfu send -t "Title" -p high "body"                    # With title + priority
syncfu send -t "Title" -a "id:Label:style" "body"        # With action buttons
syncfu send -t "Confirm?" -a "yes:Yes" -a "no:No:danger" --wait "msg"  # Block until user clicks
syncfu send --progress 0.7 --progress-label "70%" "msg"  # Progress bar
syncfu send --progress 0.5 --progress-style ring "msg"   # Ring progress
syncfu send -i circle-check "Done"                       # With Lucide icon
syncfu send -s "agent-name" "msg"                        # Custom sender
syncfu send --font "JetBrains Mono" "msg"                # Custom Google Font
syncfu send --timeout never -p critical "msg"            # Never auto-dismiss
syncfu send --style-json '{"accentColor":"#22c55e"}' "styled"  # Custom styling
syncfu update <id> --progress 0.9 --body "Almost done"   # Update existing
syncfu dismiss <id>                                      # Dismiss one
syncfu dismiss-all                                       # Dismiss all
syncfu list                                              # List active (JSON)
syncfu health                                            # Server health
```

## Wait mode (decision gates)

```bash
# Block until user clicks — stdout is the action_id
ACTION=$(syncfu send -t "Deploy?" -a "yes:Yes" -a "no:No:danger" --wait "Ship to prod?")
echo "User chose: $ACTION"  # "yes", "no", "dismissed", or "timeout"

# Exit codes: 0=action clicked, 1=dismissed, 2=timeout
syncfu send -t "Continue?" -a "go:Go" -a "stop:Stop:danger" \
  --wait --wait-timeout 120 "Phase 1 done" \
  && echo "Continuing..." || echo "Aborted"

# JSON output for scripting
RESULT=$(syncfu send -t "Pick" -a "a:Option A" -a "b:Option B" --wait --json "Choose")
# {"event":"action","action_id":"a"}
```

## Progress tracking

```bash
ID=$(syncfu send -t "Migrating" --progress 0 --json "Starting..." | jq -r .id)
syncfu update "$ID" --progress 0.5 --progress-label "50%"
syncfu update "$ID" --progress 1.0 --body "Done!"
```

## All flags

| Flag | Description |
|------|-------------|
| `-t, --title` | Notification title (default: "Notification") |
| `-s, --sender` | Sender name (default: system username) |
| `-p, --priority` | `low` (6s), `normal` (8s), `high` (12s), `critical` (never) |
| `-a, --action` | Action button: `id:label:style` (repeatable, max 3) |
| `-i, --icon` | Lucide icon name |
| `-w, --wait` | Block until user acts (SSE-backed) |
| `--wait-timeout` | Timeout in seconds for --wait (default: 300) |
| `--progress` | Progress value 0.0-1.0 |
| `--progress-label` | Text label for progress |
| `--progress-style` | `bar` (default) or `ring` |
| `--timeout` | Auto-dismiss: `never`, `default`, or seconds |
| `--group` | Group/category key |
| `--theme` | `light` or `dark` (auto-follows system) |
| `--sound` | Sound name |
| `--font` | Google Font name |
| `--callback-url` | Webhook URL — POSTed when action clicked |
| `--style-json` | JSON style overrides (27 properties) |
| `--json` | Force JSON output |
| `--server` | Server URL (default: `http://127.0.0.1:9868`, env: `SYNCFU_SERVER`) |

## Pipe tricks

```bash
# Notify on success/fail
cargo build && syncfu send -t "Build done" "$(date)" \
  || syncfu send -t "Build failed" -p critical "check terminal"

# Tail errors
tail -f app.log | grep --line-buffered ERROR | while read line; do
  syncfu send -t "Error" -p high "$line"
done
```

## Claude Code hooks

```json
{"hooks":{"Stop":[{"command":"syncfu send -t 'Agent done' \"$(git diff --stat HEAD~1 2>/dev/null || echo 'No changes')\""}]}}
```

## HTTP API (when CLI isn't available)

```bash
curl -X POST localhost:9868/notify \
  -H "Content-Type: application/json" \
  -d '{"sender":"ci","title":"Build","body":"Done","priority":"high","icon":"rocket"}'
```

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/notify` | Send notification (returns `{id}`) |
| `POST` | `/notify/{id}/update` | Update body/progress |
| `POST` | `/notify/{id}/action` | Trigger action button |
| `POST` | `/notify/{id}/dismiss` | Dismiss one |
| `GET` | `/notify/{id}/wait` | SSE stream (wait for action/dismiss) |
| `POST` | `/dismiss-all` | Dismiss all |
| `GET` | `/health` | `{status, active_count}` |
| `GET` | `/active` | List all active (JSON array) |

## Common icons

`rocket` `circle-check` `circle-x` `triangle-alert` `bell` `mail` `terminal` `git-pull-request` `shield-alert` `flame` `siren` `trending-up` `calendar-clock` `message-circle` `pill` `coffee` `lightbulb` `server-crash` `webhook` `trophy` `info` `clock` `loader`

## Style keys

`accentColor` `cardBg` `cardBorderRadius` `iconColor` `iconBg` `iconBorderColor` `titleColor` `titleFontSize` `bodyColor` `bodyFontSize` `senderColor` `timeColor` `btnBg` `btnColor` `btnBorderColor` `btn2Bg` `btn2Color` `btn2BorderColor` `dangerBg` `dangerColor` `dangerBorderColor` `progressColor` `progressTrackColor` `countdownColor` `closeBg` `closeColor` `closeBorderColor`

## Prerequisites

The desktop app must be running (system tray) for notifications to display. The CLI talks to the HTTP server on `http://127.0.0.1:9868`. Override with `SYNCFU_SERVER` env var or `--server` flag.
