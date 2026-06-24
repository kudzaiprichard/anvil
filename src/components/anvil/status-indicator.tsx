import { Check, Flag } from "lucide-react";
import { cn } from "@/src/lib/utils";
import type { ProblemStatus } from "@/src/lib/types";

/**
 * Problem status mark used in lists: solved ✓ (emerald), in-progress dot
 * (amber), needs-review flag, todo hollow dot.
 */
export function StatusIndicator({
  status,
  className,
}: {
  status: ProblemStatus;
  className?: string;
}) {
  switch (status) {
    case "solved":
      return (
        <Check
          className={cn("size-3.5 stroke-[2.6] text-easy", className)}
          aria-label="Solved"
        />
      );
    case "in-progress":
      return (
        <span
          className={cn("block size-[7px] rounded-full bg-medium", className)}
          aria-label="In progress"
        />
      );
    case "needs-review":
      return (
        <Flag
          className={cn("size-3 text-medium", className)}
          aria-label="Needs review"
        />
      );
    default:
      return (
        <span
          className={cn(
            "block size-1.5 rounded-full border-[1.5px] border-muted-foreground opacity-50",
            className
          )}
          aria-label="Todo"
        />
      );
  }
}
