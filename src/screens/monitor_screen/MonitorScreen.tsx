import { useEffect, memo, FC, useRef, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { FileDropZone } from "@/components/file_drop_zone";

type monitorScreenProps = {
 
};

const BOTTOM_THRESHOLD_PX = 60;

export const MonitorScreen: FC<monitorScreenProps> = memo(
  ({ }) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const [monitorLogs, setMonitorLogs] = useState("");
    const [files, setFiles] = useState<(string | null)[]>([null, null, null]);
    const [isScrollLocked, setScrollLocked] = useState(false);


    const stripAnsi = (input: string) =>
        input.replace(
            // eslint-disable-next-line no-control-regex
            /\u001b\[[0-9;?]*[ -/]*[@-~]/g,
            ""
    );

    const selectFile = async (label: string, index: number, file: string) => {
      const updated = [...files];
      updated[index] = file;
      setFiles(updated);

      await invoke("tauri_add_file_into_scope", {
        fileType: label,
        filename: file,
      });
      setMonitorLogs((prev) => prev + `Selected ${file}\n`);
    };

    useEffect(() => {
      const unlistenPromise = listen<string>("esp-tool-monitor", (e) => {
        setMonitorLogs((prev) => prev + stripAnsi(e.payload));
      });

      return () => {
        unlistenPromise.then((unlisten) => unlisten());
      };
    }, []);

    useEffect(() => {
        const el = textareaRef.current;
        if (!el) return;

        if (!isScrollLocked) {
        el.scrollTop = el.scrollHeight;
        }
    }, [monitorLogs, isScrollLocked]);



    const isNearBottom = useCallback((el: HTMLTextAreaElement) => {
        const distance =
        el.scrollHeight - el.scrollTop - el.clientHeight;
        return distance <= BOTTOM_THRESHOLD_PX;
    }, []);

    const handleOnScroll = useCallback(() => {
        const el = textareaRef.current;
        if (!el) return;

        setScrollLocked(!isNearBottom(el));
    }, [isNearBottom]);

    

    const handleRunMonitor = async () => {
      setMonitorLogs((p) => p + "Monitor starting...\n");
      await invoke("tauri_monitor_start", { filename: "zxc" });
    };

    const handleCancel = async () => {
      await invoke("tauri_monitor_stop");
      setMonitorLogs((p) => p + "Monitor canceled.\n");
    };

    const handleClear = async () => {
      setMonitorLogs("");
    };

    return (
      <div className="space-y-3">
         {["Firmware Elf"].map((label, i) => (
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
          className="h-[55vh] flex-1 min-h-0"
          value={monitorLogs}
          onScroll={handleOnScroll}
        />

        <Button className="w-full" onClick={handleRunMonitor}>
          RUN
        </Button>

        <Button className="w-full bg-red-800" onClick={handleClear}>
          CLEAR
        </Button>

        <Button className="w-full bg-blue-800" onClick={handleCancel}>
          STOP
        </Button>
      </div>
    );
  },
);
