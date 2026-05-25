#!/usr/bin/env bash
# Refresh vendored phpstorm-stubs content from upstream.
#
# Usage:
#   ./scripts/sync-phpstorm-stubs.sh [TAG_OR_COMMIT]
#
# If no argument is given, the current HEAD of the upstream default branch is used.
#
# What this script updates:
#   crates/mir-analyzer/stubs/         — stub PHP files for extensions we support
#   crates/mir-analyzer/stubs/PhpStormStubsMap.php  — build-time symbol index
#   crates/mir-analyzer/stubs/UPSTREAM_REV          — pinned upstream commit
#
# Run this manually when you want to pick up upstream fixes or new built-ins.
# The repo works without the submodule; contributors never need to run this
# unless they are refreshing stub content.

set -euo pipefail

UPSTREAM="https://github.com/JetBrains/phpstorm-stubs.git"
CRATE_STUBS="crates/mir-analyzer/stubs"

# Extensions we vendor.  Must match the directory names in phpstorm-stubs.
EXTENSIONS=(
  Core
  SPL
  standard
  date
  curl
  json
  pcre
  hash
  filter
  bcmath
  iconv
  openssl
  dom
  Reflection
  gd
  pdo
  intl
)

cd "$(git rev-parse --show-toplevel)"

REF="${1:-HEAD}"

echo "Cloning phpstorm-stubs at $REF …"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

git clone --quiet --depth 1 "$UPSTREAM" "$TMPDIR/phpstorm-stubs"
cd "$TMPDIR/phpstorm-stubs"
ACTUAL_REV=$(git rev-parse HEAD)
cd -

echo "Upstream rev: $ACTUAL_REV"

echo "Syncing PhpStormStubsMap.php …"
cp "$TMPDIR/phpstorm-stubs/PhpStormStubsMap.php" "$CRATE_STUBS/PhpStormStubsMap.php"

echo "Syncing extension stubs …"
for ext in "${EXTENSIONS[@]}"; do
  src="$TMPDIR/phpstorm-stubs/$ext"
  dst="$CRATE_STUBS/$ext"
  if [ -d "$src" ]; then
    rm -rf "$dst"
    cp -r "$src" "$dst"
    echo "  synced $ext"
  else
    echo "  WARNING: $ext not found in upstream — skipping"
  fi
done

echo "$ACTUAL_REV" > "$CRATE_STUBS/UPSTREAM_REV"

echo ""
echo "Done.  Upstream rev pinned to: $ACTUAL_REV"
echo "Next steps:"
echo "  cargo check -p mir-analyzer   # rebuild the generated indexes"
echo "  cargo test  -p mir-analyzer   # verify fixtures still pass"
echo "  git add $CRATE_STUBS && git commit -m 'chore(stubs): refresh from phpstorm-stubs $ACTUAL_REV'"
