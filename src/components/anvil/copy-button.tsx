"use client";

import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/src/lib/utils";

/** Writes text to the clipboard. Prefers the async Clipboard API (available in
 *  the Tauri webview's secure context and modern browsers) and falls back to a
 *  hidden-textarea `execCommand("copy")` where it isn't. Returns whether it
 *  landed so the caller can toast success vs. failure. */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // fall through to the legacy path
  }
  try {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.style.position = "fixed";
    ta.style.top = "-9999px";
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}

/**
 * A copy-to-clipboard button that flips to a check for a beat and toasts the
 * outcome. `getText` is called on click so callers can copy live state (e.g.
 * the currently-selected solution language). Pass `label` for a text+icon
 * button, or omit it for an icon-only square.
 */
export function CopyButton({
  getText,
  toastMessage,
  title,
  label,
  copiedLabel = "Copied",
  className,
  iconClassName = "size-[15px]",
}: {
  getText: () => string;
  /** Toast shown on success, e.g. "Question copied". */
  toastMessage: string;
  /** Tooltip / accessible label. */
  title: string;
  /** Optional visible text; when set the button shows an icon + label. */
  label?: string;
  copiedLabel?: string;
  className?: string;
  iconClassName?: string;
}) {
  const [copied, setCopied] = useState(false);

  const onClick = async () => {
    const text = getText();
    if (!text) {
      toast.error("Nothing to copy");
      return;
    }
    if (await copyToClipboard(text)) {
      setCopied(true);
      toast.success(toastMessage);
      window.setTimeout(() => setCopied(false), 1400);
    } else {
      toast.error("Couldn't copy to clipboard");
    }
  };

  const Icon = copied ? Check : Copy;
  return (
    <button
      type="button"
      title={title}
      aria-label={title}
      onClick={onClick}
      className={className}
    >
      <Icon className={cn(iconClassName, copied && "text-primary")} />
      {label != null && <span>{copied ? copiedLabel : label}</span>}
    </button>
  );
}
