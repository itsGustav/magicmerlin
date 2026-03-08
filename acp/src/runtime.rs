use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use crate::config::{AgentHarnessConfig, AgentId};

/// ACP session state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AcpSessionStatus {
    /// Session is queued for launch.
    Queued,
    /// Session process is running.
    Running,
    /// Session completed successfully.
    Completed,
    /// Session failed.
    Failed,
    /// Session was rejected by policy.
    Rejected,
}

/// ACP session model persisted for thread binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpSession {
    /// Unique session ID.
    pub id: String,
    /// Chat thread ID that owns this session.
    pub thread_id: String,
    /// Agent identity.
    pub agent: AgentId,
    /// Current status.
    pub status: AcpSessionStatus,
    /// Spawn command.
    pub command: String,
    /// Spawn args.
    pub args: Vec<String>,
    /// UNIX timestamp when started.
    pub started_at: u64,
    /// UNIX timestamp when last active.
    pub updated_at: u64,
    /// Optional completion timestamp.
    pub ended_at: Option<u64>,
    /// Exit code when process exits.
    pub exit_code: Option<i32>,
    /// Captured stdout lines.
    pub stdout: Vec<String>,
    /// Captured stderr lines.
    pub stderr: Vec<String>,
}

/// Request for spawning an ACP session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpSessionRequest {
    /// Thread this session belongs to.
    pub thread_id: String,
    /// Target agent.
    pub agent: AgentId,
    /// Process command.
    pub command: String,
    /// Process arguments.
    #[serde(default)]
    pub args: Vec<String>,
}

/// ACPX dispatch request for external protocol integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpxRequest {
    /// Thread this session belongs to.
    pub thread_id: String,
    /// Target agent.
    pub agent: AgentId,
    /// Process command.
    pub command: String,
    /// Process arguments.
    #[serde(default)]
    pub args: Vec<String>,
}

/// Stream event for session output and lifecycle updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpEvent {
    /// Session ID that emitted the event.
    pub session_id: String,
    /// Event type string.
    pub kind: String,
    /// Event payload.
    pub payload: serde_json::Value,
}

/// Receiver type for ACP event streaming.
pub type AcpEventStream = broadcast::Receiver<AcpEvent>;

/// ACP runtime with control plane and thread-bound session persistence.
#[derive(Clone)]
pub struct AcpRuntime {
    config: AgentHarnessConfig,
    sessions: Arc<Mutex<HashMap<String, AcpSession>>>,
    events: broadcast::Sender<AcpEvent>,
    state_file: PathBuf,
}

impl AcpRuntime {
    /// Creates runtime backed by a state directory.
    pub fn new(state_dir: &Path, config: AgentHarnessConfig) -> Result<Self> {
        fs::create_dir_all(state_dir)
            .with_context(|| format!("create state dir {}", state_dir.display()))?;
        let state_file = state_dir.join("acp_sessions.json");
        let sessions = load_sessions_from_file(&state_file)?;
        let (events, _) = broadcast::channel(1024);

        Ok(Self {
            config,
            sessions: Arc::new(Mutex::new(sessions)),
            events,
            state_file,
        })
    }

    /// Subscribes to real-time ACP events.
    pub fn subscribe(&self) -> AcpEventStream {
        self.events.subscribe()
    }

    /// Returns all known sessions.
    pub async fn list_sessions(&self) -> Vec<AcpSession> {
        self.sessions.lock().await.values().cloned().collect()
    }

    /// Returns sessions for one thread.
    pub async fn sessions_for_thread(&self, thread_id: &str) -> Vec<AcpSession> {
        self.sessions
            .lock()
            .await
            .values()
            .filter(|session| session.thread_id == thread_id)
            .cloned()
            .collect()
    }

