//! Preset loading (task 0007): reads and validates every `*.json` under
//! `resources/presets/` at startup, the same trusted-bundled-content rule
//! as `problem_store` — a bad file fails startup loudly, naming the file.

use std::path::Path;

use crate::domain::preset::Preset;
use crate::error::{AppError, AppResult};

#[derive(Debug, Default)]
pub struct PresetStore {
    presets: Vec<Preset>,
}

impl PresetStore {
    pub fn load(presets_dir: &Path) -> AppResult<Self> {
        if !presets_dir.is_dir() {
            return Err(AppError::Validation(format!(
                "preset resource dir missing: {}",
                presets_dir.display()
            )));
        }
        let mut entries: Vec<_> = std::fs::read_dir(presets_dir)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|e| e.path());

        let mut presets = Vec::new();
        for entry in entries {
            let path = entry.path();
            if !path.extension().is_some_and(|ext| ext == "json") {
                continue;
            }
            let raw = std::fs::read_to_string(&path)?;
            let preset: Preset = serde_json::from_str(&raw).map_err(|e| {
                AppError::Validation(format!("{}: invalid preset JSON: {e}", path.display()))
            })?;
            preset
                .validate()
                .map_err(|msg| AppError::Validation(format!("{}: {msg}", path.display())))?;
            presets.push(preset);
        }
        if presets.is_empty() {
            return Err(AppError::Validation(format!(
                "no preset files found under {}",
                presets_dir.display()
            )));
        }
        log::info!("loaded {} presets", presets.len());
        Ok(Self { presets })
    }

    pub fn all(&self) -> &[Preset] {
        &self.presets
    }

    pub fn get(&self, id: &str) -> Option<&Preset> {
        self.presets.iter().find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn shipped_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("presets")
    }

    #[test]
    fn shipped_presets_load_and_validate() {
        let store = PresetStore::load(&shipped_dir()).expect("presets load");
        let blind = store.get("blind75").expect("blind75 present");
        let neet = store.get("neetcode150").expect("neetcode150 present");

        assert_eq!(blind.name, "Blind 75");
        assert_eq!(blind.slugs().len(), 75);
        assert_eq!(neet.name, "NeetCode 150");
        assert_eq!(neet.slugs().len(), 150);

        // uniqueness (validate() enforces it, but assert the shipped data)
        for preset in store.all() {
            let slugs = preset.slugs();
            let unique: HashSet<&&str> = slugs.iter().collect();
            assert_eq!(slugs.len(), unique.len(), "{} has duplicates", preset.id);
        }
    }

    #[test]
    fn blind75_is_a_subset_of_neetcode150_except_combination_sum_iv() {
        // Verified against the canonical lists: the original Blind 75
        // includes combination-sum-iv, which NeetCode 150 does not.
        let store = PresetStore::load(&shipped_dir()).unwrap();
        let blind = store.get("blind75").unwrap();
        let neet = store.get("neetcode150").unwrap();
        let outside: Vec<&str> = blind
            .slugs()
            .into_iter()
            .filter(|s| !neet.contains(s))
            .collect();
        assert_eq!(outside, vec!["combination-sum-iv"]);
    }

    #[test]
    fn premium_slugs_are_listed_and_belong_to_groups() {
        let store = PresetStore::load(&shipped_dir()).unwrap();
        let blind = store.get("blind75").unwrap();
        // e.g. Meeting Rooms is premium and in the Intervals group
        assert!(blind.premium.iter().any(|s| s == "meeting-rooms"));
        assert!(blind.contains("meeting-rooms"));
    }

    #[test]
    fn invalid_presets_fail_loudly() {
        use crate::domain::preset::{Preset, PresetGroup};
        use crate::domain::problem::Pattern;

        let mut p = Preset {
            id: "x".into(),
            name: "X".into(),
            groups: vec![PresetGroup {
                pattern: Pattern("Stack".into()),
                slugs: vec!["a".into(), "a".into()],
            }],
            premium: vec![],
        };
        assert!(p.validate().unwrap_err().contains("duplicate slug"));
        p.groups[0].slugs = vec!["a".into()];
        p.groups[0].pattern = Pattern("Sorcery".into());
        assert!(p.validate().unwrap_err().contains("unknown pattern"));
        p.groups[0].pattern = Pattern("Stack".into());
        p.premium = vec!["not-in-groups".into()];
        assert!(p.validate().unwrap_err().contains("premium slug"));
    }
}
