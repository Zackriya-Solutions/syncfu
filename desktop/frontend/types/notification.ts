export type Priority = "low" | "normal" | "high" | "critical";
export type ActionStyle = "primary" | "secondary" | "danger";
export type ProgressStyle = "bar" | "ring";
export type DismissReason = "timeout" | "user" | "replaced";

export interface Action {
  readonly id: string;
  readonly label: string;
  readonly style: ActionStyle;
  readonly icon?: string;
  readonly bg?: string;
  readonly color?: string;
  readonly borderColor?: string;
}

/** Per-notification style overrides. All optional — defaults from CSS variables. */
export interface StyleOverrides {
  readonly accentColor?: string;
  readonly cardBg?: string;
  readonly cardBorderRadius?: string;
  readonly iconColor?: string;
  readonly iconBg?: string;
  readonly iconBorderColor?: string;
  readonly titleColor?: string;
  readonly titleFontSize?: string;
  readonly bodyColor?: string;
  readonly bodyFontSize?: string;
  readonly senderColor?: string;
  readonly timeColor?: string;
  readonly btnBg?: string;
  readonly btnColor?: string;
  readonly btnBorderColor?: string;
  readonly btn2Bg?: string;
  readonly btn2Color?: string;
  readonly btn2BorderColor?: string;
  readonly dangerBg?: string;
  readonly dangerColor?: string;
  readonly dangerBorderColor?: string;
  readonly progressColor?: string;
  readonly progressTrackColor?: string;
  readonly countdownColor?: string;
  readonly closeBg?: string;
  readonly closeColor?: string;
  readonly closeBorderColor?: string;
}

export interface ProgressInfo {
  readonly value: number;
  readonly label?: string;
  readonly style: ProgressStyle;
}

export interface Timeout {
  readonly seconds?: number;
  readonly never?: boolean;
}

export interface NotificationPayload {
  readonly id: string;
  readonly sender: string;
  readonly title: string;
  readonly body: string;
  readonly icon?: string;
  readonly font?: string;
  readonly priority: Priority;
  readonly timeout: Timeout | "never" | "default";
  readonly actions: readonly Action[];
  readonly progress?: ProgressInfo;
  readonly group?: string;
  readonly theme?: string;
  readonly sound?: string;
  readonly callbackUrl?: string;
  readonly style?: StyleOverrides;
  readonly createdAt: string;
}

export interface HistoryEntry {
  readonly id: string;
  readonly sender: string;
  readonly title: string;
  readonly body: string;
  readonly priority: Priority;
  readonly groupKey?: string;
  readonly actionsJson?: string;
  readonly createdAt: string;
  readonly dismissedAt?: string;
  readonly actionTaken?: string;
  readonly callbackResult?: string;
}

export type WindowLabel = "overlay" | "main";
