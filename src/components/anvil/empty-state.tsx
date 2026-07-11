import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/src/lib/utils";

/** Centered empty state: icon tile, title, hint, optional action. */
export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  className,
}: {
  icon: LucideIcon;
  title: string;
  description?: ReactNode;
  action?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex min-h-[300px] flex-col items-center justify-center gap-3.5 text-center",
        className
      )}
    >
      <div className="flex size-[52px] items-center justify-center rounded-2xl bg-muted text-muted-foreground">
        <Icon className="size-6 stroke-[1.8]" />
      </div>
      <div>
        <div className="text-[15px] font-semibold">{title}</div>
        {description && (
          <div className="mx-auto mt-1.5 max-w-[340px] text-[13px] leading-relaxed text-muted-foreground">
            {description}
          </div>
        )}
      </div>
      {action}
    </div>
  );
}
