import { window as tauriWindow } from "@tauri-apps/api";
import { NotificationOverlay } from "./components/overlay/NotificationOverlay";
import { MainApp } from "./components/app/MainApp";
import type { WindowLabel } from "./types/notification";

export function App() {
  const currentWindow = tauriWindow.getCurrentWindow();
  const label = currentWindow.label as WindowLabel;

  if (label === "overlay") {
    return <NotificationOverlay />;
  }

  return <MainApp />;
}
