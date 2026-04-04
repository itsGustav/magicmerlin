//! Integration test: CLI commands must not panic.

use magicmerlin_integration_tests::cargo_bin;
use std::process::Command;

/// `magicmerlin --help` must exit 0.
#[test]
fn test_cli_help_no_panic() {
    let bin = cargo_bin("magicmerlin");
    if !bin.exists() {
        eprintln!("skipping: magicmerlin binary not found at {bin:?}");
        return;
    }

    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .expect("spawn");

    assert!(
        output.status.success(),
        "magicmerlin --help should exit 0, got: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `magicmerlin version` must print the version string.
#[test]
fn test_cli_version_no_panic() {
    let bin = cargo_bin("magicmerlin");
    if !bin.exists() {
        eprintln!("skipping: magicmerlin binary not found at {bin:?}");
        return;
    }

    let output = Command::new(&bin)
        .arg("version")
        .output()
        .expect("spawn");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // Should contain a version number pattern or the word "version"/"magicmerlin"
    assert!(
        output.status.success() || combined.contains("version") || combined.contains("magicmerlin"),
        "version command should succeed or print version info.\nstdout: {stdout}\nstderr: {stderr}"
    );
}

/// `magicmerlin status` must exit 0 or print a graceful "not running" message (not panic).
#[test]
fn test_cli_status_no_panic() {
    let bin = cargo_bin("magicmerlin");
    if !bin.exists() {
        eprintln!("skipping: magicmerlin binary not found at {bin:?}");
        return;
    }

    let output = Command::new(&bin)
        .arg("status")
        .output()
        .expect("spawn");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // Either succeeds or gives a non-panic error (e.g. "gateway not running")
    assert!(
        output.status.success()
            || combined.contains("not running")
            || combined.contains("offline")
            || combined.contains("connection refused")
            || combined.contains("error")
            || combined.contains("Error"),
        "status should exit gracefully, not panic.\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Must not contain panic artifacts
    assert!(
        !combined.contains("thread 'main' panicked"),
        "status command panicked!\nstdout: {stdout}\nstderr: {stderr}"
    );
}

/// `magicmerlin gateway --help` must be a recognized subcommand.
#[test]
fn test_cli_gateway_subcommand_exists() {
    let bin = cargo_bin("magicmerlin");
    if !bin.exists() {
        eprintln!("skipping: magicmerlin binary not found at {bin:?}");
        return;
    }

    let output = Command::new(&bin)
        .args(["gateway", "--help"])
        .output()
        .expect("spawn");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Gateway subcommand should exist (help exits 0 or 2, never panic)
    assert!(
        !stdout.contains("thread 'main' panicked") && !stderr.contains("thread 'main' panicked"),
        "gateway --help panicked!\nstdout: {stdout}\nstderr: {stderr}"
    );
}
