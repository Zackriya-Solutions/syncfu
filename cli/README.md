# syncfu CLI

Send notifications to the syncfu desktop overlay from anywhere — terminal, scripts, CI/CD, cron, AI agents. Block on user decisions with `--wait`.

## Install

```bash
cargo install --path cli
```

## Usage

```bash
syncfu send "Hello world"
```

That's it. Body is the positional argument. Sender defaults to `$USER`, priority to `normal`.

## Commands

### `syncfu send <BODY>`

```bash
# Simple
syncfu send "Backup complete"

# With title, priority, icon
syncfu send -t "Build Passed" -p high -i circle-check "All tests green"

# Action buttons (repeatable)
syncfu send -t "PR #42" \
  --action "approve:Approve:primary" \
  --action "deny:Deny:danger" \
  --callback-url "http://localhost:9870/cb" \
  "Review requested"

# Progress bar
syncfu send -t "Migrating" --progress 0.65 --progress-label "65%" "users table"

# Ring progress
syncfu send -t "Upload" --progress 0.73 --progress-style ring "Uploading..."

# Custom style (27 CSS properties)
syncfu send -t "Deploy" -i rocket \
  --style-json '{"accentColor":"#22c55e","titleColor":"#bbf7d0"}' \
  "v2.1 is live"

# Never auto-dismiss
syncfu send -t "Critical" -p critical --timeout never "Server down"

# Custom timeout (seconds)
syncfu send -t "Reminder" --timeout 30 "Check the build"
```

### `syncfu send --wait` (v0.2+)

Block until the user clicks an action button, dismisses the notification, or a timeout expires. The CLI prints the outcome to stdout and exits with a meaningful code.

```bash
# Basic: wait for user decision
syncfu send -t "Deploy?" \
  -a "yes:Yes:primary" -a "no:No:danger" \
  --wait "Deploy to production?"
# User clicks "Yes" → stdout: "yes", exit 0
# User clicks X     → stdout: "dismissed", exit 1
# 5 min passes      → stdout: "timeout", exit 2

# Custom timeout (seconds, default 300)
syncfu send -t "Quick check" -a "ok:OK" --wait --wait-timeout 30 "Looks good?"

# JSON output for scripting
syncfu send -t "Confirm" -a "go:Go" --wait --json "Proceed?"
# {"id":"abc-123"}
# {"event":"action","action_id":"go"}

# Gate a script on approval
syncfu send -t "Migrate" -a "go:Run it" -a "abort:Abort:danger" \
  --wait "Run database migration?" && rake db:migrate || echo "Aborted"

# Capture action in a variable
ACTION=$(syncfu send -t "Strategy" \
  -a "fast:Fast (skip tests)" -a "safe:Safe (full suite)" \
  --wait "Build strategy?")
if [ "$ACTION" = "fast" ]; then
  cargo build
else
  cargo build && cargo test
fi

# Wait without actions (detect dismiss/timeout only)
syncfu send -t "Heads up" --wait --wait-timeout 60 "Deployment starting in 60s"
```

**Exit codes:**

| Outcome | stdout | Exit code |
|---------|--------|-----------|
| Action clicked | action_id | `0` |
| Dismissed (X button) | `dismissed` | `1` |
| Timeout expired | `timeout` | `2` |

