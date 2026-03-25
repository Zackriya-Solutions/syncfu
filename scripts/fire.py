#!/usr/bin/env python3
"""Send test notifications to syncfu.

Usage:
    python3 scripts/fire.py                  # send one normal notification
    python3 scripts/fire.py --all            # send one of each priority
    python3 scripts/fire.py --burst 5        # send 5 rapid notifications
    python3 scripts/fire.py --progress       # send a notification with progress bar
    python3 scripts/fire.py --actions        # send a notification with action buttons
    python3 scripts/fire.py --critical       # send a critical notification
    python3 scripts/fire.py --icons          # send notifications with various icons
    python3 scripts/fire.py --fonts          # send notifications with custom Google Fonts
    python3 scripts/fire.py --webhook        # send notification with callback_url (starts listener)
    python3 scripts/fire.py --styled         # send notifications with custom style overrides
"""

import json
import sys
import time
import urllib.request

BASE = "http://127.0.0.1:9868"


def send(payload: dict) -> dict:
    data = json.dumps(payload).encode()
    req = urllib.request.Request(
        f"{BASE}/notify", data=data, headers={"Content-Type": "application/json"}
    )
    try:
        resp = urllib.request.urlopen(req)
        result = json.loads(resp.read().decode())
        print(f"  -> {result['id'][:8]}  {payload.get('priority', 'normal'):8}  {payload['title']}")
        return result
    except Exception as e:
        print(f"  ERROR: {e}")
        return {}


def health():
    try:
        resp = urllib.request.urlopen(f"{BASE}/health")
        return json.loads(resp.read().decode())
    except Exception:
        return None


