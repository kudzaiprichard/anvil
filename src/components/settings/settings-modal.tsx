"use client";

import { useCallback, useEffect, useState } from "react";
import {
  Check,
  Code2,
  Keyboard,
  RefreshCw,
  SunMedium,
  TerminalSquare,
} from "lucide-react";
import { useTheme } from "next-themes";
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from "@/src/components/shadcn/dialog";
import { Switch } from "@/src/components/shadcn/switch";
import { Spinner } from "@/src/components/anvil/spinner";
import { detectRuntimes } from "@/src/lib/api";
import type { RuntimeInfo } from "@/src/lib/types";
import {
  setEditorPrefs,
  useEditorPrefs,
  WORKSPACE_LAYOUTS,
  type WorkspaceLayout,
} from "@/src/lib/settings";
import { cn } from "@/src/lib/utils";

type Pane = "appearance" | "editor" | "runtime" | "shortcuts";

const NAV: { id: Pane; label: string; icon: React.ElementType }[] = [
  { id: "appearance", label: "Appearance", icon: SunMedium },
  { id: "editor", label: "Editor", icon: Code2 },
  { id: "runtime", label: "Runtime", icon: TerminalSquare },
  { id: "shortcuts", label: "Shortcuts", icon: Keyboard },
];

function PaneTitle({ title, sub }: { title: string; sub: string }) {
  return (
    <>
      <div className="text-[15px] font-semibold">{title}</div>
      <div className="mt-1 text-[13px] text-muted-foreground">{sub}</div>
    </>
  );
}

function ThemeCard({
  label,
  active,
  onClick,
  preview,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
  preview: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "rounded-[11px] border bg-card p-2.5 text-left transition-shadow",
        active && "border-primary shadow-[0_0_0_1px_var(--primary)]"
      )}
    >
      {preview}
      <span className="mt-2 flex items-center gap-[7px] text-[13px] font-medium">
        <span
          className={cn(
            "inline-block size-[15px] shrink-0 rounded-full",
            active
              ? "border-4 border-primary"
              : "border-[1.5px] border-muted-foreground"
          )}
        />
        {label}
      </span>
    </button>
  );
}

/** Miniature pane map: problem = light, editor = ember, results = mid. */
function LayoutDiagram({ id }: { id: WorkspaceLayout }) {
  const problem = "rounded-[3px] bg-muted-foreground/25";
  const editor = "rounded-[3px] bg-primary/75";
  const results = "rounded-[3px] bg-muted-foreground/45";
  return (
    <span className="flex h-[46px] gap-[3px] rounded-[7px] border bg-background p-1.5">
      {id === "classic" && (
        <>
          <span className={cn(problem, "w-1/3")} />
          <span className="flex flex-1 flex-col gap-[3px]">
            <span className={cn(editor, "flex-1")} />
            <span className={cn(results, "h-1/3")} />
          </span>
        </>
      )}
      {id === "mirrored" && (
        <>
          <span className="flex flex-1 flex-col gap-[3px]">
            <span className={cn(editor, "flex-1")} />
            <span className={cn(results, "h-1/3")} />
          </span>
          <span className={cn(problem, "w-1/3")} />
        </>
      )}
      {id === "bottom" && (
        <span className="flex flex-1 flex-col gap-[3px]">
          <span className="flex flex-1 gap-[3px]">
            <span className={cn(problem, "w-2/5")} />
            <span className={cn(editor, "flex-1")} />
          </span>
          <span className={cn(results, "h-1/3")} />
        </span>
      )}
      {id === "columns" && (
        <>
          <span className={cn(problem, "w-[30%]")} />
          <span className={cn(editor, "flex-1")} />
          <span className={cn(results, "w-[26%]")} />
        </>
      )}
      {id === "editor-deck" && (
        <span className="flex flex-1 flex-col gap-[3px]">
          <span className={cn(editor, "flex-1")} />
          <span className="flex h-2/5 gap-[3px]">
            <span className={cn(problem, "w-1/2")} />
            <span className={cn(results, "flex-1")} />
          </span>
        </span>
      )}
    </span>
  );
}

