#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
case "$(uname -s)" in
  Linux) platform=linux ;;
  Darwin) platform=macos ;;
  *) echo 'This script requires Linux or macOS.' >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64) arch=x64 ;;
  arm64|aarch64) arch=arm64 ;;
  *) echo 'Unsupported architecture.' >&2; exit 1 ;;
esac

destination="release/$platform-$arch"
pnpm run build
if [[ "$platform" == macos ]]; then
  # Tauri creates the native .app with its icon and embedded frontend.
  (cd tauri-app && pnpm exec tauri build --bundles app)
else
  cargo build --manifest-path tauri-app/Cargo.toml -p uniconvert --release --features custom-protocol
fi
cargo build --manifest-path tauri-app/Cargo.toml -p uniconvert-cli --target-dir tauri-app/target/cli --release
mkdir -p "$destination"
cp tauri-app/target/release/uniconvert "$destination/uniconvert"
cp tauri-app/target/cli/release/uniconvert "$destination/uniconvert-cli"
if [[ "$platform" == macos ]]; then
  ditto tauri-app/target/release/bundle/macos/uniconvert.app "$destination/UniConvert.app"
fi
echo "Built $destination"
