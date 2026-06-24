"use client";

import { Fragment, useState } from "react";
import { Bookmark, ChevronDown, ChevronRight, Eye, FileUp } from "lucide-react";
import { toast } from "sonner";
import { exportProblem } from "@/src/lib/api";
import { CodeEditor } from "@/src/components/anvil/code-editor";
import { DifficultyBadge } from "@/src/components/anvil/difficulty-badge";
import { Markdown } from "@/src/components/anvil/markdown";
import { PatternBadge } from "@/src/components/anvil/pattern-badge";
import { cn } from "@/src/lib/utils";
import type { Language, Problem } from "@/src/lib/types";
import { LANGUAGE_LABELS } from "@/src/lib/types";

type Tab = "description" | "hints" | "solution";

/** For imported problems, signals when a question is running without our
 *  hidden judge (basic) or without any tests (run-only). Full-tier and
 *  built-in/user problems show nothing. */
function TierChip({ problem }: { problem: Problem }) {
  if (problem.source !== "imported") return null;
  const hasHidden = problem.test_cases.some((tc) => tc.hidden);
  if (problem.judge && hasHidden) return null; // full tier — no chip
  const runOnly = problem.test_cases.length === 0;
  return (
    <span
      title={
        runOnly
          ? "No tests could be recovered — runs without a verdict."
          : "Judged against the statement's own examples — no hidden test pack for this question yet."
      }
      className="rounded-full border border-amber-500/40 px-2 py-0.5 text-[10.5px] font-semibold uppercase tracking-wide text-amber-600 dark:text-amber-400"
    >
      {runOnly ? "Run only" : "Basic mode"}
    </span>
  );
}

