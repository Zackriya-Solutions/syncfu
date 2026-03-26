import { vi } from "vitest";

export const isEnabled = vi.fn(() => Promise.resolve(false));
export const enable = vi.fn(() => Promise.resolve());
export const disable = vi.fn(() => Promise.resolve());
