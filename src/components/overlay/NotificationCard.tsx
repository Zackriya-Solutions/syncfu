import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import type { NotificationPayload, StyleOverrides, Action } from "@/types/notification";
import { NotificationIcon } from "./NotificationIcon";
import { RelativeTime } from "./RelativeTime";
import { useGoogleFont } from "@/hooks/useGoogleFont";

/** Auto-dismiss timeouts by priority (ms). Critical never auto-dismisses. */
const TIMEOUTS: Record<string, number | null> = {
  low: 6000,
  normal: 8000,
  high: 12000,
  critical: null,
};

const DISMISS_ANIM_MS = 280;

interface NotificationCardProps {
  readonly notification: NotificationPayload;
  readonly index?: number;
  readonly total?: number;
  readonly onDismiss: (id: string) => void;
  readonly onAction: (notificationId: string, actionId: string) => void;
}

export function NotificationCard({
  notification,
  index = 0,
  total = 1,
  onDismiss,
  onAction,
}: NotificationCardProps) {
  const { id, sender, title, body, priority, actions, progress, theme, icon, font, timeout, createdAt, style } =
    notification;

  useGoogleFont(font);

  // Build CSS custom properties from style overrides + stack index
  const styleVars = useMemo(
    () => ({ ...buildStyleVars(style, font), "--index": index, "--total": total } as React.CSSProperties),
    [style, font, index, total],
  );
  const [dismissing, setDismissing] = useState(false);
  const hovering = useRef(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Resolve auto-dismiss duration (null = never)
  const autoDismissMs = resolveTimeout(timeout, priority);

  // Animated dismiss: play slide-out, then remove from store
  const animatedDismiss = useCallback(() => {
    if (dismissing) return;
    setDismissing(true);
    setTimeout(() => onDismiss(id), DISMISS_ANIM_MS);
  }, [id, onDismiss, dismissing]);

  // Auto-dismiss timer
  useEffect(() => {
    if (autoDismissMs === null) return;

    const startTimer = () => {
      timerRef.current = setTimeout(() => {
        if (!hovering.current) {
          animatedDismiss();
        }
      }, autoDismissMs);
    };

    startTimer();
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [autoDismissMs, animatedDismiss]);

  const handleMouseEnter = useCallback(() => {
    hovering.current = true;
    if (timerRef.current) clearTimeout(timerRef.current);
  }, []);

  const handleMouseLeave = useCallback(() => {
    hovering.current = false;
    // Restart timer with full duration after hover
    if (autoDismissMs !== null && !dismissing) {
      timerRef.current = setTimeout(() => {
        animatedDismiss();
      }, autoDismissMs);
    }
  }, [autoDismissMs, animatedDismiss, dismissing]);

  const classNames = [
    "notification-card",
    priority,
    theme ?? "",
    dismissing ? "dismissing" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      data-testid="notification-card"
      className={classNames}
      style={styleVars}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <button
        className="notification-dismiss"
        onClick={() => animatedDismiss()}
        aria-label="Dismiss"
      >
        <svg width="8" height="8" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeLinecap="round" strokeWidth="2.5">
          <line x1="2" y1="2" x2="8" y2="8" />
          <line x1="8" y1="2" x2="2" y2="8" />
        </svg>
      </button>

      <div className="notification-content-row">
        {icon && (
          <div className="notification-icon">
            <NotificationIcon name={icon} size={20} strokeWidth={1.8} />
          </div>
        )}
        <div className="notification-text">
          <div className="notification-header">
            <span className="notification-sender">{sender}</span>
            <RelativeTime iso={createdAt} />
          </div>
          <div className="notification-title">{title}</div>
          <div className="notification-body">{body}</div>
        </div>
      </div>

      {progress && (
        <div className="notification-progress">
          <div
            role="progressbar"
            aria-valuenow={Math.round(progress.value * 100)}
            aria-valuemin={0}
            aria-valuemax={100}
            className={`progress-${progress.style}`}
          >
            <div
              className="progress-fill"
              style={{ width: `${progress.value * 100}%` }}
            />
          </div>
          {progress.label && (
            <span className="progress-label">{progress.label}</span>
          )}
        </div>
      )}

      {actions.length > 0 && (
        <div className="notification-actions">
          {actions.map((action) => (
            <button
              key={action.id}
              className={`action-button ${action.style}`}
              style={buildActionStyle(action)}
              onClick={() => onAction(id, action.id)}
            >
              {action.icon && (
                <NotificationIcon name={action.icon} size={14} strokeWidth={2} />
              )}
              {action.label}
            </button>
          ))}
        </div>
      )}

      {autoDismissMs !== null && (
        <div className="notification-countdown">
          <div
            className="countdown-fill"
            style={{ "--countdown-duration": `${autoDismissMs}ms` } as React.CSSProperties}
          />
        </div>
      )}
    </div>
  );
}

function resolveTimeout(
  timeout: NotificationPayload["timeout"],
  priority: string,
): number | null {
  if (timeout === "never") return null;
  if (timeout === "default") return TIMEOUTS[priority] ?? 8000;
  if (typeof timeout === "object" && timeout.never) return null;
  if (typeof timeout === "object" && timeout.seconds) return timeout.seconds * 1000;
  return TIMEOUTS[priority] ?? 8000;
}

/** Maps from StyleOverrides key to CSS custom property name */
const STYLE_VAR_MAP: Record<string, string> = {
  accentColor: "--s-accent-color",
  cardBg: "--s-card-bg",
  cardBorderRadius: "--s-card-border-radius",
  iconColor: "--s-icon-color",
  iconBg: "--s-icon-bg",
  iconBorderColor: "--s-icon-border-color",
  titleColor: "--s-title-color",
  titleFontSize: "--s-title-font-size",
  bodyColor: "--s-body-color",
  bodyFontSize: "--s-body-font-size",
  senderColor: "--s-sender-color",
  timeColor: "--s-time-color",
  btnBg: "--s-btn-bg",
  btnColor: "--s-btn-color",
  btnBorderColor: "--s-btn-border-color",
  btn2Bg: "--s-btn2-bg",
  btn2Color: "--s-btn2-color",
  btn2BorderColor: "--s-btn2-border-color",
  dangerBg: "--s-danger-bg",
  dangerColor: "--s-danger-color",
  dangerBorderColor: "--s-danger-border-color",
  progressColor: "--s-progress-color",
  progressTrackColor: "--s-progress-track-color",
  countdownColor: "--s-countdown-color",
  closeBg: "--s-close-bg",
  closeColor: "--s-close-color",
  closeBorderColor: "--s-close-border-color",
};

/** Build CSS custom properties object from style overrides */
function buildStyleVars(
  style: StyleOverrides | undefined,
  font: string | undefined,
): React.CSSProperties {
  const vars: Record<string, string> = {};

  if (font) {
    vars.fontFamily = `"${font}", sans-serif`;
  }

  if (!style) return vars as React.CSSProperties;

  for (const [key, cssVar] of Object.entries(STYLE_VAR_MAP)) {
    const value = style[key as keyof StyleOverrides];
    if (value) {
      vars[cssVar] = value;
    }
  }

  return vars as React.CSSProperties;
}

/** Build per-action inline styles from action overrides */
function buildActionStyle(action: Action): React.CSSProperties | undefined {
  const { bg, color, borderColor } = action;
  if (!bg && !color && !borderColor) return undefined;

  const style: React.CSSProperties = {};
  if (bg) style.background = bg;
  if (color) style.color = color;
  if (borderColor) style.borderColor = borderColor;
  return style;
}
