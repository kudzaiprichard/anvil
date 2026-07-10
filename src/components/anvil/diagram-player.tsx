"use client";

import { useCallback, useMemo, useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  CircleDot,
  Film,
  HelpCircle,
  RotateCcw,
  Sparkles,
  X,
} from "lucide-react";
import { Markdown } from "@/src/components/anvil/markdown";
import { cn } from "@/src/lib/utils";
import type { DiagramSpec } from "@/src/lib/types";

/**
 * Prediction-diagram player (Phase 5, LESSON_COURSE_DESIGN.md §3.5/§13.4). The
 * *renderer* is engine; the frames/trace are data, precomputed offline — no
 * server, no runtime execution. It steps through an algorithm's state
 * snapshots and, at each `predict_at` frame, **pauses to ask "what happens
 * next?"** before revealing the answer: the prediction turn is what converts a
 * passive animation into active engagement (COURSE_BLUEPRINT.md §7). A frame's
 * graded `predict` block scores the choice against the trace's ground truth;
 * without one, the pause degrades to think-then-reveal. In `perform` mode the
 * same turn is framed as "perform the step yourself".
 */
export function DiagramPlayer({
  diagram,
  className,
}: {
  diagram: DiagramSpec;
  className?: string;
}) {
  const steps = diagram.steps;
  const predictAt = useMemo(
    () => new Set(diagram.predict_at),
    [diagram.predict_at]
  );
  const perform = diagram.mode === "perform";

  const [index, setIndex] = useState(0);
  // Which prediction pauses the learner has committed to (by step index) and
  // the choice they picked — so a revisited pause shows their verdict again.
  const [answered, setAnswered] = useState<Record<number, string>>({});

  const step = steps[index];
  const isPause = predictAt.has(index);
  const committed = answered[index] !== undefined;
  // A pause blocks moving forward until the learner has engaged with it.
  const blockedForward = isPause && !committed;
  const atEnd = index === steps.length - 1;

  const goPrev = useCallback(() => setIndex((i) => Math.max(0, i - 1)), []);
  const goNext = useCallback(
    () => setIndex((i) => Math.min(steps.length - 1, i + 1)),
    [steps.length]
  );
  const replay = useCallback(() => {
    setIndex(0);
    setAnswered({});
  }, []);

  const commit = useCallback(
    (choiceId: string) => setAnswered((a) => ({ ...a, [index]: choiceId })),
    [index]
  );
  const retryPause = useCallback(
    () =>
      setAnswered((a) => {
        const next = { ...a };
        delete next[index];
        return next;
      }),
    [index]
  );

  return (
    <div
      className={cn(
        "overflow-hidden rounded-xl border bg-surface-2",
        className
      )}
    >
      {/* header */}
      <div className="flex items-center gap-2 border-b bg-card px-4 py-2.5">
        <Film className="size-[15px] text-primary" />
        <span className="microlabel text-foreground">
          {perform ? "Perform it yourself" : "Prediction diagram"}
        </span>
        <span className="font-mono text-[11px] text-muted-foreground">
          {diagram.algorithm}
        </span>
        <span className="flex-1" />
        <span className="font-mono text-[11px] text-muted-foreground">
          {index + 1} / {steps.length}
        </span>
      </div>

      {/* step dots — prediction pauses marked in ember */}
      <div className="flex items-center gap-1.5 px-4 pt-3">
        {steps.map((_, i) => {
          const pause = predictAt.has(i);
          return (
            <button
              key={i}
              type="button"
              aria-label={`Step ${i + 1}${pause ? " (prediction)" : ""}`}
              onClick={() => setIndex(i)}
              className={cn(
                "h-1.5 flex-1 rounded-full transition-colors",
                i === index
                  ? "bg-primary"
                  : i < index
                    ? pause
                      ? "bg-primary/40"
                      : "bg-pass/40"
                    : "bg-muted"
              )}
            />
          );
        })}
      </div>

      {/* state visualization */}
      <div className="px-4 pb-1 pt-4">
        <StateView state={step.state} />
      </div>

      {/* caption */}
      <div className="px-4 pb-4 pt-1">
        <Markdown className="text-[13px] leading-relaxed [&_p]:!my-0">
          {step.caption_md}
        </Markdown>
      </div>

      {/* prediction pause */}
      {isPause && (
        <PredictionPanel
          key={index}
          step={step}
          perform={perform}
          committedChoice={answered[index]}
          onCommit={commit}
          onRetry={retryPause}
        />
      )}

      {/* controls */}
      <div className="flex items-center gap-2 border-t bg-card px-4 py-2.5">
        <button
          type="button"
          onClick={goPrev}
          disabled={index === 0}
          className="inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-[12.5px] font-medium text-muted-foreground transition-colors hover:bg-accent disabled:opacity-40 disabled:hover:bg-transparent"
        >
          <ArrowLeft className="size-[14px]" />
          Back
        </button>
        <span className="flex-1" />
        {blockedForward && (
          <span className="text-[11.5px] text-muted-foreground">
            {perform ? "Make your move to continue" : "Predict to continue"}
          </span>
        )}
        {atEnd ? (
          <button
            type="button"
            onClick={replay}
            className="inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-[12.5px] font-medium text-muted-foreground transition-colors hover:bg-accent"
          >
            <RotateCcw className="size-[13px]" />
            Replay
          </button>
        ) : (
          <button
            type="button"
            onClick={goNext}
            disabled={blockedForward}
            className={cn(
              "inline-flex items-center gap-1.5 rounded-md px-3.5 py-1.5 text-[12.5px] font-semibold transition-[filter,opacity]",
              blockedForward
                ? "cursor-not-allowed bg-muted text-muted-foreground"
                : "bg-primary text-primary-foreground hover:brightness-110"
            )}
          >
            Next
            <ArrowRight className="size-[14px] stroke-[2.2]" />
          </button>
        )}
      </div>
    </div>
  );
}