/** Renders text with `inline code` spans styled like the mockup. */
function InlineCode({ text }: { text: string }) {
  const parts = text.split(/`([^`]*)`/);
  return (
    <>
      {parts.map((part, i) =>
        i % 2 === 1 ? (
          <code
            key={i}
            className="rounded-[5px] bg-muted px-[5px] py-px font-mono text-xs"
          >
            {part}
          </code>
        ) : (
          <Fragment key={i}>{part}</Fragment>
        )
      )}
    </>
  );
}

function DescriptionTab({
  problem,
  bookmarked,
  onToggleBookmark,
}: {
  problem: Problem;
  bookmarked: boolean;
  onToggleBookmark?: () => void;
}) {
  return (
    <>
      <div className="flex items-start justify-between gap-3">
        <h1 className="text-xl font-semibold leading-snug tracking-tight">
          {problem.number}. {problem.title}
        </h1>
        <div className="flex shrink-0 items-center gap-1.5">
          {problem.source !== "built-in" && (
            <button
              type="button"
              title="Export as a shareable problem file"
              onClick={async () => {
                try {
                  if (await exportProblem(problem.id)) {
                    toast.success("Problem exported.");
                  }
                } catch (err) {
                  toast.error(
                    err instanceof Error ? err.message : "Export failed"
                  );
                }
              }}
              className="flex size-[30px] items-center justify-center rounded-lg border bg-card text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
            >
              <FileUp className="size-[15px]" />
            </button>
          )}
          <button
            type="button"
            title={bookmarked ? "Remove bookmark" : "Bookmark"}
            aria-pressed={bookmarked}
            onClick={onToggleBookmark}
            className={cn(
              "flex size-[30px] items-center justify-center rounded-lg border bg-card transition-colors hover:bg-muted",
              bookmarked
                ? "text-primary"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Bookmark
              className={cn("size-[15px]", bookmarked && "fill-primary")}
            />
          </button>
        </div>
      </div>
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <DifficultyBadge difficulty={problem.difficulty} variant="pill" />
        <PatternBadge pattern={problem.pattern} />
        <TierChip problem={problem} />
      </div>

      {/* Imported problems carry a server-sanitized HTML statement (ammonia
          allowlist) for full fidelity; built-in/user problems use markdown +
          structured examples. */}
      {problem.body_html ? (
        <div
          className="mt-[18px] text-[13.5px] leading-relaxed [&_code]:rounded-[5px] [&_code]:bg-muted [&_code]:px-[5px] [&_code]:py-px [&_code]:font-mono [&_code]:text-xs [&_li]:mt-1 [&_p]:mt-3 [&_pre]:mt-2 [&_pre]:overflow-auto [&_pre]:rounded-[10px] [&_pre]:border [&_pre]:bg-surface-2 [&_pre]:px-3.5 [&_pre]:py-3 [&_pre]:font-mono [&_pre]:text-[12.5px] [&_pre]:leading-[1.85] [&_pre_code]:bg-transparent [&_pre_code]:p-0 [&_strong]:font-semibold [&_ul]:mt-2.5 [&_ul]:list-disc [&_ul]:pl-5"
          dangerouslySetInnerHTML={{ __html: problem.body_html }}
        />
      ) : (
        <Markdown className="mt-[18px]">{problem.description_md}</Markdown>
      )}

      {problem.examples.map((ex, i) => (
        <div key={i}>
          <div className="mt-[22px] text-[13px] font-semibold">
            Example {i + 1}
          </div>
          <div className="mt-2 rounded-[10px] border bg-surface-2 px-3.5 py-[13px] font-mono text-[12.5px] leading-[1.85]">
            <div>
              <span className="text-muted-foreground">Input:&nbsp;&nbsp;</span>
              {ex.input}
            </div>
            <div>
              <span className="text-muted-foreground">Output:&nbsp;</span>
              {ex.output}
            </div>
            {ex.explanation_md && (
              <div className="mt-1 font-sans text-[12.5px] leading-relaxed text-muted-foreground">
                {ex.explanation_md}
              </div>
            )}
          </div>
        </div>
      ))}

      {!problem.body_html && problem.constraints.length > 0 && (
        <>
          <div className="mt-[22px] text-[13px] font-semibold">Constraints</div>
          <ul className="mt-2.5 flex flex-col gap-2">
            {problem.constraints.map((c, i) => (
              <li
                key={i}
                className="flex items-baseline gap-[9px] text-[13.5px] text-muted-foreground"
              >
                <span className="size-1 shrink-0 -translate-y-[3px] rounded-full bg-muted-foreground" />
                <span>
                  <InlineCode text={c} />
                </span>
              </li>
            ))}
          </ul>
        </>
      )}

      {problem.follow_up && (
        <div className="mt-5 rounded-lg border bg-surface-2 px-[13px] py-[11px] text-[13px]">
          <span className="text-[12.5px] font-semibold">Follow-up&nbsp;&nbsp;</span>
          <span className="text-muted-foreground">{problem.follow_up}</span>
        </div>
      )}
    </>
  );
}

function HintsTab({ problem }: { problem: Problem }) {
  const [revealed, setRevealed] = useState(0);
  const hints = problem.hints;

  if (hints.length === 0) {
    return (
      <p className="text-[13px] text-muted-foreground">
        This problem has no hints — trust your instincts.
      </p>
    );
  }
  return (
    <div className="flex flex-col gap-2.5">
      <p className="text-[13px] leading-relaxed text-muted-foreground">
        Hints reveal one at a time — stop as soon as something clicks. The last
        one outlines the full approach.
      </p>
      {hints.slice(0, revealed + 1).map((hint, i) => {
        const isRevealed = i < revealed;
        const isLast = i === hints.length - 1;
        const label = isLast ? "Full approach" : `Hint ${i + 1}`;
        return isRevealed ? (
          <div key={i} className="rounded-[10px] border bg-surface-2 px-3.5 py-3">
            <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-primary">
              <ChevronDown className="size-3.5" />
              {label}
            </div>
            <Markdown className="mt-2">{hint}</Markdown>
          </div>
        ) : (
          <button
            key={i}
            type="button"
            onClick={() => setRevealed(i + 1)}
            className="flex items-center gap-2 rounded-[10px] border bg-card px-3.5 py-3 text-left text-[13px] font-medium transition-colors hover:bg-muted"
          >
            <ChevronRight className="size-3.5 text-muted-foreground" />
            {label}
            <span className="ml-auto text-xs font-normal text-muted-foreground">
              click to reveal
            </span>
          </button>
        );
      })}
    </div>
  );
}

function SolutionTab({ problem }: { problem: Problem }) {
  const [revealed, setRevealed] = useState(false);
  const solution = problem.reference_solution;
  const languages = (["python", "javascript"] as const).filter(
    (l) => solution?.[l]
  );
  const [lang, setLang] = useState<Language>(languages[0] ?? "python");

  if (!solution || languages.length === 0) {
    return (
      <p className="text-[13px] text-muted-foreground">
        No reference solution is available for this problem.
      </p>
    );
  }

  if (!revealed) {
    return (
      <div className="flex min-h-[260px] flex-col items-center justify-center gap-3.5 text-center">
        <div className="flex size-[52px] items-center justify-center rounded-2xl bg-muted text-muted-foreground">
          <Eye className="size-6 stroke-[1.8]" />
        </div>
        <div>
          <div className="text-[15px] font-semibold">Reveal the solution?</div>
          <p className="mx-auto mt-1.5 max-w-[320px] text-[13px] leading-relaxed text-muted-foreground">
            Working through it yourself is where the learning happens. Sure you
            want to peek?
          </p>
        </div>
        <button
          type="button"
          onClick={() => setRevealed(true)}
          className="rounded-[9px] border px-3.5 py-2 text-[13px] font-semibold transition-colors hover:bg-muted"
        >
          Reveal solution
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center gap-0.5">
        {languages.map((l) => (
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
        {solution.complexity && (
          <span className="ml-auto font-mono text-xs text-muted-foreground">
            Time {solution.complexity.time} · Space {solution.complexity.space}
          </span>
        )}
      </div>
      <div className="mt-2.5 overflow-hidden rounded-[10px] border">
        <CodeEditor
          value={solution[lang] ?? ""}
          language={lang}
          readOnly
          className="max-h-[340px]"
        />
      </div>
      {problem.explanation_md && (
        <>
          <div className="mt-5 text-[13px] font-semibold">Explanation</div>
          <Markdown className="mt-2">{problem.explanation_md}</Markdown>
        </>
      )}
    </div>
  );
}

/** Left pane of the workspace: Description / Hints / Solution. */
export function ProblemPane({
  problem,
  bookmarked = false,
  onToggleBookmark,
}: {
  problem: Problem;
  bookmarked?: boolean;
  onToggleBookmark?: () => void;
}) {
  const [tab, setTab] = useState<Tab>("description");
  const tabs: { id: Tab; label: string }[] = [
    { id: "description", label: "Description" },
    { id: "hints", label: "Hints" },
    { id: "solution", label: "Solution" },
  ];

  return (
    <div className="flex h-full min-w-0 flex-col">
      <div className="flex h-10 shrink-0 items-center gap-0.5 border-b px-3">
        {tabs.map(({ id, label }) => (
          <button
            key={id}
            type="button"
            onClick={() => setTab(id)}
            className={cn(
              "relative px-2.5 py-2.5 text-[13px] transition-colors",
              tab === id
                ? "font-semibold text-foreground"
                : "font-medium text-muted-foreground hover:text-foreground"
            )}
          >
            {label}
            {tab === id && (
              <span className="absolute inset-x-1.5 -bottom-px h-0.5 rounded-sm bg-primary" />
            )}
          </button>
        ))}
      </div>
      <div className="min-h-0 flex-1 overflow-auto px-6 pb-7 pt-[22px]">
        {tab === "description" && (
          <DescriptionTab
            problem={problem}
            bookmarked={bookmarked}
            onToggleBookmark={onToggleBookmark}
          />
        )}
        {tab === "hints" && <HintsTab key={problem.id} problem={problem} />}
        {tab === "solution" && (
          <SolutionTab key={problem.id} problem={problem} />
        )}
      </div>
    </div>
  );
}
