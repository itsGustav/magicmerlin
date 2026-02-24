use magicmerlin_compat::providers::{SnapshotBackedProviders, ToolRegistryProvider};

#[test]
fn can_load_snapshot_bundle_and_hash() {
  let providers = SnapshotBackedProviders::load().expect("load snapshot providers");
  let hashes = providers.hashes().expect("hashes");
  assert!(!hashes.fingerprint.is_empty());
  assert!(hashes.files.contains_key("openclawHelp"));
  assert!(hashes.files.contains_key("openclawStatusJson"));
  // At least a few tools
  assert!(providers.tool_names().len() >= 5);
}
