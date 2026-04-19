#!/usr/bin/env bash
# Downloads the laravel/framework fixture used by the real-world benchmarks.
# Pins the version so every machine analyses the exact same PHP files.
# To upgrade: delete benches/fixtures/laravel/, bump LARAVEL_VERSION, re-run.
#
# Prerequisites: git, composer
set -euo pipefail

LARAVEL_VERSION="v11.44.7"
DEST="$(dirname "$0")/fixtures/laravel"

if [ -d "$DEST" ]; then
    echo "Fixture already exists at $DEST — skipping."
    echo "Delete it and re-run to refresh: rm -rf $DEST"
    exit 0
fi

echo "Cloning laravel/framework $LARAVEL_VERSION into $DEST ..."
git clone --depth=1 --branch "$LARAVEL_VERSION" \
    https://github.com/laravel/framework "$DEST"

echo ""
echo "Installing vendor dependencies (composer install) ..."
composer install \
    --working-dir="$DEST" \
    --no-scripts \
    --no-plugins \
    --no-interaction \
    --prefer-dist \
    --quiet

echo ""
echo "Done. Run benchmarks with:"
echo "  cargo bench -p mir-analyzer --bench analyze_real_world"
