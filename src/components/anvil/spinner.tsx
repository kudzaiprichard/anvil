import { cn } from "@/src/lib/utils";

/** Small circular spinner used for running/loading states. */
export function Spinner({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "inline-block size-3.5 animate-spin rounded-full border-2 border-border border-t-primary",
        className
      )}
      role="status"
      aria-label="Loading"
    />
  );
}
