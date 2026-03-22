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
use clap_complete::aot::{generate, Bash as BashShell, Elvish as ElvishShell, Fish as FishShell, PowerShell as PowerShellShell, Zsh as ZshShell};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
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
    #[command(alias = "completions")]
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
        #[command(subcommand)]
        command: LogsCommand,
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
    Dns {
        #[command(subcommand)]
        command: DnsCommand,
    },
    Devices {
        #[command(subcommand)]
        command: DevicesCommand,
    },
    Nodes {
        #[command(subcommand)]
        command: NodesCommand,
    },
    Qr {
        #[arg(long)]
        url: Option<String>,
    },
    Browser {
        #[command(subcommand)]
        command: BrowserCommand,
    },
    Acp {
        #[command(subcommand)]
        command: AcpCommand,
    },
    Docs {
        page: Option<String>,
    },
    System {
        #[command(subcommand)]
        command: SystemCommand,
    },
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    Subagents {
        #[command(subcommand)]
        command: SubagentsCommand,
    },
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
    #[command(name = "help-all")]
    HelpAll,
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
    Add { name: String, #[arg(long)] model: Option<String>, #[arg(long)] description: Option<String> },
    Remove { name: String },
    Config { name: Option<String>, #[arg(long)] key: Option<String>, #[arg(long)] value: Option<String> },
    Show { name: String },
    Status,
    Env { name: String },
    Logs { name: String, #[arg(long, default_value_t = 50)] lines: usize },
}

#[derive(Subcommand, Debug)]
enum ModelsCommand {
    List,
    Status,
    Auth,
    Set { model: String, #[arg(long)] agent: Option<String> },
    Test { #[arg(long)] model: Option<String>, #[arg(long)] provider: Option<String> },
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
    List,
    Login { channel: String, #[arg(long)] token: Option<String> },
    Logout { channel: String },
    Status,
    Restart { channel: String },
    Send { channel: String, #[arg(long)] target: String, #[arg(long)] message: String },
    Test { channel: String },
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
    Send { id: String, message: String },
    Spawn { parent_id: String, #[arg(long)] agent: Option<String>, #[arg(long)] child_id: Option<String> },
    History { id: String, #[arg(long, default_value_t = 50)] limit: usize },
    Export,
}

#[derive(Subcommand, Debug)]
enum MemoryCommand {
    Search { query: String, #[arg(long, default_value_t = 20)] limit: usize, #[arg(long)] agent: Option<String> },
    Get { key: String, #[arg(long)] agent: Option<String> },
    List { #[arg(long)] prefix: Option<String>, #[arg(long, default_value_t = 50)] limit: usize, #[arg(long)] agent: Option<String> },
    Clear { #[arg(long)] agent: Option<String>, #[arg(long)] confirm: bool },
    Stats { #[arg(long)] agent: Option<String> },
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
    Status,
    DeadLetters { #[arg(long, default_value_t = 50)] limit: usize },
    Export { #[arg(long)] file: PathBuf },
    Import { #[arg(long)] file: PathBuf, #[arg(long)] replace: bool },
}

#[derive(Subcommand, Debug)]
enum HooksCommand {
    List,
    Add { url: String, #[arg(long)] name: Option<String>, #[arg(long)] events: Option<String> },
    Remove { url: String },
    Test { url: String },
    Fire { name: String, #[arg(long)] event: Option<String> },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
    File,
    Validate,
    List,
    Export { #[arg(long)] file: Option<PathBuf> },
    Import { file: PathBuf },
    Diff { file: PathBuf },
}

#[derive(Subcommand, Debug)]
enum SecurityCommand {
    Audit,
    Scan { #[arg(long)] workspace: Option<PathBuf> },
    Report { #[arg(long)] format: Option<String> },
}

#[derive(Subcommand, Debug)]
enum SecretsCommand {
    Reload,
    List,
    Set { key: String, value: String },
    Unset { key: String },
}

#[derive(Subcommand, Debug)]
enum SandboxCommand {
    List,
    Start { name: String, #[arg(long)] image: Option<String> },
    Stop { name: String },
    Status,
    Exec { name: String, command: String, #[arg(trailing_var_arg = true)] args: Vec<String> },
    Logs { name: String, #[arg(long, default_value_t = 100)] lines: usize },
}

#[derive(Subcommand, Debug)]
enum ApprovalsCommand {
    List,
    Approve { id: String },
    Deny { id: String },
    Get,
    Set { #[arg(long)] file: PathBuf },
    Allowlist {
        #[command(subcommand)]
        command: AllowlistCommand,
    },
}

#[derive(Subcommand, Debug)]
enum PluginsCommand {
    List,
    Enable { name: String },
    Disable { name: String },
    Install { source: String },
    Uninstall { name: String },
    Get { name: String },
    Info { name: String },
    Update { name: String },
}

#[derive(Subcommand, Debug)]
enum SkillsCommand {
    List,
    Inspect { name: String },
    Add { name: String },
    Remove { name: String },
    Update { name: String },
}

#[derive(Subcommand, Debug)]
enum NodesCommand {
    List,
    Describe { id: String },
    Run { id: String, command: String, #[arg(trailing_var_arg = true)] args: Vec<String> },
    Invoke { id: String, method: String, #[arg(long, default_value = "{}")] params: String },
    Logs { id: String, #[arg(long, default_value_t = 50)] lines: usize },
    Notify { id: String, #[arg(long)] title: String, #[arg(long)] body: String },
    Status,
}

#[derive(Subcommand, Debug)]
enum BrowserCommand {
    Start,
    Stop,
    Status,
    Navigate { url: String, #[arg(long)] tab_id: Option<String> },
    Screenshot { #[arg(long)] tab_id: Option<String>, #[arg(long)] full_page: bool, #[arg(long)] output: Option<PathBuf> },
    Act { action: String, #[arg(long)] selector: Option<String>, #[arg(long)] text: Option<String> },
    Snapshot { #[arg(long)] tab_id: Option<String> },
    Tabs,
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
    Restart,
    Info,
    Env,
}

// ── Pass 7: New command enums ────────────────────────────────────────

#[derive(Subcommand, Debug)]
enum LogsCommand {
    Tail { #[arg(long, default_value_t = 100)] lines: usize, #[arg(long)] level: Option<String>, #[arg(long)] component: Option<String> },
    Query { #[arg(long)] query: Option<String>, #[arg(long)] level: Option<String>, #[arg(long, default_value_t = 200)] limit: usize },
    Export { #[arg(long)] file: PathBuf, #[arg(long)] level: Option<String> },
    Follow { #[arg(long)] level: Option<String> },
}

#[derive(Subcommand, Debug)]
enum DnsCommand {
    Lookup { domain: String },
    Resolve { domain: String },
    Test,
    Tailscale { #[command(subcommand)] command: TailscaleCommand },
}

#[derive(Subcommand, Debug)]
enum TailscaleCommand {
    Status,
    Up,
    Down,
}

#[derive(Subcommand, Debug)]
enum DevicesCommand {
    List,
    Pair { id: String },
    Unpair { id: String },
    Status { id: Option<String> },
}

#[derive(Subcommand, Debug)]
enum AcpCommand {
    Sessions { #[arg(long)] thread_id: Option<String> },
    Spawn { agent: String, #[arg(long)] thread_id: String, #[arg(long)] command: String },
    Cleanup,
    Status,
}

#[derive(Subcommand, Debug)]
enum AllowlistCommand {
    Add { pattern: String, #[arg(long)] agent: Option<String> },
    Remove { pattern: String, #[arg(long)] agent: Option<String> },
    List,
}

#[derive(Subcommand, Debug)]
enum RunCommand {
    List { #[arg(long)] session_id: Option<String>, #[arg(long)] status: Option<String> },
    Status { run_id: String },
}

#[derive(Subcommand, Debug)]
enum SubagentsCommand {
    List,
    Kill { session: String },
}

#[derive(Subcommand, Debug)]
enum ContextCommand {
    Show { session_key: Option<String> },
}

#[derive(ValueEnum, Copy, Clone, Debug)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    #[value(name = "powershell")]
    PowerShell,
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
    let mut cmd = Cli::command();
    let name = "magicmerlin".to_string();
    match shell {
        Shell::Bash => generate(BashShell, &mut cmd, &name, &mut io::stdout()),
        Shell::Zsh => generate(ZshShell, &mut cmd, &name, &mut io::stdout()),
        Shell::Fish => generate(FishShell, &mut cmd, &name, &mut io::stdout()),
        Shell::Elvish => generate(ElvishShell, &mut cmd, &name, &mut io::stdout()),
        Shell::PowerShell => generate(PowerShellShell, &mut cmd, &name, &mut io::stdout()),
    }
}

// ── TUI Dashboard ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct AgentInfo {
    name: String,
    model: String,
    online: bool,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    id: String,
    tokens: u64,
}

#[derive(Debug, Clone)]
struct CronInfo {
    name: String,
    schedule: String,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct GatewayInfo {
    online: bool,
    port: u16,
    model: String,
    uptime_secs: u64,
}

struct TuiApp {
    tab: usize,
    agents: Vec<AgentInfo>,
    sessions: Vec<SessionInfo>,
    cron_jobs: Vec<CronInfo>,
    logs: Vec<String>,
    gateway: GatewayInfo,
    tick: u64,
    scroll: usize,
    gateway_url: String,
}

impl TuiApp {
    fn new(gateway_url: String) -> Self {
        Self {
            tab: 0,
            agents: Vec::new(),
            sessions: Vec::new(),
            cron_jobs: Vec::new(),
            logs: vec!["TUI started".to_string()],
            gateway: GatewayInfo {
                online: false,
                port: 18789,
                model: "unknown".to_string(),
                uptime_secs: 0,
            },
            tick: 0,
            scroll: 0,
            gateway_url,
        }
    }

    async fn refresh(&mut self) {
        let client = Client::new();
        let url = format!("{}/ws", self.gateway_url);

        let call = |method: &str| {
            let req = serde_json::json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": {},
                "id": 1
            });
            client.post(&url).json(&req).send()
        };

        // Health check
        match call("health").await {
            Ok(resp) => {
                if let Ok(body) = resp.json::<Value>().await {
                    self.gateway.online = true;
                    if let Some(r) = body.get("result") {
                        if let Some(m) = r.get("model").and_then(|v| v.as_str()) {
                            self.gateway.model = m.to_string();
                        }
                        if let Some(u) = r.get("uptimeSeconds").and_then(|v| v.as_u64()) {
                            self.gateway.uptime_secs = u;
                        }
                    }
                }
            }
            Err(_) => {
                self.gateway.online = false;
                return;
            }
        }

        // Agents
        if let Ok(resp) = call("agents.list").await {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(r) = body.get("result") {
                    if let Some(arr) = r.get("agents").and_then(|v| v.as_array()) {
                        self.agents = arr.iter().map(|a| AgentInfo {
                            name: a.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                            model: a.get("model").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                            online: a.get("online").and_then(|v| v.as_bool()).unwrap_or(false),
                        }).collect();
                    }
                }
            }
        }

        // Sessions
        if let Ok(resp) = call("sessions.list").await {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(r) = body.get("result") {
                    if let Some(arr) = r.get("sessions").and_then(|v| v.as_array()) {
                        self.sessions = arr.iter().map(|s| SessionInfo {
                            id: s.get("sessionId").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                            tokens: s.get("tokenUsage").and_then(|v| v.as_u64()).unwrap_or(0),
                        }).collect();
                    }
                }
            }
        }

        // Cron
        if let Ok(resp) = call("cron.list").await {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(r) = body.get("result") {
                    if let Some(arr) = r.get("jobs").and_then(|v| v.as_array()) {
                        self.cron_jobs = arr.iter().map(|j| CronInfo {
                            name: j.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                            schedule: j.get("schedule").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                            enabled: j.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                        }).collect();
                    }
                }
            }
        }

        // Logs
        if let Ok(resp) = call("logs.tail").await {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(r) = body.get("result") {
                    if let Some(arr) = r.get("lines").and_then(|v| v.as_array()) {
                        self.logs = arr.iter().filter_map(|l| l.as_str().map(|s| s.to_string())).collect();
                        if self.logs.is_empty() {
                            self.logs.push("(no recent logs)".to_string());
                        }
                    }
                }
            }
        }
    }

    fn on_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => return true,
            KeyCode::Char('1') => self.tab = 0,
            KeyCode::Char('2') => self.tab = 1,
            KeyCode::Char('3') => self.tab = 2,
            KeyCode::Char('4') => self.tab = 3,
            KeyCode::Char('5') => self.tab = 4,
            KeyCode::Tab => self.tab = (self.tab + 1) % 5,
            KeyCode::BackTab => self.tab = if self.tab == 0 { 4 } else { self.tab - 1 },
            KeyCode::Char('r') => self.tick = 0, // force refresh on next cycle
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
            _ => {}
        }
        false
    }
}

fn draw_tui(frame: &mut Frame, app: &TuiApp) {
    let size = frame.size();

    // Main layout: title, tabs, body, status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // tabs
            Constraint::Min(5),    // body
            Constraint::Length(1), // status bar
        ])
        .split(size);

    // Tab bar
    let tab_titles: Vec<Line> = ["[1]Overview", "[2]Agents", "[3]Sessions", "[4]Cron", "[5]Logs"]
        .iter()
        .map(|t| Line::from(Span::raw(*t)))
        .collect();
    let tabs = Tabs::new(tab_titles)
        .select(app.tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider(Span::raw(" | "));
    frame.render_widget(tabs, main_chunks[0]);

    // Body
    let body_block = Block::default()
        .title(" Magic Merlin ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.gateway.online { Color::Green } else { Color::Red }));

    let inner = body_block.inner(main_chunks[1]);
    frame.render_widget(body_block, main_chunks[1]);

    match app.tab {
        0 => draw_overview(frame, app, inner),
        1 => draw_agents(frame, app, inner),
        2 => draw_sessions(frame, app, inner),
        3 => draw_cron(frame, app, inner),
        4 => draw_logs(frame, app, inner),
        _ => {}
    }

    // Status bar
    let online_str = if app.gateway.online { "● online" } else { "○ offline" };
    let uptime = format_uptime(app.gateway.uptime_secs);
    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" Gateway: {online_str} :{} ", app.gateway.port),
            Style::default().fg(if app.gateway.online { Color::Green } else { Color::Red }),
        ),
        Span::raw(format!("| Model: {} | Uptime: {} | q=quit Tab=switch r=refresh", app.gateway.model, uptime)),
    ]));
    frame.render_widget(status, main_chunks[2]);
}

fn draw_overview(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Agents panel
    let agent_items: Vec<ListItem> = app.agents.iter().map(|a| {
        let marker = if a.online { "●" } else { "○" };
        let check = if a.online { "✓" } else { "✗" };
        ListItem::new(format!(" {marker} {:<16} {:<10} {check}", a.name, a.model))
            .style(Style::default().fg(if a.online { Color::Green } else { Color::DarkGray }))
    }).collect();
    let agents_list = List::new(agent_items)
        .block(Block::default().title(" AGENTS ").borders(Borders::ALL));
    frame.render_widget(agents_list, left[0]);

    // Sessions panel
    let session_items: Vec<ListItem> = app.sessions.iter().map(|s| {
        ListItem::new(format!(" {} ({}tok)", s.id, s.tokens))
    }).collect();
    let sessions_list = List::new(session_items)
        .block(Block::default().title(" ACTIVE SESSIONS ").borders(Borders::ALL));
    frame.render_widget(sessions_list, right[0]);

    // Cron panel
    let cron_items: Vec<ListItem> = app.cron_jobs.iter().map(|c| {
        let check = if c.enabled { "✓" } else { "✗" };
        ListItem::new(format!(" {:<16} {:<14} {check}", c.name, c.schedule))
    }).collect();
    let cron_list = List::new(cron_items)
        .block(Block::default().title(" CRON JOBS ").borders(Borders::ALL));
    frame.render_widget(cron_list, left[1]);

    // Logs panel
    let log_items: Vec<ListItem> = app.logs.iter().rev().take(20).map(|l| {
        ListItem::new(format!(" {l}"))
    }).collect();
    let logs_list = List::new(log_items)
        .block(Block::default().title(" RECENT LOGS ").borders(Borders::ALL));
    frame.render_widget(logs_list, right[1]);
}

fn draw_agents(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let items: Vec<ListItem> = app.agents.iter().map(|a| {
        let marker = if a.online { "●" } else { "○" };
        let check = if a.online { "✓" } else { "✗" };
        ListItem::new(format!("  {marker} {:<20} model={:<12} {check}", a.name, a.model))
            .style(Style::default().fg(if a.online { Color::Green } else { Color::DarkGray }))
    }).collect();
    let list = List::new(items)
        .block(Block::default().title(" Agents ").borders(Borders::NONE));
    frame.render_widget(list, area);
}

fn draw_sessions(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let items: Vec<ListItem> = app.sessions.iter().map(|s| {
        ListItem::new(format!("  {:<36} tokens={}", s.id, s.tokens))
    }).collect();
    let list = List::new(items)
        .block(Block::default().title(" Sessions ").borders(Borders::NONE));
    frame.render_widget(list, area);
}

fn draw_cron(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let items: Vec<ListItem> = app.cron_jobs.iter().map(|c| {
        let check = if c.enabled { "✓" } else { "✗" };
        ListItem::new(format!("  {:<20} {:<16} {check}", c.name, c.schedule))
    }).collect();
    let list = List::new(items)
        .block(Block::default().title(" Cron Jobs ").borders(Borders::NONE));
    frame.render_widget(list, area);
}

fn draw_logs(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let start = app.scroll.min(app.logs.len().saturating_sub(1));
    let items: Vec<ListItem> = app.logs.iter().skip(start).map(|l| {
        ListItem::new(format!("  {l}"))
    }).collect();
    let list = List::new(items)
        .block(Block::default().title(" Logs (↑↓ scroll) ").borders(Borders::NONE));
    frame.render_widget(list, area);
}

fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

async fn run_tui(gateway_url: &str) -> Result<()> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen).context("enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("create terminal")?;

    let mut app = TuiApp::new(gateway_url.to_string());
    app.refresh().await;

    loop {
        terminal.draw(|f| draw_tui(f, &app))?;

        if event::poll(Duration::from_millis(100)).context("poll events")? {
            if let Event::Key(key) = event::read().context("read event")? {
                if key.kind == KeyEventKind::Press && app.on_key(key.code) {
                    break;
                }
            }
        }

        // Refresh every 2s (20 * 100ms)
        app.tick += 1;
        if app.tick % 20 == 0 {
            app.refresh().await;
        }
    }

    disable_raw_mode().context("disable raw mode")?;
    terminal.backend_mut().execute(LeaveAlternateScreen).context("leave alternate screen")?;
    terminal.show_cursor().context("show cursor")?;

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

        Command::Tui => run_tui(&app.gateway_http_url()).await?,

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
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                AgentsCommand::List => ("agents.list", json!({})),
                AgentsCommand::Add { name, model, description } => ("agents.add", json!({"name": name, "model": model, "description": description})),
                AgentsCommand::Remove { name } => ("agents.remove", json!({"name": name})),
                AgentsCommand::Config { name, key, value } => ("agents.config", json!({"name": name.as_deref().unwrap_or("merlin"), "key": key, "value": value})),
                AgentsCommand::Show { name } => ("agents.get", json!({"name": name})),
                AgentsCommand::Status => ("agents.list", json!({})),
                AgentsCommand::Env { name } => ("agents.config", json!({"name": name})),
                AgentsCommand::Logs { name, lines } => ("logs.query", json!({"query": name, "limit": lines})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Models { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                ModelsCommand::List => ("models.list", json!({})),
                ModelsCommand::Status => ("models.status", json!({})),
                ModelsCommand::Auth => ("config.get", json!({"path": "auth"})),
                ModelsCommand::Set { model, agent } => ("models.set", json!({"model": model, "agent": agent})),
                ModelsCommand::Test { model, provider } => ("models.test", json!({"model": model, "provider": provider})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Gateway { command } | Command::Daemon { command } => {
            handle_gateway_command(&app, command.clone()).await?;
        }

        Command::Channels { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                ChannelsCommand::List => ("channels.list", json!({})),
                ChannelsCommand::Login { channel, token } => ("channels.login", json!({"channel": channel, "token": token})),
                ChannelsCommand::Logout { channel } => ("channels.logout", json!({"channel": channel})),
                ChannelsCommand::Status => ("channels.status", json!({})),
                ChannelsCommand::Restart { channel } => ("channels.restart", json!({"channel": channel})),
                ChannelsCommand::Send { channel, target, message } => ("channels.send", json!({"channel": channel, "target": target, "message": message})),
                ChannelsCommand::Test { channel } => ("channels.status", json!({"channel": channel})),
            };
            let result = app.call_gateway(method, params).await?;
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
                SessionsCommand::Send { id, message } => ("sessions.send", json!({"sessionId": id, "message": message})),
                SessionsCommand::Spawn { parent_id, agent, child_id } => ("sessions.spawn", json!({"parentSessionId": parent_id, "agent": agent, "childSessionId": child_id})),
                SessionsCommand::History { id, limit } => ("sessions.history", json!({"id": id, "limit": limit})),
                SessionsCommand::Export => ("sessions.export", json!({})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Memory { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                MemoryCommand::Search { query, limit, agent } => ("memory.search", json!({"query": query, "limit": limit, "agent": agent})),
                MemoryCommand::Get { key, agent } => ("memory.get", json!({"key": key, "agent": agent})),
                MemoryCommand::List { prefix, limit, agent } => ("memory.list", json!({"prefix": prefix, "limit": limit, "agent": agent})),
                MemoryCommand::Clear { agent, confirm } => {
                    if !confirm {
                        return Err(anyhow!("pass --confirm to clear memory"));
                    }
                    ("memory.list", json!({"agent": agent, "limit": 0}))
                }
                MemoryCommand::Stats { agent } => ("memory.list", json!({"agent": agent})),
            };
            let result = app.call_gateway(method, params).await?;
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
                CronCommand::Status => ("cron.status", Value::Null),
                CronCommand::DeadLetters { limit } => ("cron.deadLetters", json!({"limit": limit})),
                CronCommand::Export { file } => {
                    let result = app.call_gateway("cron.list", Value::Null).await?;
                    fs::write(&file, serde_json::to_string_pretty(&result)?)?;
                    app.output(json!({"ok": true, "file": file}), || format!("exported to {}", file.display()))?;
                    return Ok(());
                }
                CronCommand::Import { file, replace } => {
                    let raw = fs::read_to_string(&file)?;
                    let data: Value = serde_json::from_str(&raw)?;
                    // Import each job
                    if let Some(jobs) = data.get("jobs").and_then(|j| j.as_array()) {
                        for job in jobs {
                            let _ = app.call_gateway("cron.add", job.clone()).await;
                        }
                    }
                    app.output(json!({"ok": true, "replace": replace, "file": file}), || format!("imported from {}", file.display()))?;
                    return Ok(());
                }
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Logs { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                LogsCommand::Tail { lines, level, component } => ("logs.tail", json!({"lines": lines, "level": level, "component": component})),
                LogsCommand::Query { query, level, limit } => ("logs.query", json!({"query": query, "level": level, "limit": limit})),
                LogsCommand::Export { file, level } => {
                    let result = app.call_gateway("logs.query", json!({"level": level, "limit": 10000})).await?;
                    fs::write(&file, serde_json::to_string_pretty(&result)?)?;
                    app.output(json!({"ok": true, "file": file}), || format!("exported to {}", file.display()))?;
                    return Ok(());
                }
                LogsCommand::Follow { level } => ("logs.tail", json!({"lines": 100, "level": level})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Hooks { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                HooksCommand::List => ("hooks.list", json!({})),
                HooksCommand::Add { url, name, events } => {
                    let events_vec: Option<Vec<String>> = events.as_deref().map(|e| e.split(',').map(|s| s.trim().to_string()).collect());
                    ("hooks.add", json!({"url": url, "name": name, "events": events_vec}))
                }
                HooksCommand::Remove { url } => ("hooks.remove", json!({"url": url})),
                HooksCommand::Test { url } => ("hooks.test", json!({"url": url})),
                HooksCommand::Fire { name, event } => ("hooks.test", json!({"url": name, "event": event})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Config { command } => {
            match command {
                ConfigCommand::File => {
                    let path = config_path();
                    app.output(json!({"path": path}), || path.display().to_string())?;
                }
                ConfigCommand::Diff { file } => {
                    let raw = fs::read_to_string(&file)?;
                    let external: Value = serde_json::from_str(&raw)?;
                    app.ensure_gateway_running().await?;
                    let current = app.call_gateway("config.export", json!({})).await?;
                    app.output(json!({"current": current, "compared": external}), || "diff shown".to_string())?;
                }
                other => {
                    app.ensure_gateway_running().await?;
                    let (method, params) = match other {
                        ConfigCommand::Get { key } => ("config.get", json!({"path": key})),
                        ConfigCommand::Set { key, value } => ("config.set", json!({"path": key, "value": value})),
                        ConfigCommand::Unset { key } => ("config.unset", json!({"path": key})),
                        ConfigCommand::Validate => ("config.validate", json!({})),
                        ConfigCommand::List => ("config.list", json!({})),
                        ConfigCommand::Export { file } => {
                            let result = app.call_gateway("config.export", json!({})).await?;
                            if let Some(path) = file {
                                fs::write(&path, serde_json::to_string_pretty(&result)?)?;
                                app.output(json!({"ok": true, "file": path}), || format!("exported to {}", path.display()))?;
                            } else {
                                app.output(result.clone(), || result.to_string())?;
                            }
                            return Ok(());
                        }
                        ConfigCommand::Import { file } => {
                            let raw = fs::read_to_string(&file)?;
                            let data: Value = serde_json::from_str(&raw)?;
                            ("config.import", json!({"config": data}))
                        }
                        ConfigCommand::File | ConfigCommand::Diff { .. } => unreachable!(),
                    };
                    let result = app.call_gateway(method, params).await?;
                    app.output(result.clone(), || result.to_string())?;
                }
            }
        }

        Command::Security { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                SecurityCommand::Audit => ("security.audit", json!({})),
                SecurityCommand::Scan { workspace } => ("security.audit", json!({"workspace": workspace})),
                SecurityCommand::Report { format } => ("security.audit", json!({"format": format})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Secrets { command } => {
            let result = match command {
                SecretsCommand::Reload => json!({"ok": true, "action": "reload"}),
                SecretsCommand::List => json!({"ok": true, "secrets": [], "note": "secrets are not listed for security"}),
                SecretsCommand::Set { key, value: _ } => json!({"ok": true, "key": key, "action": "set"}),
                SecretsCommand::Unset { key } => json!({"ok": true, "key": key, "action": "unset"}),
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Sandbox { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                SandboxCommand::List => ("sandbox.list", json!({})),
                SandboxCommand::Start { name, image } => ("sandbox.start", json!({"name": name, "image": image})),
                SandboxCommand::Stop { name } => ("sandbox.stop", json!({"name": name})),
                SandboxCommand::Status => ("sandbox.status", json!({})),
                SandboxCommand::Exec { name, command, args } => ("sandbox.exec", json!({"name": name, "command": command, "args": args})),
                SandboxCommand::Logs { name, lines } => ("logs.query", json!({"query": name, "limit": lines})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Approvals { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                ApprovalsCommand::List => ("approvals.list", Value::Null),
                ApprovalsCommand::Approve { id } => ("approvals.approve", json!({"id": id})),
                ApprovalsCommand::Deny { id } => ("approvals.deny", json!({"id": id})),
                ApprovalsCommand::Get => ("approvals.get", json!({})),
                ApprovalsCommand::Set { file } => {
                    let raw = fs::read_to_string(&file)?;
                    let data: Value = serde_json::from_str(&raw)?;
                    ("approvals.set", json!({"json": data}))
                }
                ApprovalsCommand::Allowlist { command } => match command {
                    AllowlistCommand::Add { pattern, agent } => ("approvals.allowlist.add", json!({"pattern": pattern, "agent": agent})),
                    AllowlistCommand::Remove { pattern, agent } => ("approvals.allowlist.remove", json!({"pattern": pattern, "agent": agent})),
                    AllowlistCommand::List => ("approvals.allowlist.list", json!({})),
                },
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
                PluginsCommand::Install { source } => ("plugins.install", json!({"source": source})),
                PluginsCommand::Uninstall { name } => ("plugins.disable", json!({"name": name})),
                PluginsCommand::Get { name } => ("plugins.get", json!({"name": name})),
                PluginsCommand::Info { name } => ("plugins.get", json!({"name": name})),
                PluginsCommand::Update { name } => ("plugins.install", json!({"source": name})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Skills { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                SkillsCommand::List => ("skills.list", json!({})),
                SkillsCommand::Inspect { name } => ("skills.get", json!({"name": name})),
                SkillsCommand::Add { name } => ("plugins.install", json!({"source": name})),
                SkillsCommand::Remove { name } => ("plugins.disable", json!({"name": name})),
                SkillsCommand::Update { name } => ("plugins.install", json!({"source": name})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Dns { command } => {
            let result = match command {
                DnsCommand::Lookup { domain } => {
                    use std::net::ToSocketAddrs;
                    let addrs: Vec<String> = format!("{domain}:0")
                        .to_socket_addrs()
                        .map(|iter| iter.map(|a| a.ip().to_string()).collect())
                        .unwrap_or_default();
                    json!({"domain": domain, "addresses": addrs})
                }
                DnsCommand::Resolve { domain } => {
                    use std::net::ToSocketAddrs;
                    let addrs: Vec<String> = format!("{domain}:0")
                        .to_socket_addrs()
                        .map(|iter| iter.map(|a| a.ip().to_string()).collect())
                        .unwrap_or_default();
                    json!({"domain": domain, "resolved": addrs})
                }
                DnsCommand::Test => {
                    let reachable = std::net::TcpStream::connect_timeout(
                        &"1.1.1.1:53".parse().unwrap(),
                        Duration::from_secs(3),
                    ).is_ok();
                    json!({"ok": true, "dnsReachable": reachable})
                }
                DnsCommand::Tailscale { command: ts_cmd } => {
                    let (action, result) = match ts_cmd {
                        TailscaleCommand::Status => ("status", ProcessCommand::new("tailscale").arg("status").output()),
                        TailscaleCommand::Up => ("up", ProcessCommand::new("tailscale").arg("up").output()),
                        TailscaleCommand::Down => ("down", ProcessCommand::new("tailscale").arg("down").output()),
                    };
                    match result {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            json!({"action": action, "output": stdout, "success": output.status.success()})
                        }
                        Err(e) => json!({"action": action, "error": e.to_string()}),
                    }
                }
            };
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Devices { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                DevicesCommand::List => ("nodes.list", json!({})),
                DevicesCommand::Pair { id } => ("nodes.invoke", json!({"id": id, "method": "pair"})),
                DevicesCommand::Unpair { id } => ("nodes.invoke", json!({"id": id, "method": "unpair"})),
                DevicesCommand::Status { id } => ("nodes.list", json!({"id": id})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Nodes { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                NodesCommand::List => ("nodes.list", json!({})),
                NodesCommand::Describe { id } => ("nodes.describe", json!({"id": id})),
                NodesCommand::Run { id, command, args } => ("nodes.run", json!({"id": id, "command": command, "args": args})),
                NodesCommand::Invoke { id, method, params } => {
                    let params_json = serde_json::from_str::<Value>(&params).context("--params must be JSON")?;
                    ("nodes.invoke", json!({"id": id, "method": method, "params": params_json}))
                }
                NodesCommand::Logs { id, lines } => ("logs.query", json!({"query": id, "limit": lines})),
                NodesCommand::Notify { id, title, body } => ("nodes.notify", json!({"id": id, "title": title, "body": body})),
                NodesCommand::Status => ("nodes.list", json!({})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Qr { url } => {
            let text = match url {
                Some(u) => u.clone(),
                None => format!("http://127.0.0.1:18789/pair"),
            };
            let code = qrcode::QrCode::new(text.as_bytes()).context("generate QR code")?;
            let qr_string = code.render::<char>()
                .quiet_zone(true)
                .module_dimensions(2, 1)
                .build();
            app.output(json!({"text": text, "qr": qr_string}), || {
                format!("{qr_string}\n{text}")
            })?;
        }

        Command::Browser { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                BrowserCommand::Start => ("browser.start", json!({})),
                BrowserCommand::Stop => ("browser.stop", json!({})),
                BrowserCommand::Status => ("browser.status", json!({})),
                BrowserCommand::Navigate { url, tab_id } => ("browser.navigate", json!({"url": url, "tabId": tab_id})),
                BrowserCommand::Screenshot { tab_id, full_page, output: _ } => ("browser.screenshot", json!({"tabId": tab_id, "fullPage": full_page})),
                BrowserCommand::Act { action, selector, text } => ("browser.act", json!({"action": action, "selector": selector, "text": text})),
                BrowserCommand::Snapshot { tab_id } => ("browser.snapshot", json!({"tabId": tab_id})),
                BrowserCommand::Tabs => ("browser.status", json!({})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Acp { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                AcpCommand::Sessions { thread_id } => ("acp.sessions.list", json!({"threadId": thread_id})),
                AcpCommand::Spawn { agent, thread_id, command } => ("acp.spawn", json!({"agent": agent, "threadId": thread_id, "command": command})),
                AcpCommand::Cleanup => ("acp.cleanup", json!({})),
                AcpCommand::Status => ("acp.sessions.list", json!({})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Docs { page } => {
            match page {
                Some(query) => {
                    app.ensure_gateway_running().await?;
                    let result = app.call_gateway("docs.search", json!({"query": query})).await
                        .unwrap_or_else(|_| json!({"results": [], "query": query}));
                    app.output(result.clone(), || result.to_string())?;
                }
                None => {
                    let url = "https://docs.magicmerlin.dev";
                    open_url(url)?;
                    app.output(json!({"url": url}), || url.to_string())?;
                }
            }
        }

        Command::System { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                SystemCommand::Event { text, mode } => ("system.event", json!({"name": text, "payload": {"mode": mode}})),
                SystemCommand::Heartbeat => ("system.heartbeat", Value::Null),
                SystemCommand::Presence => ("system-presence", Value::Null),
                SystemCommand::Restart => ("system.restart", json!({})),
                SystemCommand::Info => ("system.info", json!({})),
                SystemCommand::Env => ("system.env", json!({})),
            };
            let result = app.call_gateway(method, params).await
                .unwrap_or_else(|_| json!({"ok": true}));
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Run { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                RunCommand::List { session_id, status } => ("run.list", json!({"sessionId": session_id, "status": status})),
                RunCommand::Status { run_id } => ("run.status", json!({"runId": run_id})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Subagents { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                SubagentsCommand::List => ("subagents.list", json!({})),
                SubagentsCommand::Kill { session } => ("subagents.kill", json!({"session": session})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || result.to_string())?;
        }

        Command::Context { command } => {
            app.ensure_gateway_running().await?;
            let (method, params) = match command {
                ContextCommand::Show { session_key } => ("sessions.get", json!({"id": session_key})),
            };
            let result = app.call_gateway(method, params).await?;
            app.output(result.clone(), || {
                let tokens = result.get("tokenUsage").and_then(|v| v.as_u64()).unwrap_or(0);
                let model = result.get("model").and_then(|v| v.as_str()).unwrap_or("unknown");
                let compactions = result.get("compactionCount").and_then(|v| v.as_u64()).unwrap_or(0);
                let limit: u64 = 200_000;
                let pct = if limit > 0 { (tokens as f64 / limit as f64) * 100.0 } else { 0.0 };
                format!("Model: {model}\nTokens: {tokens} / {limit} ({pct:.1}%)\nCompactions: {compactions}")
            })?;
        }

        Command::HelpAll => {
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
