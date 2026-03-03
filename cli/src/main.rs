use std::{
    fs,
    net::IpAddr,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "magicmerlin")]
#[command(about = "MagicMerlin CLI (OpenClaw-shaped)")]
struct Args {
    #[command(subcommand)]
    command: CommandGroup,
}

#[derive(Subcommand, Debug)]
enum CommandGroup {
    /// Run the onboarding wizard (config + optional daemon setup)
    Onboard {
        /// Install a user daemon (macOS LaunchAgent). Prints the bootstrap command.
        #[arg(long)]
        install_daemon: bool,

        /// Port for the gateway web UI.
        #[arg(long, default_value_t = 18789)]
        port: u16,

        /// Bind address.
        #[arg(long, default_value = "127.0.0.1")]
        bind: IpAddr,
    },

    /// Gateway controls
    Gateway {
        #[command(subcommand)]
        command: GatewayCommand,
    },

    /// Open the Control UI in a browser
    Dashboard {
        #[arg(long)]
        port: Option<u16>,

        #[arg(long)]
        bind: Option<IpAddr>,
    },

    /// Send a test message (minimal)
    Message {
        #[command(subcommand)]
        command: MessageCommand,
    },
}

#[derive(Subcommand, Debug)]
enum GatewayCommand {
    /// Check whether the gateway is running
    Status,

    /// Run gateway in the foreground (similar to `openclaw gateway --port`)
    Run {
        #[arg(long, default_value_t = 18789)]
        port: u16,

        #[arg(long, default_value = "127.0.0.1")]
        bind: IpAddr,

        /// Start scheduler loop
        #[arg(long, default_value_t = true)]
        daemon: bool,
    },
}

#[derive(Subcommand, Debug)]
enum MessageCommand {
    /// Send a message
    Send {
        /// Target: `local` (Control UI) or an http/https webhook URL.
        #[arg(long)]
        target: String,

        #[arg(long)]
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MagicMerlinConfig {
    port: u16,
    bind: String,
}

fn state_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MAGICMERLIN_STATE_DIR") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("MAGICMERLIN_HOME") {
        return PathBuf::from(p);
    }

    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    home.join(".magicmerlin")
}

fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("MAGICMERLIN_CONFIG_PATH") {
        return PathBuf::from(p);
    }
    state_dir().join("config.json")
}

fn load_or_default_config() -> Result<MagicMerlinConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(MagicMerlinConfig {
            port: 18789,
            bind: "127.0.0.1".to_string(),
        });
    }
    let raw = fs::read_to_string(&path).context("read config")?;
    let cfg: MagicMerlinConfig = serde_json::from_str(&raw).context("parse config")?;
    Ok(cfg)
}

fn write_config(cfg: &MagicMerlinConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&path, serde_json::to_string_pretty(cfg)? + "\n").context("write config")?;
    Ok(())
}

fn base_url(cfg: &MagicMerlinConfig) -> String {
    format!("http://{}:{}", cfg.bind, cfg.port)
}

fn gateway_health(cfg: &MagicMerlinConfig) -> Result<()> {
    let url = format!("{}/health", base_url(cfg));
    let res = reqwest::blocking::get(url).context("GET /health")?;
    if !res.status().is_success() {
        return Err(anyhow!("gateway unhealthy: {}", res.status()));
    }
    Ok(())
}

fn spawn_gateway_background(cfg: &MagicMerlinConfig, daemon: bool) -> Result<()> {
    let mut cmd = Command::new("magicmerlin-gateway");
    cmd.arg("--serve")
        .arg(cfg.port.to_string())
        .arg("--bind")
        .arg(&cfg.bind);

    if daemon {
        cmd.arg("--daemon");
    }

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    cmd.spawn().context("spawn magicmerlin-gateway")?;

    // Give it a moment.
    thread::sleep(Duration::from_millis(900));
    Ok(())
}

fn open_url(url: &str) {
    // Best-effort.
    let _ = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", url]).status()
    } else {
        Command::new("xdg-open").arg(url).status()
    };
}

