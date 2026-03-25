import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { App } from "./App";
import { window as tauriWindow } from "@tauri-apps/api";

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders overlay view when window label is 'overlay'", () => {
    vi.mocked(tauriWindow.getCurrentWindow).mockReturnValue({
      label: "overlay",
      setIgnoreCursorEvents: vi.fn(() => Promise.resolve()),
      show: vi.fn(() => Promise.resolve()),
      hide: vi.fn(() => Promise.resolve()),
      close: vi.fn(() => Promise.resolve()),
      onCloseRequested: vi.fn(() => Promise.resolve(() => {})),
    } as unknown as ReturnType<typeof tauriWindow.getCurrentWindow>);

    render(<App />);
    expect(screen.getByTestId("overlay-root")).toBeInTheDocument();
  });

  it("renders main app view when window label is 'main'", () => {
    vi.mocked(tauriWindow.getCurrentWindow).mockReturnValue({
      label: "main",
      setIgnoreCursorEvents: vi.fn(() => Promise.resolve()),
      show: vi.fn(() => Promise.resolve()),
      hide: vi.fn(() => Promise.resolve()),
      close: vi.fn(() => Promise.resolve()),
      onCloseRequested: vi.fn(() => Promise.resolve(() => {})),
    } as unknown as ReturnType<typeof tauriWindow.getCurrentWindow>);

    render(<App />);
    expect(screen.getByTestId("main-app-root")).toBeInTheDocument();
  });
});
