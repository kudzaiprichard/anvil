//! Python runtime spec. Quirks: none — the harness handles entry-function
//! resolution and traceback cleanup itself. The bare `"python"` program name
//! is replaced by the detected interpreter path in task 0007.

use super::LanguageSpec;

pub static SPEC: LanguageSpec = LanguageSpec {
    solution_filename: "solution.py",
    harness_filename: "harness.py",
    harness_source: include_str!("harness/harness.py"),
    program: "python",
    args: &[],
    solution_suffix: "",
};
