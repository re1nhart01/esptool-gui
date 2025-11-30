import { useState, useRef } from "react";
import { cn } from "@/lib/utils";
import { Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { appDataDir } from "@tauri-apps/api/path";
import { writeFile } from "@tauri-apps/plugin-fs";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  label: string;
  file: string | null;
  onSelect: (path: string) => void;
}

export function FileDropZone({ label, file, onSelect }: Props) {
  const [isDragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const selectByDialog = async () => {
    try {
      const filepath = await open({
        multiple: false,
        filters: [{ name: "Bin Files", extensions: ["bin"] }],
      });
      if (typeof filepath === "string") onSelect(filepath);
    } catch (e) {
      console.log(e);
    }
  };

  const selectByDrop = async (file: File | null) => {
    if (!file) return;

    if (!file.name.endsWith(".bin")) {
      alert("Only .bin files allowed!");
      return;
    }

    const buf = new Uint8Array(await file.arrayBuffer());
    const dir = await appDataDir();
    const fullPath = `${dir}/${file.name}`;

    await writeFile(fullPath, buf);
    onSelect(fullPath);
  };

  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-2",
        "border p-4 rounded-md cursor-pointer transition-colors",
        "bg-secondary text-secondary-foreground",
        isDragging ? "border-primary bg-primary/15" : "border-border",
      )}
      onClick={selectByDialog}
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragging(false);
        selectByDrop(e.dataTransfer.files[0]);
      }}
    >
      <Upload className="w-6 h-6 opacity-70" />

      {file ? (
        <div className="text-xs text-primary font-semibold max-w-full truncate">
          📍 {file}
        </div>
      ) : (
        <span className="text-sm opacity-60">{label}</span>
      )}

      <input
        ref={inputRef}
        type="file"
        className="hidden"
        onChange={(e) => selectByDrop(e.target.files?.[0] ?? null)}
      />

      <Button
        variant="outline"
        size="sm"
        onClick={(e) => {
          e.stopPropagation();
          selectByDialog();
        }}
      >
        Browse
      </Button>
    </div>
  );
}
