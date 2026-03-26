import { vi } from "vitest";

export const ask = vi.fn(() => Promise.resolve(true));
export const message = vi.fn(() => Promise.resolve());
export const confirm = vi.fn(() => Promise.resolve(true));
