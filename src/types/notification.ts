export type Priority = "low" | "normal" | "high" | "critical";
export type ActionStyle = "primary" | "secondary" | "danger";
export type ProgressStyle = "bar" | "ring";
export type DismissReason = "timeout" | "user" | "replaced";

export interface Action {
  readonly id: string;
  readonly label: string;
  readonly style: ActionStyle;
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
