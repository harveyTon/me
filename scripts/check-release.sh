#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --locked --release
bash -n scripts/build.sh
bash -n scripts/build-dist.sh
bash -n scripts/install.sh
ruby -c Formula/me.rb
MANWIDTH=80 man ./man/man1/me.1 >/dev/null