**All flags:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--title` | `-t` | `Notification` | Title |
| `--sender` | `-s` | `$USER` | Sender name |
| `--priority` | `-p` | `normal` | `low`, `normal`, `high`, `critical` |
| `--icon` | `-i` | — | Lucide icon name |
| `--action` | `-a` | — | `"id:label:style"` (repeatable). Style: `primary`/`secondary`/`danger`, defaults to `primary` |
| `--timeout` | | — | `"never"`, `"default"`, or seconds |
| `--progress` | | — | `0.0`–`1.0` |
| `--progress-label` | | — | Label text for progress bar |
| `--progress-style` | | `bar` | `bar` or `ring` |
| `--group` | | — | Group key |
| `--theme` | | — | Theme name |
| `--sound` | | — | Sound name |
| `--font` | | — | Google Font name |
| `--callback-url` | | — | Webhook URL for action callbacks |
| `--style-json` | | — | JSON string of style overrides |
| `--wait` | `-w` | — | Block until action/dismiss/timeout |
| `--wait-timeout` | | `300` | Max wait time in seconds |

### `syncfu update <ID>`

Update a live notification's body or progress.

```bash
syncfu update abc-123 --progress 0.9 --progress-label "90%"
syncfu update abc-123 --body "Step 3/4 complete"
```

### `syncfu action <ID> <ACTION_ID>`

Trigger an action button programmatically.

```bash
syncfu action abc-123 approve
```

### `syncfu dismiss <ID>`

```bash
syncfu dismiss abc-123
```

### `syncfu dismiss-all`

```bash
syncfu dismiss-all
```

### `syncfu list`

List active notifications as JSON. Pipe to `jq` for filtering.

```bash
syncfu list
syncfu list | jq '.[].title'
syncfu list | jq '.[] | select(.priority == "critical")'
```

### `syncfu health`

```bash
syncfu health
# {"status":"ok","active_count":3}
```

Exit code `0` if server is reachable, `1` if not.

## Global options

| Flag | Env | Default | Description |
|------|-----|---------|-------------|
| `--server` | `SYNCFU_SERVER` | `http://127.0.0.1:9868` | Server URL |
| `--json` | | — | Force JSON output for `send` and `dismiss-all` |

## Examples

### Shell aliases

```bash
alias notify='syncfu send -t'
alias notify-done='syncfu send -t "Done" -p low -i circle-check'
alias notify-fail='syncfu send -t "Failed" -p critical -i circle-x'
alias ask='syncfu send --wait -t'

cargo build --release && notify-done "Build finished" || notify-fail "Build broken"

# Ask before deploying
ask "Deploy?" -a "yes:Yes" -a "no:No:danger" "Ship to production?" && deploy.sh
```

### CI/CD

```bash
SYNCFU_SERVER=http://dev-machine:9868 syncfu send \
  -t "Deploy $VERSION" -p high -i rocket \
  --action "rollback:Rollback:danger" \
  "Deployed to production"
```

### AI agent decision gates

```bash
# Agent asks human before destructive action
ANSWER=$(syncfu send -t "Confirm" -s agent -p high \
  -a "yes:Proceed" -a "no:Cancel:danger" \
  --wait --wait-timeout 120 \
  "Delete 47 stale branches?")
[ "$ANSWER" = "yes" ] && git branch -d $(git branch --merged | grep -v main)

# ADHD medication reminder — waits for acknowledgment
syncfu send -t "Medication" -s remind -i pill -p critical \
  --timeout never -a "taken:Taken" -a "skip:Skip:danger" \
  --wait "Time to take your medication"
```

### Claude Code hooks

```json
{
  "hooks": {
    "Stop": [{
      "command": "syncfu send -t 'Agent done' \"$(git diff --stat HEAD~1 2>/dev/null || echo 'No changes')\""
    }]
  }
}
```

### Cron

```bash
# Check disk space every hour
0 * * * * df -h / | tail -1 | awk '{if ($5+0 > 90) system("syncfu send -t \"Disk Warning\" -p high \"Usage: "$5"\"")}'
```

### Multi-step script with checkpoints

```bash
#!/bin/bash
# Build → ask → deploy → ask → monitor

cargo build --release || { syncfu send -t "Build failed" -p critical "Check logs"; exit 1; }

syncfu send -t "Build OK" -a "deploy:Deploy" -a "abort:Abort:danger" \
  --wait "Build succeeded. Deploy to staging?" || exit 0

deploy-staging.sh

syncfu send -t "Staging live" -a "prod:Go to prod" -a "rollback:Rollback:danger" \
  --wait --wait-timeout 600 "Staging looks good. Promote to production?" || {
  rollback-staging.sh
  exit 1
}

deploy-prod.sh
syncfu send -t "Deployed" -p low -i rocket "v2.3 is live in production"
```

## Development

```bash
# Build
cargo build -p syncfu-cli

# Run (during dev, use -p to avoid binary name collision with Tauri app)
cargo run -p syncfu-cli -- send "test"

# Test (109 Rust tests)
cargo test -p syncfu-cli
```
