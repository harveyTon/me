#!/usr/bin/env bash
set -euo pipefail

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
[[ -n "${version}" ]] || {
  echo "could not determine version from Cargo.toml" >&2
  exit 1
}

grep -q "name = \"me\"" Cargo.lock
grep -A1 'name = "me"' Cargo.lock | grep -q "version = \"${version}\""
grep -q 'env!("CARGO_PKG_VERSION")' src/shell_integration.rs
grep -q "^## v${version}$" RELEASE_NOTES.md

echo "version consistency ok: v${version}"
