import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ConfigContextProvider } from "./context/ConfigContext";
import { Toaster } from "sonner";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <ConfigContextProvider>
    <React.StrictMode>
      <Toaster richColors />
      <App />
    </React.StrictMode>
  </ConfigContextProvider>,
);
