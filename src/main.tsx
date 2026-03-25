import React from "react";
import ReactDOM from "react-dom/client";
import { attachConsole } from "@tauri-apps/plugin-log";
import { App } from "./App";
import "./styles/globals.css";
import "./styles/overlay.css";
import "./styles/app.css";
import "./styles/animations.css";

// Bridge frontend console.log to Tauri log plugin (dev only)
attachConsole();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
