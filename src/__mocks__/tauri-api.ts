import { vi } from "vitest";

const listeners = new Map<string, Set<(event: unknown) => void>>();

export const event = {
  listen: vi.fn(
    (eventName: string, handler: (event: unknown) => void) => {
      if (!listeners.has(eventName)) {
        listeners.set(eventName, new Set());
      }
      listeners.get(eventName)!.add(handler);
      return Promise.resolve(() => {
        listeners.get(eventName)?.delete(handler);
      });
    }
  ),
  emit: vi.fn((eventName: string, payload?: unknown) => {
    listeners.get(eventName)?.forEach((handler) =>
      handler({ payload, event: eventName, id: 0 })
    );
    return Promise.resolve();
  }),
};

export const invoke = vi.fn((_cmd: string, _args?: Record<string, unknown>) => {
  return Promise.resolve(null);
});

export const core = {
  invoke,
};

const mockCurrentWindow = {
  label: "main",
  setIgnoreCursorEvents: vi.fn(() => Promise.resolve()),
  setSize: vi.fn(() => Promise.resolve()),
  show: vi.fn(() => Promise.resolve()),
  hide: vi.fn(() => Promise.resolve()),
  close: vi.fn(() => Promise.resolve()),
  onCloseRequested: vi.fn(() => Promise.resolve(() => {})),
};

export const LogicalSize = vi.fn((width: number, height: number) => ({ width, height, type: "Logical" }));

export const getCurrentWindow = vi.fn(() => mockCurrentWindow);

export const window = {
  getCurrentWindow: vi.fn(() => mockCurrentWindow),
  LogicalSize,
  Window: {
    getByLabel: vi.fn((_label: string) => null),
  },
};

export function emitMockEvent(eventName: string, payload: unknown) {
  listeners.get(eventName)?.forEach((handler) =>
    handler({ payload, event: eventName, id: 0 })
  );
}

export function clearMockListeners() {
  listeners.clear();
}
