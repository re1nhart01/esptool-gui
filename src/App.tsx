import { ChangeEvent, useEffect, useMemo, useState } from "react";
import "./App.css";
import { Button } from "./components/ui/button";
import { AboutDialog } from "./components/about_dialog";
import { FlashScreen } from "./screens/flash_screen/FlashScreen";
import { SettingsScreen } from "./screens/settings_screen/SetttingsScreen";
import { MonitorScreen } from "./screens/monitor_screen/MonitorScreen";
import { SerialPort } from "./types/types";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [tab, setTab] = useState<"flash" | "settings" | "monitor">("flash");
  const [files, setFiles] = useState<(string | null)[]>([null, null, null]);
  const [logs, setLogs] = useState("");
  const [serials, setSerials] = useState<Array<SerialPort>>([]);
  const [selectedSerial, setSelectedSerial] = useState<string>("");

  const handleSetSelectedSerial = async (e: ChangeEvent<HTMLSelectElement>) => {
    await invoke("tauri_set_selected_port", { selected: e.target.value });
    console.log(e.target.value);
    setSelectedSerial(e.target.value);
  }

  const handleGetSerial = async () => {
    try {
      const serialsRs: string = await invoke("tauri_get_serial_ports");
      const fromJson: Array<SerialPort> = JSON.parse(serialsRs);
      setSerials(fromJson);

      if (!selectedSerial && fromJson.length > 0) {
        setSelectedSerial(fromJson[0].value);
        await invoke("tauri_set_selected_port", { selected: fromJson[0].value });
        console.log(fromJson[0].value);
      }

      console.log(fromJson);
    } catch (e) {
      console.log("no serials", e);
      setSerials([]);
      setSelectedSerial("");
    }
  };

  useEffect(() => {
    handleGetSerial().then();
  }, []);

  const selectedLabel = useMemo(() => {
    return serials.find((s) => s.value === selectedSerial)?.key ?? "";
  }, [serials, selectedSerial]);

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
          variant={tab === "monitor" ? "default" : "secondary"}
          onClick={() => setTab("monitor")}
        >
          Monitor
        </Button>

        <Button
          variant={tab === "settings" ? "default" : "secondary"}
          onClick={() => setTab("settings")}
        >
          ⚙ Settings
        </Button>

        <div className="ml-auto flex items-center gap-2">
          <Button variant="secondary" onClick={handleGetSerial}>
            Get serials
          </Button>

          <select
            className="h-9 rounded-md border bg-background px-3 text-sm outline-none"
            value={selectedSerial}
            onChange={handleSetSelectedSerial}
            disabled={serials.length === 0}
            title={selectedLabel || "Select serial port"}
          >
            {serials.length === 0 ? (
              <option value="">No ports</option>
            ) : (
              serials.map((s) => (
                <option key={s.value} value={s.value}>
                  {s.key}
                </option>
              ))
            )}
          </select>

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
      {tab === "monitor" && <MonitorScreen />}
    </div>
  );
}

export default App;
