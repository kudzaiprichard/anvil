import Link from "next/link";
import { cn } from "@/src/lib/utils";

/** Anvil silhouette glyph (original geometry, reads at 16–24px). */
export function AnvilMark({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
      className={className}
    >
      {/* face + horn */}
      <path d="M3.2 5.6h15.2c1.8 0 3.3 1 4.4 2.1.4.4.1 1.1-.5 1.1h-4.1v1.1c0 .7.4 1.3 1 1.6l.9.4c.5.2.5 1 0 1.2-2.3.9-4.8 1-7.1.3-1.6-.5-3.3-.5-4.9 0-1.1.3-2.3.5-3.4.5-.6 0-.9-.7-.5-1.1l1.5-1.4c.5-.5.8-1.1.8-1.8v-.8H3.2c-.6 0-1-.4-1-1V6.6c0-.6.4-1 1-1z" />
      {/* waist */}
      <path d="M9.4 14.9h5.2l1 2.3H8.4z" />
      {/* base */}
      <path d="M6.5 18.2h11c.6 0 1 .4 1 1v.6c0 .6-.4 1-1 1h-11c-.6 0-1-.4-1-1v-.6c0-.6.4-1 1-1z" />
    </svg>
  );
}

/** The Anvil mark on an ember tile + optional wordmark. Links to the dashboard. */
export function Logo({
  withWordmark = true,
  size = "md",
  className,
}: {
  withWordmark?: boolean;
  size?: "sm" | "md";
  className?: string;
}) {
  return (
    <Link
      href="/"
      className={cn("flex items-center gap-2", className)}
      aria-label="Anvil — dashboard"
    >
      <span
        className={cn(
          "flex items-center justify-center rounded-[6px] text-primary-foreground",
          "bg-gradient-to-b from-primary to-primary/80 shadow-[inset_0_1px_0_rgb(255_255_255/0.25)]",
          size === "sm" ? "size-[22px]" : "size-6"
        )}
      >
        <AnvilMark className={size === "sm" ? "size-[15px]" : "size-4"} />
      </span>
      {withWordmark && (
        <span className="text-[15px] font-semibold tracking-tight">Anvil</span>
      )}
    </Link>
  );
}
