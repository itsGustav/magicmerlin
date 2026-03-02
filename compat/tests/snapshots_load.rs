use magicmerlin_compat::providers::{CliProvider, SnapshotBackedProviders, ToolRegistryProvider};

#[test]
fn can_load_snapshot_bundle_and_hash() {
    let providers = SnapshotBackedProviders::load().expect("load snapshot providers");
    let hashes = providers.hashes().expect("hashes");

    assert!(!hashes.fingerprint.is_empty());
    assert!(hashes.files.contains_key("openclawHelp"));
    assert!(hashes.files.contains_key("openclawStatusJson"));

    // At least a few tools
    assert!(providers.tool_names().len() >= 5);

    // Snapshot headers should be present (capture metadata).
    assert!(providers.openclaw_help_text().starts_with("# Snapshot:"));
    assert!(providers.openclaw_cron_help_text().starts_with("# Snapshot:"));

    // If the manifest provides expected hashes/fingerprint, enforce them.
    let manifest = &providers.snapshots().manifest;

    if let Some(expected_fp) = &manifest.fingerprint {
        assert_eq!(&hashes.fingerprint, expected_fp, "fingerprint mismatch");
    }

    if let Some(expected_hashes) = &manifest.snapshot_hashes {
        for (k, v) in expected_hashes {
            let got = hashes.files.get(k);
            assert_eq!(got, Some(v), "hash mismatch for {k}");
        }
    }
}

