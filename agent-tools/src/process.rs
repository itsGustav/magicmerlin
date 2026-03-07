//! Background process/session manager used by exec/process tools.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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
    /// Whether process has exited.
    pub exited: bool,
}

struct ProcessHandle {
    command: String,
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    log: Arc<Mutex<String>>,
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
            .map_err(|err| ToolError::Process(err.to_string()))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdin = child.stdin.take();

        let log = Arc::new(Mutex::new(String::new()));
        if let Some(mut out) = stdout {
            let log_clone = log.clone();
            tokio::spawn(async move {
                let mut buf = vec![0_u8; 4096];
                loop {
                    match out.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let chunk = String::from_utf8_lossy(&buf[..n]);
                            let mut lock = log_clone.lock().await;
                            lock.push_str(&chunk);
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        if let Some(mut err) = stderr {
            let log_clone = log.clone();
            tokio::spawn(async move {
                let mut buf = vec![0_u8; 4096];
                loop {
                    match err.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let chunk = String::from_utf8_lossy(&buf[..n]);
                            let mut lock = log_clone.lock().await;
                            lock.push_str(&chunk);
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
            log,
        };

        self.handles.lock().await.insert(id, handle);
        Ok(id)
    }

    /// Lists active and finished process sessions.
    pub async fn list(&self) -> Vec<ProcessSummary> {
        let handles = self.handles.lock().await;
        let mut out = Vec::new();
        for (id, handle) in handles.iter() {
            let exited = {
                let mut child = handle.child.lock().await;
                child.try_wait().ok().flatten().is_some()
            };
            out.push(ProcessSummary {
                session_id: *id,
                command: handle.command.clone(),
                exited,
            });
        }
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

    /// Reads session log with offset/limit slicing.
    pub async fn log(&self, session_id: u64, offset: usize, limit: usize) -> Result<String> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let log = handle.log.lock().await;
        let sliced = log.chars().skip(offset).take(limit).collect::<String>();
        Ok(sliced)
    }

    /// Polls one session for process status.
    pub async fn poll(&self, session_id: u64) -> Result<ProcessSummary> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let exited = {
            let mut child = handle.child.lock().await;
            child.try_wait().ok().flatten().is_some()
        };

        Ok(ProcessSummary {
            session_id,
            command: handle.command.clone(),
            exited,
        })
    }

    /// Terminates a process session.
    pub async fn kill(&self, session_id: u64) -> Result<()> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(&session_id)
            .ok_or_else(|| ToolError::Process(format!("unknown session {session_id}")))?;
        let mut child = handle.child.lock().await;
        child
            .kill()
            .await
            .map_err(|err| ToolError::Process(err.to_string()))
    }
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
