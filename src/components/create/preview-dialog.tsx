"use client";

import { useMemo, useState } from "react";
import { Eye } from "lucide-react";
import { CodeEditor } from "@/src/components/anvil/code-editor";
import { ProblemPane } from "@/src/components/workspace/problem-pane";
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from "@/src/components/shadcn/dialog";
import { cn } from "@/src/lib/utils";
import type {
  Language,
  Pattern,
  Problem,
  TestCase,
  UserProblemDraft,
} from "@/src/lib/types";
import { LANGUAGE_LABELS } from "@/src/lib/types";

/**
 * Maps the in-progress draft to a displayable `Problem` so the preview can
 * reuse the real workspace components. Never persisted: `number: 0` hides
 * the list number and `source: "built-in"` hides the export button (there's
 * nothing on disk to export). Unparsable test rows are skipped — the
 * verification step is where they get flagged.
 */
export function draftToPreviewProblem(draft: UserProblemDraft): Problem {
  const test_cases: TestCase[] = [];
  for (const tc of draft.test_cases) {
    try {
      const input: unknown = JSON.parse(tc.input);
      if (!Array.isArray(input)) continue;
      test_cases.push({
        input,
        expected: tc.expected.trim() ? JSON.parse(tc.expected) : null,
        hidden: tc.hidden,
      });
    } catch {
      // invalid JSON — preview what we can
    }
  }
  const hasSolution = Boolean(
    draft.reference_solution.python?.trim() ||
      draft.reference_solution.javascript?.trim()
  );
  return {
    id: "forge-preview",
    number: 0,
    title: draft.title.trim() || "Untitled problem",
    pattern: (draft.pattern || "Arrays & Hashing") as Pattern,
    difficulty: draft.difficulty,
    source: "built-in",
    description_md: draft.description_md || "*No statement yet.*",
    constraints: draft.constraints.filter((c) => c.trim()),
    examples: draft.examples.filter((ex) => ex.input.trim() || ex.output.trim()),
    function_signature: { ...draft.function_signature },
    test_cases,
    hints: draft.hints.filter((h) => h.trim()),
    reference_solution: hasSolution ? draft.reference_solution : undefined,
    follow_up: draft.follow_up?.trim() || undefined,
    license: "preview",
    author: "you",
  };
}

/**
 * "Open in workspace" preview for the Forge page: the real ProblemPane
 * (Description / Hints / Solution tabs) beside a live starter-code editor,
 * laid out like the classic workspace. Nothing is saved or runnable here —
 * it exists to answer "is this what I want solvers to see?".
 */
export function PreviewDialog({
  open,
  onOpenChange,
  draft,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  draft: UserProblemDraft;
}) {
  const problem = useMemo(() => draftToPreviewProblem(draft), [draft]);
  const [lang, setLang] = useState<Language>("python");
  const [codeByLang, setCodeByLang] = useState(draft.function_signature);

  // Re-seed the scratch editor each time the preview opens.
  const [prevOpen, setPrevOpen] = useState(open);
  if (open !== prevOpen) {
    setPrevOpen(open);
    if (open) setCodeByLang({ ...draft.function_signature });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        aria-describedby={undefined}
        className="top-[5%] flex h-[88vh] w-[min(1180px,94vw)] max-w-[1180px] translate-y-0 flex-col gap-0 overflow-hidden rounded-xl p-0 sm:max-w-[1180px]"
      >
        <div className="flex shrink-0 items-center gap-2.5 border-b bg-sidebar px-4 py-[11px]">
          <Eye className="size-4 text-primary" />
          <DialogTitle className="text-[14px] font-semibold tracking-tight">
            Workspace preview
          </DialogTitle>
          <span className="text-xs text-muted-foreground">
            exactly what solvers will see — nothing here is saved, and Run
            unlocks after publishing
          </span>
        </div>

        <div className="flex min-h-0 flex-1">
          {/* problem pane — the real workspace component */}
          <div className="w-[46%] min-w-0 shrink-0 border-r bg-card">
            <ProblemPane problem={problem} />
          </div>

          {/* editor pane */}
          <div className="flex min-w-0 flex-1 flex-col">
            <div className="flex h-9 shrink-0 items-center gap-0.5 border-b bg-card px-2">
              {(["python", "javascript"] as const).map((l) => (
                <button
                  key={l}
                  type="button"
                  onClick={() => setLang(l)}
                  className={cn(
                    "rounded-lg px-3 py-1.5 text-[12.5px] transition-colors",
                    l === lang
                      ? "bg-primary/10 font-semibold text-primary dark:bg-primary/20"
                      : "font-medium text-muted-foreground hover:text-foreground"
                  )}
                >
                  {LANGUAGE_LABELS[l]}
                </button>
              ))}
              <span className="ml-auto pr-1 font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground">
                scratch editor
              </span>
            </div>
            <div className="min-h-0 flex-1">
              <CodeEditor
                value={codeByLang[lang]}
                language={lang}
                onChange={(code) =>
                  setCodeByLang((prev) =>
                    prev[lang] === code ? prev : { ...prev, [lang]: code }
                  )
                }
                docKey={`preview-${lang}`}
              />
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
