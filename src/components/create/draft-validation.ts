import type { UserProblemDraft } from "@/src/lib/types";

/**
 * Client-side live validation for the create/edit form (UI_SPEC §6.5).
 * Mirrors the rules enforced by `validateUserProblem` in the API so the
 * sticky sidebar can show issues as the user types.
 */

export interface DraftIssue {
  /** DOM id of the form section to scroll to. */
  sectionId: string;
  field: string;
  message: string;
}

export interface DraftValidation {
  issues: DraftIssue[];
  /** JSON parse errors per test-case row: index → message. */
  testCaseErrors: Map<number, string>;
  checksPassed: number;
  totalChecks: number;
  /** Positive confirmations for the "Looks good" sidebar list. */
  passing: string[];
}

export function validateDraft(draft: UserProblemDraft): DraftValidation {
  const issues: DraftIssue[] = [];
  const testCaseErrors = new Map<number, string>();
  const passing: string[] = [];
  let checksPassed = 0;
  const totalChecks = 9;

  // 1. title
  if (draft.title.trim()) checksPassed++;
  else issues.push({ sectionId: "sec-basics", field: "Title", message: "is required" });

  // 2. pattern
  if (draft.pattern) checksPassed++;
  else
    issues.push({
      sectionId: "sec-basics",
      field: "Pattern",
      message: "pick one of the 15 patterns",
    });

  // 3. description
  if (draft.description_md.trim()) checksPassed++;
  else
    issues.push({
      sectionId: "sec-statement",
      field: "Description",
      message: "is required",
    });

  if (draft.title.trim() && draft.pattern && draft.description_md.trim()) {
    passing.push("Basics & statement");
  }

  // 4. examples
  const examplesOk =
    draft.examples.length > 0 &&
    draft.examples.every((ex) => ex.input.trim() && ex.output.trim());
  if (examplesOk) checksPassed++;
  else if (draft.examples.length === 0)
    issues.push({
      sectionId: "sec-examples",
      field: "Examples",
      message: "at least one is required",
    });
  else
    draft.examples.forEach((ex, i) => {
      if (!ex.input.trim() || !ex.output.trim())
        issues.push({
          sectionId: "sec-examples",
          field: `Example ${i + 1}`,
          message: "needs both input and output",
        });
    });

  // 5 + 6. signatures
  if (draft.function_signature.python.trim()) checksPassed++;
  else
    issues.push({
      sectionId: "sec-signature",
      field: "Python signature",
      message: "is empty",
    });
  if (draft.function_signature.javascript.trim()) checksPassed++;
  else
    issues.push({
      sectionId: "sec-signature",
      field: "JavaScript signature",
      message: "is empty",
    });

  // 7. test-case JSON
  let jsonOk = draft.test_cases.length > 0;
  draft.test_cases.forEach((tc, i) => {
    for (const [label, raw] of [
      ["Input", tc.input],
      ["Expected", tc.expected],
    ] as const) {
      try {
        const parsed: unknown = JSON.parse(raw);
        if (label === "Input" && !Array.isArray(parsed)) {
          throw new Error("Input must be a JSON array of arguments");
        }
      } catch (err) {
        jsonOk = false;
        const message =
          err instanceof Error && !err.message.startsWith("Unexpected")
            ? err.message
            : `${label} isn't valid JSON`;
        if (!testCaseErrors.has(i)) testCaseErrors.set(i, message);
        issues.push({
          sectionId: "sec-tests",
          field: `Test case ${i + 1}`,
          message,
        });
      }
    }
  });
  if (jsonOk) checksPassed++;

  // 8. visible + hidden coverage
  const visible = draft.test_cases.filter((tc) => !tc.hidden).length;
  const hidden = draft.test_cases.filter((tc) => tc.hidden).length;
  if (visible >= 1 && hidden >= 1) checksPassed++;
  else
    issues.push({
      sectionId: "sec-tests",
      field: "Test cases",
      message:
        visible < 1 ? "need at least 1 visible test" : "need at least 1 hidden test",
    });
  if (examplesOk && jsonOk && visible >= 1 && hidden >= 1) {
    passing.push(
      `${draft.examples.length} example${draft.examples.length === 1 ? "" : "s"} · ${hidden} hidden test${hidden === 1 ? "" : "s"}`
    );
  }

  // 9. warranty
  if (draft.originalityWarranty) {
    checksPassed++;
    passing.push("Originality warranty signed");
  } else {
    issues.push({
      sectionId: "sec-warranty",
      field: "Originality warranty",
      message: "must be accepted to save",
    });
  }

  if (
    draft.reference_solution.python?.trim() ||
    draft.reference_solution.javascript?.trim()
  ) {
    passing.push("Reference solution provided");
  }

  return { issues, testCaseErrors, checksPassed, totalChecks, passing };
}
