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
        send({"sender": "test", "title": "Low Priority", "body": "This is fine", "priority": "low", "icon": "check-circle"})
        send({"sender": "test", "title": "Normal Priority", "body": "Business as usual", "priority": "normal", "icon": "info"})
        send({"sender": "test", "title": "High Priority", "body": "Needs attention soon", "priority": "high", "icon": "alert-triangle"})
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

    elif mode == "--fonts":
        print("Sending notifications with custom Google Fonts:")
        send({"sender": "design", "title": "New Mockup Ready", "body": "Landing page v3 uploaded to Figma", "icon": "figma", "font": "Space Grotesk"})
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