    /// Spawns a session process and begins streaming events.
    pub async fn spawn_session(&self, request: AcpSessionRequest) -> Result<AcpSession> {
        if !self.config.is_allowed(&request.agent) {
            let session = self
                .insert_rejected_session(request, "agent is not allowed")
                .await?;
            return Ok(session);
        }

        let active_count = self
            .sessions
            .lock()
            .await
            .values()
            .filter(|session| session.status == AcpSessionStatus::Running)
            .count();
        if active_count >= self.config.max_concurrent_sessions {
            anyhow::bail!("maxConcurrentSessions limit reached");
        }

        let now = epoch_now();
        let session_id = Uuid::new_v4().to_string();
        let session = AcpSession {
            id: session_id.clone(),
            thread_id: request.thread_id.clone(),
            agent: request.agent.clone(),
            status: AcpSessionStatus::Running,
            command: request.command.clone(),
            args: request.args.clone(),
            started_at: now,
            updated_at: now,
            ended_at: None,
            exit_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        };

        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session_id.clone(), session.clone());
            persist_sessions(&self.state_file, &sessions)?;
        }

        self.emit_event(
            &session_id,
            "started",
            serde_json::json!({"threadId": request.thread_id, "agent": request.agent.as_str()}),
        );

        let runtime = self.clone();
        tokio::spawn(async move {
            let _ = runtime.run_process(session_id, request.command, request.args).await;
        });

        Ok(session)
    }

    /// ACPX entry point that dispatches directly to ACP session spawning.
    pub async fn dispatch_acpx(&self, request: AcpxRequest) -> Result<AcpSession> {
        self.spawn_session(AcpSessionRequest {
            thread_id: request.thread_id,
            agent: request.agent,
            command: request.command,
            args: request.args,
        })
        .await
    }

    /// Removes sessions older than configured TTL when they are terminal.
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let now = epoch_now();
        let ttl = self.config.ttl_seconds;

        let mut sessions = self.sessions.lock().await;
        let before = sessions.len();
        sessions.retain(|_, session| {
            let terminal = matches!(
                session.status,
                AcpSessionStatus::Completed | AcpSessionStatus::Failed | AcpSessionStatus::Rejected
            );
            if !terminal {
                return true;
            }
            now.saturating_sub(session.updated_at) < ttl
        });
        persist_sessions(&self.state_file, &sessions)?;
        Ok(before.saturating_sub(sessions.len()))
    }

    async fn run_process(&self, session_id: String, command: String, args: Vec<String>) -> Result<()> {
        let mut child = Command::new(&command)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("spawn command {command}"))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(stdout) = stdout {
            let runtime = self.clone();
            let session_id = session_id.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = runtime.append_stdout(&session_id, line).await;
                }
            });
        }

        if let Some(stderr) = stderr {
            let runtime = self.clone();
            let session_id = session_id.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = runtime.append_stderr(&session_id, line).await;
                }
            });
        }

        let status = child.wait().await.context("wait child")?;
        self.complete_session(&session_id, status.code()).await?;
        Ok(())
    }

    async fn append_stdout(&self, session_id: &str, line: String) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        let Some(session) = sessions.get_mut(session_id) else {
            return Ok(());
        };
        session.stdout.push(line.clone());
        session.updated_at = epoch_now();
        let thread_id = session.thread_id.clone();
        persist_sessions(&self.state_file, &sessions)?;
        self.emit_event(
            session_id,
            "stdout",
            serde_json::json!({"line": line, "threadId": thread_id}),
        );
        Ok(())
    }

    async fn append_stderr(&self, session_id: &str, line: String) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        let Some(session) = sessions.get_mut(session_id) else {
            return Ok(());
        };
        session.stderr.push(line.clone());
        session.updated_at = epoch_now();
        let thread_id = session.thread_id.clone();
        persist_sessions(&self.state_file, &sessions)?;
        self.emit_event(
            session_id,
            "stderr",
            serde_json::json!({"line": line, "threadId": thread_id}),
        );
        Ok(())
    }

    async fn complete_session(&self, session_id: &str, code: Option<i32>) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        let Some(session) = sessions.get_mut(session_id) else {
            return Ok(());
        };

        let now = epoch_now();
        session.ended_at = Some(now);
        session.updated_at = now;
        session.exit_code = code;
        session.status = if code == Some(0) {
            AcpSessionStatus::Completed
        } else {
            AcpSessionStatus::Failed
        };
        let status = session.status.clone();

        persist_sessions(&self.state_file, &sessions)?;
        self.emit_event(
            session_id,
            "completed",
            serde_json::json!({"exitCode": code, "status": format!("{:?}", status)}),
        );
        Ok(())
    }

    async fn insert_rejected_session(
        &self,
        request: AcpSessionRequest,
        reason: &str,
    ) -> Result<AcpSession> {
        let now = epoch_now();
        let session = AcpSession {
            id: Uuid::new_v4().to_string(),
            thread_id: request.thread_id,
            agent: request.agent,
            status: AcpSessionStatus::Rejected,
            command: request.command,
            args: request.args,
            started_at: now,
            updated_at: now,
            ended_at: Some(now),
            exit_code: None,
            stdout: Vec::new(),
            stderr: vec![reason.to_string()],
        };

        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session.id.clone(), session.clone());
            persist_sessions(&self.state_file, &sessions)?;
        }

        self.emit_event(
            &session.id,
            "rejected",
            serde_json::json!({"reason": reason, "threadId": session.thread_id}),
        );
        Ok(session)
    }

    fn emit_event(&self, session_id: &str, kind: &str, payload: serde_json::Value) {
        let _ = self.events.send(AcpEvent {
            session_id: session_id.to_string(),
            kind: kind.to_string(),
            payload,
        });
    }
}

