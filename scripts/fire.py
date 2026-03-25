#!/usr/bin/env python3
"""Send test notifications to syncfu.

Usage:
    python3 scripts/fire.py                  # send one normal notification
    python3 scripts/fire.py --all            # send one of each priority
    python3 scripts/fire.py --burst 5        # send 5 rapid notifications
    python3 scripts/fire.py --progress       # send a notification with progress bar
    python3 scripts/fire.py --actions        # send a notification with action buttons
    python3 scripts/fire.py --critical       # send a critical notification
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
        send({"sender": "test", "title": "Low Priority", "body": "This is fine", "priority": "low"})
        send({"sender": "test", "title": "Normal Priority", "body": "Business as usual", "priority": "normal"})
        send({"sender": "test", "title": "High Priority", "body": "Needs attention soon", "priority": "high"})
        send({"sender": "test", "title": "Critical Alert", "body": "Disk usage at 95%!", "priority": "critical"})

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
            "progress": {"value": 0.65, "label": "65% complete", "style": "bar"},
        })

    elif mode == "--actions":
        print("Sending notification with action buttons:")
        send({
            "sender": "github",
            "title": "PR #42 Ready",
            "body": "@sujith requested your review",
            "priority": "high",
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
            "actions": [{"id": "restart", "label": "Restart", "style": "danger"}],
        })

    else:
        print("Sending test notification:")
        send({"sender": "claude", "title": "Test Notification", "body": "syncfu is working!", "priority": "normal"})

    print()
    h = health()
    print(f"Active: {h['active_count']}")


if __name__ == "__main__":
    main()
