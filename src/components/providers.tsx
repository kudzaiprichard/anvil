"use client";

import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ThemeProvider } from "next-themes";
import { useEffect, type ReactNode } from "react";

export function Providers({ children }: { children: ReactNode }) {
  // The window is configured hidden (`visible: false`) — reveal it only
  // after React has painted a themed frame, so launch never flashes a white
  // WebView. Double rAF = after the first real paint.
  useEffect(() => {
    if (!isTauri()) return;
    let raf2 = 0;
    const raf1 = requestAnimationFrame(() => {
      raf2 = requestAnimationFrame(() => {
        const win = getCurrentWindow();
        void win.show();
        void win.setFocus();
      });
    });
    return () => {
      cancelAnimationFrame(raf1);
      cancelAnimationFrame(raf2);
    };
  }, []);
  // Desktop feel: inside Tauri, suppress the WebView's browser context menu
  // on chrome. Text surfaces keep it — copy/paste there is genuinely useful.
  useEffect(() => {
    if (!isTauri()) return;
    const onContextMenu = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (
        target?.closest(
          'input, textarea, [contenteditable="true"], .anvil-prose, pre, code, .select-text'
        )
      ) {
        return;
      }
      e.preventDefault();
    };
    document.addEventListener("contextmenu", onContextMenu);
    return () => document.removeEventListener("contextmenu", onContextMenu);
  }, []);

  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange
    >
      {children}
    </ThemeProvider>
  );
}
