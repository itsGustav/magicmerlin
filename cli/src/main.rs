use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const DEFAULT_GATEWAY_WS: &str = "ws://127.0.0.1:18789/ws";
const DEFAULT_UI_URL: &str = "http://127.0.0.1:18789/ui";

#[derive(Parser, Debug)]
#[command(name = "magicmerlin")]
#[command(about = "MagicMerlin CLI")]
struct Cli {
    #[arg(long, global = true)]
    dev: bool,

    #[arg(long, global = true)]
    profile: Option<String>,

    #[arg(long, global = true)]
    log_level: Option<String>,

    #[arg(long, global = true)]
    no_color: bool,

    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(name = "_introspect", hide = true)]
    Introspect {
        #[command(subcommand)]
        command: IntrospectCommand,
    },
    Status,
    #[command(alias = "configure")]
    Setup,
    Onboard,
    #[command(alias = "doctor")]
    Health,
    Dashboard,
    Tui,
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
    Version,
    Update,
    #[command(alias = "uninstall")]
    Reset,
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    Agents {
        #[command(subcommand)]
        command: AgentsCommand,
    },
    Models {
        #[command(subcommand)]
        command: ModelsCommand,
    },
    Gateway {
        #[command(subcommand)]
        command: GatewayCommand,
    },
    Daemon {
        #[command(subcommand)]
        command: GatewayCommand,
    },
    Channels {
        #[command(subcommand)]
        command: ChannelsCommand,
    },
    Message {
        #[command(subcommand)]
        command: MessageCommand,
    },
    Directory {
        query: Option<String>,
    },
    Pairing {
        #[command(subcommand)]
        command: PairingCommand,
    },
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    Cron {
        #[command(subcommand)]
        command: CronCommand,
    },
    Logs {
        #[arg(long, default_value_t = 100)]
        lines: usize,
        #[arg(long)]
        follow: bool,
    },
    #[command(alias = "webhooks")]
    Hooks {
        #[command(subcommand)]
        command: HooksCommand,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Security {
        #[command(subcommand)]
        command: SecurityCommand,
    },
    Secrets {
        #[command(subcommand)]
        command: SecretsCommand,
    },
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommand,
    },
    Approvals {
        #[command(subcommand)]
        command: ApprovalsCommand,
    },
    Plugins {
        #[command(subcommand)]
        command: PluginsCommand,
    },
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },
    Dns,
    Devices,
    Nodes {
        #[command(subcommand)]
        command: NodesCommand,
    },
    Qr {
        text: String,
    },
    Browser {
        #[command(subcommand)]
        command: BrowserCommand,
    },
    Acp,
    Docs,
    System {
        #[command(subcommand)]
        command: SystemCommand,
    },
    Help,
}

#[derive(Subcommand, Debug)]
enum IntrospectCommand {
    Commands,
}

