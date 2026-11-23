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
  # Ad-hoc signing is configured in tauri.macos.conf.json. It is free and keeps
  # the app runnable on Apple Silicon, but does not replace Apple notarization.
  pnpm tauri build --bundles app,dmg
else
  cargo build --manifest-path tauri-app/Cargo.toml -p uniconvert --release --features custom-protocol
fi
cargo build --manifest-path tauri-app/Cargo.toml -p uniconvert-cli --target-dir tauri-app/target/cli --release
mkdir -p "$destination"
cp tauri-app/target/release/uniconvert "$destination/uniconvert"
cp tauri-app/target/cli/release/uniconvert "$destination/uniconvert-cli"
if [[ "$platform" == macos ]]; then
  ditto tauri-app/target/release/bundle/macos/uniconvert.app "$destination/UniConvert.app"
  dmg_files=(tauri-app/target/release/bundle/dmg/*.dmg)
  if [[ ${#dmg_files[@]} -ne 1 || ! -f "${dmg_files[0]}" ]]; then
    echo 'Expected exactly one macOS DMG bundle.' >&2
    exit 1
  fi
  cp "${dmg_files[0]}" "$destination/"
fi
echo "Built $destination"