def main():
    h = health()
    if not h:
        print("syncfu is not running (port 9868 not responding)")
        sys.exit(1)
    print(f"syncfu is up — {h['active_count']} active notifications\n")

    args = sys.argv[1:]
    mode = args[0] if args else "--one"

    if mode == "--all":
        print("Sending one of each priority:")
        send({"sender": "test", "title": "Low Priority", "body": "This is fine", "priority": "low", "icon": "circle-check"})
        send({"sender": "test", "title": "Normal Priority", "body": "Business as usual", "priority": "normal", "icon": "info"})
        send({"sender": "test", "title": "High Priority", "body": "Needs attention soon", "priority": "high", "icon": "triangle-alert"})
        send({"sender": "test", "title": "Critical Alert", "body": "Disk usage at 95%!", "priority": "critical", "icon": "flame"})

    elif mode == "--burst":
        count = int(args[1]) if len(args) > 1 else 5
        print(f"Sending {count} rapid notifications:")
        for i in range(count):
            send({"sender": "burst", "title": f"Notification #{i+1}", "body": f"Rapid fire test {i+1} of {count}"})

    elif mode == "--progress":
        print("Sending notification with progress bar:")
        send({
            "sender": "ci-pipeline",
            "title": "Build in Progress",
            "body": "Running test suite...",
            "priority": "normal",
            "icon": "loader",
            "progress": {"value": 0.65, "label": "65% complete", "style": "bar"},
        })

    elif mode == "--actions":
        print("Sending notification with action buttons:")
        send({
            "sender": "github",
            "title": "PR #42 Ready",
            "body": "@sujith requested your review",
            "priority": "high",
            "icon": "git-pull-request",
            "actions": [
                {"id": "approve", "label": "Approve", "style": "primary"},
                {"id": "dismiss", "label": "Later", "style": "secondary"},
            ],
        })

    elif mode == "--critical":
        print("Sending critical notification:")
        send({
            "sender": "monitor",
            "title": "Service Down",
            "body": "API server not responding for 5 minutes",
            "priority": "critical",
            "icon": "server-crash",
            "actions": [{"id": "restart", "label": "Restart", "style": "danger"}],
        })

    elif mode == "--icons":
        print("Sending notifications with various icons:")
        send({"sender": "slack", "title": "New Message", "body": "Hey, are you free?", "icon": "message-circle"})
        send({"sender": "calendar", "title": "Meeting in 5m", "body": "Standup with the team", "icon": "calendar-clock", "priority": "high"})
        send({"sender": "mail", "title": "New Email", "body": "Invoice from AWS", "icon": "mail"})
        send({"sender": "deploy", "title": "Deploy Complete", "body": "v2.1.0 is live", "icon": "rocket", "priority": "low"})
        send({"sender": "security", "title": "Login Attempt", "body": "New sign-in from Tokyo", "icon": "shield-alert", "priority": "high"})

    elif mode == "--webhook":
        print("Starting callback listener on :9870 and sending notification with callback_url...")
        import http.server
        import threading

        class CallbackHandler(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                length = int(self.headers.get("Content-Length", 0))
                body = self.rfile.read(length).decode() if length else ""
                print(f"\n  WEBHOOK RECEIVED:")
                print(f"  {json.dumps(json.loads(body), indent=2)}")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(b'{"ok":true}')

            def log_message(self, format, *args):
                pass  # suppress default logging

        server = http.server.HTTPServer(("127.0.0.1", 9870), CallbackHandler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        print("  Callback listener running on http://127.0.0.1:9870/callback")

        send({
            "sender": "webhook-test",
            "title": "Click an action!",
            "body": "Clicking Approve or Reject will fire a webhook to :9870",
            "priority": "high",
            "icon": "webhook",
            "actions": [
                {"id": "approve", "label": "Approve", "style": "primary"},
                {"id": "reject", "label": "Reject", "style": "danger"},
            ],
            "callback_url": "http://127.0.0.1:9870/callback",
        })

        print("\n  Waiting for webhook callback (Ctrl+C to stop)...")
        try:
            thread.join()
        except KeyboardInterrupt:
            server.shutdown()
            print("\n  Listener stopped.")
        return

    elif mode == "--styled":
        print("Sending styled notifications with custom colors:")

        # Green deploy theme
        send({
            "sender": "deploy",
            "title": "Deploy Complete",
            "body": "v2.1.0 is live in production",
            "icon": "rocket",
            "priority": "low",
            "style": {
                "accentColor": "#22c55e",
                "cardBg": "rgba(10, 40, 20, 0.96)",
                "iconColor": "#4ade80",
                "iconBg": "rgba(34, 197, 94, 0.15)",
                "titleColor": "#bbf7d0",
                "bodyColor": "#86efac",
                "progressColor": "#22c55e",
            },
            "progress": {"value": 1.0, "label": "Complete", "style": "bar"},
        })

        # Purple/brand theme with custom buttons
        send({
            "sender": "pen-tool",
            "title": "Design Review Ready",
            "body": "Landing page v3 needs your feedback",
            "icon": "pen-tool",
            "priority": "high",
            "style": {
                "accentColor": "#a855f7",
                "iconColor": "#c084fc",
                "iconBg": "rgba(168, 85, 247, 0.15)",
                "titleColor": "#e9d5ff",
                "btnBg": "#7c3aed",
                "btnColor": "#ffffff",
                "btn2Color": "#c084fc",
            },
            "actions": [
                {"id": "review", "label": "Review", "style": "primary", "icon": "eye"},
                {"id": "later", "label": "Later", "style": "secondary"},
            ],
        })

        # Red alert with per-button overrides
        send({
            "sender": "pagerduty",
            "title": "Incident #4821",
            "body": "API latency > 2s for 5 minutes",
            "icon": "siren",
            "priority": "critical",
            "style": {
                "accentColor": "#ef4444",
                "titleColor": "#fecaca",
                "bodyColor": "#fca5a5",
                "countdownColor": "#ef4444",
            },
            "actions": [
                {"id": "ack", "label": "Acknowledge", "style": "primary", "bg": "#dc2626", "color": "#ffffff", "borderColor": "#b91c1c"},
                {"id": "snooze", "label": "Snooze 15m", "style": "secondary"},
            ],
            "timeout": "never",
        })

        # Ocean blue minimal
        send({
            "sender": "analytics",
            "title": "Weekly Report",
            "body": "Revenue up 12% week-over-week",
            "icon": "trending-up",
            "style": {
                "accentColor": "#06b6d4",
                "iconColor": "#22d3ee",
                "iconBg": "rgba(6, 182, 212, 0.12)",
                "senderColor": "#67e8f9",
                "titleColor": "#cffafe",
                "bodyColor": "#a5f3fc",
            },
        })

    elif mode == "--fonts":
        print("Sending notifications with custom Google Fonts:")
        send({"sender": "design", "title": "New Mockup Ready", "body": "Landing page v3 uploaded to Figma", "icon": "pen-tool", "font": "Space Grotesk"})
        send({"sender": "terminal", "title": "Build Complete", "body": "All 247 tests passed", "icon": "terminal", "font": "JetBrains Mono", "priority": "low"})
        send({"sender": "editorial", "title": "Article Published", "body": "How We Scaled to 1M Users", "icon": "newspaper", "font": "Playfair Display"})
        send({"sender": "playful", "title": "Achievement Unlocked", "body": "You shipped 10 features this week!", "icon": "trophy", "font": "Nunito", "priority": "low"})

    else:
        print("Sending test notification:")
        send({"sender": "claude", "title": "Test Notification", "body": "syncfu is working!", "priority": "normal", "icon": "bell"})

    print()
    h = health()
    print(f"Active: {h['active_count']}")


if __name__ == "__main__":
    main()
