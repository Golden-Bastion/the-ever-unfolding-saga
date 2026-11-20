# Changelog

This file records completed work in the current unreleased development line. Older releases are preserved in [CHANGELOG-HISTORY.md](CHANGELOG-HISTORY.md), while unfinished work remains in [ROADMAP.md](ROADMAP.md). The concise, manually authored summary for the release being prepared or most recently published belongs in [RELEASE-NOTES.md](RELEASE-NOTES.md).

## Unreleased — v0.53.x

### Release and workflow maintenance

- Added a manually maintained `RELEASE-NOTES.md` for concise, version-specific GitHub Release summaries. CI/CD validates its version heading and uses its title and body instead of extracting the large development changelog.
- Removed automatic workflow-run cleanup from GitHub Actions. The standalone cleanup script now targets a specified release, previews deletions by default, requires `--apply` to delete, retries transient API errors, and preserves the selected release runs and active runs.
