# /syncfu — Send a notification to your screen

Send a desktop overlay notification via syncfu. Use this to alert yourself about task completion, errors, reminders, or anything that needs attention.

## Instructions

When the user runs `/syncfu`, parse their intent and run the appropriate `syncfu` CLI command via Bash.

### Quick patterns

**Simple message** — `/syncfu tests passed`
```bash
syncfu send "tests passed"
```

**With title** — `/syncfu Build: all 47 tests green`
```bash
syncfu send -t "Build" "all 47 tests green"
```

**Priority** — `/syncfu critical: API is down`
```bash
syncfu send -t "API is down" -p critical -i triangle-alert "API is down"
```

**Done/success** — `/syncfu done: refactoring complete`
```bash
syncfu send -t "Done" -p low -i circle-check "refactoring complete"
```

### How to decide

1. If the message starts with `critical:` or `error:` or `fail:` → use `-p critical -i triangle-alert`
2. If the message starts with `done:` or `success:` or `pass:` → use `-p low -i circle-check`
3. If the message starts with `warn:` or `warning:` → use `-p high -i triangle-alert`
4. If the message mentions PR, review, approve → add `--action "open:Open:primary"`
5. Otherwise → use default priority, pick a sensible icon from: `rocket`, `bell`, `info`, `terminal`, `circle-check`, `triangle-alert`, `flame`, `mail`, `eye`, `loader`

### With actions

If the user's intent implies a decision point:
```bash
syncfu send -t "PR #42" -i git-pull-request \
  --action "approve:Approve:primary" --action "skip:Skip:secondary" \
  "Review requested"
```

### With progress

If the user mentions progress or completion percentage:
```bash
syncfu send -t "Migration" --progress 0.65 --progress-label "65%" "users table"
```

### Wait for response

If the user needs to make a decision and you need the answer back (e.g. "ask me whether to deploy"):
```bash
syncfu send -t "Deploy?" -a "yes:Yes:primary" -a "no:No:danger" --wait --wait-timeout 120 "Deploy to production?"
```

`--wait` blocks until the user clicks an action or dismisses. Output:
- **Action clicked** → prints the action_id to stdout, exits 0
- **Dismissed** (X button) → prints `dismissed`, exits 1
- **Timeout** → prints `timeout`, exits 2

With `--json`, output is structured: `{"event":"action","action_id":"yes"}`

Use `--wait` when:
- The user asks you to confirm something before proceeding
- A decision point requires user input via notification
- You need to gate further work on user approval

### Args reference

```
syncfu send <BODY>
  -t, --title <TITLE>       Title text
  -s, --sender <SENDER>     Sender name (default: $USER)
  -p, --priority <PRI>      low | normal | high | critical
  -i, --icon <ICON>         Lucide icon name
  -a, --action <SPEC>       "id:label:style" (repeatable, max 3)
  --progress <0.0-1.0>      Progress bar value
  --progress-label <TEXT>    Progress label
  --progress-style <STYLE>  bar (default) | ring
  --timeout <TIMEOUT>       "never" | "default" | seconds
  --group <GROUP>            Group/category key
  --theme <THEME>            light | dark (auto-follows system)
  --sound <SOUND>            Sound name
  --font <FONT>              Google Font name
  --callback-url <URL>       Webhook for action callbacks
  --style-json <JSON>        Style overrides (27 properties)
  -w, --wait                 Block until action/dismiss/timeout
  --wait-timeout <SECS>      Max wait time (default: 300)
  --json                     Force JSON output
  --server <URL>             Server URL (default: http://127.0.0.1:9868)

syncfu update <ID>          Update existing notification
  --body <TEXT>              New body text
  --progress <0.0-1.0>      New progress value
  --progress-label <TEXT>    New progress label

syncfu list                 List active (JSON)
syncfu health               Server health
syncfu dismiss <ID>         Dismiss by ID
syncfu dismiss-all          Clear all
```

### Exit codes (with --wait)

| Outcome | stdout | Exit code |
|---------|--------|-----------|
| Action clicked | action_id | 0 |
| Dismissed | `dismissed` | 1 |
| Timeout | `timeout` | 2 |

### Important

- Always use Bash tool to run the command
- If syncfu is not reachable (error), tell the user: "syncfu app doesn't seem to be running. Start it from the system tray or run the desktop app."
- Keep it simple — don't overthink the icon or priority. Just send the notification.
