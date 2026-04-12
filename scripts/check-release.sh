#!/usr/bin/env bash
set -euo pipefail

check_windows_target() {
  local target="$1"

  if command -v rustup >/dev/null 2>&1; then
    rustup target add "${target}"
    RUSTC="$(rustup which rustc)" cargo check --locked --target "${target}"
  else
    cargo check --locked --target "${target}"
  fi
}

cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
check_windows_target x86_64-pc-windows-msvc
check_windows_target aarch64-pc-windows-msvc
cargo build --locked --release
bash -n scripts/build.sh
bash -n scripts/build-dist.sh
bash -n scripts/install.sh
MANWIDTH=80 man ./man/man1/me.1 >/dev/null
