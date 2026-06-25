"use client";

import {
  Suspense,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import {
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  Check,
  ChevronRight,
  CircleAlert,
  Eye,
  Plus,
  Trash2,
  TriangleAlert,
} from "lucide-react";
import { toast } from "sonner";
import { AppShell } from "@/src/components/anvil/app-shell";
import { CodeEditor } from "@/src/components/anvil/code-editor";
import { Markdown } from "@/src/components/anvil/markdown";
import { Spinner } from "@/src/components/anvil/spinner";
import {
  validateDraft,
  type DraftValidation,
} from "@/src/components/create/draft-validation";
import { PreviewDialog } from "@/src/components/create/preview-dialog";
import {
  VerificationPanel,
  type VerifyState,
} from "@/src/components/create/verification-panel";
import { Checkbox } from "@/src/components/shadcn/checkbox";
import { Input } from "@/src/components/shadcn/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/shadcn/select";
import { Switch } from "@/src/components/shadcn/switch";
import { Textarea } from "@/src/components/shadcn/textarea";
import {
  deleteDraft,
  getDraft,
  getProblem,
  listDrafts,
  saveDraft,
  saveUserProblem,
  validateUserProblem,
} from "@/src/lib/api";
import { cn } from "@/src/lib/utils";
import type {
  Difficulty,
  DraftSummary,
  Language,
  UserProblemDraft,
} from "@/src/lib/types";
import { LANGUAGE_LABELS, PATTERNS } from "@/src/lib/types";

const EMPTY_DRAFT: UserProblemDraft = {
  title: "",
  pattern: "",
  difficulty: "Easy",
  description_md: "",
  constraints: [""],
  follow_up: "",
  examples: [{ input: "", output: "", explanation_md: "" }],
  function_signature: {
    python: "def solve():\n    # write your starter signature\n    pass",
    javascript: "function solve() {\n  // write your starter signature\n}",
  },
  test_cases: [
    { input: "", expected: "", hidden: false },
    { input: "", expected: "", hidden: true },
  ],
  hints: [],
  reference_solution: { python: "", javascript: "", complexity: { time: "", space: "" } },
  originalityWarranty: false,
};

const DIFF_ACTIVE: Record<Difficulty, string> = {
  Easy: "bg-easy/10 font-semibold text-easy dark:bg-easy/15",
  Medium: "bg-medium/10 font-semibold text-medium dark:bg-medium/15",
  Hard: "bg-hard/10 font-semibold text-hard dark:bg-hard/15",
};

/** The Forge stepper. Navigation is free (no gating) — the sidebar marks
 *  which steps still have issues; publishing enforces everything. */
const STEPS: {
  id: string;
  label: string;
  /** Section ids (validation anchors) that live in this step. */
  sections: string[];
  optional?: boolean;
}[] = [
  { id: "basics", label: "Basics", sections: ["sec-basics"] },
  { id: "statement", label: "Statement", sections: ["sec-statement"] },
  { id: "examples", label: "Examples", sections: ["sec-examples"] },
  { id: "starter", label: "Starter code", sections: ["sec-signature"] },
  { id: "tests", label: "Test cases", sections: ["sec-tests"] },
  {
    id: "extras",
    label: "Hints & solution",
    sections: ["sec-hints", "sec-solution"],
    optional: true,
  },
  { id: "publish", label: "Verify & publish", sections: ["sec-warranty"] },
];

const stepForSection = (sectionId: string) =>
  Math.max(
    0,
    STEPS.findIndex((s) => s.sections.includes(sectionId))
  );

function SectionCard({
  id,
  number,
  title,
  caption,
  done,
  children,
}: {
  id: string;
  /** Step number chip; omitted → neutral dot (auxiliary cards). */
  number?: number;
  title: string;
  caption?: React.ReactNode;
  /** Flips the number chip to a green check once the section is complete. */
  done?: boolean;
  children: React.ReactNode;
}) {
  return (
    <section id={id} className="scroll-mt-6 rounded-lg border bg-card p-5">
      <div className="mb-4 flex items-center gap-2.5">
        <span
          className={cn(
            "flex size-[22px] shrink-0 items-center justify-center rounded-[7px] font-mono text-xs font-bold transition-colors",
            done
              ? "bg-pass/10 text-pass dark:bg-pass/15"
              : "bg-primary/10 text-primary dark:bg-primary/20"
          )}
        >
          {done ? (
            <Check className="size-3.5 stroke-[3]" />
          ) : (
            (number ?? <span className="size-[7px] rounded-full bg-primary/60" />)
          )}
        </span>
        <span className="text-sm font-semibold">{title}</span>
        {caption && (
          <span className="text-xs text-muted-foreground">{caption}</span>
        )}
      </div>
      {children}
    </section>
  );
}

function FieldLabel({
  children,
  required,
  className,
}: {
  children: React.ReactNode;
  required?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("mb-1.5 text-[12.5px] font-semibold", className)}>
      {children}
      {required && <span className="text-fail"> *</span>}
    </div>
  );
}

function RemoveButton({ onClick, label }: { onClick: () => void; label: string }) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={label}
      className="flex size-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-fail/10 hover:text-fail"
    >
      <Trash2 className="size-3.5" />
    </button>
  );
}