fn find_in_path(bin: &str) -> Option<PathBuf> {
    let out = Command::new("bash")
        .args(["-lc", &format!("command -v {bin}")])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

fn write_launch_agent(port: u16, bind: &IpAddr) -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("HOME not set"))?;
    let plist_path = home
        .join("Library")
        .join("LaunchAgents")
        .join("ai.magicmerlin.gateway.plist");

    let gateway_bin = find_in_path("magicmerlin-gateway")
        .ok_or_else(|| anyhow!("magicmerlin-gateway not found in PATH"))?;

    let program_args = format!(
        "<array>\n  <string>{}</string>\n  <string>--serve</string>\n  <string>{}</string>\n  <string>--bind</string>\n  <string>{}</string>\n  <string>--daemon</string>\n</array>",
        gateway_bin.display(),
        port,
        bind
    );

    let plist = format!(
        r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
  <key>Label</key><string>ai.magicmerlin.gateway</string>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>ProgramArguments</key>
  {program_args}
</dict>
</plist>
"#
    );

    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&plist_path, plist).context("write plist")?;

    Ok(plist_path)
}

fn codex_login_status() -> Result<String> {
    let out = Command::new("codex")
        .args(["login", "status"])
        .output()
        .context("run codex login status")?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    Ok(s)
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        CommandGroup::Onboard {
            install_daemon,
            port,
            bind,
        } => {
            let sd = state_dir();
            fs::create_dir_all(&sd).ok();

            let cfg = MagicMerlinConfig {
                port,
                bind: bind.to_string(),
            };
            write_config(&cfg)?;

            eprintln!("Wrote config: {}", config_path().display());

            // Check Codex OAuth
            let status =
                codex_login_status().unwrap_or_else(|_| "(codex not available)".to_string());
            if !status.to_lowercase().contains("logged in") {
                eprintln!("Codex not logged in. Run: codex login");
            } else {
                eprintln!("Codex status: {}", status.trim());
            }

            if install_daemon {
                if cfg!(target_os = "macos") {
                    let plist = write_launch_agent(port, &bind)?;
                    eprintln!("Wrote LaunchAgent: {}", plist.display());
                    eprintln!(
                        "Enable it with: launchctl bootstrap gui/$UID {}",
                        plist.display()
                    );
                } else {
                    eprintln!("--install-daemon is currently implemented for macOS only.");
                }
            }

            Ok(())
        }

        CommandGroup::Gateway { command } => {
            let cfg = load_or_default_config()?;
            match command {
                GatewayCommand::Status => gateway_health(&cfg)
                    .map(|_| {
                        println!("ok {}", base_url(&cfg));
                    })
                    .or_else(|e| {
                        println!("down {} ({})", base_url(&cfg), e);
                        Ok(())
                    }),

                GatewayCommand::Run { port, bind, daemon } => {
                    let mut cmd = Command::new("magicmerlin-gateway");
                    cmd.arg("--serve")
                        .arg(port.to_string())
                        .arg("--bind")
                        .arg(bind.to_string());
                    if daemon {
                        cmd.arg("--daemon");
                    }
                    let status = cmd.status().context("run magicmerlin-gateway")?;
                    if !status.success() {
                        return Err(anyhow!("gateway exited non-zero: {status}"));
                    }
                    Ok(())
                }
            }
        }

        CommandGroup::Dashboard { port, bind } => {
            let mut cfg = load_or_default_config()?;
            if let Some(p) = port {
                cfg.port = p;
            }
            if let Some(b) = bind {
                cfg.bind = b.to_string();
            }

            // Ensure running.
            if gateway_health(&cfg).is_err() {
                spawn_gateway_background(&cfg, true)?;
            }

            let url = base_url(&cfg);
            println!("{url}");
            open_url(&url);
            Ok(())
        }

        CommandGroup::Message { command } => match command {
            MessageCommand::Send { target, message } => {
                if target == "local" {
                    let cfg = load_or_default_config()?;
                    if gateway_health(&cfg).is_err() {
                        spawn_gateway_background(&cfg, true)?;
                    }
                    let url = format!("{}/chat", base_url(&cfg));
                    let client = reqwest::blocking::Client::new();
                    let res: serde_json::Value = client
                        .post(url)
                        .json(&serde_json::json!({"message": message}))
                        .send()
                        .context("POST /chat")?
                        .json()
                        .context("parse json")?;
                    println!(
                        "{}",
                        res.get("reply").and_then(|v| v.as_str()).unwrap_or("")
                    );
                    return Ok(());
                }

                if target.starts_with("http://") || target.starts_with("https://") {
                    let client = reqwest::blocking::Client::new();
                    let res = client
                        .post(&target)
                        .json(&serde_json::json!({"message": message}))
                        .send()
                        .context("POST webhook")?;
                    println!("{}", res.status());
                    return Ok(());
                }

                Err(anyhow!(
                    "unsupported target: {target} (use --target local or a webhook URL)"
                ))
            }
        },
    }
}
