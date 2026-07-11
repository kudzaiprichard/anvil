"use client";

import { useSyncExternalStore } from "react";
import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { cn } from "@/src/lib/utils";

// Hydration-safe "has mounted" check (false on the server, true on the client).
const emptySubscribe = () => () => {};
function useMounted() {
  return useSyncExternalStore(
    emptySubscribe,
    () => true,
    () => false
  );
}

/**
 * Icon button that flips light/dark (System is offered in the settings
 * modal). Ghost variant matches the workspace top bar; outline matches the
 * page header in the mockups.
 */
export function ThemeToggle({
  variant = "outline",
  className,
}: {
  variant?: "outline" | "ghost";
  className?: string;
}) {
  const { resolvedTheme, setTheme } = useTheme();
  const mounted = useMounted();

  return (
    <button
      type="button"
      title="Toggle theme"
      aria-label="Toggle theme"
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
      className={cn(
        "flex items-center justify-center rounded-[9px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
        variant === "outline"
          ? "size-[34px] border bg-card"
          : "size-8 rounded-lg",
        className
      )}
    >
      {mounted && resolvedTheme === "dark" ? (
        <Sun className="size-4" />
      ) : (
        <Moon className="size-4" />
      )}
    </button>
  );
}
