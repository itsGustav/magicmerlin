use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::{start_chrome, stop_chrome, BrowserLaunchOptions, BrowserProcess};
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserProfile {
    Default,
    Relay,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BrowserStatus {
    pub running: bool,
    pub port: u16,
    pub profile: BrowserProfile,
    pub launched_at_unix_ms: Option<u128>,
    pub uptime_ms: Option<u128>,
    pub debugging_url: Option<String>,
    pub user_data_dir: Option<PathBuf>,
}

#[derive(Debug)]
struct BrowserHandle {
    process: BrowserProcess,
    profile: BrowserProfile,
    port: u16,
    launched_at: Instant,
    launched_at_unix_ms: u128,
    user_data_dir: Option<PathBuf>,
}

#[derive(Clone, Default)]
pub struct BrowserManager {
    handles: Arc<Mutex<HashMap<u16, BrowserHandle>>>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(
        &self,
        mut options: BrowserLaunchOptions,
        profile: BrowserProfile,
    ) -> Result<BrowserStatus> {
        let user_data = self.profile_user_data_dir(profile, options.remote_debugging_port);
        if options.user_data_dir.is_none() {
            options.user_data_dir = user_data.clone();
        }

        if self.is_running(options.remote_debugging_port).await {
            return self.status(options.remote_debugging_port).await;
        }

        let started = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let process = start_chrome(options.clone()).await?;
        let handle = BrowserHandle {
            process,
            profile,
            port: options.remote_debugging_port,
            launched_at: Instant::now(),
            launched_at_unix_ms: started,
            user_data_dir: options.user_data_dir,
        };

        self.handles.lock().await.insert(handle.port, handle);
        self.status(options.remote_debugging_port).await
    }

    pub async fn stop(&self, port: u16) -> Result<()> {
        let mut handles = self.handles.lock().await;
        let Some(mut handle) = handles.remove(&port) else {
            return Ok(());
        };
        stop_chrome(&mut handle.process).await
    }

    pub async fn stop_all(&self) -> Result<()> {
        let ports = self
            .handles
            .lock()
            .await
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for port in ports {
            self.stop(port).await?;
        }
        Ok(())
    }

    pub async fn is_running(&self, port: u16) -> bool {
        self.handles.lock().await.contains_key(&port)
    }

    pub async fn list_ports(&self) -> Vec<u16> {
        let mut ports = self
            .handles
            .lock()
            .await
            .keys()
            .copied()
            .collect::<Vec<_>>();
        ports.sort_unstable();
        ports
    }

    pub async fn status(&self, port: u16) -> Result<BrowserStatus> {
        let handles = self.handles.lock().await;
        let Some(handle) = handles.get(&port) else {
            return Ok(BrowserStatus {
                running: false,
                port,
                profile: BrowserProfile::Default,
                launched_at_unix_ms: None,
                uptime_ms: None,
                debugging_url: None,
                user_data_dir: None,
            });
        };

        Ok(BrowserStatus {
            running: true,
            port,
            profile: handle.profile,
            launched_at_unix_ms: Some(handle.launched_at_unix_ms),
            uptime_ms: Some(handle.launched_at.elapsed().as_millis()),
            debugging_url: Some(handle.process.debugging_url.clone()),
            user_data_dir: handle.user_data_dir.clone(),
        })
    }

    fn profile_user_data_dir(&self, profile: BrowserProfile, port: u16) -> Option<PathBuf> {
        let root = std::env::temp_dir().join("magicmerlin-browser");
        let suffix = match profile {
            BrowserProfile::Default => "default",
            BrowserProfile::Relay => "relay",
        };
        Some(root.join(format!("{suffix}-{port}")))
    }

    pub fn launch_options_for_profile(
        profile: BrowserProfile,
        port: u16,
        startup_timeout: Duration,
    ) -> BrowserLaunchOptions {
        let mut opts = BrowserLaunchOptions::default();
        opts.remote_debugging_port = port;
        opts.startup_timeout = startup_timeout;
        if profile == BrowserProfile::Relay {
            opts.headless = false;
            opts.extra_args.push("--start-maximized".to_string());
            opts.extra_args.push("--disable-extensions".to_string());
        }
        opts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MediaError;

    #[tokio::test]
    async fn default_status_is_not_running() {
        let manager = BrowserManager::new();
        let status = manager.status(9555).await.expect("status");
        assert!(!status.running);
        assert_eq!(status.port, 9555);
    }

    #[tokio::test]
    async fn list_ports_is_sorted() {
        let manager = BrowserManager::new();
        manager.handles.lock().await.insert(
            9005,
            BrowserHandle {
                process: BrowserProcess {
                    child: tokio::process::Command::new("sh")
                        .arg("-lc")
                        .arg("sleep 2")
                        .spawn()
                        .map_err(|e| MediaError::Execution(e.to_string()))
                        .expect("spawn"),
                    debugging_url: "http://127.0.0.1:9005".to_string(),
                },
                profile: BrowserProfile::Default,
                port: 9005,
                launched_at: Instant::now(),
                launched_at_unix_ms: 1,
                user_data_dir: None,
            },
        );
        manager.handles.lock().await.insert(
            9001,
            BrowserHandle {
                process: BrowserProcess {
                    child: tokio::process::Command::new("sh")
                        .arg("-lc")
                        .arg("sleep 2")
                        .spawn()
                        .map_err(|e| MediaError::Execution(e.to_string()))
                        .expect("spawn"),
                    debugging_url: "http://127.0.0.1:9001".to_string(),
                },
                profile: BrowserProfile::Default,
                port: 9001,
                launched_at: Instant::now(),
                launched_at_unix_ms: 1,
                user_data_dir: None,
            },
        );

        let ports = manager.list_ports().await;
        assert_eq!(ports, vec![9001, 9005]);
        let _ = manager.stop_all().await;
    }
}
