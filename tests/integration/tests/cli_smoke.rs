//! Integration tests: CLI binary smoke tests.

use std::process::Command;

fn cli_bin() -> std::path::PathBuf {
    magicmerlin_integration_tests::cargo_bin("magicmerlin")
}

#[test]
fn test_cli_version() {
    let bin = cli_bin();
    if !bin.exists() {
        eprintln!("CLI binary not found at {}, skipping", bin.display());
        return;
    }

    let out = Command::new(&bin).arg("version").output().unwrap();
    assert!(out.status.success(), "version command failed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("magicmerlin"),
        "version output should contain 'magicmerlin', got: {stdout}"
    );
}

#[test]
fn test_cli_version_json() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    let out = Command::new(&bin)
        .args(["--json", "version"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Should be valid JSON
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_default();
    assert_eq!(v["name"], "magicmerlin");
}

#[test]
fn test_cli_completions_bash() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    let out = Command::new(&bin)
        .args(["completion", "bash"])
        .output()
        .unwrap();
    assert!(out.status.success(), "bash completions failed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("magicmerlin") || stdout.contains("complete"),
        "bash completions should reference magicmerlin"
    );
}

#[test]
fn test_cli_completions_zsh() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    let out = Command::new(&bin)
        .args(["completion", "zsh"])
        .output()
        .unwrap();
    assert!(out.status.success(), "zsh completions failed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("compdef") || stdout.contains("magicmerlin"),
        "zsh completions should contain compdef or magicmerlin"
    );
}

#[test]
fn test_cli_completions_fish() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    let out = Command::new(&bin)
        .args(["completion", "fish"])
        .output()
        .unwrap();
    assert!(out.status.success(), "fish completions failed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.is_empty());
}

#[test]
fn test_cli_help() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    let out = Command::new(&bin).arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("MagicMerlin") || stdout.contains("magicmerlin"));
}

#[test]
fn test_cli_health_offline() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    // When gateway is offline, health should report and exit non-zero or report offline status.
    let out = Command::new(&bin).arg("health").output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let combined = format!("{stdout}{stderr}");
    // Should not panic (i.e. process should exit, not crash).
    // Either exit code is non-zero, or output mentions offline/refused/false/✗
    assert!(
        !out.status.success()
            || combined.contains("false")
            || combined.contains("offline")
            || combined.contains("refused")
            || combined.contains('\u{2717}')  // ✗
            || combined.contains('\u{2718}')  // ✘
            || combined.contains("not reachable")
            || combined.contains("RPC"),
        "health command should indicate offline gateway, got: {combined}"
    );
}

#[test]
fn test_cli_status_offline() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    // Status without gateway should fail gracefully
    let out = Command::new(&bin).arg("status").output().unwrap();
    // Should not crash — we just check it terminates
    let _stderr = String::from_utf8(out.stderr).unwrap();
}

#[test]
fn test_cli_gateway_subcommand_exists() {
    let bin = cli_bin();
    if !bin.exists() {
        return;
    }

    // Verify gateway subcommand is recognized (--help should succeed)
    let out = Command::new(&bin)
        .args(["gateway", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("start") || stdout.contains("status") || stdout.contains("gateway"),
        "gateway help should list subcommands"
    );
}
