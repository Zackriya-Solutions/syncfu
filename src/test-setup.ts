import "@testing-library/jest-dom/vitest";

// jsdom doesn't have ResizeObserver
global.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;
