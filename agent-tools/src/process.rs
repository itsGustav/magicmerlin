//! Background process/session manager used by exec/process tools.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;

use crate::error::{Result, ToolError};

/// Lightweight process summary for list/poll operations.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessSummary {
    /// Process session ID.
    pub session_id: u64,
    /// Original command.
    pub command: String,
    /// Process ID if known.
    pub pid: Option<u32>,
    /// Whether process has exited.
    pub exited: bool,
    /// Exit code if process is completed.
    pub exit_code: Option<i32>,
    /// Runtime in milliseconds.
    pub runtime_ms: u128,
    /// Last output character offset.
    pub cursor: usize,
    /// Whether process was started in TTY-like mode.
    pub tty: bool,
    /// Process start timestamp.
    pub started_at_unix_ms: u128,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessLogChunk {
    pub session_id: u64,
    pub from: usize,
    pub to: usize,
    pub text: String,
    pub exited: bool,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessOutput {
    pub combined: String,
    pub stdout: String,
    pub stderr: String,
}

struct ProcessHandle {
    command: String,
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    output: Arc<Mutex<ProcessOutput>>,
    cursor: Arc<Mutex<usize>>,
    start: Instant,
    started_at_unix_ms: u128,
    tty: bool,
}

/// Concurrent process manager for background sessions.
#[derive(Clone, Default)]
pub struct ProcessManager {
    next_id: Arc<AtomicU64>,
    handles: Arc<Mutex<HashMap<u64, ProcessHandle>>>,
}

impl ProcessManager {
    /// Creates a new process manager.
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawns a new background command and returns session id.
    pub async fn spawn(
        &self,
        command: &str,
        cwd: Option<&std::path::Path>,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<u64> {
        self.spawn_with_options(command, cwd, env, false, None)
            .await
    }

    /// Spawns process with optional TTY mode and timeout monitor.
    pub async fn spawn_with_options(
        &self,
        command: &str,
        cwd: Option<&std::path::Path>,
        env: &std::collections::HashMap<String, String>,
        tty: bool,
        timeout: Option<Duration>,
    ) -> Result<u64> {
        let mut cmd = shell_command(command);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env);

        let mut child = cmd
            .spawn()
            .map_err(|err| ToolError::Process(format!("spawn failed: {err}")))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdin = child.stdin.take();

        let output = Arc::new(Mutex::new(ProcessOutput {
            combined: String::new(),
            stdout: String::new(),
            stderr: String::new(),
        }));

        if let Some(mut out) = stdout {
            let target = output.clone();
            tokio::spawn(async move {
                let mut buf = vec![0_u8; 8192];
                loop {
                    match out.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                            let mut lock = target.lock().await;
                            lock.stdout.push_str(&chunk);
                            lock.combined.push_str(&chunk);
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        if let Some(mut err) = stderr {
            let target = output.clone();
            tokio::spawn(async move {
                let mut buf = vec![0_u8; 8192];
                loop {
                    match err.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                            let mut lock = target.lock().await;
                            lock.stderr.push_str(&chunk);
                            lock.combined.push_str(&chunk);
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let handle = ProcessHandle {
            command: command.to_string(),
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            output,
            cursor: Arc::new(Mutex::new(0)),
            start: Instant::now(),
            started_at_unix_ms: now_ms(),
            tty,
        };

        if let Some(limit) = timeout {
            let child_ref = handle.child.clone();
            tokio::spawn(async move {
                tokio::time::sleep(limit).await;
                let mut child = child_ref.lock().await;
                if child.try_wait().ok().flatten().is_none() {
                    let _ = child.kill().await;
                }
            });
        }

        self.handles.lock().await.insert(id, handle);
        Ok(id)
    }

    /// Lists active and finished process sessions.
    pub async fn list(&self) -> Vec<ProcessSummary> {
        let handles = self.handles.lock().await;
        let mut out = Vec::new();
        for (id, handle) in handles.iter() {
            let (exited, code, pid) = {
                let mut child = handle.child.lock().await;
                let status = child.try_wait().ok().flatten();
                (status.is_some(), status.and_then(|s| s.code()), child.id())
            };
            let cursor = *handle.cursor.lock().await;
            out.push(ProcessSummary {
                session_id: *id,
                command: handle.command.clone(),
                pid,
                exited,
                exit_code: code,
                runtime_ms: handle.start.elapsed().as_millis(),
                cursor,
                tty: handle.tty,
                started_at_unix_ms: handle.started_at_unix_ms,
            });
        }
        out.sort_by_key(|x| x.session_id);
        out
    }

    /// Writes stdin bytes for a session.
    pub async fn write(&self, session_id: u64, text: &str) -> Result<()> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let mut stdin = handle.stdin.lock().await;
        let Some(stdin) = stdin.as_mut() else {
            return Err(ToolError::Process("stdin closed".to_string()));
        };
        stdin
            .write_all(text.as_bytes())
            .await
            .map_err(|err| ToolError::Process(err.to_string()))
    }

    /// Close stdin after sending optional text.
    pub async fn submit(&self, session_id: u64, text: Option<&str>) -> Result<()> {
        if let Some(chunk) = text {
            self.write(session_id, chunk).await?;
        }
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let mut stdin = handle.stdin.lock().await;
        *stdin = None;
        Ok(())
    }

    /// Reads session log with offset/limit slicing.
    pub async fn log(&self, session_id: u64, offset: usize, limit: usize) -> Result<String> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let output = handle.output.lock().await;
        let sliced = output
            .combined
            .chars()
            .skip(offset)
            .take(limit)
            .collect::<String>();
        Ok(sliced)
    }

    /// Polls one session for status and new output up to timeout.
    pub async fn poll_wait(&self, session_id: u64, timeout_ms: u64) -> Result<ProcessLogChunk> {
        let start = Instant::now();
        loop {
            let chunk = self.poll_chunk(session_id).await?;
            if !chunk.text.is_empty() || chunk.exited || timeout_ms == 0 {
                return Ok(chunk);
            }
            if start.elapsed() >= Duration::from_millis(timeout_ms) {
                return Ok(chunk);
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// Polls one session for process status.
    pub async fn poll(&self, session_id: u64) -> Result<ProcessSummary> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let (exited, code, pid) = {
            let mut child = handle.child.lock().await;
            let status = child.try_wait().ok().flatten();
            (status.is_some(), status.and_then(|s| s.code()), child.id())
        };
        let cursor = *handle.cursor.lock().await;

        Ok(ProcessSummary {
            session_id,
            command: handle.command.clone(),
            pid,
            exited,
            exit_code: code,
            runtime_ms: handle.start.elapsed().as_millis(),
            cursor,
            tty: handle.tty,
            started_at_unix_ms: handle.started_at_unix_ms,
        })
    }

    /// Returns new output since prior cursor.
    pub async fn poll_chunk(&self, session_id: u64) -> Result<ProcessLogChunk> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;

        let output = handle.output.lock().await;
        let mut cursor = handle.cursor.lock().await;
        let from = *cursor;
        let available = output.combined.chars().count();
        let text = output.combined.chars().skip(from).collect::<String>();
        *cursor = available;

        let (exited, code) = {
            let mut child = handle.child.lock().await;
            let status = child.try_wait().ok().flatten();
            (status.is_some(), status.and_then(|s| s.code()))
        };

        Ok(ProcessLogChunk {
            session_id,
            from,
            to: available,
            text,
            exited,
            exit_code: code,
        })
    }

    /// Sends terminal key sequences to process.
    pub async fn send_keys(&self, session_id: u64, keys: &str) -> Result<()> {
        let expanded = expand_key_sequence(keys);
        self.write(session_id, &expanded).await
    }

    /// Sends bracketed paste sequence.
    pub async fn paste(&self, session_id: u64, text: &str) -> Result<()> {
        let wrapped = format!("\u{001b}[200~{text}\u{001b}[201~");
        self.write(session_id, &wrapped).await
    }

    /// Terminates a process session.
    pub async fn kill(&self, session_id: u64) -> Result<()> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let mut child = handle.child.lock().await;

        #[cfg(unix)]
        {
            if let Some(pid) = child.id() {
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .status()
                    .await;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        if child.try_wait().ok().flatten().is_none() {
            child
                .kill()
                .await
                .map_err(|err| ToolError::Process(err.to_string()))?;
        }

        Ok(())
    }
}

fn expand_key_sequence(input: &str) -> String {
    let mut out = input.to_string();
    let mappings = [
        ("<enter>", "\n"),
        ("<tab>", "\t"),
        ("<esc>", "\u{001b}"),
        ("<up>", "\u{001b}[A"),
        ("<down>", "\u{001b}[B"),
        ("<right>", "\u{001b}[C"),
        ("<left>", "\u{001b}[D"),
        ("<ctrl-c>", "\u{0003}"),
        ("<ctrl-d>", "\u{0004}"),
    ];
    for (needle, value) in mappings {
        out = out.replace(needle, value);
    }
    out
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn shell_command(command: &str) -> Command {
    #[cfg(target_family = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(command);
        cmd
    }
    #[cfg(not(target_family = "windows"))]
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-lc").arg(command);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_poll_log_and_kill() {
        let manager = ProcessManager::new();
        let id = manager
            .spawn("echo hello", None, &HashMap::new())
            .await
            .expect("spawn");
        let chunk = manager.poll_wait(id, 500).await.expect("poll wait");
        assert!(chunk.text.contains("hello") || chunk.exited);
        let summary = manager.poll(id).await.expect("poll");
        assert_eq!(summary.session_id, id);
        let _ = manager.kill(id).await;
    }

    #[tokio::test]
    async fn submit_closes_stdin() {
        let manager = ProcessManager::new();
        let id = manager
            .spawn("cat", None, &HashMap::new())
            .await
            .expect("spawn");
        manager.submit(id, Some("a\n")).await.expect("submit");
        let chunk = manager.poll_wait(id, 300).await.expect("poll");
        assert!(chunk.text.contains("a") || chunk.exited);
        let _ = manager.kill(id).await;
    }

    #[test]
    fn key_expander_translates_sequences() {
        let text = expand_key_sequence("hi<enter><tab><up>");
        assert!(text.contains('\n'));
        assert!(text.contains('\t'));
        assert!(text.contains("\u{001b}[A"));
    }
}
