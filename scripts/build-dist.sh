#!/usr/bin/env bash
set -euo pipefail

if (($# == 0)); then
  exec "$(dirname "$0")/build.sh" host
fi

exec "$(dirname "$0")/build.sh" "$@"
