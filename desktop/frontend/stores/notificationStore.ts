import { create } from "zustand";
import type { NotificationPayload, ProgressInfo } from "@/types/notification";

const MAX_VISIBLE = 5;

interface NotificationUpdate {
  readonly body?: string;
  readonly progress?: ProgressInfo;
}

interface NotificationState {
  readonly notifications: readonly NotificationPayload[];
  readonly queued: readonly NotificationPayload[];
  add: (notification: NotificationPayload) => void;
  dismiss: (id: string) => void;
  dismissAll: () => void;
  update: (id: string, partial: NotificationUpdate) => void;
  clear: () => void;
}

export const useNotificationStore = create<NotificationState>((set) => ({
  notifications: [],
  queued: [],

  add: (notification) =>
    set((state) => {
      const withoutDuplicate = state.notifications.filter(
        (n) => n.id !== notification.id
      );
      const queuedWithoutDuplicate = state.queued.filter(
        (n) => n.id !== notification.id
      );
      const merged = [notification, ...withoutDuplicate];

      if (merged.length <= MAX_VISIBLE) {
        return {
          notifications: merged,
          queued: queuedWithoutDuplicate,
        };
      }

      return {
        notifications: merged.slice(0, MAX_VISIBLE),
        queued: [...queuedWithoutDuplicate, ...merged.slice(MAX_VISIBLE)],
      };
    }),

  dismiss: (id) =>
    set((state) => {
      const remaining = state.notifications.filter((n) => n.id !== id);
      const newQueued = [...state.queued];

      if (
        remaining.length < MAX_VISIBLE &&
        remaining.length < state.notifications.length &&
        newQueued.length > 0
      ) {
        const promoted = newQueued.shift()!;
        return {
          notifications: [...remaining, promoted],
          queued: newQueued,
        };
      }

      return { notifications: remaining, queued: newQueued };
    }),

  dismissAll: () =>
    set({ notifications: [], queued: [] }),

  update: (id, partial) =>
    set((state) => ({
      notifications: state.notifications.map((n) =>
        n.id === id ? { ...n, ...partial } : n
      ),
    })),

  clear: () =>
    set({ notifications: [], queued: [] }),
}));
