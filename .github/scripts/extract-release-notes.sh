#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "Usage: $0 CHANGELOG.md VERSION RELEASE_NOTES_OUTPUT" >&2
    exit 2
fi

changelog_file=$1
version=$2
release_notes_file=$3

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Invalid release version: $version" >&2
    exit 2
fi

if [[ ! -f "$changelog_file" ]]; then
    echo "Changelog file not found: $changelog_file" >&2
    exit 2
fi

if [[ -z "${GITHUB_OUTPUT:-}" ]]; then
    echo "GITHUB_OUTPUT must point to the GitHub Actions output file." >&2
    exit 2
fi

codename_file="$(mktemp)"
trap 'rm -f "$codename_file"' EXIT

awk -v version="$version" -v codename_file="$codename_file" '
    BEGIN {
        heading = "## v" version
    }
    $0 == heading || index($0, heading " ") == 1 {
        found = 1
        next
    }
    found && /^## / {
        exit
    }
    found {
        line = $0
        sub(/\r$/, "", line)

        if (line ~ /^\*\*Codename:\*\*/) {
            if (codename_seen) {
                error = "The release section contains more than one Codename field."
                next
            }

            codename_seen = 1
            sub(/^\*\*Codename:\*\*[[:space:]]*/, "", line)
            sub(/[[:space:]]*$/, "", line)

            if (line == "") {
                error = "The Codename field must contain a non-empty, single-line value."
            } else {
                codename = line
            }
            next
        }

        print
    }
    END {
        if (!found) {
            print "No matching release section for v" version " in CHANGELOG.md." > "/dev/stderr"
            exit 1
        }
        if (error != "") {
            print error > "/dev/stderr"
            exit 1
        }
        if (codename_seen) {
            print codename > codename_file
        }
    }
' "$changelog_file" > "$release_notes_file"

if ! grep -q '[^[:space:]]' "$release_notes_file"; then
    echo "Release notes section for v$version is empty." >&2
    exit 1
fi

release_name="v$version"
if [[ -s "$codename_file" ]]; then
    IFS= read -r codename < "$codename_file"
    release_name+=" — $codename"
fi

printf 'release_name=%s\n' "$release_name" >> "$GITHUB_OUTPUT"
echo "Extracted v$version release notes and release name from CHANGELOG.md."
