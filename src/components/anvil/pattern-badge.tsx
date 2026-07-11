import { cn } from "@/src/lib/utils";
import type { Pattern } from "@/src/lib/types";

/** Neutral pattern pill (problem header, cards). */
export function PatternBadge({
  pattern,
  className,
}: {
  pattern: Pattern;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-md bg-muted px-2.5 py-1 text-xs font-medium text-muted-foreground",
        className
      )}
    >
      {pattern}
    </span>
  );
}
