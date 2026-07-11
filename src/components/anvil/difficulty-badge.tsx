import { cn } from "@/src/lib/utils";
import type { Difficulty } from "@/src/lib/types";

const COLOR: Record<Difficulty, string> = {
  Easy: "text-easy",
  Medium: "text-medium",
  Hard: "text-hard",
};

const SOFT_BG: Record<Difficulty, string> = {
  Easy: "bg-easy/10 dark:bg-easy/15",
  Medium: "bg-medium/10 dark:bg-medium/15",
  Hard: "bg-hard/10 dark:bg-hard/15",
};

/**
 * Difficulty label per UI_SPEC §3: Easy=emerald, Medium=amber, Hard=rose.
 * `variant="text"` renders the plain colored word (list rows);
 * `variant="pill"` renders the dot + soft pill (problem header, resume row).
 */
export function DifficultyBadge({
  difficulty,
  variant = "text",
  className,
}: {
  difficulty: Difficulty;
  variant?: "text" | "pill";
  className?: string;
}) {
  if (variant === "text") {
    return (
      <span className={cn("text-xs font-semibold", COLOR[difficulty], className)}>
        {difficulty}
      </span>
    );
  }
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold",
        COLOR[difficulty],
        SOFT_BG[difficulty],
        className
      )}
    >
      <span className="size-1.5 rounded-full bg-current" />
      {difficulty}
    </span>
  );
}
