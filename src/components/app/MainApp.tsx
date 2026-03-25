import { HistoryView } from "./HistoryView";

export function MainApp() {
  return (
    <div data-testid="main-app-root" className="main-app">
      <aside className="sidebar">
        <nav>
          <button className="nav-item active" aria-current="page">
            History
          </button>
        </nav>
      </aside>
      <main className="content">
        <HistoryView />
      </main>
    </div>
  );
}
