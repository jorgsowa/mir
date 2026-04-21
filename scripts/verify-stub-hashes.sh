#!/usr/bin/env bash
# Verifies that each stubs/{ext}/ input tree matches the `input-hash` header
# committed in crates/mir-analyzer/src/generated/stubs_{ext}.rs. Fast check —
# does not compile the workspace.
#
# The hash format must match `hash_input_tree` in crates/mir-stubs-gen/src/main.rs:
# blake3 over, for each file in sorted relative-path order, `relpath \0 content \0`.
#
# Requires: b3sum.

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
stubs_dir="$repo_root/stubs"
gen_dir="$repo_root/crates/mir-analyzer/src/generated"

if ! command -v b3sum >/dev/null 2>&1; then
    echo "::error::b3sum not found. Install via 'apt install b3sum' or 'cargo install b3sum'."
    exit 1
fi

failed=0

compute_hash() {
    local ext_dir=$1
    (
        cd "$ext_dir"
        find . -type f | sed 's|^\./||' | LC_ALL=C sort | while IFS= read -r rel; do
            printf '%s\0' "$rel"
            cat "./$rel"
            printf '\0'
        done
    ) | b3sum --no-names | awk '{print $1}'
}

for ext_dir in "$stubs_dir"/*/; do
    ext_name=$(basename "$ext_dir")
    module_name="stubs_${ext_name//-/_}"
    generated_file="$gen_dir/${module_name}.rs"

    if [[ ! -f "$generated_file" ]]; then
        echo "::error file=stubs/$ext_name::Missing generated file $generated_file"
        failed=1
        continue
    fi

    expected=$(grep -oE '^// input-hash: blake3:[0-9a-f]+' "$generated_file" | sed 's|.*blake3:||') || true
    if [[ -z "$expected" ]]; then
        echo "::error file=$generated_file::Missing 'input-hash: blake3:<hex>' header"
        failed=1
        continue
    fi

    actual=$(compute_hash "$ext_dir")

    if [[ "$expected" != "$actual" ]]; then
        echo "::error file=stubs/$ext_name::Hash mismatch for $ext_name"
        echo "  committed: $expected"
        echo "  current:   $actual"
        echo "  Run 'cargo run -p mir-stubs-gen -- $ext_name' and commit the result."
        failed=1
    fi
done

if (( failed )); then
    exit 1
fi

echo "All stub input-hashes match."
