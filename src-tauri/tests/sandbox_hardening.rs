//! Hostile-input tests for the sandbox guards (task 0006). Runtime-gated
//! like the runner suites; the orphan-process assertions are Windows-only
//! (tasklist) — Unix guards are compile-gated for v1.

mod common;

use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::runner;
use common::fixture_problem;

const MEMORY_MESSAGE: &str = "Memory limit exceeded (512 MB) — execution stopped.";

/// These tests count OS processes and hammer memory — run them one at a
/// time so a neighbour's interpreter never skews a baseline.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(windows)]
fn count_processes(image_name: &str) -> usize {
    let out = std::process::Command::new("tasklist")
        .args([
            "/FI",
            &format!("IMAGENAME eq {image_name}"),
            "/FO",
            "CSV",
            "/NH",
        ])
        .output()
        .expect("tasklist runs");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.to_lowercase().contains(&image_name.to_lowercase()))
        .count()
}

/// Children die asynchronously after a job-object kill; give the OS a
/// moment before declaring an orphan leak.
#[cfg(windows)]
fn assert_no_orphans(image_name: &str, baseline: usize) {
    for _ in 0..20 {
        if count_processes(image_name) <= baseline {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    panic!(
        "orphaned {image_name} processes remain: {} > baseline {baseline}",
        count_processes(image_name)
    );
}

#[test]
fn python_huge_allocation_hits_the_memory_cap() {
    let _serial = serial();
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        "def solve(a, b):\n    data = bytearray(10**10)\n    return a + b",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Error);
    assert_eq!(result.error.as_deref(), Some(MEMORY_MESSAGE));
}

#[test]
fn node_huge_allocation_hits_the_memory_cap() {
    let _serial = serial();
    require_runtime!("node");
    let result = runner::execute(
        &fixture_problem(),
        Language::Javascript,
        "function solve(a, b) {\n  const hog = Buffer.alloc(2 ** 31 - 1);\n  return hog.length && a + b;\n}",
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Error);
    assert_eq!(result.error.as_deref(), Some(MEMORY_MESSAGE));
}

#[cfg(windows)]
#[test]
fn fork_bomb_is_capped_and_leaves_no_orphans() {
    let _serial = serial();
    require_runtime!("python");
    let baseline = count_processes("python.exe");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        concat!(
            "def solve(a, b):\n",
            "    import subprocess, sys\n",
            "    for _ in range(100):\n",
            "        subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'])\n",
            "    return a + b",
        ),
        true,
    )
    .unwrap();
    // The active-process cap makes spawning fail (error) or the run times
    // out — either way the app survives and the tree dies with the job.
    assert!(
        matches!(result.status, RunStatus::Error | RunStatus::Timeout),
        "unexpected status: {:?}",
        result.status
    );
    assert_no_orphans("python.exe", baseline);
}

#[cfg(windows)]
#[test]
fn timeout_kills_the_whole_spawned_tree() {
    let _serial = serial();
    require_runtime!("python");
    let baseline = count_processes("python.exe");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        concat!(
            "def solve(a, b):\n",
            "    import subprocess, sys\n",
            "    for _ in range(3):\n",
            "        subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'])\n",
            "    while True:\n",
            "        pass",
        ),
        true,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Timeout);
    assert_no_orphans("python.exe", baseline);
}

/// v1 does not block writes outside the CWD (documented) — but the CWD must
/// be the per-run temp dir, and it must be gone after the run.
#[test]
fn cwd_is_an_isolated_temp_dir_that_gets_cleaned_up() {
    let _serial = serial();
    require_runtime!("python");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        concat!(
            "def solve(a, b):\n",
            "    import os\n",
            "    with open('probe.txt', 'w') as f:\n",
            "        f.write('x')\n",
            "    return os.getcwd()",
        ),
        false,
    )
    .unwrap();
    assert_eq!(result.status, RunStatus::Fail); // cwd string != expected int
    let display = result.cases[0].output.clone().expect("visible case output");
    let cwd: String = serde_json::from_str(&display).expect("output is a JSON string");
    let lower = cwd.to_lowercase();
    assert!(
        lower.contains("tmp") || lower.contains("temp"),
        "cwd was not a temp dir: {cwd}"
    );
    assert!(
        !std::path::Path::new(&cwd).exists(),
        "temp dir should be removed after the run: {cwd}"
    );
}

/// The scrubbed environment must not leak proxy settings or arbitrary
/// variables from the parent process into user code.
#[test]
fn environment_is_scrubbed() {
    let _serial = serial();
    require_runtime!("python");
    // current process env is inherited by `cargo test` children unless
    // scrubbed; pick a var that is guaranteed present in the parent.
    std::env::set_var("ANVIL_SECRET_CANARY", "leak-me");
    std::env::set_var("HTTP_PROXY", "http://proxy.example:8080");
    let result = runner::execute(
        &fixture_problem(),
        Language::Python,
        concat!(
            "def solve(a, b):\n",
            "    import os\n",
            "    return [os.environ.get('ANVIL_SECRET_CANARY'), os.environ.get('HTTP_PROXY')]",
        ),
        false,
    )
    .unwrap();
    std::env::remove_var("ANVIL_SECRET_CANARY");
    std::env::remove_var("HTTP_PROXY");
    assert_eq!(result.status, RunStatus::Fail);
    assert_eq!(result.cases[0].output.as_deref(), Some("[null, null]"));
}
