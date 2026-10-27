#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

case "${1:-}" in
  --prove-strictness)
    if cargo test --test features_runner -- --input tests/fixtures/strictness.feature; then
      echo "strictness proof failed: an unbound step passed" >&2
      exit 1
    fi
    echo "strictness proof passed: the unbound step failed the run"
    ;;
  --e2e)
    trap 'podman-compose -f compose.e2e.yaml down -v' EXIT
    podman-compose -f compose.e2e.yaml up -d --build
    cargo test --test features_runner -- --tags '@e2e'
    ;;
  *)
    cargo test --test features_runner -- --tags 'not @e2e'
    ;;
esac
