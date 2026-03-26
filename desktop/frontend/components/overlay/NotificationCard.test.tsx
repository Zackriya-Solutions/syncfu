import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { NotificationCard } from "./NotificationCard";
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

describe("NotificationCard", () => {
  it("renders title and body", () => {
    render(
      <NotificationCard
        notification={makeNotification()}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    expect(screen.getByText("Build Complete")).toBeInTheDocument();
    expect(screen.getByText("All tests passed")).toBeInTheDocument();
  });

  it("renders sender name", () => {
    render(
      <NotificationCard
        notification={makeNotification()}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    expect(screen.getByText("ci-pipeline")).toBeInTheDocument();
  });

  it("applies priority class", () => {
    render(
      <NotificationCard
        notification={makeNotification({ priority: "critical" })}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    const card = screen.getByTestId("notification-card");
    expect(card.className).toContain("critical");
  });

  it("renders action buttons", () => {
    const notification = makeNotification({
      actions: [
        { id: "approve", label: "Approve", style: "primary" },
        { id: "reject", label: "Reject", style: "danger" },
      ],
    });

    render(
      <NotificationCard
        notification={notification}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    expect(screen.getByText("Approve")).toBeInTheDocument();
    expect(screen.getByText("Reject")).toBeInTheDocument();
  });

  it("calls onAction when action button clicked", () => {
    const onAction = vi.fn();
    const notification = makeNotification({
      actions: [{ id: "approve", label: "Approve", style: "primary" }],
    });

    render(
      <NotificationCard
        notification={notification}
        onDismiss={vi.fn()}
        onAction={onAction}
      />
    );

    fireEvent.click(screen.getByText("Approve"));
    expect(onAction).toHaveBeenCalledWith("test-1", "approve");
  });

  it("calls onDismiss when dismiss button clicked", () => {
    vi.useFakeTimers();
    const onDismiss = vi.fn();

    render(
      <NotificationCard
        notification={makeNotification()}
        onDismiss={onDismiss}
        onAction={vi.fn()}
      />
    );

    fireEvent.click(screen.getByLabelText("Dismiss"));
    vi.advanceTimersByTime(300);
    expect(onDismiss).toHaveBeenCalledWith("test-1");
    vi.useRealTimers();
  });

  it("renders progress bar when progress is provided", () => {
    const notification = makeNotification({
      progress: { value: 0.65, label: "65%", style: "bar" },
    });

    render(
      <NotificationCard
        notification={notification}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    const progressBar = screen.getByRole("progressbar");
    expect(progressBar).toBeInTheDocument();
    expect(progressBar.getAttribute("aria-valuenow")).toBe("65");
    expect(screen.getByText("65%")).toBeInTheDocument();
  });

  it("does not render progress bar when no progress", () => {
    render(
      <NotificationCard
        notification={makeNotification()}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    expect(screen.queryByRole("progressbar")).not.toBeInTheDocument();
  });

  it("applies custom theme class", () => {
    render(
      <NotificationCard
        notification={makeNotification({ theme: "github-dark" })}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    const card = screen.getByTestId("notification-card");
    expect(card.className).toContain("github-dark");
  });

  it("applies style overrides as CSS custom properties", () => {
    render(
      <NotificationCard
        notification={makeNotification({
          style: {
            accentColor: "#22c55e",
            titleColor: "#bbf7d0",
            cardBg: "rgba(10, 40, 20, 0.96)",
          },
        })}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    const card = screen.getByTestId("notification-card");
    expect(card.style.getPropertyValue("--s-accent-color")).toBe("#22c55e");
    expect(card.style.getPropertyValue("--s-title-color")).toBe("#bbf7d0");
    expect(card.style.getPropertyValue("--s-card-bg")).toBe("rgba(10, 40, 20, 0.96)");
  });

  it("renders without style overrides (no extra CSS vars)", () => {
    render(
      <NotificationCard
        notification={makeNotification()}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    const card = screen.getByTestId("notification-card");
    expect(card.style.getPropertyValue("--s-accent-color")).toBe("");
  });

  it("applies per-action inline styles", () => {
    render(
      <NotificationCard
        notification={makeNotification({
          actions: [
            { id: "deploy", label: "Deploy", style: "primary", bg: "#22c55e", color: "#fff" },
          ],
        })}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    const button = screen.getByText("Deploy");
    // Browser normalizes hex to rgb
    expect(button.style.background).toBeTruthy();
    expect(button.style.color).toBeTruthy();
  });

  it("renders action button with icon", () => {
    render(
      <NotificationCard
        notification={makeNotification({
          actions: [
            { id: "view", label: "View", style: "primary", icon: "eye" },
          ],
        })}
        onDismiss={vi.fn()}
        onAction={vi.fn()}
      />
    );

    const button = screen.getByText("View");
    // Button should contain an SVG icon from NotificationIcon
    expect(button.querySelector("svg")).toBeTruthy();
  });
});
