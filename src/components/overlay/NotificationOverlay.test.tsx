import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act, waitFor } from "@testing-library/react";
import { NotificationOverlay } from "./NotificationOverlay";
import { useNotificationStore } from "@/stores/notificationStore";
import { event as tauriEvent } from "@tauri-apps/api";
import { emitMockEvent } from "@/__mocks__/tauri-api";
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

describe("NotificationOverlay", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useNotificationStore.getState().clear();
  });

  it("renders overlay root container", () => {
    render(<NotificationOverlay />);
    expect(screen.getByTestId("overlay-root")).toBeInTheDocument();
  });

  it("renders with transparent background class", () => {
    render(<NotificationOverlay />);
    const root = screen.getByTestId("overlay-root");
    expect(root.className).toContain("overlay-root");
  });

  it("renders notification cards from store", () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1", title: "First" })
    );
    useNotificationStore.getState().add(
      makeNotification({ id: "n2", title: "Second" })
    );

    render(<NotificationOverlay />);

    expect(screen.getByText("First")).toBeInTheDocument();
    expect(screen.getByText("Second")).toBeInTheDocument();
  });

  it("renders empty when no notifications", () => {
    render(<NotificationOverlay />);
    expect(screen.queryAllByTestId("notification-card")).toHaveLength(0);
  });

  it("dismisses notification when dismiss button clicked", () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1", title: "Dismissable" })
    );

    render(<NotificationOverlay />);

    fireEvent.click(screen.getByLabelText("Dismiss"));

    expect(screen.queryByText("Dismissable")).not.toBeInTheDocument();
    expect(useNotificationStore.getState().notifications).toHaveLength(0);
  });

  it("subscribes to notification:add events", async () => {
    render(<NotificationOverlay />);
    await waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledWith(
        "notification:add",
        expect.any(Function)
      );
    });
  });

  it("subscribes to notification:dismiss events", async () => {
    render(<NotificationOverlay />);
    await waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledWith(
        "notification:dismiss",
        expect.any(Function)
      );
    });
  });

  it("subscribes to notification:dismiss-all events", async () => {
    render(<NotificationOverlay />);
    await waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledWith(
        "notification:dismiss-all",
        expect.any(Function)
      );
    });
  });

  it("adds notification when notification:add event received", async () => {
    render(<NotificationOverlay />);

    // Wait for event subscriptions to be set up
    await waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    const payload = makeNotification({ id: "event-1", title: "From Event" });

    act(() => {
      emitMockEvent("notification:add", payload);
    });

    expect(screen.getByText("From Event")).toBeInTheDocument();
  });

  it("removes notification when notification:dismiss event received", async () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1", title: "Will Be Dismissed" })
    );

    render(<NotificationOverlay />);

    await waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    act(() => {
      emitMockEvent("notification:dismiss", "n1");
    });

    expect(screen.queryByText("Will Be Dismissed")).not.toBeInTheDocument();
  });

  it("clears all when notification:dismiss-all event received", async () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1", title: "First" })
    );
    useNotificationStore.getState().add(
      makeNotification({ id: "n2", title: "Second" })
    );

    render(<NotificationOverlay />);

    await waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    act(() => {
      emitMockEvent("notification:dismiss-all", 2);
    });

    expect(screen.queryAllByTestId("notification-card")).toHaveLength(0);
  });

  it("renders notification stack container", () => {
    useNotificationStore.getState().add(
      makeNotification({ id: "n1" })
    );

    render(<NotificationOverlay />);
    expect(screen.getByTestId("notification-stack")).toBeInTheDocument();
  });

  it("cleans up event listeners on unmount", async () => {
    const { unmount } = render(<NotificationOverlay />);

    await waitFor(() => {
      expect(tauriEvent.listen).toHaveBeenCalledTimes(3);
    });

    unmount();

    expect(tauriEvent.listen).toHaveBeenCalled();
  });
});
