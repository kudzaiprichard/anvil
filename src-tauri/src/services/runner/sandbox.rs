//! All v1 process guards live here (BACKEND_PLAN §6.2): whole-run timeout,
//! memory cap, process cap, environment scrub, and reliable kill-the-tree
//! semantics. Windows uses a Job Object (`KILL_ON_JOB_CLOSE` is the
//! fork-bomb defense); Unix uses `setsid` + rlimits + `killpg`. Explicitly
//! NOT container-grade isolation — that is out of scope for v1 (spec §7.3).

use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::error::{AppError, AppResult};

/// Single source of truth for guard defaults.
pub const DEFAULT_TIMEOUT_MS: u64 = 3000;
pub const DEFAULT_MEMORY_BYTES: u64 = 512 * 1024 * 1024;
pub const DEFAULT_MAX_PROCESSES: u32 = 16;

#[derive(Debug, Clone, Copy)]
pub struct Guards {
    pub timeout_ms: u64,
    pub memory_bytes: u64,
    pub max_processes: u32,
}

impl Default for Guards {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_TIMEOUT_MS,
            memory_bytes: DEFAULT_MEMORY_BYTES,
            max_processes: DEFAULT_MAX_PROCESSES,
        }
    }
}

pub enum WaitOutcome {
    Exited {
        status: ExitStatus,
        stdout: String,
        stderr: String,
        /// Peak memory where cheaply available (Windows Job Object
        /// accounting); `None` on Unix in v1.
        peak_memory_bytes: Option<u64>,
    },
    TimedOut,
    /// The runtime died from the memory cap before producing a structured
    /// error (e.g. a V8 heap abort). Python-level `MemoryError`s surface as
    /// normal harness error lines instead and are mapped by the runner.
    MemoryKilled,
}

pub struct GuardedChild {
    child: Child,
    #[cfg(windows)]
    job: windows_job::Job,
}

/// Spawns `cmd` with all guards applied: scrubbed environment, piped stdio,
/// memory/process caps. The caller sets program/args/cwd; everything
/// security-relevant happens here.
pub fn spawn_guarded(mut cmd: Command, guards: &Guards) -> AppResult<GuardedChild> {
    scrub_env(&mut cmd);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let program = cmd.get_program().to_string_lossy().into_owned();

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
        let job = windows_job::Job::new(guards)?;
        let child = cmd.spawn().map_err(|e| spawn_error(&program, &e))?;
        // std cannot CREATE_SUSPENDED; assigning right after spawn leaves a
        // tiny window before limits apply — accepted for v1.
        job.assign(&child)?;
        Ok(GuardedChild { child, job })
    }

    #[cfg(unix)]
    {
        apply_unix_guards(&mut cmd, guards);
        let child = cmd.spawn().map_err(|e| spawn_error(&program, &e))?;
        Ok(GuardedChild { child })
    }
}

fn spawn_error(program: &str, e: &std::io::Error) -> AppError {
    AppError::Runner(format!(
        "failed to start {program} — is it installed and on PATH? ({e})"
    ))
}

/// Minimal environment: only what the runtimes need to boot. Everything
/// else — proxies (the best-effort no-network stance), API keys, HOME — is
/// dropped by `env_clear`.
fn scrub_env(cmd: &mut Command) {
    cmd.env_clear();
    const ALLOWED: &[&str] = &[
        "PATH",
        "SYSTEMROOT",
        "SYSTEMDRIVE",
        "TEMP",
        "TMP",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
    ];
    for key in ALLOWED {
        if let Ok(value) = std::env::var(key) {
            cmd.env(key, value);
        }
    }
    // Deterministic unicode output on Windows consoles.
    cmd.env("PYTHONIOENCODING", "utf-8");
}

