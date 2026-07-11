//! Runtime detection (spec §7.3 hybrid strategy, step 1): probe PATH for a
//! compatible Python and Node, reporting name/path/version/found for the
//! Settings → Runtime pane. Never errors — a missing runtime is data
//! (`found: false`), not a failure. The bundled-runtime fallback (step 2)
//! is deferred past 0012 (decision §9.4).

use std::process::Command;

use serde::{Deserialize, Serialize};

pub const MIN_PYTHON: (u32, u32, u32) = (3, 10, 0);
pub const MIN_NODE: (u32, u32, u32) = (18, 0, 0);

/// Mirrors `RuntimeInfo` in `src/lib/types.ts` (all-lowercase field names —
/// no rename needed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub tag: String,
    pub name: String,
    pub path: String,
    pub version: String,
    pub found: bool,
}

pub fn detect() -> Vec<RuntimeInfo> {
    let detected = vec![detect_python(), detect_node()];
    for rt in &detected {
        if rt.found {
            log::info!("runtime {}: {} at {}", rt.name, rt.version, rt.path);
        } else {
            log::info!("runtime {}: not found", rt.name);
        }
    }
    detected
}

/// Candidates in probe order; `py -3` covers the Windows launcher.
const PYTHON_CANDIDATES: &[(&str, &[&str])] = &[("python", &[]), ("python3", &[]), ("py", &["-3"])];

fn detect_python() -> RuntimeInfo {
    for (program, base_args) in PYTHON_CANDIDATES {
        let mut version_args = base_args.to_vec();
        version_args.push("--version");
        let Some(text) = run_capture(program, &version_args) else {
            continue;
        };
        let Some(version) = parse_version(&text) else {
            continue;
        };
        if version < MIN_PYTHON {
            continue;
        }
        // sys.executable resolves shims and the py launcher to the real
        // interpreter path the runner should spawn.
        let mut path_args = base_args.to_vec();
        path_args.extend(["-c", "import sys; print(sys.executable)"]);
        let path = run_capture(program, &path_args).unwrap_or_default();
        return found("Py", "Python", path, version);
    }
    not_found("Py", "Python")
}

fn detect_node() -> RuntimeInfo {
    if let Some(text) = run_capture("node", &["--version"]) {
        if let Some(version) = parse_version(&text) {
            if version >= MIN_NODE {
                let path = run_capture("node", &["-e", "console.log(process.execPath)"])
                    .unwrap_or_default();
                return found("JS", "Node.js", path, version);
            }
        }
    }
    not_found("JS", "Node.js")
}

fn found(tag: &str, name: &str, path: String, version: (u32, u32, u32)) -> RuntimeInfo {
    RuntimeInfo {
        tag: tag.into(),
        name: name.into(),
        path,
        version: format!("v{}.{}.{}", version.0, version.1, version.2),
        found: true,
    }
}

fn not_found(tag: &str, name: &str) -> RuntimeInfo {
    RuntimeInfo {
        tag: tag.into(),
        name: name.into(),
        path: String::new(),
        version: String::new(),
        found: false,
    }
}

/// Runs `program args…` and returns trimmed stdout (falling back to stderr —
/// some interpreters print versions there). `None` on any failure: missing
/// binary, non-zero exit (covers the Windows Store python alias stub), or
/// empty output.
fn run_capture(program: &str, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Some(stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    (!stderr.is_empty()).then_some(stderr)
}

/// Extracts the first dotted version from strings like `"Python 3.12.1"` or
/// `"v20.11.0"`.
fn parse_version(text: &str) -> Option<(u32, u32, u32)> {
    let token = text
        .split_whitespace()
        .map(|t| t.trim_start_matches('v'))
        .find(|t| t.contains('.') && t.chars().next().is_some_and(|c| c.is_ascii_digit()))?;
    let mut parts = token.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts
        .next()
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_python_style_versions() {
        assert_eq!(parse_version("Python 3.12.1"), Some((3, 12, 1)));
        assert_eq!(parse_version("Python 3.10.0rc1"), Some((3, 10, 0)));
        assert_eq!(parse_version("Python 3.14"), Some((3, 14, 0)));
    }

    #[test]
    fn parses_node_style_versions() {
        assert_eq!(parse_version("v20.11.0"), Some((20, 11, 0)));
        assert_eq!(parse_version("v18.0.0"), Some((18, 0, 0)));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_version("command not found"), None);
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("vX.Y.Z"), None);
    }

    #[test]
    fn version_ordering_enforces_minimums() {
        assert!(parse_version("Python 3.9.19").unwrap() < MIN_PYTHON);
        assert!(parse_version("Python 3.10.0").unwrap() >= MIN_PYTHON);
        assert!(parse_version("v17.9.1").unwrap() < MIN_NODE);
    }
}