function AppearancePane() {
  const { theme, setTheme } = useTheme();
  const prefs = useEditorPrefs();
  const swatch = (bg: string, bar: string, bar2?: string) => (
    <span
      className="flex h-[46px] items-end gap-[3px] rounded-[7px] border p-1.5"
      style={{ background: bg }}
    >
      <span className="h-[5px] w-3.5 rounded-sm" style={{ background: bar }} />
      {bar2 && (
        <span
          className="h-[5px] w-[22px] rounded-sm"
          style={{ background: bar2 }}
        />
      )}
    </span>
  );
  return (
    <div>
      <PaneTitle
        title="Appearance"
        sub="Choose how Anvil looks. System follows your OS setting."
      />
      <div className="mt-[18px] text-[13px] font-semibold">Theme</div>
      <div className="mt-2.5 grid grid-cols-3 gap-2.5">
        <ThemeCard
          label="Light"
          active={theme === "light"}
          onClick={() => setTheme("light")}
          preview={swatch("#faf8f5", "#c25a18", "#e7e2da")}
        />
        <ThemeCard
          label="Dark"
          active={theme === "dark"}
          onClick={() => setTheme("dark")}
          preview={swatch("#211e1b", "#e8915a", "#3a3531")}
        />
        <ThemeCard
          label="System"
          active={theme === "system" || theme === undefined}
          onClick={() => setTheme("system")}
          preview={swatch(
            "linear-gradient(105deg,#faf8f5 50%,#211e1b 50%)",
            "#c25a18"
          )}
        />
      </div>
      <div className="mt-[22px] text-[13px] font-semibold">
        Workspace layout
      </div>
      <p className="mt-0.5 text-xs text-muted-foreground">
        How the problem, code, and results panes are arranged.
      </p>
      <div className="mt-2.5 grid grid-cols-3 gap-2.5">
        {WORKSPACE_LAYOUTS.map(({ id, label }) => (
          <ThemeCard
            key={id}
            label={label}
            active={prefs.workspaceLayout === id}
            onClick={() => setEditorPrefs({ workspaceLayout: id })}
            preview={<LayoutDiagram id={id} />}
          />
        ))}
      </div>
      <div className="mt-[22px] flex items-center justify-between border-t py-[13px]">
        <div>
          <div className="text-[13px] font-semibold">Practice timer</div>
          <div className="mt-px text-xs text-muted-foreground">
            Per-problem stopwatch; stops and records when all tests pass.
          </div>
        </div>
        <Switch
          checked={prefs.showTimer}
          onCheckedChange={(showTimer) => setEditorPrefs({ showTimer })}
        />
      </div>
      <div
        className={cn(
          "flex items-center justify-between border-t py-[13px]",
          !prefs.showTimer && "opacity-50"
        )}
      >
        <div>
          <div className="text-[13px] font-semibold">Auto-start timer</div>
          <div className="mt-px text-xs text-muted-foreground">
            Start the clock when a problem opens; off = press play yourself.
          </div>
        </div>
        <Switch
          disabled={!prefs.showTimer}
          checked={prefs.timerAutoStart}
          onCheckedChange={(timerAutoStart) => setEditorPrefs({ timerAutoStart })}
        />
      </div>
      <div className="flex items-center justify-between border-t py-[13px]">
        <div>
          <div className="text-[13px] font-semibold">Reduce motion</div>
          <div className="mt-px text-xs text-muted-foreground">
            Minimize transitions and the pass celebration.
          </div>
        </div>
        <Switch
          checked={prefs.reduceMotion}
          onCheckedChange={(reduceMotion) => setEditorPrefs({ reduceMotion })}
        />
      </div>
    </div>
  );
}

function EditorPane() {
  const prefs = useEditorPrefs();
  const segBtn = (active: boolean) =>
    cn(
      "flex-1 rounded-[7px] py-1.5 text-center text-[12.5px] transition-colors",
      active
        ? "bg-primary/10 font-semibold text-primary dark:bg-primary/20"
        : "font-medium text-muted-foreground hover:text-foreground"
    );
  return (
    <div>
      <PaneTitle
        title="Editor"
        sub="Preferences applied to the workspace code editor."
      />
      <div className="mt-[18px] flex flex-col gap-5">
        <div>
          <div className="text-[13px] font-semibold">Font size</div>
          <div className="mt-2 flex w-[280px] gap-1.5 rounded-[9px] border bg-editor p-[3px]">
            {[12, 13, 14, 16].map((size) => (
              <button
                key={size}
                type="button"
                onClick={() => setEditorPrefs({ fontSize: size })}
                className={segBtn(prefs.fontSize === size)}
              >
                {size}px
              </button>
            ))}
          </div>
        </div>
        <div>
          <div className="text-[13px] font-semibold">Tab size</div>
          <div className="mt-2 flex w-[180px] gap-1.5 rounded-[9px] border bg-editor p-[3px]">
            {([2, 4] as const).map((size) => (
              <button
                key={size}
                type="button"
                onClick={() => setEditorPrefs({ tabSize: size })}
                className={segBtn(prefs.tabSize === size)}
              >
                {size} spaces
              </button>
            ))}
          </div>
        </div>
        <div className="flex items-center justify-between border-t py-[13px]">
          <div>
            <div className="text-[13px] font-semibold">Line wrap</div>
            <div className="mt-px text-xs text-muted-foreground">
              Wrap long lines instead of scrolling horizontally.
            </div>
          </div>
          <Switch
            checked={prefs.lineWrap}
            onCheckedChange={(lineWrap) => setEditorPrefs({ lineWrap })}
          />
        </div>
      </div>
    </div>
  );
}

