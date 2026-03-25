# syncfu cheatsheet

## basics

```bash
syncfu send "hello world"
syncfu send -t "Title" "body text"
syncfu send -t "Alert" -p critical -i triangle-alert "DB is down"
syncfu send -t "Done" -p low -i circle-check "All green"
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

## styled

```bash
syncfu send -t "Deploy" -i rocket \
  --style-json '{"accentColor":"#22c55e","titleColor":"#bbf7d0","cardBg":"rgba(10,40,20,0.96)"}' \
  "v2.1 is live"
```

## manage

```bash
syncfu list | jq '.[].title'
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

## env

```bash
export SYNCFU_SERVER=http://remote:9868   # point to remote machine
```

## icons (lucide v1.x names)

`rocket` `circle-check` `circle-x` `triangle-alert` `bell` `mail` `terminal` `git-pull-request` `shield-alert` `flame` `siren` `trending-up` `calendar-clock` `message-circle` `eye` `pen-tool` `loader` `server-crash` `webhook` `trophy` `newspaper` `info`

## priorities

`low` (6s) `normal` (8s) `high` (12s) `critical` (never auto-dismiss)

## style keys

`accentColor` `cardBg` `cardBorderRadius` `iconColor` `iconBg` `iconBorderColor` `titleColor` `titleFontSize` `bodyColor` `bodyFontSize` `senderColor` `timeColor` `btnBg` `btnColor` `btnBorderColor` `btn2Bg` `btn2Color` `btn2BorderColor` `dangerBg` `dangerColor` `dangerBorderColor` `progressColor` `progressTrackColor` `countdownColor` `closeBg` `closeColor` `closeBorderColor`

## api endpoints

```
POST /notify                → send notification (returns {id})
POST /notify/{id}/update    → update body/progress
POST /notify/{id}/action    → trigger action button
POST /notify/{id}/dismiss   → dismiss one
POST /dismiss-all           → dismiss all
GET  /health                → {status, active_count}
GET  /active                → list all active (JSON array)
```
