import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useContext } from "react";
import { ConfigContext } from "@/context/ConfigContext";

export function AboutDialog() {
  const ctx = useContext(ConfigContext);

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="ghost">ℹ️ Info</Button>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle>About</DialogTitle>
        </DialogHeader>

        <div className="text-sm space-y-1">
          <div>{ctx?.value?.about.name}</div>
          <div>Version: {ctx?.value?.about.version}</div>
          <div>Author: {ctx?.value?.about.author}</div>
          <div>Date of Release: {ctx?.value?.about.date_of_release}</div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