/* --------------------------------------------------------------------- */
/* Prediction pause                                                      */
/* --------------------------------------------------------------------- */

function PredictionPanel({
  step,
  perform,
  committedChoice,
  onCommit,
  onRetry,
}: {
  step: DiagramSpec["steps"][number];
  perform: boolean;
  committedChoice?: string;
  onCommit: (choiceId: string) => void;
  onRetry: () => void;
}) {
  const predict = step.predict;

  // Fallback: a pause without a graded block is a think-then-reveal prompt.
  if (!predict) {
    const revealed = committedChoice !== undefined;
    return (
      <div className="mx-4 mb-4 rounded-lg border border-primary/30 bg-primary/5 p-3.5">
        <p className="flex items-center gap-1.5 text-[12.5px] font-semibold text-primary">
          <HelpCircle className="size-[14px]" />
          {perform ? "Perform the step" : "Predict: what happens next?"}
        </p>
        {revealed ? (
          <p className="mt-1.5 text-[12.5px] text-muted-foreground">
            Now step forward and check your prediction against the next frame.
          </p>
        ) : (
          <button
            type="button"
            onClick={() => onCommit("revealed")}
            className="mt-2.5 inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-[12.5px] font-semibold text-primary-foreground transition-[filter] hover:brightness-110"
          >
            I&apos;ve made my prediction
          </button>
        )}
      </div>
    );
  }

  const revealed = committedChoice !== undefined;
  const correct = committedChoice === predict.answer;

  return (
    <div className="mx-4 mb-4 rounded-lg border border-primary/30 bg-primary/5 p-3.5">
      <p className="flex items-center gap-1.5 text-[12.5px] font-semibold text-primary">
        <HelpCircle className="size-[14px]" />
        {perform ? "Perform the step" : "Predict: what happens next?"}
      </p>
      <Markdown className="mt-1.5 text-[13px] [&_p]:!my-0">
        {predict.prompt_md}
      </Markdown>

      <ul className="mt-2.5 flex flex-col gap-1.5">
        {predict.choices.map((choice) => {
          const isPick = committedChoice === choice.id;
          const isAnswer = revealed && choice.id === predict.answer;
          const isWrongPick = revealed && isPick && !correct;
          return (
            <li key={choice.id}>
              <button
                type="button"
                onClick={() => !revealed && onCommit(choice.id)}
                disabled={revealed}
                className={cn(
                  "flex w-full items-center gap-2.5 rounded-md border bg-card px-3 py-2 text-left text-[13px] transition-colors",
                  !revealed && "hover:bg-accent",
                  revealed && "cursor-default",
                  isAnswer && "border-pass bg-pass/10",
                  isWrongPick && "border-fail bg-fail/10",
                  revealed && !isAnswer && !isWrongPick && "opacity-60"
                )}
              >
                <span
                  className={cn(
                    "flex size-[16px] shrink-0 items-center justify-center rounded-full border",
                    isAnswer && "border-pass bg-pass text-white",
                    isWrongPick && "border-fail bg-fail text-white",
                    !revealed && "border-muted-foreground/40"
                  )}
                >
                  {isAnswer && <Check className="size-[11px] stroke-[3]" />}
                  {isWrongPick && <X className="size-[11px] stroke-[3]" />}
                </span>
                <span className="min-w-0 flex-1">
                  <Markdown className="[&_p]:!my-0">{choice.label_md}</Markdown>
                </span>
              </button>
            </li>
          );
        })}
      </ul>

      {revealed && (
        <div
          className={cn(
            "mt-3 rounded-md border-l-2 pl-3 text-[12.5px]",
            correct ? "border-pass" : "border-fail"
          )}
        >
          <p
            className={cn(
              "flex items-center gap-1.5 font-semibold",
              correct ? "text-pass" : "text-fail"
            )}
          >
            {correct ? (
              <>
                <Check className="size-[13px] stroke-[2.6]" />
                {perform ? "Correct move" : "Nailed it"}
              </>
            ) : (
              <>
                <X className="size-[13px] stroke-[2.6]" />
                Not quite
              </>
            )}
            <button
              type="button"
              onClick={onRetry}
              className="ml-auto inline-flex items-center gap-1 text-[11px] font-medium text-muted-foreground transition-colors hover:text-foreground"
            >
              <RotateCcw className="size-[11px]" />
              Try again
            </button>
          </p>
          <Markdown className="mt-1 text-muted-foreground [&_p]:!my-0">
            {predict.explanation_md}
          </Markdown>
        </div>
      )}
    </div>
  );
}

