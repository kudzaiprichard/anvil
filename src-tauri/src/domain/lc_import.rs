//! LeetCode import domain (task 0005, CONTENT_DESIGN.md §6–8). Two halves:
//!
//! 1. The **scrape file** shape (`my_questions.json`) — the user's own export,
//!    deserialized leniently (the external scraper owns this format; unknown
//!    fields are ignored so a scraper revision never breaks import). This is
//!    the user's data; it is never shipped, committed, or sent anywhere but
//!    the local DB.
//! 2. The **IPC reports** the importer returns: a pre-import `ScrapePreview`
//!    (so the UI can show coverage + per-question selection) and a post-import
//!    `ImportSummary`. Field names/casing mirror `src/lib/types.ts`.
//!
//! No `tauri::` here — pure types + the `topic_slugs → pattern` table, so the
//! whole matching/merge layer unit-tests as plain Rust.

use serde::{Deserialize, Serialize};

use super::problem::{Difficulty, Pattern};

/// The experience tier a matched question lands in (CONTENT_DESIGN.md §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportTier {
    /// slug matched + fingerprint verified: statement + our pack (hidden
    /// judge, hints, solutions, reveal-failing-case).
    Full,
    /// no pack / fingerprint mismatch: visible tests parsed from the
    /// statement's own examples.
    Basic,
    /// examples unparseable: execute and show output, no verdict.
    RunOnly,
}

impl ImportTier {
    pub fn as_str(self) -> &'static str {
        match self {
            ImportTier::Full => "full",
            ImportTier::Basic => "basic",
            ImportTier::RunOnly => "run-only",
        }
    }

    pub fn from_tag(s: &str) -> Option<Self> {
        match s {
            "full" => Some(ImportTier::Full),
            "basic" => Some(ImportTier::Basic),
            "run-only" => Some(ImportTier::RunOnly),
            _ => None,
        }
    }
}

/* ---------------- scrape file (`my_questions.json`) ---------------- */

/// Top-level envelope. `schema_version` is accepted as string or number; we
/// only require the `questions` array to be present and well-formed.
#[derive(Debug, Clone, Deserialize)]
pub struct ScrapeFile {
    #[serde(default)]
    pub schema_version: serde_json::Value,
    #[serde(default)]
    pub questions: Vec<ScrapeQuestion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScrapeQuestion {
    #[serde(default)]
    pub qid: String,
    pub slug: String,
    #[serde(default)]
    pub title: String,
    /// "Easy" | "Medium" | "Hard" — matches our `Difficulty` serde exactly.
    #[serde(default = "default_difficulty")]
    pub difficulty: Difficulty,
    /// The scraper emits this as the string "true"/"false" or a bool; accept
    /// both. Premium questions are flagged, not silently dropped.
    #[serde(default, deserialize_with = "de_flexible_bool")]
    pub is_premium: bool,
    #[serde(default)]
    pub body_html: String,
    #[serde(default)]
    pub body_text: String,
    #[serde(default)]
    pub hints: Vec<String>,
    #[serde(default)]
    pub topic_slugs: Vec<String>,
    #[serde(default)]
    pub code_stubs: CodeStubs,
    #[serde(default)]
    pub example_tests: Vec<ExampleTest>,
    #[serde(default)]
    pub scraped_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CodeStubs {
    #[serde(default)]
    pub python: String,
    #[serde(default)]
    pub javascript: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExampleTest {
    /// One JSON-encoded value per parameter (the structured input the example
    /// parser prefers); may be absent on older scrapes.
    #[serde(default)]
    pub input_lines: Vec<String>,
}

fn default_difficulty() -> Difficulty {
    Difficulty::Medium
}

/// Accepts `true`/`false`, the strings `"true"`/`"false"`, or 0/1.
fn de_flexible_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Bool(b) => Ok(b),
        serde_json::Value::String(s) => Ok(matches!(s.to_lowercase().as_str(), "true" | "1")),
        serde_json::Value::Number(n) => Ok(n.as_i64().unwrap_or(0) != 0),
        serde_json::Value::Null => Ok(false),
        other => Err(D::Error::custom(format!("invalid is_premium: {other}"))),
    }
}

/* ---------------- IPC: preview + summary ---------------- */

/// Pre-import preview so the UI can show coverage and offer per-question
/// selection without a second file read (the parsed scrape is cached server
/// side between `select_scrape` and `import_selected`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScrapePreview {
    pub total: usize,
    /// How many scraped questions have a bundled pack (full-tier candidates).
    #[serde(rename = "withPack")]
    pub with_pack: usize,
    /// How many are already in the local library (re-import candidates).
    #[serde(rename = "alreadyImported")]
    pub already_imported: usize,
    pub questions: Vec<PreviewItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PreviewItem {
    pub slug: String,
    pub title: String,
    pub difficulty: Difficulty,
    #[serde(rename = "hasPack")]
    pub has_pack: bool,
    #[serde(rename = "alreadyImported")]
    pub already_imported: bool,
    #[serde(rename = "isPremium")]
    pub is_premium: bool,
}

/// Per-question outcome the summary screen surfaces (CONTENT_MAP "Import
/// Summary Screen"). Counts are mutually exclusive across the three tiers for
/// the questions that were stored; `skipped` is everything not stored.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ImportSummary {
    /// Newly added problems.
    pub imported: u32,
    /// Re-imported with a newer scrape — content refreshed, history kept.
    pub updated: u32,
    /// Re-imported but the stored copy was as new or newer — left untouched.
    pub unchanged: u32,
    pub full: u32,
    pub basic: u32,
    #[serde(rename = "runOnly")]
    pub run_only: u32,
    /// `(slug, reason)` for questions that were not stored (premium, missing
    /// stubs, parse failure) — shown collapsibly in the summary.
    pub skipped: Vec<SkippedQuestion>,
    /// Notes worth surfacing but not failures — e.g. a topic that fell back
    /// to the closest pattern, or stress cases skipped for a missing runtime.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkippedQuestion {
    pub slug: String,
    pub reason: String,
}

