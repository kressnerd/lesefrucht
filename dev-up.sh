#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
podman-compose -f compose.dev.yaml up -d --build
