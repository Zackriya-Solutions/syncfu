import { describe, it, expect, beforeEach, vi } from "vitest";
import { useHistoryStore } from "./historyStore";
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

describe("historyStore", () => {
  beforeEach(() => {
    useHistoryStore.getState().reset();
  });

  describe("setEntries", () => {
    it("sets history entries", () => {
      const entries = [
        makeHistoryEntry({ id: "h-1" }),
        makeHistoryEntry({ id: "h-2" }),
      ];
      useHistoryStore.getState().setEntries(entries);

      expect(useHistoryStore.getState().entries).toHaveLength(2);
    });

    it("replaces existing entries (immutable)", () => {
      useHistoryStore
        .getState()
        .setEntries([makeHistoryEntry({ id: "h-1" })]);
      const before = useHistoryStore.getState().entries;

      useHistoryStore
        .getState()
        .setEntries([makeHistoryEntry({ id: "h-2" })]);
      const after = useHistoryStore.getState().entries;

      expect(before).not.toBe(after);
      expect(after).toHaveLength(1);
      expect(after[0].id).toBe("h-2");
    });
  });

  describe("prependEntry", () => {
    it("adds new entry to the top", () => {
      useHistoryStore
        .getState()
        .setEntries([makeHistoryEntry({ id: "h-1" })]);
      useHistoryStore
        .getState()
        .prependEntry(makeHistoryEntry({ id: "h-new" }));

      const entries = useHistoryStore.getState().entries;
      expect(entries[0].id).toBe("h-new");
      expect(entries).toHaveLength(2);
    });
  });

  describe("selectedId", () => {
    it("selects an entry by id", () => {
      useHistoryStore.getState().selectEntry("h-1");
      expect(useHistoryStore.getState().selectedId).toBe("h-1");
    });

    it("clears selection", () => {
      useHistoryStore.getState().selectEntry("h-1");
      useHistoryStore.getState().selectEntry(null);
      expect(useHistoryStore.getState().selectedId).toBeNull();
    });
  });

  describe("filters", () => {
    it("sets search filter", () => {
      useHistoryStore.getState().setSearch("deploy");
      expect(useHistoryStore.getState().search).toBe("deploy");
    });

    it("sets sender filter", () => {
      useHistoryStore.getState().setSenderFilter("ci-pipeline");
      expect(useHistoryStore.getState().senderFilter).toBe("ci-pipeline");
    });

    it("sets priority filter", () => {
      useHistoryStore.getState().setPriorityFilter("critical");
      expect(useHistoryStore.getState().priorityFilter).toBe("critical");
    });

    it("sets date range filter", () => {
      useHistoryStore.getState().setDateRange("7d");
      expect(useHistoryStore.getState().dateRange).toBe("7d");
    });

    it("clears all filters", () => {
      useHistoryStore.getState().setSearch("test");
      useHistoryStore.getState().setSenderFilter("ci");
      useHistoryStore.getState().setPriorityFilter("high");
      useHistoryStore.getState().setDateRange("7d");

      useHistoryStore.getState().clearFilters();

      const state = useHistoryStore.getState();
      expect(state.search).toBe("");
      expect(state.senderFilter).toBeNull();
      expect(state.priorityFilter).toBeNull();
      expect(state.dateRange).toBe("all");
    });
  });

  describe("loading state", () => {
    it("tracks loading state", () => {
      expect(useHistoryStore.getState().loading).toBe(false);
      useHistoryStore.getState().setLoading(true);
      expect(useHistoryStore.getState().loading).toBe(true);
    });
  });

  describe("senders list", () => {
    it("sets available senders", () => {
      useHistoryStore.getState().setSenders(["ci", "remind", "deploy"]);
      expect(useHistoryStore.getState().senders).toEqual([
        "ci",
        "remind",
        "deploy",
      ]);
    });
  });

  describe("reset", () => {
    it("resets all state to defaults", () => {
      useHistoryStore
        .getState()
        .setEntries([makeHistoryEntry()]);
      useHistoryStore.getState().setSearch("test");
      useHistoryStore.getState().setSenderFilter("ci");

      useHistoryStore.getState().reset();

      const state = useHistoryStore.getState();
      expect(state.entries).toHaveLength(0);
      expect(state.search).toBe("");
      expect(state.senderFilter).toBeNull();
      expect(state.selectedId).toBeNull();
    });
  });
});
