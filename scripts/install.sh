#!/usr/bin/env bash
set -euo pipefail

OWNER="harveyTon"
REPO="me"
DEFAULT_INSTALL_DIR="/usr/local/bin"
FALLBACK_INSTALL_DIR="${HOME}/.local/bin"
TMPDIR_CLEANUP=""

usage() {
  cat <<'EOF'
Install me from GitHub Releases.

Usage:
  scripts/install.sh [version]

Examples:
  scripts/install.sh
  scripts/install.sh v0.1.2
EOF
}

log() {
  printf '%s\n' "$*"
}

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

cleanup() {
  if [[ -n "${TMPDIR_CLEANUP:-}" ]]; then
    rm -rf "${TMPDIR_CLEANUP}"
  fi
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

detect_os() {
  case "$(uname -s)" in
    Darwin) echo "macos" ;;
    Linux) echo "linux" ;;
    *) fail "unsupported OS: $(uname -s); use the release archives manually on this platform" ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64) echo "x64" ;;
    arm64|aarch64) echo "arm64" ;;
    *) fail "unsupported architecture: $(uname -m)" ;;
  esac
}

resolve_version() {
  local requested="${1:-latest}"
  if [[ "${requested}" != "latest" ]]; then
    printf '%s\n' "${requested}"
    return
  fi

  local effective
  effective="$(
    curl -fsSL -o /dev/null -w '%{url_effective}' \
      "https://github.com/${OWNER}/${REPO}/releases/latest"
  )" || fail "failed to resolve the latest release"

  local version="${effective##*/}"
  [[ -n "${version}" && "${version}" == v* ]] || fail "could not determine latest release version"
  printf '%s\n' "${version}"
}

artifact_name() {
  local version="$1"
  local os="$2"
  local arch="$3"
  printf 'me-%s-%s-%s.tar.gz\n' "${version}" "${os}" "${arch}"
}

checksum_tool() {
  if command -v sha256sum >/dev/null 2>&1; then
    echo "sha256sum"
  elif command -v shasum >/dev/null 2>&1; then
    echo "shasum"
  else
    echo ""
  fi
}

verify_checksum() {
  local archive_path="$1"
  local checksum_path="$2"
  local artifact="$3"
  local tool
  local checksum_line
  tool="$(checksum_tool)"

  [[ -n "${tool}" ]] || {
    log "warning: no SHA-256 tool found; skipping checksum verification"
    return
  }

  checksum_line="$(
    awk -v artifact="${artifact}" '
      $2 == artifact || $2 == ("./" artifact) {
        print $1 "  " artifact
        exit
      }
    ' "${checksum_path}"
  )"

  if [[ -z "${checksum_line}" ]]; then
    fail "checksum entry for ${artifact} was not found"
  fi

  (
    cd "$(dirname "${archive_path}")"
    if [[ "${tool}" == "sha256sum" ]]; then
      printf '%s\n' "${checksum_line}" | sha256sum --check --status -
    else
      printf '%s\n' "${checksum_line}" | shasum -a 256 --check --status
    fi
  ) || fail "checksum verification failed for ${artifact}"
}

pick_install_dir() {
  if [[ -d "${DEFAULT_INSTALL_DIR}" && -w "${DEFAULT_INSTALL_DIR}" ]]; then
    printf '%s\n' "${DEFAULT_INSTALL_DIR}"
    return
  fi

  if [[ ! -e "${DEFAULT_INSTALL_DIR}" && -d "$(dirname "${DEFAULT_INSTALL_DIR}")" && -w "$(dirname "${DEFAULT_INSTALL_DIR}")" ]]; then
    mkdir -p "${DEFAULT_INSTALL_DIR}"
    printf '%s\n' "${DEFAULT_INSTALL_DIR}"
    return
  fi

  mkdir -p "${FALLBACK_INSTALL_DIR}"
  printf '%s\n' "${FALLBACK_INSTALL_DIR}"
}

main() {
  case "${1:-}" in
    -h|--help)
      usage
      exit 0
      ;;
  esac

  require_command curl
  require_command tar

  local version os arch artifact install_dir release_base tmpdir archive_path checksum_path binary_path staged_binary_path
  version="$(resolve_version "${1:-latest}")"
  os="$(detect_os)"
  arch="$(detect_arch)"
  artifact="$(artifact_name "${version}" "${os}" "${arch}")"
  install_dir="$(pick_install_dir)"
  release_base="https://github.com/${OWNER}/${REPO}/releases/download/${version}"

  tmpdir="$(mktemp -d)"
  TMPDIR_CLEANUP="${tmpdir}"
  trap cleanup EXIT

  archive_path="${tmpdir}/${artifact}"
  checksum_path="${tmpdir}/SHA256SUMS.txt"
  binary_path="${install_dir}/me"
  staged_binary_path="${tmpdir}/me.install"

  log "Downloading ${artifact} from ${version}..."
  curl -fsSL "${release_base}/${artifact}" -o "${archive_path}" \
    || fail "failed to download ${artifact}"
  curl -fsSL "${release_base}/SHA256SUMS.txt" -o "${checksum_path}" \
    || fail "failed to download SHA256SUMS.txt"

  verify_checksum "${archive_path}" "${checksum_path}" "${artifact}"

  tar -C "${tmpdir}" -xzf "${archive_path}" || fail "failed to extract ${artifact}"
  [[ -f "${tmpdir}/me" ]] || fail "archive did not contain 'me'"

  if [[ -f "${binary_path}" ]]; then
    log "Replacing existing ${binary_path}"
  else
    log "Installing to ${binary_path}"
  fi

  install -m 0755 "${tmpdir}/me" "${staged_binary_path}" || fail "failed to stage ${binary_path}"
  "${staged_binary_path}" --help >/dev/null 2>&1 || fail "downloaded binary failed verification on this system"
  install -m 0755 "${staged_binary_path}" "${binary_path}" || fail "failed to install ${binary_path}"

  log "Installed me ${version} to ${binary_path}"
  if [[ ":${PATH}:" != *":${install_dir}:"* ]]; then
    log "Add ${install_dir} to PATH if 'me' is not found in a new shell."
  fi
}

main "$@"
