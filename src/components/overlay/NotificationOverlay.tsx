import { useEffect, useRef, useCallback } from "react";
import { window as tauriWindow } from "@tauri-apps/api";
import { useNotifications } from "@/hooks/useNotifications";
import { NotificationCard } from "./NotificationCard";

/** Resize the overlay window to fit only the notification content. */
function resizeToContent(el: HTMLElement | null) {
  if (!el) return;
  const win = tauriWindow.getCurrentWindow();
  // Measure actual content height + small padding
  const height = el.scrollHeight + 4;
  const width = 400;
  win.setSize(new tauriWindow.LogicalSize(width, Math.max(height, 10)));
}

export function NotificationOverlay() {
  const { notifications, dismiss } = useNotifications();
  const rootRef = useRef<HTMLDivElement>(null);

  // Set transparent background on the overlay window (Cap pattern)
  useEffect(() => {
    document.documentElement.setAttribute("data-transparent-window", "true");
    document.body.style.background = "transparent";
  }, []);

  // Hide panel when empty, resize to fit content when not
  useEffect(() => {
    const win = tauriWindow.getCurrentWindow();
    if (notifications.length === 0) {
      win.hide();
    } else {
      // Small delay to let React render the cards before measuring
      requestAnimationFrame(() => resizeToContent(rootRef.current));
    }
  }, [notifications.length]);

  // Also observe DOM changes for dynamic content (progress updates etc)
  useEffect(() => {
    const el = rootRef.current;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      if (notifications.length > 0) {
        resizeToContent(el);
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [notifications.length]);

  const handleAction = useCallback(
    (_notificationId: string, _actionId: string) => {
      // TODO: send action callback via Tauri command
    },
    []
  );

  return (
    <div data-testid="overlay-root" className="overlay-root" ref={rootRef}>
      {notifications.length > 0 && (
        <div data-testid="notification-stack" className="notification-stack">
          {notifications.map((notification) => (
            <NotificationCard
              key={notification.id}
              notification={notification}
              onDismiss={dismiss}
              onAction={handleAction}
            />
          ))}
        </div>
      )}
    </div>
  );
}
