import { useEffect, useRef, useCallback, useState } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useNotifications } from "@/hooks/useNotifications";
import { NotificationCard } from "./NotificationCard";

const PANEL_WIDTH = 400;
/** Height reserved per stacked card behind the front one (px) */
const STACK_PEEK = 8;
/** Gap between cards when expanded */
const EXPANDED_GAP = 10;

/** Resize the overlay window to fit only the notification content. */
function resizeToContent(el: HTMLElement | null, expanded: boolean, count: number) {
  if (!el) return;
  const stack = el.querySelector(".notification-stack") as HTMLElement | null;
  if (!stack) {
    getCurrentWindow()
      .setSize(new LogicalSize(PANEL_WIDTH, 10))
      .catch(() => {});
    return;
  }

  if (expanded) {
    // Expanded: measure actual rendered height
    const contentHeight = stack.scrollHeight + 16;
    getCurrentWindow()
      .setSize(new LogicalSize(PANEL_WIDTH, Math.max(contentHeight, 10)))
      .catch(() => {});
  } else {
    // Collapsed: front card height + peek per stacked card + padding
    const frontCard = stack.querySelector(".notification-card") as HTMLElement | null;
    const frontHeight = frontCard ? frontCard.offsetHeight : 0;
    const stackedPeek = Math.max(0, count - 1) * STACK_PEEK;
    const contentHeight = frontHeight + stackedPeek + 16;
    getCurrentWindow()
      .setSize(new LogicalSize(PANEL_WIDTH, Math.max(contentHeight, 10)))
      .catch(() => {});
  }
}

export function NotificationOverlay() {
  const { notifications, dismiss } = useNotifications();
  const rootRef = useRef<HTMLDivElement>(null);
  const [expanded, setExpanded] = useState(false);

  // Set transparent background on the overlay window (Cap pattern)
  useEffect(() => {
    document.documentElement.setAttribute("data-transparent-window", "true");
    document.body.style.background = "transparent";
  }, []);

  // Hide panel when empty, resize to fit content when not
  useEffect(() => {
    if (notifications.length === 0) {
      getCurrentWindow().hide();
      setExpanded(false);
    } else {
      requestAnimationFrame(() =>
        resizeToContent(rootRef.current, expanded, notifications.length)
      );
    }
  }, [notifications.length, expanded]);

  // Observe the notification stack for content size changes (progress updates, etc)
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const stack = root.querySelector(".notification-stack");
    if (!stack) return;
    const observer = new ResizeObserver(() => {
      resizeToContent(root, expanded, notifications.length);
    });
    observer.observe(stack);
    return () => observer.disconnect();
  }, [notifications.length, expanded]);

  const handleAction = useCallback(
    (notificationId: string, actionId: string) => {
      invoke("action_callback", {
        notificationId,
        actionId,
      }).catch((err) =>
        console.error("[syncfu] action_callback failed:", err)
      );
    },
    []
  );

  return (
    <div data-testid="overlay-root" className="overlay-root" ref={rootRef}>
      {notifications.length > 0 && (
        <div
          data-testid="notification-stack"
          className="notification-stack"
          data-expanded={expanded}
          onMouseEnter={() => setExpanded(true)}
          onMouseLeave={() => setExpanded(false)}
          style={
            {
              "--stack-count": notifications.length,
              "--expanded-gap": `${EXPANDED_GAP}px`,
              "--stack-peek": `${STACK_PEEK}px`,
            } as React.CSSProperties
          }
        >
          {notifications.map((notification, index) => (
            <NotificationCard
              key={notification.id}
              notification={notification}
              index={index}
              total={notifications.length}
              onDismiss={dismiss}
              onAction={handleAction}
            />
          ))}
        </div>
      )}
    </div>
  );
}