impl GuardedChild {
    /// Waits up to `timeout`, draining stdout/stderr on threads so a chatty
    /// child can never fill the pipe buffer and deadlock the watchdog. On
    /// timeout the entire process tree is killed.
    pub fn wait_with_timeout(mut self, timeout: Duration) -> AppResult<WaitOutcome> {
        let mut stdout_pipe = self
            .child
            .stdout
            .take()
            .ok_or_else(|| AppError::Runner("stdout was not piped".into()))?;
        let mut stderr_pipe = self
            .child
            .stderr
            .take()
            .ok_or_else(|| AppError::Runner("stderr was not piped".into()))?;
        let stdout_thread = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = stdout_pipe.read_to_string(&mut s);
            s
        });
        let stderr_thread = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = stderr_pipe.read_to_string(&mut s);
            s
        });

        match self.child.wait_timeout(timeout)? {
            Some(status) => {
                let stdout = stdout_thread.join().unwrap_or_default();
                let stderr = stderr_thread.join().unwrap_or_default();
                let peak_memory_bytes = self.peak_memory_bytes();
                if !status.success() && is_memory_abort(&stderr) {
                    return Ok(WaitOutcome::MemoryKilled);
                }
                Ok(WaitOutcome::Exited {
                    status,
                    stdout,
                    stderr,
                    peak_memory_bytes,
                })
            }
            None => {
                self.kill_tree();
                let _ = self.child.wait();
                Ok(WaitOutcome::TimedOut)
            }
        }
    }

    fn kill_tree(&mut self) {
        #[cfg(windows)]
        self.job.terminate();
        #[cfg(unix)]
        unsafe {
            // the child called setsid(), so its pid is the group id
            libc::killpg(self.child.id() as i32, libc::SIGKILL);
        }
        let _ = self.child.kill();
    }

    fn peak_memory_bytes(&self) -> Option<u64> {
        #[cfg(windows)]
        {
            self.job.peak_memory_bytes()
        }
        #[cfg(unix)]
        {
            None
        }
    }
}

/// Runtime-abort signatures of the memory cap (V8 heap abort, interpreter-
/// level OOM on stderr). Python `MemoryError` raised inside the solve call
/// is caught by the harness and surfaces as a traceback instead.
pub fn is_memory_abort(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("out of memory")
        || lower.contains("allocation failed")
        || lower.contains("memoryerror")
}

#[cfg(unix)]
fn apply_unix_guards(cmd: &mut Command, guards: &Guards) {
    use std::os::unix::process::CommandExt;
    let memory_bytes = guards.memory_bytes;
    let max_processes = guards.max_processes;
    unsafe {
        cmd.pre_exec(move || {
            // own session/process group so killpg reaches grandchildren
            libc::setsid();
            let mem = libc::rlimit {
                rlim_cur: memory_bytes as libc::rlim_t,
                rlim_max: memory_bytes as libc::rlim_t,
            };
            libc::setrlimit(libc::RLIMIT_AS, &mem);
            let nproc = libc::rlimit {
                rlim_cur: max_processes as libc::rlim_t,
                rlim_max: max_processes as libc::rlim_t,
            };
            libc::setrlimit(libc::RLIMIT_NPROC, &nproc);
            Ok(())
        });
    }
}

#[cfg(windows)]
mod windows_job {
    //! Thin Job Object wrapper: process-memory + active-process limits and
    //! `KILL_ON_JOB_CLOSE`, so dropping the job (even on panic) reaps the
    //! whole child tree.

    use std::os::windows::io::AsRawHandle;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    };

    use super::Guards;
    use crate::error::{AppError, AppResult};

    pub struct Job(HANDLE);

    // SAFETY: job object handles are thread-safe kernel handles.
    unsafe impl Send for Job {}

    fn last_os_error(context: &str) -> AppError {
        AppError::Runner(format!("{context}: {}", std::io::Error::last_os_error()))
    }

    impl Job {
        pub fn new(guards: &Guards) -> AppResult<Self> {
            // SAFETY: standard Job Object creation/configuration FFI; the
            // info struct is zero-initialized POD and outlives the call.
            unsafe {
                let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if handle.is_null() {
                    return Err(last_os_error("CreateJobObject failed"));
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY
                    | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                    | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                info.ProcessMemoryLimit = guards.memory_bytes as usize;
                info.BasicLimitInformation.ActiveProcessLimit = guards.max_processes;
                let ok = SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                );
                if ok == 0 {
                    let err = last_os_error("SetInformationJobObject failed");
                    CloseHandle(handle);
                    return Err(err);
                }
                Ok(Self(handle))
            }
        }

        pub fn assign(&self, child: &std::process::Child) -> AppResult<()> {
            // SAFETY: both handles are valid for the duration of the call.
            let ok = unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle() as HANDLE) };
            if ok == 0 {
                return Err(last_os_error("AssignProcessToJobObject failed"));
            }
            Ok(())
        }

        pub fn terminate(&self) {
            // SAFETY: valid job handle; exit code 1 for the killed tree.
            unsafe {
                TerminateJobObject(self.0, 1);
            }
        }

        pub fn peak_memory_bytes(&self) -> Option<u64> {
            // SAFETY: zero-initialized POD out-param sized to the struct.
            unsafe {
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                let ok = QueryInformationJobObject(
                    self.0,
                    JobObjectExtendedLimitInformation,
                    &mut info as *mut _ as *mut core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    std::ptr::null_mut(),
                );
                (ok != 0).then_some(info.PeakProcessMemoryUsed as u64)
            }
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            // KILL_ON_JOB_CLOSE makes this the last-resort tree reaper.
            // SAFETY: handle owned by self, closed exactly once.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}
