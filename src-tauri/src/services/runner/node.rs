//! Node runtime spec. Quirk: the shipped JS starter is `function solve() {}`
//! with no export, so a `module.exports` shim is appended to the user's file
//! on disk (their editor content is untouched) — see `solution_suffix`.
//! The bare `"node"` program name is replaced by the detected path in 0007.

use super::LanguageSpec;

pub static SPEC: LanguageSpec = LanguageSpec {
    solution_filename: "solution.js",
    harness_filename: "harness.js",
    harness_source: include_str!("harness/harness.js"),
    program: "node",
    args: &[],
    solution_suffix:
        "\nmodule.exports = typeof solve !== \"undefined\" ? solve : module.exports;\n",
};