/* --------------------------------------------------------------------- */
/* Generic state visualization                                           */
/* --------------------------------------------------------------------- */

type Json = null | boolean | number | string | Json[] | { [k: string]: Json };

function isPlainObject(v: Json): v is { [k: string]: Json } {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}
function isScalarArray(v: Json): v is (string | number | boolean | null)[] {
  return Array.isArray(v) && v.every((e) => !isPlainObject(e) && !Array.isArray(e));
}

/** Renders an opaque algorithm-state snapshot generically off value *types*
 *  (not field names, so the renderer stays engine, not per-lesson): scalars
 *  become labeled registers, arrays become cell rows, objects become a
 *  key→value map card (a hash bucket filling up). */
function StateView({ state }: { state: unknown }) {
  const obj = isPlainObject(state as Json) ? (state as { [k: string]: Json }) : null;
  if (!obj) {
    // Non-object state: show it as one raw value.
    return <RawValue value={state as Json} />;
  }
  const entries = Object.entries(obj);
  const scalars = entries.filter(
    ([, v]) => !isPlainObject(v) && !Array.isArray(v)
  );
  const arrays = entries.filter(([, v]) => isScalarArray(v));
  const maps = entries.filter(([, v]) => isPlainObject(v));

  return (
    <div className="flex flex-col gap-3">
      {scalars.length > 0 && (
        <div className="flex flex-wrap items-stretch gap-2">
          {scalars.map(([k, v]) => (
            <Register key={k} label={k} value={v} />
          ))}
        </div>
      )}
      {arrays.map(([k, v]) => (
        <CellRow key={k} label={k} values={v as (string | number | boolean | null)[]} />
      ))}
      {maps.map(([k, v]) => (
        <MapCard key={k} label={k} map={v as { [k: string]: Json }} />
      ))}
    </div>
  );
}

function fmt(v: Json): string {
  if (v === null) return "∅";
  if (typeof v === "string") return v;
  return JSON.stringify(v);
}

function Register({ label, value }: { label: string; value: Json }) {
  const empty = value === null;
  return (
    <div className="flex flex-col rounded-lg border bg-card px-3 py-1.5">
      <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
        {label}
      </span>
      <span
        className={cn(
          "font-mono text-[14px] font-semibold",
          empty ? "text-muted-foreground" : "text-foreground"
        )}
      >
        {fmt(value)}
      </span>
    </div>
  );
}

function CellRow({
  label,
  values,
}: {
  label: string;
  values: (string | number | boolean | null)[];
}) {
  return (
    <div className="flex items-center gap-2.5">
      <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
        {label}
      </span>
      {values.length === 0 ? (
        <span className="font-mono text-[12px] text-muted-foreground">
          empty
        </span>
      ) : (
        <div className="flex items-center gap-1">
          {values.map((v, i) => (
            <span
              key={i}
              className="flex min-w-[30px] items-center justify-center rounded-md border bg-card px-2 py-1 font-mono text-[13px] font-semibold"
            >
              {fmt(v as Json)}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

function MapCard({ label, map }: { label: string; map: { [k: string]: Json } }) {
  const entries = Object.entries(map);
  return (
    <div className="rounded-lg border bg-card px-3 py-2">
      <div className="flex items-center gap-1.5">
        <CircleDot className="size-[11px] text-primary" />
        <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
          {label}
        </span>
        {entries.length === 0 && (
          <span className="font-mono text-[11.5px] text-muted-foreground">
            empty
          </span>
        )}
      </div>
      {entries.length > 0 && (
        <div className="mt-1.5 flex flex-wrap gap-1.5">
          {entries.map(([k, v]) => (
            <span
              key={k}
              className="inline-flex items-center gap-1 rounded-md border bg-surface-2 px-2 py-1 font-mono text-[12px]"
            >
              <span className="text-foreground">{k}</span>
              <ArrowRight className="size-[11px] text-muted-foreground" />
              <span className="font-semibold text-primary">{fmt(v)}</span>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

function RawValue({ value }: { value: Json }) {
  return (
    <div className="flex items-center gap-2">
      <Sparkles className="size-[13px] text-primary" />
      <span className="font-mono text-[13px] font-semibold">{fmt(value)}</span>
    </div>
  );
}
