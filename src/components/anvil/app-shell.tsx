"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import {
  GraduationCap,
  Hammer,
  LayoutDashboard,
  Library,
  RotateCcw,
  Settings,
} from "lucide-react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useTheme } from "next-themes";
import { toast } from "sonner";
import { SettingsModal } from "@/src/components/settings/settings-modal";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/src/components/shadcn/tooltip";
import {
  getEditorPrefs,
  setEditorPrefs,
  useEditorPrefs,
  WORKSPACE_LAYOUTS,
} from "@/src/lib/settings";
import { cn } from "@/src/lib/utils";
import { CommandPalette } from "./command-palette";
import { AnvilMark } from "./logo";
import { ThemeToggle } from "./theme-toggle";
import { TitleBar } from "./title-bar";
import { useAppShortcuts } from "./use-app-shortcuts";

const NAV = [
  { href: "/", label: "Dashboard", icon: LayoutDashboard, match: ["/"] },
  { href: "/learn", label: "Course", icon: GraduationCap, match: ["/learn"] },
  { href: "/review", label: "Review", icon: RotateCcw, match: ["/review"] },
  {
    href: "/problems",
    label: "Library",
    icon: Library,
    match: ["/problems", "/problem"],
  },
  { href: "/create", label: "Forge a problem", icon: Hammer, match: ["/create"] },
] as const;

function RailButton({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>{children}</TooltipTrigger>
      <TooltipContent side="right" sideOffset={6}>
        {label}
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * Persistent desktop chrome: a 52px icon rail on the left (logo, navigation,
 * theme, settings) and a one-line status bar along the bottom. Every screen
 * renders inside this frame so the app keeps one continuous window identity
 * instead of swapping website-style headers.
 */
export function AppShell({
  children,
  status,
}: {
  children: React.ReactNode;
  /** Right-hand segment of the status bar (page-specific readout). */
  status?: React.ReactNode;
}) {
  const pathname = usePathname();
  const router = useRouter();
  const { resolvedTheme, setTheme } = useTheme();
  const prefs = useEditorPrefs();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);

  // Re-apply the persisted WebView zoom once per window.
  useEffect(() => {
    if (!isTauri()) return;
    const zoom = getEditorPrefs().uiZoom;
    if (zoom !== 1) void getCurrentWebview().setZoom(zoom);
  }, []);

  const handleZoom = useCallback((delta: 1 | -1 | 0) => {
    if (!isTauri()) return; // the browser has its own zoom
    const current = getEditorPrefs().uiZoom;
    const next =
      delta === 0
        ? 1
        : Math.min(1.5, Math.max(0.7, Math.round((current + delta * 0.1) * 10) / 10));
    setEditorPrefs({ uiZoom: next });
    void getCurrentWebview().setZoom(next);
    toast(`UI zoom ${Math.round(next * 100)}%`);
  }, []);

  useAppShortcuts({
    onOpenPalette: () => setPaletteOpen((open) => !open),
    onOpenSettings: () => setSettingsOpen(true),
    onCycleLayout: () => {
      const i = WORKSPACE_LAYOUTS.findIndex(
        (l) => l.id === prefs.workspaceLayout
      );
      const next = WORKSPACE_LAYOUTS[(i + 1) % WORKSPACE_LAYOUTS.length];
      setEditorPrefs({ workspaceLayout: next.id });
      toast(`Workspace layout: ${next.label}`);
    },
    onToggleTheme: () =>
      setTheme(resolvedTheme === "dark" ? "light" : "dark"),
    onNavigate: (i) => router.push(NAV[i].href),
    onZoom: handleZoom,
  });

  return (
    <TooltipProvider delayDuration={350}>
      <div className="flex h-dvh flex-col overflow-hidden">
        <TitleBar />
        <div className="flex min-h-0 flex-1">
          {/* icon rail */}
        <aside className="flex w-[52px] shrink-0 flex-col items-center border-r bg-sidebar pb-2.5 pt-3">
          <Link
            href="/"
            aria-label="Anvil — dashboard"
            className="flex size-[30px] items-center justify-center rounded-[8px] bg-gradient-to-b from-primary to-primary/80 text-primary-foreground shadow-[inset_0_1px_0_rgb(255_255_255/0.25)] transition-transform hover:scale-105 active:scale-95"
          >
            <AnvilMark className="size-[19px]" />
          </Link>

          <nav className="mt-4 flex flex-col items-center gap-1">
            {NAV.map(({ href, label, icon: Icon, match }) => {
              const active = (match as readonly string[]).includes(pathname);
              return (
                <RailButton key={href} label={label}>
                  <Link
                    href={href}
                    aria-label={label}
                    aria-current={active ? "page" : undefined}
                    className={cn(
                      "relative flex size-9 items-center justify-center rounded-lg transition-colors",
                      active
                        ? "bg-sidebar-accent text-primary"
                        : "text-muted-foreground hover:bg-sidebar-accent hover:text-foreground"
                    )}
                  >
                    <Icon className="size-[18px]" strokeWidth={active ? 2.1 : 1.8} />
                    {active && (
                      <span className="absolute -left-[7px] h-[18px] w-[2.5px] rounded-r-full bg-primary" />
                    )}
                  </Link>
                </RailButton>
              );
            })}
          </nav>

          <div className="flex-1" />

          <div className="flex flex-col items-center gap-1">
            <RailButton label="Toggle theme">
              <span>
                <ThemeToggle variant="ghost" className="size-9 rounded-lg" />
              </span>
            </RailButton>
            <RailButton label="Settings">
              <button
                type="button"
                aria-label="Settings"
                onClick={() => setSettingsOpen(true)}
                className="flex size-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-foreground"
              >
                <Settings className="size-[18px]" strokeWidth={1.8} />
              </button>
            </RailButton>
          </div>
        </aside>

        {/* content + status bar */}
        <div className="flex min-w-0 flex-1 flex-col">
          <div className="flex min-h-0 flex-1 flex-col">{children}</div>
          <footer className="flex h-[25px] shrink-0 items-center gap-4 border-t bg-sidebar px-3 font-mono text-[10.5px] text-muted-foreground">
            <span className="flex items-center gap-1.5">
              <span className="size-[6px] rounded-full bg-pass" />
              local runner ready
            </span>
            <span className="hidden sm:inline">offline · no account</span>
            <span className="flex-1" />
            {status}
            <span className="opacity-70">anvil v0.1.0</span>
          </footer>
          </div>
        </div>
      </div>
      <SettingsModal open={settingsOpen} onOpenChange={setSettingsOpen} />
      <CommandPalette
        open={paletteOpen}
        onOpenChange={setPaletteOpen}
        onOpenSettings={() => setSettingsOpen(true)}
      />
    </TooltipProvider>
  );
}
