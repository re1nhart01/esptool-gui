import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

export function AboutDialog() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="ghost">ℹ️ Info</Button>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle>Про програму</DialogTitle>
        </DialogHeader>

        <div className="text-sm space-y-1">
          <div>ESPTool GUI</div>
          <div>Версія: 1.0.0</div>
          <div>Автор: Evgeniy</div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
