import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@tauri-apps/api": path.resolve(__dirname, "./src/__mocks__/tauri-api.ts"),
      "@tauri-apps/plugin-autostart": path.resolve(
        __dirname,
        "./src/__mocks__/tauri-plugin-autostart.ts"
      ),
      "@tauri-apps/plugin-dialog": path.resolve(
        __dirname,
        "./src/__mocks__/tauri-plugin-dialog.ts"
      ),
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.{ts,tsx}",
        "src/__mocks__/**",
        "src/test-setup.ts",
        "src/main.tsx",
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 80,
        statements: 80,
      },
    },
  },
});
