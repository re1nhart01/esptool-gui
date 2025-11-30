import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { FileDropZone } from "@/components/file_drop_zone";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";

export function FlashScreen() {
  const [logs, setLogs] = useState("");
  const [files, setFiles] = useState<(string | null)[]>([null, null, null]);

  const selectFile = (index: number, file: string) => {
    const updated = [...files];
    updated[index] = file;
    setFiles(updated);

    setLogs((prev) => prev + `Selected ${file}\n`);
  };

  useEffect(() => {
    const unlistenPromise = listen<string>("esp-tool-log", (e) => {
      setLogs((prev) => prev + e.payload + "\n");
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleFlash = async () => {
    setLogs((p) => p + "Flash started...\n");
    await invoke("tauri_execute_and_listen", { filename: "zxc" });
  };

  const handleCancel = async () => {
    await invoke("tauri_free_listen_handle");
    setLogs((p) => p + "Flash canceled.\n");
  };

  return (
    <div className="space-y-3 h-full">
      {["Bootloader", "Partition Table", "Firmware"].map((label, i) => (
        <FileDropZone
          key={i}
          label={label}
          file={files[i]}
          onSelect={(file) => selectFile(i, file)}
        />
      ))}

      <Textarea readOnly className="h-52 flex-1 min-h-0" value={logs} />

      <Button className="w-full" onClick={handleFlash}>
        FLASH 🚀
      </Button>

      <Button className="w-full bg-blue-800" onClick={handleCancel}>
        CANCEL
      </Button>
    </div>
  );
}
