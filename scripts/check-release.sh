#!/usr/bin/env bash
set -euo pipefail

check_windows_target() {
  local target="$1"
  local cargo_cmd=(cargo)

  if command -v rustup >/dev/null 2>&1; then
    rustup target add "${target}"
    cargo_cmd=(rustup run stable cargo)
    export RUSTC
    RUSTC="$(rustup which --toolchain stable rustc)"
  fi

  "${cargo_cmd[@]}" check --locked --target "${target}"
}

cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
bash scripts/check-version-consistency.sh
check_windows_target x86_64-pc-windows-msvc
check_windows_target aarch64-pc-windows-msvc
cargo build --locked --release
bash -n scripts/build.sh
bash -n scripts/build-dist.sh
bash -n scripts/install.sh
MANWIDTH=80 man ./man/man1/me.1 >/dev/null
