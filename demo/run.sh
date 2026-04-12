#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURES="$ROOT/demo/fixtures"

cd "$ROOT"

echo '$ cargo run --quiet -- target demo/fixtures'
cargo run --quiet -- target "$FIXTURES"

echo
echo '$ cargo run --quiet -- --path rs-find/architecture demo/fixtures'
cargo run --quiet -- --path rs-find/architecture "$FIXTURES"

echo
echo '$ cargo run --quiet -- --ignore-case BORROW demo/fixtures'
cargo run --quiet -- --ignore-case BORROW "$FIXTURES"

echo
echo '$ cargo run --quiet -- link demo/fixtures'
cargo run --quiet -- link "$FIXTURES"
