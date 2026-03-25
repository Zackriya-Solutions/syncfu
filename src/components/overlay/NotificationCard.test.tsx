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
    const onDismiss = vi.fn();

    render(
      <NotificationCard
        notification={makeNotification()}
        onDismiss={onDismiss}
        onAction={vi.fn()}
      />
    );

    fireEvent.click(screen.getByLabelText("Dismiss"));
    expect(onDismiss).toHaveBeenCalledWith("test-1");
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
});
