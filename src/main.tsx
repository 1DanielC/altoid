import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import ActivityLogWindow from "./components/ActivityLogWindow";
import { AppProvider } from "./contexts/AppContext";

// Check if this is the activity log window
const params = new URLSearchParams(window.location.search);
const windowType = params.get('window');

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {windowType === 'activity-log' ? (
      <ActivityLogWindow />
    ) : (
      <AppProvider>
        <App />
      </AppProvider>
    )}
  </React.StrictMode>,
);