fn load_sessions_from_file(path: &Path) -> Result<HashMap<String, AcpSession>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let sessions = serde_json::from_str::<HashMap<String, AcpSession>>(&body)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(sessions)
}

fn persist_sessions(path: &Path, sessions: &HashMap<String, AcpSession>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create state dir {}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(sessions)?;
    fs::write(path, format!("{body}\n")).with_context(|| format!("write {}", path.display()))
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn runtime_with_config(max_concurrent_sessions: usize, ttl_seconds: u64) -> AcpRuntime {
        let tmp = std::env::temp_dir().join(format!("magicmerlin-acp-{}", Uuid::new_v4()));
        fs::create_dir_all(&tmp).expect("tmp create");
        let config = AgentHarnessConfig {
            max_concurrent_sessions,
            ttl_seconds,
            ..AgentHarnessConfig::default()
        };
        AcpRuntime::new(&tmp, config).expect("runtime")
    }

    #[tokio::test]
    async fn rejects_disallowed_agents() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut config = AgentHarnessConfig::default();
        config.allowed_agents.clear();
        let runtime = AcpRuntime::new(tmp.path(), config).expect("runtime");

        let session = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "t1".to_string(),
                agent: AgentId::Codex,
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
            })
            .await
            .expect("spawn");

        assert_eq!(session.status, AcpSessionStatus::Rejected);
    }

    #[tokio::test]
    async fn spawns_session_and_captures_output() {
        let runtime = runtime_with_config(4, 3600);

        let session = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-a".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo hello".to_string()],
            })
            .await
            .expect("spawn");

        tokio::time::sleep(Duration::from_millis(250)).await;
        let sessions = runtime.list_sessions().await;
        let found = sessions
            .iter()
            .find(|s| s.id == session.id)
            .expect("session");
        assert!(matches!(
            found.status,
            AcpSessionStatus::Completed | AcpSessionStatus::Running
        ));
    }

    #[tokio::test]
    async fn acpx_dispatch_works() {
        let runtime = runtime_with_config(4, 3600);
        let session = runtime
            .dispatch_acpx(AcpxRequest {
                thread_id: "thread-x".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo acpx".to_string()],
            })
            .await
            .expect("dispatch");
        assert_eq!(session.thread_id, "thread-x");
    }

    #[tokio::test]
    async fn lists_sessions_for_thread() {
        let runtime = runtime_with_config(4, 3600);

        let _ = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-1".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo 1".to_string()],
            })
            .await
            .expect("spawn1");

        let _ = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-2".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo 2".to_string()],
            })
            .await
            .expect("spawn2");

        let list = runtime.sessions_for_thread("thread-1").await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].thread_id, "thread-1");
    }

    #[tokio::test]
    async fn enforces_concurrency_limit() {
        let runtime = runtime_with_config(1, 3600);

        let _ = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-a".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "sleep 1".to_string()],
            })
            .await
            .expect("spawn");

        let err = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-b".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo x".to_string()],
            })
            .await
            .expect_err("limit error");

        assert!(err.to_string().contains("maxConcurrentSessions"));
    }

    #[tokio::test]
    async fn emits_events() {
        let runtime = runtime_with_config(4, 3600);
        let mut events = runtime.subscribe();

        let _ = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-a".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo event".to_string()],
            })
            .await
            .expect("spawn");

        let event = tokio::time::timeout(Duration::from_secs(2), events.recv())
            .await
            .expect("event timeout")
            .expect("event recv");
        assert!(matches!(event.kind.as_str(), "started" | "stdout" | "completed"));
    }

    #[tokio::test]
    async fn cleanup_removes_terminal_sessions_after_ttl() {
        let runtime = runtime_with_config(4, 0);

        let _ = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-a".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo done".to_string()],
            })
            .await
            .expect("spawn");

        tokio::time::sleep(Duration::from_millis(200)).await;
        let removed = runtime.cleanup_expired().await.expect("cleanup");
        assert!(removed >= 1);
    }

    #[tokio::test]
    async fn state_is_persisted_and_reloaded() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let runtime = AcpRuntime::new(tmp.path(), AgentHarnessConfig::default()).expect("runtime");

        let _ = runtime
            .spawn_session(AcpSessionRequest {
                thread_id: "thread-a".to_string(),
                agent: AgentId::Codex,
                command: "sh".to_string(),
                args: vec!["-c".to_string(), "echo persist".to_string()],
            })
            .await
            .expect("spawn");

        tokio::time::sleep(Duration::from_millis(150)).await;
        let reloaded = AcpRuntime::new(tmp.path(), AgentHarnessConfig::default()).expect("reload");
        assert!(!reloaded.list_sessions().await.is_empty());
    }
}
