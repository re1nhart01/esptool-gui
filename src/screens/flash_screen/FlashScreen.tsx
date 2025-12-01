import { useEffect, memo, FC, Dispatch, SetStateAction, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { FileDropZone } from "@/components/file_drop_zone";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";

type flashScreenProps = {
  logs: string;
  setLogs: Dispatch<SetStateAction<string>>;
  files: (string | null)[];
  setFiles: Dispatch<SetStateAction<(string | null)[]>>;
};

export const FlashScreen: FC<flashScreenProps> = memo(
  ({ files, setFiles, logs, setLogs }) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const selectFile = async (label: string, index: number, file: string) => {
      const updated = [...files];
      updated[index] = file;
      setFiles(updated);

      await invoke("tauri_add_file_into_scope", {
        fileType: label,
        filename: file,
      });
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

    useEffect(() => {
      if (textareaRef.current) {
        textareaRef.current.scrollTop = textareaRef.current.scrollHeight;
      }
    }, [logs]);

    const handleFlash = async () => {
      setLogs((p) => p + "Flash started...\n");
      await invoke("tauri_execute_and_listen", { filename: "zxc" });
    };

    const handleCancel = async () => {
      await invoke("tauri_free_listen_handle");
      setLogs((p) => p + "Flash canceled.\n");
    };

    return (
      <div className="space-y-3">
        {["Bootloader", "Partition Table", "Firmware"].map((label, i) => (
          <FileDropZone
            key={i}
            label={label}
            file={files[i]}
            onSelect={async (file) => await selectFile(label, i, file)}
          />
        ))}

        <Textarea
          ref={textareaRef}
          readOnly
          className="h-52 flex-1 min-h-0"
          value={logs}
        />

        <Button className="w-full" onClick={handleFlash}>
          FLASH 🚀
        </Button>

        <Button className="w-full bg-blue-800" onClick={handleCancel}>
          CANCEL
        </Button>
      </div>
    );
  },
);
