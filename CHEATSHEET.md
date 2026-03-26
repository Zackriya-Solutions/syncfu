# syncfu cheatsheet

## basics

```bash
syncfu send "hello world"
syncfu send -t "Title" "body text"
syncfu send -t "Alert" -p critical -i triangle-alert "DB is down"
syncfu send -t "Done" -p low -i circle-check "All green"
```

## wait for user response (v0.2+)

```bash
# Block until user clicks an action button or dismisses
syncfu send -t "Deploy?" \
  --action "yes:Yes:primary" --action "no:No:danger" \
  --wait "Deploy to production?"
# stdout: action_id ("yes"/"no"), "dismissed", or "timeout"
# exit: 0=action, 1=dismissed, 2=timeout

# With custom timeout (seconds)
syncfu send -t "Approve?" -a "ok:OK" --wait --wait-timeout 60 "Merge PR #42?"

# JSON output for scripting
RESULT=$(syncfu send -t "Continue?" \
  -a "yes:Yes" -a "no:No:danger" \
  --wait --json "Run the migration?")
# {"event":"action","action_id":"yes"}

# Gate a script on user approval
syncfu send -t "Deploy" -a "go:Ship it" -a "abort:Abort:danger" \
  --wait --wait-timeout 120 "v2.3 ready for production" \
  && echo "Deploying..." || echo "Aborted"

# Capture the action_id in a variable
ACTION=$(syncfu send -t "Pick" \
  -a "fast:Fast (skip tests)" -a "safe:Safe (full suite)" \
  --wait "How should we build?")
echo "User chose: $ACTION"
```

## actions + callbacks

```bash
syncfu send -t "PR #42" \
  --action "approve:Approve:primary" \
  --action "deny:Deny:danger" \
  --callback-url "http://localhost:9870/hook" \
  "Review requested"
```

## progress

```bash
ID=$(syncfu send -t "Migrating" --progress 0.0 --json "Starting..." | jq -r .id)
syncfu update $ID --progress 0.5 --progress-label "50%"
syncfu update $ID --progress 1.0 --body "Done!"
```

## ring progress

```bash
syncfu send -t "Upload" --progress 0.73 --progress-label "73%" \
  --progress-style ring "Uploading assets..."
```

## styled

```bash
syncfu send -t "Deploy" -i rocket \
  --style-json '{"accentColor":"#22c55e","titleColor":"#bbf7d0","cardBg":"rgba(10,40,20,0.96)"}' \
  "v2.1 is live"
```

## manage

```bash
syncfu list | jq '.[].title'
syncfu list | jq '.[] | select(.priority == "critical")'
syncfu health
syncfu dismiss <id>
syncfu dismiss-all
```

## pipe tricks

```bash
# notify on success/fail
cargo build && syncfu send -t "Build done" "$(date)" \
  || syncfu send -t "Build failed" -p critical "check terminal"

# tail errors
tail -f app.log | grep --line-buffered ERROR | while read line; do
  syncfu send -t "Error" -p high "$line"
done

# disk check (cron)
0 * * * * df -h / | awk 'NR==2{if($5+0>90) system("syncfu send -t \"Disk\" -p high \"" $5 "\"")}'

# wait for user before continuing a script
syncfu send -t "Checkpoint" -a "go:Continue" -a "stop:Abort:danger" \
  --wait --wait-timeout 300 "Phase 1 complete. Ready for Phase 2?" \
  || exit 1
```

## AI agent patterns

```bash
# Claude Code hook: notify when agent finishes
syncfu send -t "Agent done" -s claude \
  "$(git diff --stat HEAD~1 2>/dev/null || echo 'No changes')"

# Ask user before destructive action
syncfu send -t "Confirm" -s agent -p high \
  -a "yes:Proceed" -a "no:Cancel:danger" \
  --wait --wait-timeout 120 \
  "Delete 47 stale branches?"

# Autonomous loop progress
syncfu send -t "Phase 3/5" -s loop-operator \
  --progress 0.6 --progress-label "3 of 5" \
  "Integration tests passed. Starting E2E..."

# Medication/ADHD reminder that waits for confirmation
syncfu send -t "Medication" -s remind -i pill -p critical \
  --timeout never -a "taken:Taken" -a "skip:Skip:danger" \
  --wait "Time to take your medication"
```

## claude code hooks

```json
{"hooks":{"Stop":[{"command":"syncfu send -t 'Agent done' \"$(git diff --stat HEAD~1 2>/dev/null || echo 'No changes')\""}]}}
```

## curl (when CLI isn't available)

```bash
curl -X POST localhost:9868/notify \
  -H "Content-Type: application/json" \
  -d '{"sender":"ci","title":"Build","body":"Done","priority":"high","icon":"rocket"}'
```

## python

```python
import requests
requests.post("http://localhost:9868/notify", json={
    "sender": "bot", "title": "Alert", "body": "CPU 95%",
    "priority": "critical", "icon": "triangle-alert",
    "actions": [{"id": "ack", "label": "Ack", "style": "danger"}]
})
```

## SSE wait endpoint (for custom clients)

```bash
# Send notification, then open SSE stream to wait for resolution
ID=$(curl -s -X POST localhost:9868/notify \
  -H "Content-Type: application/json" \
  -d '{"sender":"ci","title":"Approve?","body":"Deploy v2","actions":[{"id":"yes","label":"Yes","style":"primary"}]}' \
  | jq -r .id)

curl -N "localhost:9868/notify/$ID/wait"
# event: message
# data: {"event":"connected"}
#
# event: message
# data: {"event":"action","action_id":"yes"}
```

## env

```bash
export SYNCFU_SERVER=http://remote:9868   # point to remote machine
```

## icons (lucide v1.x names)

`rocket` `circle-check` `circle-x` `triangle-alert` `bell` `mail` `terminal` `git-pull-request` `shield-alert` `flame` `siren` `trending-up` `calendar-clock` `message-circle` `eye` `pen-tool` `loader` `server-crash` `webhook` `trophy` `newspaper` `info` `pill` `coffee` `lightbulb` `user` `folder` `repeat` `clock`

## priorities

`low` (6s) `normal` (8s) `high` (12s) `critical` (never auto-dismiss)

## exit codes (with --wait)

| Outcome | stdout | Exit code |
|---------|--------|-----------|
| Action clicked | `action_id` | `0` |
| Dismissed (X) | `dismissed` | `1` |
| Timeout | `timeout` | `2` |

## style keys

`accentColor` `cardBg` `cardBorderRadius` `iconColor` `iconBg` `iconBorderColor` `titleColor` `titleFontSize` `bodyColor` `bodyFontSize` `senderColor` `timeColor` `btnBg` `btnColor` `btnBorderColor` `btn2Bg` `btn2Color` `btn2BorderColor` `dangerBg` `dangerColor` `dangerBorderColor` `progressColor` `progressTrackColor` `countdownColor` `closeBg` `closeColor` `closeBorderColor`

## api endpoints

```
POST /notify                → send notification (returns {id})
POST /notify/{id}/update    → update body/progress
POST /notify/{id}/action    → trigger action button
POST /notify/{id}/dismiss   → dismiss one
GET  /notify/{id}/wait      → SSE stream (wait for action/dismiss)
POST /dismiss-all           → dismiss all
GET  /health                → {status, active_count}
GET  /active                → list all active (JSON array)
```
