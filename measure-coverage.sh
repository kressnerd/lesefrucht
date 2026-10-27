#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

mkdir -p coverage/profraw/non-e2e
rm -f coverage/profraw/non-e2e/*.profraw coverage/non-e2e.profdata

RUSTFLAGS="-C instrument-coverage" \
LLVM_PROFILE_FILE="$(pwd)/coverage/profraw/non-e2e/%p-%m.profraw" \
cargo test --test features_runner -- --tags 'not @e2e'

llvm-profdata merge -sparse coverage/profraw/non-e2e/*.profraw -o coverage/non-e2e.profdata
grcov coverage/profraw/non-e2e \
  -b target/debug \
  -t cobertura \
  -o coverage/non-e2e-cobertura.xml \
  --ignore 'tests/*' \
  --ignore 'xtask/*' \
  --ignore 'twins/*'
