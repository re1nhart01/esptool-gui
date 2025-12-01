import { useState } from "react";
import "./App.css";
import { Button } from "./components/ui/button";
import { AboutDialog } from "./components/about_dialog";
import { FlashScreen } from "./screens/flash_screen/FlashScreen";
import { SettingsScreen } from "./screens/settings_screen/SetttingsScreen";

function App() {
  const [tab, setTab] = useState<"flash" | "settings">("flash");
  const [files, setFiles] = useState<(string | null)[]>([null, null, null]);
  const [logs, setLogs] = useState("");

  return (
    <div className="w-full p-4 space-y-4 max-h-screen">
      <div className="flex items-center gap-3">
        <Button
          variant={tab === "flash" ? "default" : "secondary"}
          onClick={() => setTab("flash")}
        >
          Flash
        </Button>

        <Button
          variant={tab === "settings" ? "default" : "secondary"}
          onClick={() => setTab("settings")}
        >
          ⚙ Налаштування
        </Button>

        <div className="ml-auto">
          <AboutDialog />
        </div>
      </div>

      {tab === "flash" && (
        <FlashScreen
          files={files}
          setFiles={setFiles}
          logs={logs}
          setLogs={setLogs}
        />
      )}
      {tab === "settings" && <SettingsScreen />}
    </div>
  );
}

export default App;
