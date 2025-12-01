import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ConfigContext } from "@/context/ConfigContext";
import { Config } from "@/types/types";
import { Label } from "@radix-ui/react-label";
import { useContext, useEffect, useState } from "react";
import { toast } from "sonner";

export function SettingsScreen() {
  const ctx = useContext(ConfigContext);

  if (!ctx || !ctx.value) {
    return <p className="p-4 text-muted-foreground">Loading config...</p>;
  }

  const [form, setForm] = useState<Config>({ ...ctx.value });

  const updateField = (field: string, value: string | number | string[]) => {
    setForm((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const handleSave = async () => {
    try {
      await ctx.updateConfig(form);
      toast.success("Config saved!");
    } catch (err) {
      toast.error("Failed to save config");
      console.error(err);
    }
  };

  useEffect(() => {
    ctx.getInitialConfig();
  }, []);

  return (
    <Card className="mt-20 w-full max-w-3xl mx-auto">
      <CardHeader>
        <CardTitle>Flash Configuration</CardTitle>
      </CardHeader>

      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label>Chip</Label>
            <Input
              value={form.chip}
              onChange={(e) => updateField("chip", e.target.value)}
            />
          </div>

          <div>
            <Label>Baud Rate</Label>
            <Input
              type="number"
              value={form.baud_rate}
              onChange={(e) => updateField("baud_rate", Number(e.target.value))}
            />
          </div>

          <div>
            <Label>Flash Mode</Label>
            <Input
              value={form.flash_mode}
              onChange={(e) => updateField("flash_mode", e.target.value)}
            />
          </div>

          <div>
            <Label>Flash Size</Label>
            <Input
              value={form.flash_size}
              onChange={(e) => updateField("flash_size", e.target.value)}
            />
          </div>

          <div>
            <Label>Flash Frequency</Label>
            <Input
              value={form.flash_freq}
              onChange={(e) => updateField("flash_freq", e.target.value)}
            />
          </div>

          <div>
            <Label>Bootloader Offset</Label>
            <Input
              value={form.bootloader_start}
              onChange={(e) => updateField("bootloader_start", e.target.value)}
            />
          </div>

          <div>
            <Label>Partition Offset</Label>
            <Input
              value={form.partition_start}
              onChange={(e) => updateField("partition_start", e.target.value)}
            />
          </div>

          <div>
            <Label>Firmware Offset</Label>
            <Input
              value={form.firmware_start}
              onChange={(e) => updateField("firmware_start", e.target.value)}
            />
          </div>

          <div>
            <Label>Flags Before</Label>
            <Input
              value={form.before_flags.join(" ")}
              onChange={(e) =>
                updateField("before_flags", e.target.value.split(" "))
              }
            />
          </div>

          <div>
            <Label>Flags After</Label>
            <Input
              value={form.after_flags.join(" ")}
              onChange={(e) =>
                updateField("after_flags", e.target.value.split(" "))
              }
            />
          </div>
        </div>
      </CardContent>

      <CardFooter className="justify-end">
        <Button onClick={handleSave}>Save Configuration</Button>
      </CardFooter>
    </Card>
  );
}