function RuntimePane() {
  const [runtimes, setRuntimes] = useState<RuntimeInfo[] | null>(null);
  const [probing, setProbing] = useState(false);

  useEffect(() => {
    let cancelled = false;
    detectRuntimes().then((detected) => {
      if (!cancelled) setRuntimes(detected);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const probe = useCallback(() => {
    setProbing(true);
    detectRuntimes()
      .then(setRuntimes)
      .finally(() => setProbing(false));
  }, []);

  return (
    <div>
      <PaneTitle
        title="Runtime"
        sub="Anvil runs your code locally with these detected interpreters."
      />
      <div className="mt-[18px] flex flex-col gap-2.5">
        {runtimes === null && (
          <div className="flex items-center gap-2.5 rounded-[11px] border bg-sidebar p-3.5 text-[13px] text-muted-foreground">
            <Spinner className="size-[13px]" />
            Probing for interpreters…
          </div>
        )}
        {(runtimes ?? []).map((rt) => (
          <div
            key={rt.name}
            className="flex items-center gap-[13px] rounded-[11px] border bg-sidebar p-3.5"
          >
            <span className="flex size-9 shrink-0 items-center justify-center rounded-[9px] bg-editor font-mono text-[13px] font-semibold text-muted-foreground">
              {rt.tag}
            </span>
            <div className="min-w-0 flex-1">
              <div className="text-[13.5px] font-semibold">{rt.name}</div>
              <div className="mt-px truncate font-mono text-xs text-muted-foreground">
                {rt.path || "not on PATH"}
              </div>
            </div>
            {rt.found ? (
              <span className="inline-flex items-center gap-1.5 rounded-full bg-pass/10 px-2.5 py-1 text-xs font-semibold text-pass dark:bg-pass/15">
                <Check className="size-3 stroke-[2.6]" />
                Found · {rt.version}
              </span>
            ) : (
              <span className="inline-flex items-center rounded-full bg-fail/10 px-2.5 py-1 text-xs font-semibold text-fail dark:bg-fail/15">
                Not found
              </span>
            )}
          </div>
        ))}
      </div>
      <div className="mt-3 flex justify-end">
        <button
          type="button"
          onClick={probe}
          disabled={probing}
          className="flex items-center gap-1.5 rounded-lg border bg-card px-[11px] py-1.5 text-xs font-semibold transition-colors hover:bg-muted disabled:opacity-50"
        >
          <RefreshCw className={cn("size-[13px]", probing && "animate-spin")} />
          Re-detect
        </button>
      </div>
      <div className="mt-3.5 flex items-start gap-[9px] rounded-[10px] border bg-primary/5 px-[13px] py-3 dark:bg-primary/10">
        <TerminalSquare className="mt-px size-[15px] shrink-0 text-primary" />
        <span className="text-[12.5px] leading-relaxed text-muted-foreground">
          No interpreter? Anvil can fall back to a{" "}
          <span className="font-medium text-foreground">bundled runtime</span>{" "}
          so practice always works offline.
        </span>
      </div>
    </div>
  );
}

const SHORTCUT_GROUPS: {
  title: string;
  items: { action: string; keys: string[] }[];
}[] = [
  {
    title: "Everywhere",
    items: [
      { action: "Command palette", keys: ["Ctrl", "K"] },
      { action: "Open settings", keys: ["Ctrl", ","] },
      { action: "Toggle light / dark theme", keys: ["Ctrl", "Shift", "T"] },
      { action: "Cycle workspace layout", keys: ["Ctrl", "Shift", "L"] },
      { action: "Go to Dashboard / Library / Forge", keys: ["Ctrl", "1–3"] },
      { action: "Zoom in / out / reset", keys: ["Ctrl", "+ − 0"] },
    ],
  },
  {
    title: "Library",
    items: [
      { action: "Focus search", keys: ["/"] },
      { action: "Move selection", keys: ["↑", "↓"] },
      { action: "Open selected problem", keys: ["Enter"] },
    ],
  },
  {
    title: "Workspace",
    items: [
      { action: "Run all tests", keys: ["Ctrl", "Enter"] },
      { action: "Previous problem", keys: ["Ctrl", "["] },
      { action: "Next problem", keys: ["Ctrl", "]"] },
      { action: "Problem list", keys: ["Ctrl", "P"] },
      { action: "Maximize / restore editor", keys: ["Ctrl", "Shift", "M"] },
      { action: "Reset to starter code", keys: ["Ctrl", "Alt", "R"] },
    ],
  },
  {
    title: "Editor",
    items: [
      { action: "Indent / outdent selection", keys: ["Tab", "Shift+Tab"] },
      { action: "Toggle line comment", keys: ["Ctrl", "/"] },
      { action: "Move line up / down", keys: ["Alt", "↑ ↓"] },
      { action: "Duplicate line up / down", keys: ["Shift", "Alt", "↑ ↓"] },
      { action: "Select next occurrence (multi-cursor)", keys: ["Ctrl", "D"] },
      { action: "Add cursor", keys: ["Alt", "Click"] },
      { action: "Delete line", keys: ["Ctrl", "Shift", "K"] },
      { action: "Find / find & replace", keys: ["Ctrl", "F"] },
      { action: "Select all occurrences of selection", keys: ["Ctrl", "Shift", "L"] },
      { action: "Undo / redo", keys: ["Ctrl", "Z / Y"] },
    ],
  },
];

function ShortcutsPane() {
  return (
    <div>
      <PaneTitle title="Shortcuts" sub="Key bindings (not editable yet)." />
      {SHORTCUT_GROUPS.map((group) => (
        <div key={group.title} className="mt-[18px]">
          <div className="microlabel">{group.title}</div>
          <div className="mt-1 flex flex-col">
            {group.items.map((sc) => (
              <div
                key={sc.action}
                className="flex items-center justify-between border-b py-2.5 text-[13px] last:border-b-0"
              >
                <span>{sc.action}</span>
                <span className="flex items-center gap-1">
                  {sc.keys.map((k) => (
                    <kbd
                      key={k}
                      className="rounded-md border bg-muted px-1.5 py-0.5 font-mono text-[11px] font-medium text-muted-foreground"
                    >
                      {k}
                    </kbd>
                  ))}
                </span>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

/** Centered settings modal (UI_SPEC §7): left nav + right content pane. */
export function SettingsModal({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [pane, setPane] = useState<Pane>("appearance");

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        aria-describedby={undefined}
        className="flex h-[480px] w-[740px] max-w-[740px] flex-col gap-0 overflow-hidden rounded-2xl p-0 sm:max-w-[740px]"
      >
        <div className="flex shrink-0 items-center border-b px-[18px] py-[15px]">
          <DialogTitle className="text-[15px] font-semibold tracking-tight">
            Settings
          </DialogTitle>
        </div>
        <div className="flex min-h-0 flex-1">
          <nav className="flex w-[190px] shrink-0 flex-col gap-0.5 border-r bg-sidebar p-3">
            {NAV.map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                type="button"
                onClick={() => setPane(id)}
                className={cn(
                  "flex items-center gap-[9px] rounded-lg px-2.5 py-2 text-[13px] transition-colors",
                  pane === id
                    ? "bg-primary/10 font-semibold text-primary dark:bg-primary/20"
                    : "font-medium text-muted-foreground hover:text-foreground"
                )}
              >
                <Icon className="size-[15px]" />
                {label}
              </button>
            ))}
          </nav>
          <div className="min-w-0 flex-1 overflow-auto px-6 py-[22px]">
            {pane === "appearance" && <AppearancePane />}
            {pane === "editor" && <EditorPane />}
            {pane === "runtime" && <RuntimePane />}
            {pane === "shortcuts" && <ShortcutsPane />}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
