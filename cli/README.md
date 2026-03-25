# syncfu CLI

Send notifications to the syncfu desktop overlay from anywhere — terminal, scripts, CI/CD, cron, AI agents.

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
syncfu send -t "Build Passed" -p high -i check-circle "All tests green"

# Action buttons (repeatable)
syncfu send -t "PR #42" \
  --action "approve:Approve:primary" \
  --action "deny:Deny:danger" \
  --callback-url "http://localhost:9870/cb" \
  "Review requested"

# Progress bar
syncfu send -t "Migrating" --progress 0.65 --progress-label "65%" "users table"

# Custom style (27 CSS properties)
syncfu send -t "Deploy" -i rocket \
  --style-json '{"accentColor":"#22c55e","titleColor":"#bbf7d0"}' \
  "v2.1 is live"

# Never auto-dismiss
syncfu send -t "Critical" -p critical --timeout never "Server down"

# Custom timeout (seconds)
syncfu send -t "Reminder" --timeout 30 "Check the build"
```

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
alias notify-done='syncfu send -t "Done"'
alias notify-fail='syncfu send -t "Failed" -p critical'

cargo build --release && notify-done "Build finished" || notify-fail "Build broken"
```

### CI/CD

```bash
SYNCFU_SERVER=http://dev-machine:9868 syncfu send \
  -t "Deploy $VERSION" -p high -i rocket \
  --action "rollback:Rollback:danger" \
  "Deployed to production"
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

## Development

```bash
# Build
cargo build -p syncfu-cli

# Run (during dev, use -p to avoid binary name collision with Tauri app)
cargo run -p syncfu-cli -- send "test"

# Test
cargo test -p syncfu-cli
```
