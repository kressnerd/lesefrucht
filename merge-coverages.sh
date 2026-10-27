#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

python3 scripts/merge_cobertura.py \
  coverage/non-e2e-cobertura.xml \
  coverage/e2e-cobertura.xml \
  coverage/cobertura-coverage.xml
