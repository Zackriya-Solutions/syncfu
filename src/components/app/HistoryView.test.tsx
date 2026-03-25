import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { HistoryView } from "./HistoryView";
import { useHistoryStore } from "@/stores/historyStore";
import type { HistoryEntry } from "@/types/notification";

function makeHistoryEntry(
  overrides: Partial<HistoryEntry> = {}
): HistoryEntry {
  return {
    id: "hist-1",
    sender: "test",
    title: "Test Notification",
    body: "Test body",
    priority: "normal",
    createdAt: new Date().toISOString(),
    ...overrides,
  };
}

describe("HistoryView", () => {
  beforeEach(() => {
    useHistoryStore.getState().reset();
  });

  it("renders empty state when no history", () => {
    render(<HistoryView />);
    expect(screen.getByText(/no notifications yet/i)).toBeInTheDocument();
  });

  it("renders history entries", () => {
    useHistoryStore.getState().setEntries([
      makeHistoryEntry({ id: "h-1", title: "Build passed", sender: "ci" }),
      makeHistoryEntry({ id: "h-2", title: "Tests failed", sender: "test-watcher" }),
    ]);

    render(<HistoryView />);

    expect(screen.getByText("Build passed")).toBeInTheDocument();
    expect(screen.getByText("Tests failed")).toBeInTheDocument();
    expect(screen.getByText("ci")).toBeInTheDocument();
    expect(screen.getByText("test-watcher")).toBeInTheDocument();
  });

  it("shows priority indicator for each entry", () => {
    useHistoryStore.getState().setEntries([
      makeHistoryEntry({ id: "h-1", priority: "critical" }),
      makeHistoryEntry({ id: "h-2", priority: "low" }),
    ]);

    render(<HistoryView />);

    const rows = screen.getAllByTestId("history-row");
    expect(rows[0].querySelector("[data-priority='critical']")).toBeInTheDocument();
    expect(rows[1].querySelector("[data-priority='low']")).toBeInTheDocument();
  });

  it("shows relative time for entries", () => {
    const now = new Date();
    const fiveMinAgo = new Date(now.getTime() - 5 * 60 * 1000).toISOString();

    useHistoryStore.getState().setEntries([
      makeHistoryEntry({ id: "h-1", createdAt: fiveMinAgo }),
    ]);

    render(<HistoryView />);

    expect(screen.getByText(/5m ago/)).toBeInTheDocument();
  });
});
