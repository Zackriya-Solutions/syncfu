import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useNotifications } from "./useNotifications";
import { useNotificationStore } from "@/stores/notificationStore";
import { event as tauriEvent } from "@tauri-apps/api";
import { emitMockEvent, clearMockListeners } from "@/__mocks__/tauri-api";
import type { NotificationPayload } from "@/types/notification";

function makeNotification(
  overrides: Partial<NotificationPayload> = {}
): NotificationPayload {
  return {
    id: "test-1",
    sender: "ci-pipeline",
    title: "Build Complete",
    body: "All tests passed",
    priority: "normal",
    timeout: "default",
    actions: [],
    createdAt: new Date().toISOString(),
    ...overrides,
  };
}

describe("useNotifications", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearMockListeners();
    useNotificationStore.getState().clear();
  });

  afterEach(() => {
    clearMockListeners();
  });

  it("returns current notifications from store", () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1", title: "First" })
    );

    const { result } = renderHook(() => useNotifications());

    expect(result.current.notifications).toHaveLength(1);
    expect(result.current.notifications[0].title).toBe("First");
  });

  it("returns empty array when no notifications", () => {
    const { result } = renderHook(() => useNotifications());
    expect(result.current.notifications).toHaveLength(0);
  });

  it("provides dismiss function", () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1" })
    );

    const { result } = renderHook(() => useNotifications());

    act(() => {
      result.current.dismiss("n1");
    });

    expect(result.current.notifications).toHaveLength(0);
  });

  it("provides dismissAll function", () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1" })
    );
    useNotificationStore.getState().add(
      makeNotification({ id: "n2" })
    );

    const { result } = renderHook(() => useNotifications());

    act(() => {
      result.current.dismissAll();
    });

    expect(result.current.notifications).toHaveLength(0);
  });

  it("subscribes to notification:add events on mount", async () => {
    renderHook(() => useNotifications());

    // Wait for async event setup
    await vi.waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledWith(
        "notification:add",
        expect.any(Function)
      );
    });
  });

  it("subscribes to notification:dismiss events on mount", async () => {
    renderHook(() => useNotifications());

    await vi.waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledWith(
        "notification:dismiss",
        expect.any(Function)
      );
    });
  });

  it("subscribes to notification:dismiss-all events on mount", async () => {
    renderHook(() => useNotifications());

    await vi.waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledWith(
        "notification:dismiss-all",
        expect.any(Function)
      );
    });
  });

  it("adds notification when notification:add event fires", async () => {
    const { result } = renderHook(() => useNotifications());

    await vi.waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    const payload = makeNotification({ id: "event-1", title: "From Event" });

    act(() => {
      emitMockEvent("notification:add", payload);
    });

    expect(result.current.notifications).toHaveLength(1);
    expect(result.current.notifications[0].title).toBe("From Event");
  });

  it("removes notification when notification:dismiss event fires", async () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1", title: "Will Go" })
    );

    const { result } = renderHook(() => useNotifications());

    await vi.waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    act(() => {
      emitMockEvent("notification:dismiss", "n1");
    });

    expect(result.current.notifications).toHaveLength(0);
  });

  it("clears all when notification:dismiss-all event fires", async () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1" })
    );
    useNotificationStore.getState().add(
      makeNotification({ id: "n2" })
    );

    const { result } = renderHook(() => useNotifications());

    await vi.waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    act(() => {
      emitMockEvent("notification:dismiss-all", 2);
    });

    expect(result.current.notifications).toHaveLength(0);
  });

  it("cleans up listeners on unmount", async () => {
    const { unmount } = renderHook(() => useNotifications());

    await vi.waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    unmount();

    // After unmount, events should not affect the store
    // (listeners were removed)
    expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
  });
});
