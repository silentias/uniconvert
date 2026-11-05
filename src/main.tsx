import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/app/App";
import { applyTheme, getPreferredDarkTheme } from "@/shared/lib/theme";

applyTheme(getPreferredDarkTheme());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