function AddButton({ onClick, children }: { onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="mt-3 inline-flex items-center gap-1.5 rounded-[9px] border border-dashed px-3 py-[7px] text-[12.5px] font-medium text-muted-foreground transition-colors hover:border-solid hover:bg-muted hover:text-foreground"
    >
      <Plus className="size-3.5" />
      {children}
    </button>
  );
}

function LangTabs({
  value,
  onChange,
  required,
}: {
  value: Language;
  onChange: (l: Language) => void;
  required?: boolean;
}) {
  return (
    <div className="mb-2.5 flex gap-0.5">
      {(["python", "javascript"] as const).map((l) => (
        <button
          key={l}
          type="button"
          onClick={() => onChange(l)}
          className={cn(
            "rounded-lg px-3 py-1.5 text-[12.5px] transition-colors",
            l === value
              ? "bg-primary/10 font-semibold text-primary dark:bg-primary/20"
              : "font-medium text-muted-foreground hover:text-foreground"
          )}
        >
          {LANGUAGE_LABELS[l]}
          {required && <span className="text-fail"> *</span>}
        </button>
      ))}
    </div>
  );
}

function CreateForm() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const editId = searchParams.get("id");

  const [draft, setDraft] = useState<UserProblemDraft>(EMPTY_DRAFT);
  const [loading, setLoading] = useState(Boolean(editId));
  const [saving, setSaving] = useState(false);
  const [showIssues, setShowIssues] = useState(false);
  const [drafts, setDrafts] = useState<DraftSummary[]>([]);
  /** When set, "Save as draft" overwrites this draft instead of creating one. */
  const [activeDraftId, setActiveDraftId] = useState<string | null>(null);
  const [step, setStep] = useState(0);
  const [verifyState, setVerifyState] = useState<VerifyState>(null);
  const [previewOpen, setPreviewOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    listDrafts().then((d) => {
      if (!cancelled) setDrafts(d);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const handleSaveDraft = async () => {
    try {
      const id = await saveDraft(draft, activeDraftId ?? undefined);
      setActiveDraftId(id);
      setDrafts(await listDrafts());
      toast.success("Draft saved.");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not save draft");
    }
  };

  const handleResumeDraft = async (id: string) => {
    try {
      const resumed = await getDraft(id);
      if (!resumed) {
        toast.error("Draft not found");
        return;
      }
      setDraft(resumed);
      setActiveDraftId(id);
      setVerifyState(null);
      setStep(0);
      toast(`Resumed draft "${resumed.title.trim() || "Untitled draft"}".`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not load draft");
    }
  };

  const handleDeleteDraft = async (id: string) => {
    try {
      await deleteDraft(id);
      if (activeDraftId === id) setActiveDraftId(null);
      setDrafts(await listDrafts());
      toast("Draft deleted.");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not delete draft");
    }
  };
  const [statementTab, setStatementTab] = useState<"write" | "split" | "preview">(
    "write"
  );
  const [sigLang, setSigLang] = useState<Language>("python");
  const [solLang, setSolLang] = useState<Language>("python");

  // Edit mode: prefill from the existing problem.
  useEffect(() => {
    if (!editId) return;
    let cancelled = false;
    getProblem(editId).then((p) => {
      if (cancelled) return;
      if (!p) {
        toast.error("Problem not found");
        router.replace("/create");
        setLoading(false);
        return;
      }
      setDraft({
        id: p.id,
        title: p.title,
        pattern: p.pattern,
        difficulty: p.difficulty,
        description_md: p.description_md,
        constraints: p.constraints.length ? [...p.constraints] : [""],
        follow_up: p.follow_up ?? "",
        examples: p.examples.map((ex) => ({ ...ex })),
        function_signature: { ...p.function_signature },
        test_cases: p.test_cases.map((tc) => ({
          input: JSON.stringify(tc.input),
          expected: JSON.stringify(tc.expected),
          hidden: tc.hidden,
        })),
        hints: [...p.hints],
        reference_solution: {
          python: p.reference_solution?.python ?? "",
          javascript: p.reference_solution?.javascript ?? "",
          complexity: p.reference_solution?.complexity ?? { time: "", space: "" },
        },
        originalityWarranty: false,
      });
      setLoading(false);
    });
    return () => {
      cancelled = true;
    };
  }, [editId, router]);

  const patch = useCallback((partial: Partial<UserProblemDraft>) => {
    setDraft((d) => ({ ...d, ...partial }));
    // Content changed → any earlier verification result is stale. The
    // warranty checkbox is the one field that isn't verification input.
    const keys = Object.keys(partial);
    if (!(keys.length === 1 && keys[0] === "originalityWarranty")) {
      setVerifyState(null);
    }
  }, []);

  const validation: DraftValidation = useMemo(
    () => validateDraft(draft),
    [draft]
  );

  const sectionDone = (id: string) =>
    !validation.issues.some((i) => i.sectionId === id);
  const solutionProvided = Boolean(
    draft.reference_solution.python?.trim() ||
      draft.reference_solution.javascript?.trim()
  );

  const runVerification = async () => {
    setShowIssues(true);
    const blocking = validation.issues.filter(
      (i) => i.sectionId !== "sec-warranty"
    );
    if (blocking.length > 0) {
      toast.error(
        `Fix ${blocking.length} issue${blocking.length === 1 ? "" : "s"} before verifying.`
      );
      setStep(stepForSection(blocking[0].sectionId));
      return;
    }
    setVerifyState("running");
    try {
      // The warranty is a publish-time requirement, not verification input —
      // force it so the backend's schema check doesn't reject the dry run.
      setVerifyState(
        await validateUserProblem({ ...draft, originalityWarranty: true })
      );
    } catch (err) {
      setVerifyState(null);
      toast.error(err instanceof Error ? err.message : "Verification failed");
    }
  };

  const handleSave = async () => {
    setShowIssues(true);
    if (validation.issues.length > 0) {
      toast.error(
        `${validation.issues.length} issue${validation.issues.length === 1 ? "" : "s"} to fix before saving.`
      );
      setStep(stepForSection(validation.issues[0].sectionId));
      return;
    }
    setSaving(true);
    try {
      const result = await validateUserProblem(draft);
      if (!result.ok) {
        // Surface the failure in the verification panel, not just a toast.
        setVerifyState(result);
        setStep(STEPS.length - 1);
        toast.error("Verification failed — see the details below.");
        return;
      }
      const failed = result.caseResults?.filter((c) => !c.passed) ?? [];
      if (failed.length > 0) {
        setVerifyState(result);
        setStep(STEPS.length - 1);
        toast.error(
          `Reference solution failed ${failed.length} test case${failed.length === 1 ? "" : "s"}.`
        );
        return;
      }
      const saved = await saveUserProblem(draft);
      // a published draft is no longer a draft
      if (activeDraftId) {
        await deleteDraft(activeDraftId).catch(() => undefined);
        setActiveDraftId(null);
      }
      toast.success(
        editId ? "Problem updated." : "Problem saved — it's now in your library."
      );
      router.push(`/problem?id=${saved.id}`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Save failed");
    } finally {
      setSaving(false);
    }
  };

  const hiddenCount = draft.test_cases.filter((tc) => tc.hidden).length;
  const visibleIssues = showIssues ? validation.issues : [];
  const visibleCount = draft.test_cases.length - hiddenCount;

  const verified =
    typeof verifyState === "object" &&
    verifyState !== null &&
    verifyState.ok &&
    (verifyState.caseResults?.every((c) => c.passed) ?? false);

  const stepDone = (index: number): boolean => {
    const s = STEPS[index];
    if (s.id === "extras") return draft.hints.length > 0 || solutionProvided;
    if (s.id === "publish") return draft.originalityWarranty;
    return s.sections.every(sectionDone);
  };
  const stepHint = (index: number): string | undefined => {
    switch (STEPS[index].id) {
      case "examples":
        return String(draft.examples.length);
      case "tests":
        return `${visibleCount} · ${hiddenCount} hidden`;
      case "extras":
        return draft.hints.length > 0 || solutionProvided
          ? [
              draft.hints.length > 0 ? `${draft.hints.length} hints` : null,
              solutionProvided ? "solution" : null,
            ]
              .filter(Boolean)
              .join(" · ")
          : "optional";
      case "publish":
        return verified ? "verified" : undefined;
      default:
        return undefined;
    }
  };

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Spinner className="size-6" />
      </div>
    );
  }

  const fieldMono =
    "rounded-lg border bg-editor px-3 py-2 font-mono text-xs outline-none transition-colors focus:border-ring";

  return (
    <div className="grid items-start gap-6 px-7 pb-10 pt-[26px] lg:grid-cols-[minmax(0,1fr)_304px]">
      {/* ===== main form (one step at a time) ===== */}
      <div className="flex min-w-0 flex-col gap-[18px]">
        <div>
          <h1 className="text-[22px] font-semibold tracking-tight">
            {editId ? "Edit problem" : "Forge a problem"}
          </h1>
          <p className="mt-1 text-[13.5px] text-muted-foreground">
            {STEPS[step].id === "publish"
              ? "Prove the problem works, preview it, and publish it to your library."
              : `Step ${step + 1} of ${STEPS.length} — ${STEPS[step].label}. Fields marked `}
            {STEPS[step].id !== "publish" && (
              <>
                <span className="text-fail">*</span> are required.
              </>
            )}
          </p>
        </div>

        {/* 1. Basics */}
        {step === 0 && (
        <SectionCard
          id="sec-basics"
          number={1}
          title="Basics"
          done={sectionDone("sec-basics")}
        >
          <div className="flex flex-col gap-4">
            <div>
              <FieldLabel required>Title</FieldLabel>
              <Input
                value={draft.title}
                onChange={(e) => patch({ title: e.target.value })}
                placeholder="e.g. Pair With Target Sum"
                className="bg-editor"
              />
            </div>
            <div className="grid gap-4 md:grid-cols-[1fr_280px]">
              <div>
                <FieldLabel required>Pattern</FieldLabel>
                <Select
                  value={draft.pattern || undefined}
                  onValueChange={(pattern) =>
                    patch({ pattern: pattern as UserProblemDraft["pattern"] })
                  }
                >
                  <SelectTrigger className="w-full bg-editor">
                    <SelectValue placeholder="Select a pattern…" />
                  </SelectTrigger>
                  <SelectContent>
                    {PATTERNS.map((p) => (
                      <SelectItem key={p} value={p}>
                        {p}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <FieldLabel required>Difficulty</FieldLabel>
                <div className="flex gap-1.5 rounded-[9px] border bg-editor p-[3px]">
                  {(["Easy", "Medium", "Hard"] as const).map((d) => (
                    <button
                      key={d}
                      type="button"
                      onClick={() => patch({ difficulty: d })}
                      className={cn(
                        "flex-1 rounded-[7px] py-1.5 text-center text-[12.5px] transition-colors",
                        draft.difficulty === d
                          ? DIFF_ACTIVE[d]
                          : "font-medium text-muted-foreground hover:text-foreground"
                      )}
                    >
                      {d}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </SectionCard>
        )}

        {/* 2. Statement */}
        {step === 1 && (
        <SectionCard
          id="sec-statement"
          number={2}
          title="Statement"
          done={sectionDone("sec-statement")}
        >
          <FieldLabel required>
            Description{" "}
            <span className="font-normal text-muted-foreground">· Markdown</span>
          </FieldLabel>
          <div className="overflow-hidden rounded-[10px] border">
            <div className="flex items-center gap-0.5 border-b bg-surface-2 px-2 py-[5px]">
              {(["write", "split", "preview"] as const).map((t) => (
                <button
                  key={t}
                  type="button"
                  onClick={() => setStatementTab(t)}
                  className={cn(
                    "rounded-md px-2.5 py-1 text-xs capitalize transition-colors",
                    statementTab === t
                      ? "bg-card font-semibold shadow-sm"
                      : "font-medium text-muted-foreground hover:text-foreground"
                  )}
                >
                  {t}
                </button>
              ))}
              <div className="flex-1" />
              <span className="font-mono text-[11px] text-muted-foreground">
                GFM · ** _ ` []()
              </span>
            </div>
            {statementTab === "write" && (
              <Textarea
                value={draft.description_md}
                onChange={(e) => patch({ description_md: e.target.value })}
                placeholder={
                  "Given an array of integers `nums` …\n\nDescribe the task in your own original words."
                }
                className="min-h-[200px] resize-y rounded-none border-0 font-mono text-xs leading-[1.7] shadow-none focus-visible:ring-0"
              />
            )}
            {statementTab === "split" && (
              <div className="grid md:grid-cols-2">
                <Textarea
                  value={draft.description_md}
                  onChange={(e) => patch({ description_md: e.target.value })}
                  placeholder={
                    "Given an array of integers `nums` …\n\nDescribe the task in your own original words."
                  }
                  className="min-h-[200px] resize-y rounded-none border-0 border-r font-mono text-xs leading-[1.7] shadow-none focus-visible:ring-0"
                />
                <div className="px-3.5 py-3">
                  {draft.description_md.trim() ? (
                    <Markdown className="text-[13px]">
                      {draft.description_md}
                    </Markdown>
                  ) : (
                    <span className="text-[13px] text-muted-foreground">
                      Live preview appears here.
                    </span>
                  )}
                </div>
              </div>
            )}
            {statementTab === "preview" && (
              <div className="min-h-[200px] px-3.5 py-3">
                <Markdown>{draft.description_md || "*Nothing to preview yet.*"}</Markdown>
              </div>
            )}
          </div>

          <FieldLabel className="mt-4">Constraints</FieldLabel>
          <div className="flex flex-col gap-2">
            {draft.constraints.map((c, i) => (
              <div key={i} className="flex gap-2">
                <input
                  value={c}
                  onChange={(e) => {
                    const constraints = [...draft.constraints];
                    constraints[i] = e.target.value;
                    patch({ constraints });
                  }}
                  placeholder="e.g. 2 <= nums.length <= 10^4"
                  className={cn(fieldMono, "flex-1")}
                />
                <RemoveButton
                  label={`Remove constraint ${i + 1}`}
                  onClick={() =>
                    patch({
                      constraints: draft.constraints.filter((_, j) => j !== i),
                    })
                  }
                />
              </div>
            ))}
          </div>
          <AddButton
            onClick={() => patch({ constraints: [...draft.constraints, ""] })}
          >
            Add constraint
          </AddButton>

          <FieldLabel className="mt-4">
            Follow-up{" "}
            <span className="font-normal text-muted-foreground">· optional</span>
          </FieldLabel>
          <Input
            value={draft.follow_up ?? ""}
            onChange={(e) => patch({ follow_up: e.target.value })}
            placeholder="e.g. Can you devise an algorithm that runs in O(n) time?"
            className="bg-editor"
          />
        </SectionCard>
        )}

        {/* 3. Examples */}
        {step === 2 && (
        <SectionCard
          id="sec-examples"
          number={3}
          title="Examples"
          caption="at least one required"
          done={sectionDone("sec-examples")}
        >
          <div className="flex flex-col gap-3">
            {draft.examples.map((ex, i) => (
              <div key={i} className="rounded-[10px] border bg-surface-2 px-[13px] py-3">
                <div className="mb-2.5 flex items-center justify-between">
                  <span className="text-[12.5px] font-semibold">
                    Example {i + 1}
                  </span>
                  <RemoveButton
                    label={`Remove example ${i + 1}`}
                    onClick={() =>
                      patch({ examples: draft.examples.filter((_, j) => j !== i) })
                    }
                  />
                </div>
                <div className="grid gap-2.5 md:grid-cols-2">
                  <div>
                    <FieldLabel required className="mb-1 text-[11.5px] text-muted-foreground">
                      Input
                    </FieldLabel>
                    <input
                      value={ex.input}
                      onChange={(e) => {
                        const examples = [...draft.examples];
                        examples[i] = { ...ex, input: e.target.value };
                        patch({ examples });
                      }}
                      placeholder="nums = [2,7,11,15], target = 9"
                      className={cn(fieldMono, "w-full")}
                    />
                  </div>
                  <div>
                    <FieldLabel required className="mb-1 text-[11.5px] text-muted-foreground">
                      Output
                    </FieldLabel>
                    <input
                      value={ex.output}
                      onChange={(e) => {
                        const examples = [...draft.examples];
                        examples[i] = { ...ex, output: e.target.value };
                        patch({ examples });
                      }}
                      placeholder="[0,1]"
                      className={cn(fieldMono, "w-full")}
                    />
                  </div>
                </div>
                <div className="mt-2.5">
                  <FieldLabel className="mb-1 text-[11.5px] text-muted-foreground">
                    Explanation
                  </FieldLabel>
                  <input
                    value={ex.explanation_md ?? ""}
                    onChange={(e) => {
                      const examples = [...draft.examples];
                      examples[i] = { ...ex, explanation_md: e.target.value };
                      patch({ examples });
                    }}
                    placeholder="Because nums[0] + nums[1] == 9, we return [0, 1]."
                    className="w-full rounded-lg border bg-editor px-3 py-2 text-[13px] outline-none transition-colors focus:border-ring"
                  />
                </div>
              </div>
            ))}
          </div>
          <AddButton
            onClick={() =>
              patch({
                examples: [
                  ...draft.examples,
                  { input: "", output: "", explanation_md: "" },
                ],
              })
            }
          >
            Add example
          </AddButton>
        </SectionCard>
        )}

        {/* 4. Function signature */}
        {step === 3 && (
        <SectionCard
          id="sec-signature"
          number={4}
          title="Function signature"
          caption="starter code per language"
          done={sectionDone("sec-signature")}
        >
          <LangTabs value={sigLang} onChange={setSigLang} required />
          <div className="h-[120px] overflow-hidden rounded-[10px] border">
            <CodeEditor
              value={draft.function_signature[sigLang]}
              language={sigLang}
              onChange={(code) =>
                patch({
                  function_signature: {
                    ...draft.function_signature,
                    [sigLang]: code,
                  },
                })
              }
              fontSize={12.5}
            />
          </div>
        </SectionCard>
        )}

        {/* 5. Test cases */}
        {step === 4 && (
        <SectionCard
          id="sec-tests"
          number={5}
          title="Test cases"
          done={sectionDone("sec-tests")}
          caption={
            hiddenCount === 0 && (
              <span className="inline-flex items-center gap-1.5 rounded-full bg-fail/10 px-2 py-0.5 text-[11.5px] font-semibold text-fail dark:bg-fail/15">
                <TriangleAlert className="size-[11px] stroke-[2.4]" />
                needs ≥1 hidden
              </span>
            )
          }
        >
          <div className="grid grid-cols-[32px_1fr_1fr_96px_32px] items-center gap-2.5 px-0.5 pb-2 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
            <span />
            <span>Input · JSON</span>
            <span>Expected · JSON</span>
            <span className="text-center">Hidden</span>
            <span />
          </div>
          <div className="flex flex-col gap-2">
            {draft.test_cases.map((tc, i) => {
              const error = showIssues
                ? validation.testCaseErrors.get(i)
                : undefined;
              const setRow = (row: Partial<typeof tc>) => {
                const test_cases = [...draft.test_cases];
                test_cases[i] = { ...tc, ...row };
                patch({ test_cases });
              };
              return (
                <div key={i}>
                  <div className="grid grid-cols-[32px_1fr_1fr_96px_32px] items-center gap-2.5">
                    <span className="text-center font-mono text-xs text-muted-foreground">
                      {i + 1}
                    </span>
                    <input
                      value={tc.input}
                      onChange={(e) => setRow({ input: e.target.value })}
                      placeholder="[[2,7,11,15], 9]"
                      className={cn(fieldMono, "min-w-0")}
                    />
                    <input
                      value={tc.expected}
                      onChange={(e) => setRow({ expected: e.target.value })}
                      placeholder="[0,1]"
                      className={cn(
                        fieldMono,
                        "min-w-0",
                        error &&
                          "border-[1.5px] border-fail bg-fail/5 dark:bg-fail/10"
                      )}
                    />
                    <div className="flex justify-center">
                      <Switch
                        checked={tc.hidden}
                        onCheckedChange={(hidden) => setRow({ hidden })}
                        aria-label={`Test case ${i + 1} hidden`}
                      />
                    </div>
                    <RemoveButton
                      label={`Remove test case ${i + 1}`}
                      onClick={() =>
                        patch({
                          test_cases: draft.test_cases.filter((_, j) => j !== i),
                        })
                      }
                    />
                  </div>
                  {error && (
                    <div className="ml-[42px] mt-1.5 flex items-center gap-1.5 text-[11.5px] text-fail">
                      <CircleAlert className="size-3 stroke-[2.2]" />
                      {error}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
          <AddButton
            onClick={() =>
              patch({
                test_cases: [
                  ...draft.test_cases,
                  { input: "", expected: "", hidden: false },
                ],
              })
            }
          >
            Add test case
          </AddButton>
        </SectionCard>
        )}

        {/* 6. Hints + reference solution */}
        {step === 5 && (
        <>
        <SectionCard
          id="sec-hints"
          title="Hints"
          caption="ordered · optional"
          done={draft.hints.length > 0}
        >
          <div className="flex flex-col gap-2">
            {draft.hints.map((hint, i) => (
              <div key={i} className="flex items-center gap-2">
                <div className="flex shrink-0 flex-col">
                  <button
                    type="button"
                    aria-label={`Move hint ${i + 1} up`}
                    disabled={i === 0}
                    onClick={() => {
                      const hints = [...draft.hints];
                      [hints[i - 1], hints[i]] = [hints[i], hints[i - 1]];
                      patch({ hints });
                    }}
                    className="text-muted-foreground transition-colors hover:text-foreground disabled:opacity-30"
                  >
                    <ArrowUp className="size-3" />
                  </button>
                  <button
                    type="button"
                    aria-label={`Move hint ${i + 1} down`}
                    disabled={i === draft.hints.length - 1}
                    onClick={() => {
                      const hints = [...draft.hints];
                      [hints[i], hints[i + 1]] = [hints[i + 1], hints[i]];
                      patch({ hints });
                    }}
                    className="text-muted-foreground transition-colors hover:text-foreground disabled:opacity-30"
                  >
                    <ArrowDown className="size-3" />
                  </button>
                </div>
                <span className="w-[18px] shrink-0 font-mono text-xs text-muted-foreground">
                  {i + 1}
                </span>
                <input
                  value={hint}
                  onChange={(e) => {
                    const hints = [...draft.hints];
                    hints[i] = e.target.value;
                    patch({ hints });
                  }}
                  placeholder="Nudge the solver one step closer…"
                  className="min-w-0 flex-1 rounded-lg border bg-editor px-3 py-2 text-[13px] outline-none transition-colors focus:border-ring"
                />
                <RemoveButton
                  label={`Remove hint ${i + 1}`}
                  onClick={() =>
                    patch({ hints: draft.hints.filter((_, j) => j !== i) })
                  }
                />
              </div>
            ))}
          </div>
          <AddButton onClick={() => patch({ hints: [...draft.hints, ""] })}>
            Add hint
          </AddButton>
        </SectionCard>

        {/* Reference solution (same step as hints) */}
        <SectionCard
          id="sec-solution"
          title="Reference solution"
          caption="powers the verification step — strongly recommended"
          done={solutionProvided}
        >
          <LangTabs value={solLang} onChange={setSolLang} />
          <div className="h-[180px] overflow-hidden rounded-[10px] border">
            <CodeEditor
              value={draft.reference_solution[solLang] ?? ""}
              language={solLang}
              onChange={(code) =>
                patch({
                  reference_solution: {
                    ...draft.reference_solution,
                    [solLang]: code,
                  },
                })
              }
              fontSize={12.5}
            />
          </div>
          <div className="mt-3.5 grid gap-3.5 md:grid-cols-2">
            <div>
              <FieldLabel className="mb-1 text-[11.5px] text-muted-foreground">
                Time complexity
              </FieldLabel>
              <input
                value={draft.reference_solution.complexity?.time ?? ""}
                onChange={(e) =>
                  patch({
                    reference_solution: {
                      ...draft.reference_solution,
                      complexity: {
                        time: e.target.value,
                        space: draft.reference_solution.complexity?.space ?? "",
                      },
                    },
                  })
                }
                placeholder="O(n)"
                className={cn(fieldMono, "w-full")}
              />
            </div>
            <div>
              <FieldLabel className="mb-1 text-[11.5px] text-muted-foreground">
                Space complexity
              </FieldLabel>
              <input
                value={draft.reference_solution.complexity?.space ?? ""}
                onChange={(e) =>
                  patch({
                    reference_solution: {
                      ...draft.reference_solution,
                      complexity: {
                        time: draft.reference_solution.complexity?.time ?? "",
                        space: e.target.value,
                      },
                    },
                  })
                }
                placeholder="O(n)"
                className={cn(fieldMono, "w-full")}
              />
            </div>
          </div>
        </SectionCard>
        </>
        )}

        {/* 7. Verify & publish */}
        {step === 6 && (
          <>
            <SectionCard
              id="sec-verify"
              title="Verify the problem works"
              caption="sandboxed dry run — nothing is saved"
              done={verified}
            >
              <VerificationPanel
                solutionProvided={solutionProvided}
                state={verifyState}
                onRun={runVerification}
              />
            </SectionCard>

            <SectionCard
              id="sec-preview"
              title="Preview"
              caption="open the problem the way a solver will see it"
            >
              <button
                type="button"
                onClick={() => setPreviewOpen(true)}
                className="flex items-center gap-2 rounded-md border bg-card px-3.5 py-2 text-[13px] font-semibold transition-colors hover:bg-accent"
              >
                <Eye className="size-[15px]" />
                Open workspace preview
              </button>
            </SectionCard>

            <section
              id="sec-warranty"
              className={cn(
                "flex scroll-mt-6 items-start gap-3 rounded-lg border bg-surface-2 px-[18px] py-4",
                showIssues && !draft.originalityWarranty && "border-fail/50"
              )}
            >
              <Checkbox
                id="warranty"
                checked={draft.originalityWarranty}
                onCheckedChange={(checked) =>
                  patch({ originalityWarranty: checked === true })
                }
                className="mt-0.5"
              />
              <label htmlFor="warranty" className="cursor-pointer text-[13px] leading-relaxed">
                <span className="font-semibold">
                  Originality warranty <span className="text-fail">*</span>
                </span>
                <br />
                <span className="text-muted-foreground">
                  I warrant this content is original, or that I have the right
                  to share it. Required to publish.
                </span>
              </label>
            </section>

            <div className="flex items-center gap-3 rounded-lg border bg-card p-5">
              <button
                type="button"
                onClick={handleSave}
                disabled={saving}
                className="flex items-center justify-center gap-1.5 rounded-md bg-primary px-5 py-[10px] text-[13.5px] font-semibold text-primary-foreground transition-[filter,transform] hover:brightness-110 active:scale-[0.99] disabled:opacity-60"
              >
                {saving ? (
                  <Spinner className="size-3.5 border-primary-foreground/40 border-t-primary-foreground" />
                ) : (
                  <Check className="size-[15px] stroke-[2.2]" />
                )}
                {saving
                  ? "Validating…"
                  : editId
                    ? "Save changes"
                    : "Publish to library"}
              </button>
              <span className="text-[11.5px] leading-relaxed text-muted-foreground">
                Publishing re-runs verification server-side — an unverified or
                failing problem never lands in your library silently.
              </span>
            </div>
          </>
        )}

        {/* step navigation */}
        <div className="mt-1 flex items-center justify-between">
          <button
            type="button"
            onClick={() => setStep((s) => Math.max(0, s - 1))}
            disabled={step === 0}
            className="flex items-center gap-1.5 rounded-md border bg-card px-3.5 py-2 text-[13px] font-medium transition-colors hover:bg-accent disabled:opacity-40"
          >
            <ArrowLeft className="size-3.5" />
            Back
          </button>
          <span className="font-mono text-[11px] text-muted-foreground">
            {step + 1} / {STEPS.length}
          </span>
          {step < STEPS.length - 1 ? (
            <button
              type="button"
              onClick={() => setStep((s) => s + 1)}
              className="flex items-center gap-1.5 rounded-md bg-primary px-3.5 py-2 text-[13px] font-semibold text-primary-foreground transition-[filter] hover:brightness-110"
            >
              Continue
              <ArrowRight className="size-3.5" />
            </button>
          ) : (
            <span className="w-[92px]" />
          )}
        </div>
      </div>

      {/* ===== sticky sidebar ===== */}
      <div className="sticky top-6 flex flex-col gap-3.5">
        <div className="rounded-lg border bg-card px-[17px] py-4">
          <div className="text-[13.5px] font-semibold">
            {validation.checksPassed === validation.totalChecks
              ? "Ready to save"
              : "Almost there"}
          </div>
          <div className="mt-2 flex items-center gap-2.5">
            <div className="h-[7px] flex-1 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-primary transition-[width]"
                style={{
                  width: `${(validation.checksPassed / validation.totalChecks) * 100}%`,
                }}
              />
            </div>
            <span className="font-mono text-xs font-semibold text-muted-foreground">
              {validation.checksPassed}/{validation.totalChecks}
            </span>
          </div>

          {/* steps — live status, click to jump */}
          <div className="microlabel mt-4">Steps</div>
          <div className="mt-1.5 flex flex-col gap-px">
            {STEPS.map((s, i) => {
              const done = stepDone(i);
              const active = i === step;
              const hint = stepHint(i);
              return (
                <button
                  key={s.id}
                  type="button"
                  onClick={() => setStep(i)}
                  aria-current={active ? "step" : undefined}
                  className={cn(
                    "group flex items-center gap-2.5 rounded-md px-2 py-[5px] text-left transition-colors",
                    active ? "bg-accent" : "hover:bg-accent"
                  )}
                >
                  {done ? (
                    <Check className="size-[13px] shrink-0 stroke-[2.8] text-pass" />
                  ) : showIssues && !s.optional ? (
                    <CircleAlert className="size-[13px] shrink-0 stroke-[2.2] text-fail" />
                  ) : (
                    <span
                      className={cn(
                        "flex size-[13px] shrink-0 items-center justify-center font-mono text-[9.5px] font-semibold",
                        active ? "text-primary" : "text-muted-foreground/70"
                      )}
                    >
                      {i + 1}
                    </span>
                  )}
                  <span
                    className={cn(
                      "min-w-0 flex-1 truncate text-[12.5px]",
                      active
                        ? "font-semibold text-foreground"
                        : done
                          ? "font-medium text-foreground"
                          : "font-medium text-muted-foreground group-hover:text-foreground"
                    )}
                  >
                    {s.label}
                  </span>
                  {hint && (
                    <span
                      className={cn(
                        "shrink-0 font-mono text-[10.5px]",
                        hint === "verified" ? "text-pass" : "text-muted-foreground"
                      )}
                    >
                      {hint}
                    </span>
                  )}
                </button>
              );
            })}
          </div>

          {visibleIssues.length > 0 && (
            <>
              <div className="microlabel mt-4 text-fail">Needs attention</div>
              <div className="mt-2 flex flex-col gap-2">
                {visibleIssues.map((issue, i) => (
                  <button
                    key={`${issue.field}-${i}`}
                    type="button"
                    onClick={() => setStep(stepForSection(issue.sectionId))}
                    className="flex items-start gap-2 text-left"
                  >
                    <CircleAlert className="mt-px size-3.5 shrink-0 stroke-[2.2] text-fail" />
                    <span className="text-[12.5px] leading-snug">
                      {issue.field}{" "}
                      <span className="text-muted-foreground">
                        — {issue.message}
                      </span>
                    </span>
                  </button>
                ))}
              </div>
            </>
          )}
        </div>

        <button
          type="button"
          onClick={() => setPreviewOpen(true)}
          className="flex w-full items-center justify-center gap-1.5 rounded-md border bg-card px-4 py-2.5 text-[13px] font-medium transition-colors hover:bg-accent"
        >
          <Eye className="size-[14px]" />
          Preview in workspace
        </button>
        <button
          type="button"
          onClick={handleSaveDraft}
          className="w-full rounded-md border bg-card px-4 py-2.5 text-[13px] font-medium transition-colors hover:bg-accent"
        >
          Save as draft
        </button>
        <p className="text-center text-[11.5px] leading-relaxed text-muted-foreground">
          Publish from the final step — it verifies your reference solution
          against every test case first.
        </p>

        {drafts.length > 0 && (
          <div className="rounded-lg border bg-card px-[17px] py-4">
            <div className="microlabel">Drafts</div>
            <div className="mt-2.5 flex flex-col gap-1">
              {drafts.map((d) => (
                <div
                  key={d.id}
                  className={cn(
                    "flex items-center gap-2 rounded-md px-2 py-1.5 transition-colors",
                    d.id === activeDraftId ? "bg-primary/5 dark:bg-primary/10" : "hover:bg-muted"
                  )}
                >
                  <button
                    type="button"
                    onClick={() => handleResumeDraft(d.id)}
                    className="min-w-0 flex-1 text-left"
                  >
                    <span className="block truncate text-[12.5px] font-medium">
                      {d.title}
                    </span>
                  </button>
                  <button
                    type="button"
                    title="Delete draft"
                    onClick={() => handleDeleteDraft(d.id)}
                    className="flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-fail/10 hover:text-fail"
                  >
                    <Trash2 className="size-[13px]" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      <PreviewDialog
        open={previewOpen}
        onOpenChange={setPreviewOpen}
        draft={draft}
      />
    </div>
  );
}

function CreateHeader({ editing }: { editing: boolean }) {
  const router = useRouter();
  return (
    <header className="flex h-[42px] shrink-0 items-center gap-[14px] border-b bg-card px-5">
      <div className="flex items-center gap-2 text-[13px]">
        <Link
          href="/problems"
          className="text-muted-foreground transition-colors hover:text-foreground"
        >
          Problems
        </Link>
        <ChevronRight className="size-3.5 text-muted-foreground" />
        <span className="font-semibold">
          {editing ? "Edit problem" : "New problem"}
        </span>
      </div>
      <div className="flex-1" />
      <button
        type="button"
        onClick={() => router.back()}
        className="rounded-md border bg-card px-3 py-1.5 text-[12.5px] font-medium transition-colors hover:bg-accent"
      >
        Cancel
      </button>
    </header>
  );
}

function CreatePageInner() {
  const searchParams = useSearchParams();
  return (
    <>
      <CreateHeader editing={Boolean(searchParams.get("id"))} />
      <div className="min-h-0 flex-1 overflow-auto">
        <CreateForm />
      </div>
    </>
  );
}

export default function CreatePage() {
  return (
    <AppShell>
      <Suspense
        fallback={
          <div className="flex flex-1 items-center justify-center">
            <Spinner className="size-6" />
          </div>
        }
      >
        <CreatePageInner />
      </Suspense>
    </AppShell>
  );
}