/// Manage-controls payload: how many imported problems exist and when the
/// last import happened (local ISO; the seam formats it).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImportedStats {
    pub count: u32,
    #[serde(rename = "lastImport", skip_serializing_if = "Option::is_none")]
    pub last_import: Option<String>,
}

/* ---------------- topic_slugs → pattern ---------------- */

/// LeetCode `topic_slug → Anvil pattern`, ordered most-specific first: the
/// first topic the question carries that appears here wins, so "sliding
/// window" beats the generic "array" it is always co-tagged with. Preset
/// membership overrides this entirely (the curated grouping is authoritative);
/// this table is the fallback for questions outside any preset.
const TOPIC_PATTERN: &[(&str, &str)] = &[
    ("sliding-window", "Sliding Window"),
    ("two-pointers", "Two Pointers"),
    ("binary-search", "Binary Search"),
    ("monotonic-stack", "Stack"),
    ("stack", "Stack"),
    ("linked-list", "Linked List"),
    ("trie", "Trees"),
    ("binary-search-tree", "Trees"),
    ("binary-tree", "Trees"),
    ("tree", "Trees"),
    ("heap-priority-queue", "Heap / Priority Queue"),
    ("priority-queue", "Heap / Priority Queue"),
    ("backtracking", "Backtracking"),
    ("union-find", "Graphs"),
    ("topological-sort", "Graphs"),
    ("shortest-path", "Graphs"),
    ("graph", "Graphs"),
    ("dynamic-programming", "1-D DP"),
    ("greedy", "Greedy"),
    ("bit-manipulation", "Bit Manipulation"),
    ("hash-table", "Arrays & Hashing"),
];

/// The bucket used when no topic maps and the question is in no preset.
pub const PATTERN_FALLBACK: &str = "Arrays & Hashing";

/// Best Anvil pattern for a question's topic slugs. `None` ⇒ caller uses the
/// fallback bucket and records it in the summary.
pub fn pattern_from_topics(topic_slugs: &[String]) -> Option<Pattern> {
    for (topic, pattern) in TOPIC_PATTERN {
        if topic_slugs.iter().any(|t| t == topic) {
            return Some(Pattern((*pattern).to_string()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flexible_bool_accepts_scraper_string_form() {
        let q: ScrapeQuestion = serde_json::from_value(json!({
            "slug": "two-sum", "is_premium": "false"
        }))
        .unwrap();
        assert!(!q.is_premium);
        let q: ScrapeQuestion =
            serde_json::from_value(json!({ "slug": "x", "is_premium": "true" })).unwrap();
        assert!(q.is_premium);
        let q: ScrapeQuestion =
            serde_json::from_value(json!({ "slug": "y", "is_premium": true })).unwrap();
        assert!(q.is_premium);
        // absent ⇒ not premium
        let q: ScrapeQuestion = serde_json::from_value(json!({ "slug": "z" })).unwrap();
        assert!(!q.is_premium);
    }

    #[test]
    fn unknown_scrape_fields_are_ignored() {
        let q: ScrapeQuestion = serde_json::from_value(json!({
            "slug": "two-sum", "title": "Two Sum", "difficulty": "Easy",
            "companies": ["acme"], "similar_questions": ["3sum"], "source": "leetcode"
        }))
        .unwrap();
        assert_eq!(q.slug, "two-sum");
        assert_eq!(q.difficulty, Difficulty::Easy);
    }

    #[test]
    fn most_specific_topic_wins() {
        // array is always co-tagged; the specific pattern must win.
        assert_eq!(
            pattern_from_topics(&["array".into(), "sliding-window".into()]),
            Some(Pattern("Sliding Window".into()))
        );
        assert_eq!(
            pattern_from_topics(&["array".into(), "hash-table".into()]),
            Some(Pattern("Arrays & Hashing".into()))
        );
        assert_eq!(
            pattern_from_topics(&["depth-first-search".into(), "graph".into()]),
            Some(Pattern("Graphs".into()))
        );
        // nothing recognized ⇒ caller falls back
        assert_eq!(
            pattern_from_topics(&["math".into(), "simulation".into()]),
            None
        );
    }

    #[test]
    fn tier_round_trips_through_strings() {
        for tier in [ImportTier::Full, ImportTier::Basic, ImportTier::RunOnly] {
            assert_eq!(ImportTier::from_tag(tier.as_str()), Some(tier));
        }
        assert_eq!(ImportTier::from_tag("bogus"), None);
        // kebab serde matches as_str()
        assert_eq!(
            serde_json::to_value(ImportTier::RunOnly).unwrap(),
            json!("run-only")
        );
    }
}
