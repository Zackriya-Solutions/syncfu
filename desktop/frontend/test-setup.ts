import "@testing-library/jest-dom/vitest";

// jsdom doesn't have ResizeObserver
(globalThis as Record<string, unknown>).ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;
