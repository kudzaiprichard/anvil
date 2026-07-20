//! Preset slug lists (task 0007): Blind 75 and
//! NeetCode 150 as ordered LeetCode slugs grouped by Anvil pattern. Slug
//! lists are uncopyrightable facts — we ship these, and never any statement
//! text or test data alongside them. Field names mirror `src/lib/types.ts`.

use serde::{Deserialize, Serialize};

use super::problem::Pattern;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetGroup {
    /// One of the 15 Anvil patterns, so the Library filter aligns.
    pub pattern: Pattern,
    pub slugs: Vec<String>,
}

/// Flattened preset for IPC (`list_presets`): the Library/import filter only
/// needs the id, label, and the slug set — not the pattern grouping.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PresetInfo {
    pub id: String,
    pub name: String,
    pub slugs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub groups: Vec<PresetGroup>,
    /// Slugs that are LeetCode-premium: real catalog entries, but absent
    /// from a free-account export — the importer flags rather than matches.
    #[serde(default)]
    pub premium: Vec<String>,
}

impl Preset {
    /// IPC projection for the Library/import filter.
    pub fn info(&self) -> PresetInfo {
        PresetInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            slugs: self.slugs().iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Every slug in display order.
    pub fn slugs(&self) -> Vec<&str> {
        self.groups
            .iter()
            .flat_map(|g| g.slugs.iter().map(String::as_str))
            .collect()
    }

    pub fn contains(&self, slug: &str) -> bool {
        self.groups
            .iter()
            .any(|g| g.slugs.iter().any(|s| s == slug))
    }

    /// The curated pattern this preset files `slug` under — the authoritative
    /// grouping the importer prefers over the topic-slug heuristic.
    pub fn pattern_of(&self, slug: &str) -> Option<&Pattern> {
        self.groups
            .iter()
            .find(|g| g.slugs.iter().any(|s| s == slug))
            .map(|g| &g.pattern)
    }

    /// Structural rules enforced at startup: non-empty, known patterns,
    /// globally unique slugs, premium ⊆ slugs.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err("preset id/name is empty".into());
        }
        if self.groups.is_empty() {
            return Err(format!("preset '{}' has no groups", self.id));
        }
        let mut seen = std::collections::HashSet::new();
        for group in &self.groups {
            if !group.pattern.is_known() {
                return Err(format!(
                    "preset '{}': unknown pattern '{}'",
                    self.id, group.pattern.0
                ));
            }
            if group.slugs.is_empty() {
                return Err(format!(
                    "preset '{}': empty group '{}'",
                    self.id, group.pattern.0
                ));
            }
            for slug in &group.slugs {
                if slug.trim().is_empty() {
                    return Err(format!("preset '{}': empty slug", self.id));
                }
                if !seen.insert(slug.as_str()) {
                    return Err(format!("preset '{}': duplicate slug '{slug}'", self.id));
                }
            }
        }
        for slug in &self.premium {
            if !seen.contains(slug.as_str()) {
                return Err(format!(
                    "preset '{}': premium slug '{slug}' is not in any group",
                    self.id
                ));
            }
        }
        Ok(())
    }
}
