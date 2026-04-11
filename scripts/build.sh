#!/usr/bin/env bash
set -euo pipefail

VERSION="${VERSION:-v0.2.0}"
DIST_DIR="${DIST_DIR:-dist}"
TARGET="${1:-host}"

ALL_TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-pc-windows-msvc"
  "aarch64-pc-windows-msvc"
)

mkdir -p "${DIST_DIR}"

usage() {
  cat <<'EOF'
Usage:
  scripts/build.sh [host|all|<rust-target>]

Examples:
  scripts/build.sh
  scripts/build.sh all
  scripts/build.sh aarch64-unknown-linux-gnu
EOF
}

host_targets() {
  case "$(uname -s)" in
    Darwin)
      printf '%s\n' "aarch64-apple-darwin" "x86_64-apple-darwin"
      ;;
    Linux)
      printf '%s\n' "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu"
      ;;
    MINGW*|MSYS*|CYGWIN*)
      printf '%s\n' "x86_64-pc-windows-msvc" "aarch64-pc-windows-msvc"
      ;;
    *)
      echo "unsupported host OS: $(uname -s)" >&2
      exit 1
      ;;
  esac
}

artifact_name() {
  case "$1" in
    aarch64-apple-darwin) echo "me-${VERSION}-macos-arm64.tar.gz" ;;
    x86_64-apple-darwin) echo "me-${VERSION}-macos-x64.tar.gz" ;;
    x86_64-unknown-linux-gnu) echo "me-${VERSION}-linux-x64.tar.gz" ;;
    aarch64-unknown-linux-gnu) echo "me-${VERSION}-linux-arm64.tar.gz" ;;
    x86_64-pc-windows-msvc) echo "me-${VERSION}-windows-x64.zip" ;;
    aarch64-pc-windows-msvc) echo "me-${VERSION}-windows-arm64.zip" ;;
    *)
      echo "unsupported target: $1" >&2
      exit 1
      ;;
  esac
}

build_command() {
  if [[ "${ME_FORCE_CROSS:-0}" == "1" ]]; then
    if ! command -v cross >/dev/null 2>&1; then
      echo "ME_FORCE_CROSS=1 requires cross; install with: cargo install cross --locked" >&2
      exit 1
    fi
    echo cross
    return
  fi

  if requires_cross "$1"; then
    if ! command -v cross >/dev/null 2>&1; then
      echo "target $1 requires cross on this host; install with: cargo install cross --locked" >&2
      exit 1
    fi
    echo cross
  else
    echo cargo
  fi
}

requires_cross() {
  case "$1" in
    aarch64-unknown-linux-gnu)
      [[ "$(uname -s)" != "Linux" ]] || [[ "$(uname -m)" != "aarch64" ]]
      ;;
    *)
      return 1
      ;;
  esac
}

ensure_supported_host() {
  case "$1" in
    aarch64-apple-darwin|x86_64-apple-darwin)
      [[ "$(uname -s)" == "Darwin" ]] || {
        echo "target $1 requires a macOS runner" >&2
        exit 1
      }
      ;;
    x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu)
      if [[ "$(uname -s)" != "Linux" ]] && [[ "$1" != "aarch64-unknown-linux-gnu" ]]; then
        echo "target $1 requires a Linux runner" >&2
        exit 1
      fi
      ;;
    x86_64-pc-windows-msvc|aarch64-pc-windows-msvc)
      case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) ;;
        *)
          echo "target $1 requires a Windows MSVC runner" >&2
          exit 1
          ;;
      esac
      ;;
  esac
}

package_target() {
  local rust_target="$1"
  local artifact
  local package_dir
  local builder

  artifact="$(artifact_name "${rust_target}")"
  package_dir="$(mktemp -d)"
  builder="$(build_command "${rust_target}")"

  ensure_supported_host "${rust_target}"
  rustup target add "${rust_target}"
  "${builder}" build --locked --release --target "${rust_target}"

  if [[ "${rust_target}" == *windows-msvc ]]; then
    local binary_path="${package_dir}/me.exe"
    local artifact_path="${DIST_DIR}/${artifact}"
    cp "target/${rust_target}/release/me.exe" "${binary_path}"
    if command -v cygpath >/dev/null 2>&1; then
      binary_path="$(cygpath -w "${binary_path}")"
      artifact_path="$(cygpath -w "${artifact_path}")"
    fi
    if command -v powershell.exe >/dev/null 2>&1; then
      powershell.exe -NoProfile -Command "Compress-Archive -Path '${binary_path}' -DestinationPath '${artifact_path}' -Force"
    elif command -v pwsh >/dev/null 2>&1; then
      pwsh -NoProfile -Command "Compress-Archive -Path '${binary_path}' -DestinationPath '${artifact_path}' -Force"
    else
      echo "zip packaging requires PowerShell for ${rust_target}" >&2
      exit 1
    fi
  else
    cp "target/${rust_target}/release/me" "${package_dir}/me"
    tar -C "${package_dir}" -czf "${DIST_DIR}/${artifact}" me
  fi

  rm -rf "${package_dir}"
  echo "wrote ${DIST_DIR}/${artifact}"
}

case "${TARGET}" in
  -h|--help)
    usage
    ;;
  host)
    while IFS= read -r rust_target; do
      package_target "${rust_target}"
    done < <(host_targets)
    ;;
  all)
    for rust_target in "${ALL_TARGETS[@]}"; do
      package_target "${rust_target}"
    done
    ;;
  *)
    package_target "${TARGET}"
    ;;
esac
