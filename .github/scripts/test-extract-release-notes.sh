#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
extractor="$script_dir/extract-release-notes.sh"
test_dir="$(mktemp -d)"
trap 'rm -rf "$test_dir"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

run_extractor() {
    local changelog=$1
    local version=$2
    local notes=$3

    : > "$test_dir/github-output"
    GITHUB_OUTPUT="$test_dir/github-output" bash "$extractor" "$changelog" "$version" "$notes"
}

cat > "$test_dir/codenamed.md" <<'EOF'
## Unreleased — v0.53.x

No unreleased notes.

## v0.52.70 — Example release

**Codename:** Echoes of the Fallen

Body line one.

### Highlights
- Body line two.

## v0.52.69 — Previous release

Older release content must not be included.
EOF

run_extractor "$test_dir/codenamed.md" "0.52.70" "$test_dir/notes.md"
grep -Fqx 'release_name=v0.52.70 — Echoes of the Fallen' "$test_dir/github-output" \
    || fail "codename was not included in the release name"
grep -Fq 'Body line one.' "$test_dir/notes.md" \
    || fail "release body content was not extracted"
grep -Fq 'Body line two.' "$test_dir/notes.md" \
    || fail "the complete release body was not extracted"
! grep -Fq '**Codename:**' "$test_dir/notes.md" \
    || fail "codename metadata leaked into the release body"
! grep -Fq 'Older release content' "$test_dir/notes.md" \
    || fail "content from the next release section was included"

cat > "$test_dir/legacy.md" <<'EOF'
## v0.52.69 — Legacy release

Legacy release notes remain publishable.
EOF

run_extractor "$test_dir/legacy.md" "0.52.69" "$test_dir/legacy-notes.md"
grep -Fqx 'release_name=v0.52.69' "$test_dir/github-output" \
    || fail "release without a codename did not fall back to its version"
grep -Fq 'Legacy release notes remain publishable.' "$test_dir/legacy-notes.md" \
    || fail "legacy release body was not extracted"

# Exercise the parser against the actual release section being prepared.
run_extractor "$repo_root/CHANGELOG.md" "0.52.70" "$test_dir/current-release-notes.md"
grep -Eq '^release_name=v0\.52\.70( — .+)?$' "$test_dir/github-output" \
    || fail "the current v0.52.70 release name was not generated"
grep -Fq 'These notes cover the complete v0.52.x development line' \
    "$test_dir/current-release-notes.md" \
    || fail "the current v0.52.70 release body was not extracted"

cat > "$test_dir/missing.md" <<'EOF'
## v0.52.69

Some other release.
EOF

if run_extractor "$test_dir/missing.md" "0.52.70" "$test_dir/missing-notes.md"; then
    fail "a missing release section was accepted"
fi

cat > "$test_dir/duplicate.md" <<'EOF'
## v0.52.70

**Codename:** First name
**Codename:** Second name

Release body.
EOF

if run_extractor "$test_dir/duplicate.md" "0.52.70" "$test_dir/duplicate-notes.md"; then
    fail "duplicate codename fields were accepted"
fi

cat > "$test_dir/empty.md" <<'EOF'
## v0.52.70

**Codename:**

Release body.
EOF

if run_extractor "$test_dir/empty.md" "0.52.70" "$test_dir/empty-notes.md"; then
    fail "an empty codename field was accepted"
fi

echo "Release metadata extraction tests passed."
