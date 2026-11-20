#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
extractor="$script_dir/extract-release-notes.sh"
test_dir="$(mktemp -d)"
trap 'rm -rf "$test_dir"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

run_extractor() {
    local release_file=$1
    local version=$2
    local notes=$3

    : > "$test_dir/github-output"
    GITHUB_OUTPUT="$test_dir/github-output" bash "$extractor" "$release_file" "$version" "$notes"
}

cat > "$test_dir/valid.md" <<'EOF'
# v0.53.0 — The Ashen Threshold

A concise summary written specifically for this release.

### Highlights
- A meaningful player-facing change.
- Another important improvement.
EOF

run_extractor "$test_dir/valid.md" "0.53.0" "$test_dir/notes.md"
grep -Fqx 'release_name=v0.53.0 — The Ashen Threshold' "$test_dir/github-output" \
    || fail "the release title was not read from the first heading"
grep -Fq 'A concise summary written specifically for this release.' "$test_dir/notes.md" \
    || fail "release body content was not extracted"
grep -Fq 'Another important improvement.' "$test_dir/notes.md" \
    || fail "the complete concise release body was not extracted"
! grep -Fq '# v0.53.0' "$test_dir/notes.md" \
    || fail "the title heading leaked into the release body"

cat > "$test_dir/wrong-version.md" <<'EOF'
# v0.52.70 — The Ashen Threshold

This file belongs to a different release.
EOF
if run_extractor "$test_dir/wrong-version.md" "0.53.0" "$test_dir/wrong-version-notes.md"; then
    fail "release notes for a different version were accepted"
fi

cat > "$test_dir/missing-title.md" <<'EOF'
# v0.53.0

Release body.
EOF
if run_extractor "$test_dir/missing-title.md" "0.53.0" "$test_dir/missing-title-notes.md"; then
    fail "a missing codename was accepted"
fi

cat > "$test_dir/empty-body.md" <<'EOF'
# v0.53.0 — The Ashen Threshold

   
EOF
if run_extractor "$test_dir/empty-body.md" "0.53.0" "$test_dir/empty-body-notes.md"; then
    fail "empty release notes were accepted"
fi

# The checked-in file is deliberately a template and must not pass as real release notes.
if run_extractor "$repo_root/RELEASE-NOTES.md" "0.53.0" "$test_dir/template-notes.md"; then
    fail "the unfilled release-notes template was accepted"
fi

echo "Manual release-note extraction tests passed."