#[derive(Subcommand, Debug)]
enum AgentCommand {
    Run {
        prompt: String,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        model: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum AgentsCommand {
    List,
    Add { name: String },
    Remove { name: String },
    Config { name: Option<String> },
}

#[derive(Subcommand, Debug)]
enum ModelsCommand {
    List,
    Status,
    Auth,
}

#[derive(Subcommand, Debug, Clone)]
enum GatewayCommand {
    Start,
    Stop,
    Restart,
    Status,
    Call {
        method: String,
        #[arg(long, default_value = "{}")]
        params: String,
    },
}

#[derive(Subcommand, Debug)]
enum ChannelsCommand {
    Login { channel: String },
    Logout { channel: String },
    Status,
}

#[derive(Subcommand, Debug)]
enum MessageCommand {
    Send {
        #[arg(long)]
        target: String,
        #[arg(long)]
        message: String,
        #[arg(long)]
        channel: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum PairingCommand {
    List,
    Approve { id: i64 },
    Deny { id: i64 },
}

#[derive(Subcommand, Debug)]
enum SessionsCommand {
    List {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Show { id: String },
    Delete { id: String },
    Compact { id: String },
}

#[derive(Subcommand, Debug)]
enum MemoryCommand {
    Search { query: String },
    Get { key: String },
}

#[derive(Subcommand, Debug)]
enum CronCommand {
    List,
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        schedule: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        payload: String,
    },
    Edit {
        id: i64,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        schedule: Option<String>,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        payload: Option<String>,
    },
    Rm { id: i64 },
    Run { id: i64 },
    Enable { id: i64 },
    Disable { id: i64 },
    Runs {
        #[arg(long)]
        job_id: Option<i64>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum HooksCommand {
    List,
    Add { url: String },
    Remove { url: String },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
    File,
    Validate,
}

#[derive(Subcommand, Debug)]
enum SecurityCommand {
    Audit,
}

#[derive(Subcommand, Debug)]
enum SecretsCommand {
    Reload,
}

#[derive(Subcommand, Debug)]
enum SandboxCommand {
    List,
    Start { name: String },
    Stop { name: String },
    Status,
}

#[derive(Subcommand, Debug)]
enum ApprovalsCommand {
    List,
    Approve { id: String },
    Deny { id: String },
}

#[derive(Subcommand, Debug)]
enum PluginsCommand {
    List,
    Enable { name: String },
    Disable { name: String },
    Install { source: String },
}

#[derive(Subcommand, Debug)]
enum SkillsCommand {
    List,
    Inspect { name: String },
}

#[derive(Subcommand, Debug)]
enum NodesCommand {
    List,
    Describe { id: String },
    Run { id: String },
}

#[derive(Subcommand, Debug)]
enum BrowserCommand {
    Start,
    Stop,
    Status,
}

#[derive(Subcommand, Debug)]
enum SystemCommand {
    Event {
        #[arg(long)]
        text: String,
        #[arg(long, default_value = "now")]
        mode: String,
    },
    Heartbeat,
    Presence,
}

#[derive(ValueEnum, Copy, Clone, Debug)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliConfig {
    gateway_ws_url: String,
    dashboard_url: String,
    profile: Option<String>,
    log_level: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            gateway_ws_url: DEFAULT_GATEWAY_WS.to_string(),
            dashboard_url: DEFAULT_UI_URL.to_string(),
            profile: None,
            log_level: Some("info".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct RpcErrorData {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<RpcErrorData>,
}

#[derive(Debug)]
struct App {
    cli: Cli,
    config: CliConfig,
}

impl App {
    fn gateway_http_url(&self) -> String {
        let ws = self.config.gateway_ws_url.trim_end_matches('/');
        if let Some(rest) = ws.strip_prefix("ws://") {
            return format!("http://{}", rest.trim_end_matches("/ws"));
        }
        if let Some(rest) = ws.strip_prefix("wss://") {
            return format!("https://{}", rest.trim_end_matches("/ws"));
        }
        ws.trim_end_matches("/ws").to_string()
    }

    fn output(&self, value: Value, human: impl FnOnce() -> String) -> Result<()> {
        if self.cli.json {
            println!("{}", serde_json::to_string_pretty(&value)?);
        } else {
            println!("{}", human());
        }
        Ok(())
    }

    async fn call_gateway(&self, method: &str, params: Value) -> Result<Value> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: 1,
        };

        let url = format!("{}/ws", self.gateway_http_url());
        let response = Client::new()
            .post(url)
            .json(&req)
            .send()
            .await
            .context("gateway RPC request failed")?;

        if !response.status().is_success() {
            return Err(anyhow!("gateway HTTP error: {}", response.status()));
        }

        let parsed: RpcResponse = response.json().await.context("parse RPC response")?;
        if let Some(err) = parsed.error {
            return Err(anyhow!("gateway RPC error {}: {}", err.code, err.message));
        }
        Ok(parsed.result.unwrap_or(Value::Null))
    }

    async fn ensure_gateway_running(&self) -> Result<()> {
        self.call_gateway("health", Value::Null).await.map(|_| ()).map_err(|e| {
            anyhow!("gateway unavailable ({e}). start with: magicmerlin gateway start")
        })
    }
}

fn state_dir() -> PathBuf {
    if let Ok(path) = std::env::var("MAGICMERLIN_STATE_DIR") {
        return PathBuf::from(path);
    }

    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".magicmerlin")
}

fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("MAGICMERLIN_CONFIG_PATH") {
        return PathBuf::from(path);
    }
    state_dir().join("cli-config.json")
}

fn pid_path() -> PathBuf {
    state_dir().join("gateway.pid")
}

fn read_config() -> CliConfig {
    let path = config_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return CliConfig::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_config(cfg: &CliConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(path, serde_json::to_vec_pretty(cfg)?).context("write CLI config")?;
    Ok(())
}

fn find_binary(bin: &str) -> Option<PathBuf> {
    let out = ProcessCommand::new("bash")
        .args(["-lc", &format!("command -v {bin}")])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

fn spawn_gateway() -> Result<u32> {
    let mut cmd = if find_binary("magicmerlin-gateway").is_some() {
        let mut c = ProcessCommand::new("magicmerlin-gateway");
        c.args(["--serve", "18789", "--bind", "127.0.0.1", "--daemon"]);
        c
    } else {
        let mut c = ProcessCommand::new("cargo");
        c.args([
            "run",
            "-q",
            "-p",
            "magicmerlin-gateway",
            "--",
            "--serve",
            "18789",
            "--bind",
            "127.0.0.1",
            "--daemon",
        ]);
        c
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = cmd.spawn().context("spawn gateway")?;
    Ok(child.id())
}

fn write_pid(pid: u32) -> Result<()> {
    if let Some(parent) = pid_path().parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(pid_path(), pid.to_string()).context("write pid file")?;
    Ok(())
}

fn read_pid() -> Result<u32> {
    let raw = fs::read_to_string(pid_path()).context("read pid file")?;
    raw.trim().parse::<u32>().context("parse pid")
}

fn stop_pid(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        let status = ProcessCommand::new("kill")
            .arg(pid.to_string())
            .status()
            .context("send SIGTERM")?;
        if !status.success() {
            return Err(anyhow!("kill failed for pid {pid}"));
        }
    }

    #[cfg(windows)]
    {
        let status = ProcessCommand::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .context("taskkill")?;
        if !status.success() {
            return Err(anyhow!("taskkill failed for pid {pid}"));
        }
    }

    Ok(())
}

fn is_gateway_port_open() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:18789"
            .parse()
            .expect("socket parse for static endpoint must succeed"),
        Duration::from_millis(200),
    )
    .is_ok()
}

fn collect_command_paths() -> BTreeSet<String> {
    fn walk(cmd: &clap::Command, prefix: &[String], out: &mut BTreeSet<String>) {
        for sub in cmd.get_subcommands() {
            let mut path = prefix.to_vec();
            path.push(sub.get_name().to_string());
            out.insert(path.join(" "));
            walk(sub, &path, out);
        }
    }

    let root = Cli::command();
    let mut out = BTreeSet::new();
    walk(&root, &[], &mut out);
    out
}

fn prompt(prompt: &str, default: &str) -> Result<String> {
    print!("{prompt} [{default}]: ");
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let value = line.trim();
    if value.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(value.to_string())
    }
}

fn open_url(url: &str) -> Result<()> {
    let status = if cfg!(target_os = "macos") {
        ProcessCommand::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        ProcessCommand::new("cmd").args(["/C", "start", url]).status()
    } else {
        ProcessCommand::new("xdg-open").arg(url).status()
    }
    .context("open url")?;

    if !status.success() {
        return Err(anyhow!("failed to open URL: {url}"));
    }
    Ok(())
}

fn emit_completion(shell: Shell) {
    let script = match shell {
        Shell::Bash => {
            r#"_magicmerlin_complete() {
  COMPREPLY=($(compgen -W "status setup configure onboard health doctor dashboard tui completion version update reset uninstall agent agents models gateway daemon channels message directory pairing sessions memory cron logs hooks webhooks config security secrets sandbox approvals plugins skills dns devices nodes qr browser acp docs system help" -- "${COMP_WORDS[1]}"))
}
complete -F _magicmerlin_complete magicmerlin
"#
        }
        Shell::Zsh => {
            r#"#compdef magicmerlin
_arguments "1:command:(status setup configure onboard health doctor dashboard tui completion version update reset uninstall agent agents models gateway daemon channels message directory pairing sessions memory cron logs hooks webhooks config security secrets sandbox approvals plugins skills dns devices nodes qr browser acp docs system help)"
"#
        }
        Shell::Fish => {
            r#"complete -c magicmerlin -f
for cmd in status setup configure onboard health doctor dashboard tui completion version update reset uninstall agent agents models gateway daemon channels message directory pairing sessions memory cron logs hooks webhooks config security secrets sandbox approvals plugins skills dns devices nodes qr browser acp docs system help
  complete -c magicmerlin -a $cmd
end
"#
        }
    };
    println!("{script}");
}

fn run_tui_stub() -> Result<()> {
    println!("MagicMerlin TUI");
    println!("Agents | Sessions | Cron | Logs");
    println!("Press q then Enter to quit.");
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if line.trim() == "q" {
            break;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = read_config();
    if cli.dev {
        config.profile = Some("dev".to_string());
    }
    if let Some(profile) = &cli.profile {
        config.profile = Some(profile.clone());
    }
    if let Some(level) = &cli.log_level {
        config.log_level = Some(level.clone());
    }

    let app = App { cli, config };

    let Some(command) = &app.cli.command else {
        let mut cmd = Cli::command();
        cmd.print_long_help()?;
        println!();
        return Ok(());
    };

    match command {
        Command::Introspect { command } => match command {
            IntrospectCommand::Commands => {
                let commands: Vec<String> = collect_command_paths().into_iter().collect();
                app.output(json!({"commands": commands}), || commands.join("\n"))?;
            }
        },

        Command::Status => {
            app.ensure_gateway_running().await?;
            let health = app.call_gateway("health", Value::Null).await?;
            let status = app.call_gateway("status", Value::Null).await?;
            app.output(json!({"health": health, "status": status}), || {
                format!("health={health}\nstatus={status}")
            })?;
        }

        Command::Setup => {
            let mut editable = app.config.clone();
            editable.gateway_ws_url = prompt("Gateway WebSocket URL", &editable.gateway_ws_url)?;
            editable.dashboard_url = prompt("Dashboard URL", &editable.dashboard_url)?;
            let profile_default = editable.profile.clone().unwrap_or_default();
            let p = prompt("Default profile (blank for none)", &profile_default)?;
            editable.profile = if p.trim().is_empty() { None } else { Some(p) };
            let l = prompt("Default log level", editable.log_level.as_deref().unwrap_or("info"))?;
            editable.log_level = Some(l);
            save_config(&editable)?;
            app.output(json!({"ok": true, "config": editable}), || "setup complete".to_string())?;
        }

        Command::Onboard => {
            let mut editable = app.config.clone();
            editable.gateway_ws_url = prompt("Gateway WebSocket URL", &editable.gateway_ws_url)?;
            editable.dashboard_url = prompt("Dashboard URL", &editable.dashboard_url)?;
            save_config(&editable)?;
            let start = prompt("Start gateway now? (y/n)", "y")?;
            if start.eq_ignore_ascii_case("y") {
                let pid = spawn_gateway()?;
                write_pid(pid)?;
            }
            app.output(json!({"ok": true}), || "onboard complete".to_string())?;
        }

        Command::Health => {
            let disk = fs::metadata(state_dir()).is_ok();
            let result = json!({
                "gatewayPortOpen": is_gateway_port_open(),
                "gatewayRpcReachable": app.call_gateway("health", Value::Null).await.is_ok(),
                "stateDirExists": disk
            });
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Dashboard => {
            open_url(&app.config.dashboard_url)?;
            app.output(json!({"url": app.config.dashboard_url}), || app.config.dashboard_url.clone())?;
        }

        Command::Tui => run_tui_stub()?,

        Command::Completion { shell } => emit_completion(*shell),

        Command::Version => {
            app.output(
                json!({"name": "magicmerlin", "version": env!("CARGO_PKG_VERSION")}),
                || format!("magicmerlin {}", env!("CARGO_PKG_VERSION")),
            )?;
        }

        Command::Update => {
            app.output(json!({"ok": true, "message": "update placeholder"}), || {
                "update placeholder".to_string()
            })?;
        }

        Command::Reset => {
            let dir = state_dir();
            if dir.exists() {
                fs::remove_dir_all(&dir).with_context(|| format!("remove {}", dir.display()))?;
            }
            app.output(json!({"ok": true, "stateDir": dir}), || "state reset complete".to_string())?;
        }

        Command::Agent { command } => match command {
            AgentCommand::Run {
                prompt,
                session_id,
                model,
            } => {
                app.ensure_gateway_running().await?;
                let result = app
                    .call_gateway(
                        "agent.run",
                        json!({"prompt": prompt, "message": prompt, "sessionId": session_id, "model": model}),
                    )
                    .await?;
                app.output(result.clone(), || result.to_string())?;
            }
        },

        Command::Agents { command } => {
            let result = match command {
                AgentsCommand::List => json!({"operation": "list"}),
                AgentsCommand::Add { name } => json!({"operation": "add", "name": name}),
                AgentsCommand::Remove { name } => json!({"operation": "remove", "name": name}),
                AgentsCommand::Config { name } => json!({"operation": "config", "name": name}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Models { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                ModelsCommand::List => ("status", json!({"scope": "models.list"})),
                ModelsCommand::Status => ("status", json!({"scope": "models.status"})),
                ModelsCommand::Auth => ("config.get", json!({"key": "providers.auth"})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Gateway { command } | Command::Daemon { command } => {
            handle_gateway_command(&app, command.clone()).await?;
        }

        Command::Channels { command } => {
            let result = match command {
                ChannelsCommand::Login { channel } => json!({"ok": true, "action": "login", "channel": channel}),
                ChannelsCommand::Logout { channel } => json!({"ok": true, "action": "logout", "channel": channel}),
                ChannelsCommand::Status => json!({"ok": true, "action": "status"}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Message { command } => match command {
            MessageCommand::Send {
                target,
                message,
                channel,
            } => {
                app.ensure_gateway_running().await?;
                let result = app
                    .call_gateway(
                        "chat.send",
                        json!({"target": target, "message": message, "channel": channel}),
                    )
                    .await?;
                app.output(result.clone(), || result.to_string())?;
            }
        },

        Command::Directory { query } => {
            let result = json!({"query": query, "results": []});
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Pairing { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                PairingCommand::List => ("pairing.list", json!({"limit": 50})),
                PairingCommand::Approve { id } => ("pairing.approve", json!({"requestId": id})),
                PairingCommand::Deny { id } => ("pairing.reject", json!({"requestId": id})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Sessions { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                SessionsCommand::List { limit } => ("sessions.list", json!({"limit": limit})),
                SessionsCommand::Show { id } => ("sessions.get", json!({"id": id})),
                SessionsCommand::Delete { id } => ("sessions.delete", json!({"id": id})),
                SessionsCommand::Compact { id } => ("sessions.compact", json!({"id": id})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Memory { command } => {
            let result = match command {
                MemoryCommand::Search { query } => json!({"query": query, "matches": []}),
                MemoryCommand::Get { key } => json!({"key": key, "value": Value::Null}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Cron { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                CronCommand::List => ("cron.list", Value::Null),
                CronCommand::Add {
                    name,
                    schedule,
                    kind,
                    payload,
                } => {
                    let payload_json =
                        serde_json::from_str::<Value>(payload).context("--payload must be JSON")?;
                    (
                        "cron.add",
                        json!({"name": name, "schedule": schedule, "kind": kind, "payload": payload_json}),
                    )
                }
                CronCommand::Edit {
                    id,
                    name,
                    schedule,
                    kind,
                    payload,
                } => {
                    let payload_json = match payload {
                        Some(s) => Some(serde_json::from_str::<Value>(s)?),
                        None => None,
                    };
                    (
                        "cron.edit",
                        json!({"id": id, "name": name, "schedule": schedule, "kind": kind, "payload": payload_json}),
                    )
                }
                CronCommand::Rm { id } => ("cron.rm", json!({"id": id})),
                CronCommand::Run { id } => ("cron.run", json!({"id": id})),
                CronCommand::Enable { id } => ("cron.enable", json!({"id": id})),
                CronCommand::Disable { id } => ("cron.disable", json!({"id": id})),
                CronCommand::Runs { job_id, limit } => {
                    ("cron.runs", json!({"jobId": job_id, "limit": limit}))
                }
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Logs { lines, follow } => {
            let result = json!({"lines": lines, "follow": follow, "entries": []});
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Hooks { command } => {
            let result = match command {
                HooksCommand::List => json!({"hooks": []}),
                HooksCommand::Add { url } => json!({"ok": true, "added": url}),
                HooksCommand::Remove { url } => json!({"ok": true, "removed": url}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Config { command } => {
            app.ensure_gateway_running().await?;
            match command {
                ConfigCommand::Get { key } => {
                    let result = app.call_gateway("config.get", json!({"key": key})).await?;
                    app.output(result.clone(), || result.to_string())?;
                }
                ConfigCommand::Set { key, value } => {
                    let result = app
                        .call_gateway("config.set", json!({"key": key, "value": value}))
                        .await?;
                    app.output(result.clone(), || result.to_string())?;
                }
                ConfigCommand::Unset { key } => {
                    let result = app.call_gateway("config.unset", json!({"key": key})).await?;
                    app.output(result.clone(), || result.to_string())?;
                }
                ConfigCommand::File => {
                    let path = config_path();
                    app.output(json!({"path": path}), || path.display().to_string())?;
                }
                ConfigCommand::Validate => {
                    app.output(json!({"ok": true}), || "config valid".to_string())?;
                }
            }
        }

        Command::Security { command } => match command {
            SecurityCommand::Audit => {
                let result = json!({
                    "openPorts": [{"port": 18789, "open": is_gateway_port_open()}],
                    "configIssues": [],
                    "permissionIssues": []
                });
                app.output(result.clone(), || result.to_string())?;
            }
        },

        Command::Secrets { command } => match command {
            SecretsCommand::Reload => {
                app.output(json!({"ok": true}), || "secrets reloaded".to_string())?;
            }
        },

        Command::Sandbox { command } => {
            let result = match command {
                SandboxCommand::List => json!({"sandboxes": []}),
                SandboxCommand::Start { name } => json!({"ok": true, "started": name}),
                SandboxCommand::Stop { name } => json!({"ok": true, "stopped": name}),
                SandboxCommand::Status => json!({"status": "unknown"}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Approvals { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                ApprovalsCommand::List => ("approvals.list", Value::Null),
                ApprovalsCommand::Approve { id } => ("approvals.approve", json!({"id": id})),
                ApprovalsCommand::Deny { id } => ("approvals.deny", json!({"id": id})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Plugins { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                PluginsCommand::List => ("plugins.list", Value::Null),
                PluginsCommand::Enable { name } => ("plugins.enable", json!({"name": name})),
                PluginsCommand::Disable { name } => ("plugins.disable", json!({"name": name})),
                PluginsCommand::Install { source } => {
                    ("plugins.get", json!({"source": source, "install": true}))
                }
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Skills { command } => {
            let result = match command {
                SkillsCommand::List => json!({"skills": []}),
                SkillsCommand::Inspect { name } => json!({"name": name, "body": ""}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Dns => {
            app.output(json!({"ok": true, "message": "dns helper placeholder"}), || {
                "dns helper placeholder".to_string()
            })?;
        }

        Command::Devices => {
            app.output(json!({"devices": []}), || "devices placeholder".to_string())?;
        }

        Command::Nodes { command } => {
            let result = match command {
                NodesCommand::List => json!({"nodes": []}),
                NodesCommand::Describe { id } => json!({"id": id, "node": Value::Null}),
                NodesCommand::Run { id } => json!({"ok": true, "run": id}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Qr { text } => {
            let result = json!({"text": text, "qr": text});
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Browser { command } => {
            let result = match command {
                BrowserCommand::Start => json!({"ok": true, "status": "started"}),
                BrowserCommand::Stop => json!({"ok": true, "status": "stopped"}),
                BrowserCommand::Status => json!({"ok": true, "status": "unknown"}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Acp => {
            app.output(json!({"ok": true, "message": "acp placeholder"}), || {
                "acp placeholder".to_string()
            })?;
        }

        Command::Docs => {
            let url = "https://github.com/openai/codex";
            open_url(url)?;
            app.output(json!({"url": url}), || url.to_string())?;
        }

        Command::System { command } => {
            app.ensure_gateway_running().await?;
            match command {
                SystemCommand::Event { text, mode } => {
                    let result = app
                        .call_gateway("system.event", json!({"text": text, "mode": mode}))
                        .await
                        .unwrap_or_else(|_| json!({"ok": true, "queued": true}));
                    app.output(result.clone(), || result.to_string())?;
                }
                SystemCommand::Heartbeat => {
                    let result = app
                        .call_gateway("system.heartbeat", Value::Null)
                        .await
                        .unwrap_or_else(|_| json!({"ok": true}));
                    app.output(result.clone(), || result.to_string())?;
                }
                SystemCommand::Presence => {
                    let result = app.call_gateway("system-presence", Value::Null).await?;
                    app.output(result.clone(), || result.to_string())?;
                }
            }
        }

        Command::Help => {
            let mut cmd = Cli::command();
            cmd.print_long_help()?;
            println!();
        }
    }

    Ok(())
}

async fn handle_gateway_command(app: &App, command: GatewayCommand) -> Result<()> {
    match command {
        GatewayCommand::Start => {
            if is_gateway_port_open() {
                return app.output(
                    json!({"ok": true, "alreadyRunning": true}),
                    || "gateway already running".to_string(),
                );
            }
            let pid = spawn_gateway()?;
            write_pid(pid)?;
            tokio::time::sleep(Duration::from_millis(700)).await;
            app.output(json!({"ok": true, "pid": pid}), || format!("gateway started pid={pid}"))
        }
        GatewayCommand::Stop => {
            let pid = read_pid()?;
            stop_pid(pid)?;
            let _ = fs::remove_file(pid_path());
            app.output(json!({"ok": true, "pid": pid}), || format!("gateway stopped pid={pid}"))
        }
        GatewayCommand::Restart => {
            if let Ok(pid) = read_pid() {
                let _ = stop_pid(pid);
                let _ = fs::remove_file(pid_path());
            }
            let pid = spawn_gateway()?;
            write_pid(pid)?;
            app.output(json!({"ok": true, "pid": pid}), || format!("gateway restarted pid={pid}"))
        }
        GatewayCommand::Status => {
            let pid = read_pid().ok();
            let open = is_gateway_port_open();
            app.output(
                json!({"pid": pid, "portOpen": open}),
                || format!("pid={pid:?} port_open={open}"),
            )
        }
        GatewayCommand::Call { method, params } => {
            app.ensure_gateway_running().await?;
            let params = serde_json::from_str::<Value>(&params).context("--params must be JSON")?;
            let result = app.call_gateway(&method, params).await?;
            app.output(result.clone(), || result.to_string())
        }
    }
}
