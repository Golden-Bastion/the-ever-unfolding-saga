#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "Usage: $0 RELEASE-NOTES.md VERSION RELEASE_NOTES_OUTPUT" >&2
    exit 2
fi

release_file=$1
version=$2
release_notes_file=$3

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Invalid release version: $version" >&2
    exit 2
fi

if [[ ! -f "$release_file" ]]; then
    echo "Release notes file not found: $release_file" >&2
    exit 2
fi

if [[ -z "${GITHUB_OUTPUT:-}" ]]; then
    echo "GITHUB_OUTPUT must point to the GitHub Actions output file." >&2
    exit 2
fi

heading="$(head -n 1 "$release_file" | tr -d '\r')"
expected_prefix="# v$version — "

if [[ "$heading" != "$expected_prefix"* ]]; then
    echo "RELEASE-NOTES.md must start with '$expected_prefix<release title>' for tag v$version." >&2
    exit 1
fi

release_title="${heading:2}"
release_codename="${release_title#v$version — }"
if [[ ! "$release_codename" =~ [^[:space:]] || "$release_codename" == "Release Codename" ]]; then
    echo "The release title must include a non-empty codename after the version." >&2
    exit 1
fi

if grep -Fq "Replace this template before creating the release tag." "$release_file"; then
    echo "RELEASE-NOTES.md still contains template instructions." >&2
    exit 1
fi

tail -n +2 "$release_file" > "$release_notes_file"

if ! grep -q '[^[:space:]]' "$release_notes_file"; then
    echo "RELEASE-NOTES.md must contain release notes below its title." >&2
    exit 1
fi

printf 'release_name=%s\n' "$release_title" >> "$GITHUB_OUTPUT"
echo "Read the manually maintained release title and notes for v$version."
