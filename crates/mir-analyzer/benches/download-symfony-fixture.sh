#!/usr/bin/env bash
# Downloads the Symfony fixture used by the full-project Salsa DB tests.
# Pins the exact commit so every machine analyzes the same source tree.
#
# Prerequisites: git
set -euo pipefail

SYMFONY_COMMIT="27515cd"
DEST="$(dirname "$0")/fixtures/symfony"

if [ -d "$DEST" ]; then
    echo "Fixture already exists at $DEST — skipping."
    echo "Delete it and re-run to refresh: rm -rf $DEST"
    exit 0
fi

echo "Cloning symfony/symfony at commit $SYMFONY_COMMIT into $DEST ..."
git clone --depth=1 https://github.com/symfony/symfony.git "$DEST"
git -C "$DEST" fetch --depth=1 origin "$SYMFONY_COMMIT"
git -C "$DEST" checkout --detach "$SYMFONY_COMMIT"

echo ""
echo "Done. Run the heavy Symfony integration tests with:"
echo "  for t in crates/mir-analyzer/tests/symfony_query_*.rs; do"
echo "    cargo test -p mir-analyzer --test \"\$(basename \"\${t%.rs}\")\" -- --ignored --nocapture"
echo "  done"
