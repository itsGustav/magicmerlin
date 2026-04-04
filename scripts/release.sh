#!/bin/bash
set -e
VERSION=${1:-0.9.0}
echo "Releasing MagicMerlin $VERSION"

# Update workspace version
sed -i '' "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Verify build
cargo build --release

# Tag
git tag -a "v$VERSION" -m "Release $VERSION"
git push && git push --tags

echo "Done! Create GitHub release from tag v$VERSION"
