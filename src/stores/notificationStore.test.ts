import { describe, it, expect, beforeEach } from "vitest";
import { useNotificationStore } from "./notificationStore";
import type { NotificationPayload } from "@/types/notification";

function makeNotification(
  overrides: Partial<NotificationPayload> = {}
): NotificationPayload {
  return {
    id: "test-1",
    sender: "test",
    title: "Test Notification",
    body: "Test body",
    priority: "normal",
    timeout: "default",
    actions: [],
    createdAt: new Date().toISOString(),
    ...overrides,
  };
}

describe("notificationStore", () => {
  beforeEach(() => {
    useNotificationStore.getState().clear();
  });

  describe("add", () => {
    it("adds a notification to the active list", () => {
      const notification = makeNotification();
      useNotificationStore.getState().add(notification);

      const state = useNotificationStore.getState();
      expect(state.notifications).toHaveLength(1);
      expect(state.notifications[0]).toEqual(notification);
    });

    it("prepends new notifications (most recent first)", () => {
      const first = makeNotification({ id: "first", title: "First" });
      const second = makeNotification({ id: "second", title: "Second" });

      useNotificationStore.getState().add(first);
      useNotificationStore.getState().add(second);

      const state = useNotificationStore.getState();
      expect(state.notifications[0].id).toBe("second");
      expect(state.notifications[1].id).toBe("first");
    });

    it("does not mutate existing notifications array", () => {
      const first = makeNotification({ id: "first" });
      useNotificationStore.getState().add(first);
      const before = useNotificationStore.getState().notifications;

      const second = makeNotification({ id: "second" });
      useNotificationStore.getState().add(second);
      const after = useNotificationStore.getState().notifications;

      expect(before).not.toBe(after);
      expect(before).toHaveLength(1);
    });

    it("replaces notification with same id", () => {
      const original = makeNotification({ id: "same", title: "Original" });
      const updated = makeNotification({ id: "same", title: "Updated" });

      useNotificationStore.getState().add(original);
      useNotificationStore.getState().add(updated);

      const state = useNotificationStore.getState();
      expect(state.notifications).toHaveLength(1);
      expect(state.notifications[0].title).toBe("Updated");
    });

    it("caps at max visible (5) notifications", () => {
      for (let i = 0; i < 7; i++) {
        useNotificationStore
          .getState()
          .add(makeNotification({ id: `n-${i}` }));
      }

      const state = useNotificationStore.getState();
      expect(state.notifications).toHaveLength(5);
      expect(state.queued).toHaveLength(2);
    });
  });

  describe("dismiss", () => {
    it("removes a notification by id", () => {
      const notification = makeNotification({ id: "to-dismiss" });
      useNotificationStore.getState().add(notification);
      useNotificationStore.getState().dismiss("to-dismiss");

      expect(useNotificationStore.getState().notifications).toHaveLength(0);
    });

    it("does nothing for unknown id", () => {
      const notification = makeNotification();
      useNotificationStore.getState().add(notification);
      useNotificationStore.getState().dismiss("unknown");

      expect(useNotificationStore.getState().notifications).toHaveLength(1);
    });

    it("promotes queued notification when active slot opens", () => {
      for (let i = 0; i < 6; i++) {
        useNotificationStore
          .getState()
          .add(makeNotification({ id: `n-${i}` }));
      }

      expect(useNotificationStore.getState().notifications).toHaveLength(5);
      expect(useNotificationStore.getState().queued).toHaveLength(1);

      useNotificationStore.getState().dismiss("n-4");

      expect(useNotificationStore.getState().notifications).toHaveLength(5);
      expect(useNotificationStore.getState().queued).toHaveLength(0);
    });
  });

  describe("update", () => {
    it("updates progress on an existing notification", () => {
      const notification = makeNotification({
        id: "prog",
        progress: { value: 0.3, style: "bar" },
      });
      useNotificationStore.getState().add(notification);
      useNotificationStore.getState().update("prog", {
        progress: { value: 0.7, label: "70%", style: "bar" },
      });

      const updated = useNotificationStore.getState().notifications[0];
      expect(updated.progress?.value).toBe(0.7);
      expect(updated.progress?.label).toBe("70%");
    });

    it("updates body on an existing notification", () => {
      const notification = makeNotification({ id: "body-update" });
      useNotificationStore.getState().add(notification);
      useNotificationStore.getState().update("body-update", {
        body: "New body content",
      });

      const updated = useNotificationStore.getState().notifications[0];
      expect(updated.body).toBe("New body content");
    });

    it("does not mutate the original notification object", () => {
      const notification = makeNotification({ id: "immut" });
      useNotificationStore.getState().add(notification);
      const before = useNotificationStore.getState().notifications[0];

      useNotificationStore.getState().update("immut", { body: "Changed" });
      const after = useNotificationStore.getState().notifications[0];

      expect(before).not.toBe(after);
      expect(before.body).toBe("Test body");
    });
  });

  describe("dismissAll", () => {
    it("clears all active and queued notifications", () => {
      for (let i = 0; i < 7; i++) {
        useNotificationStore
          .getState()
          .add(makeNotification({ id: `n-${i}` }));
      }

      useNotificationStore.getState().dismissAll();

      const state = useNotificationStore.getState();
      expect(state.notifications).toHaveLength(0);
      expect(state.queued).toHaveLength(0);
    });
  });

  describe("clear", () => {
    it("resets store to initial state", () => {
      useNotificationStore
        .getState()
        .add(makeNotification({ id: "n-1" }));

      useNotificationStore.getState().clear();

      const state = useNotificationStore.getState();
      expect(state.notifications).toHaveLength(0);
      expect(state.queued).toHaveLength(0);
    });
  });
});
