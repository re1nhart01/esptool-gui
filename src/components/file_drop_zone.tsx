import { useState, useRef } from "react";
import { cn } from "@/lib/utils";
import { Upload } from "lucide-react";
import { Button } from "@/components/ui/button";

interface Props {
  label: string;
  file: File | null;
  onSelect: (file: File) => void;
}

export function FileDropZone({ label, file, onSelect }: Props) {
  const [isDragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleFile = (file: File | null) => {
    if (file) onSelect(file);
  };

  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-2",
        "border p-4 rounded-md cursor-pointer transition-colors",
        "bg-secondary text-secondary-foreground",
        isDragging ? "border-primary bg-primary/15" : "border-border",
      )}
      onClick={() => inputRef.current?.click()}
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragging(false);
        handleFile(e.dataTransfer.files[0]);
      }}
    >
      <Upload className="w-6 h-6 opacity-70" />

      {file ? (
        <div className="text-xs text-primary font-semibold max-w-full truncate">
          📦 {file.name}
        </div>
      ) : (
        <span className="text-sm opacity-60">{label}</span>
      )}

      <input
        ref={inputRef}
        type="file"
        className="hidden"
        onChange={(e) => handleFile(e.target.files?.[0] ?? null)}
      />

      <Button variant="outline" size="sm">
        Browse
      </Button>
    </div>
  );
}
