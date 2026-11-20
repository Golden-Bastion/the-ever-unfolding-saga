#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ci_workflow="$repo_root/.github/workflows/ci-cd.yml"
android_workflow="$repo_root/.github/workflows/android-build.yml"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

require() {
    local file=$1
    local expected=$2
    local description=$3
    grep -Fq -- "$expected" "$file" || fail "$description"
}

prepare_line="$(grep -n '^  prepare_release:' "$ci_workflow" | cut -d: -f1)"
build_line="$(grep -n '^  build:' "$ci_workflow" | head -n1 | cut -d: -f1)"
[[ -n "$prepare_line" && -n "$build_line" && "$prepare_line" -lt "$build_line" ]] \
    || fail "draft release preparation must be declared before the desktop build job"

require "$ci_workflow" 'needs: prep_build_matrix' \
    "draft release preparation must depend on tag/matrix validation"
require "$ci_workflow" 'gh release create "$TAG_NAME" --draft' \
    "the draft release must be created before platform builds finish"
require "$ci_workflow" 'needs: [prep_build_matrix, prepare_release]' \
    "desktop builds must wait until the draft release has been prepared"
require "$ci_workflow" "needs.prepare_release.result == 'skipped'" \
    "manual build-only dispatch must remain available when release preparation is skipped"
require "$ci_workflow" 'retention-days: 7' \
    "desktop workflow artifacts must retain their independent recovery window"
require "$ci_workflow" 'needs: [prep_build_matrix, build, prepare_release]' \
    "final publication must wait for both desktop builds and draft creation"
require "$ci_workflow" 'teus-$TAG_NAME-android-x86_64.apk' \
    "final publication must require the Android x86_64 APK"
require "$ci_workflow" 'teus-$TAG_NAME-android-aarch64.apk' \
    "final publication must require the Android AArch64 APK"
require "$ci_workflow" 'teus-$TAG_NAME-android-armv7.apk' \
    "final publication must require the Android ARMv7 APK"
require "$ci_workflow" '-F draft=false' \
    "the release must only be published after asset verification"

require "$android_workflow" 'retention-days: 7' \
    "Android workflow artifacts must retain their independent recovery window"
require "$android_workflow" 'for attempt in {1..120}; do' \
    "Android publishing must allow the draft release initialization window"
require "$android_workflow" 'gh release upload "$TAG_NAME" dist/*.apk --clobber --repo "$GITHUB_REPOSITORY"' \
    "Android uploads must safely replace existing release assets on retry"

echo "Release workflow coordination checks passed."
