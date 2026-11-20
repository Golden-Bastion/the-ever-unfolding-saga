# Development process

This document records ongoing engineering and documentation practices. It is not a product roadmap or a list of future features.

## Where information belongs

- [ROADMAP.md](../ROADMAP.md) is the authoritative overview of unfinished work, current priorities, and milestone sequencing. Do not rely on open issues or scattered documentation to discover the project's major remaining goals.
- [CHANGELOG.md](../CHANGELOG.md) records completed work in the current development line.
- [CHANGELOG-HISTORY.md](../CHANGELOG-HISTORY.md) preserves completed work from earlier lines and milestones.
- [Design specifications](design/README.md) describe intended future behavior and settled design constraints.
- [Architecture](architecture.md) and [system documentation](systems/) describe current implementation ownership, contracts, and behavior.
- GitHub issues support discussion, scoped implementation tasks, and validation. They supplement the roadmap; they do not replace its project-level overview.

## Engineering practices

- Keep authoritative gameplay rules and state independent of Bevy presentation and platform-specific input where practical.
- Give each system clear ownership of its state and explicit mutation paths. Avoid parallel representations that can disagree.
- Use stable IDs for entities and references; display names are not identities.
- Keep content data-driven where practical, validate it before runtime use, and preserve authored-content precedence.
- Persist consequential world state and define clearly which state belongs to a character, the world, or presentation.
- Preserve existing behavior and save compatibility unless a change intentionally alters it. Use conservative migrations and safe defaults for older saves.
- Add focused tests for changed behavior, invariants, compatibility, and regression-prone boundaries.
- Keep platform behavior equivalent across desktop and Android, using platform-appropriate controls rather than separate gameplay rules.
- Prefer changes at responsibility boundaries. Do not split files solely to reduce line counts or add abstractions without a concrete ownership or reuse benefit.

## Development and review workflow

- Inspect the current main branch and relevant implementation before changing code or documentation.
- Work on a dedicated branch and use pull requests for normal repository changes.
- Update the roadmap when the status or sequencing of unfinished work changes; update the changelog when work is completed.
- Review every changed file for consistency, stale references, and unintended scope changes.
- Use Semantic Versioning for project releases. Documentation-only reorganization does not by itself require a version bump.
- Validate changes with the checks appropriate to their scope and inspect the final PR head before relying on CI results.
- Do not describe a planned feature as implemented or a source-level test as proof of device-level behavior.
- Keep third-party asset and audio provenance and license terms traceable in project documentation.

## Changelog and release-note rotation

Keep an `Unreleased` section for the current development line and keep the release notes for the version being published in `CHANGELOG.md` until its tag-triggered release has completed.

For a named release, add one optional, single-line `**Codename:** Name` field immediately below that version's heading. The release workflow extracts the matching version section, removes this metadata line from the release body, and sets the GitHub Release title to `vMAJOR.MINOR.PATCH — Name`. The codename is independent of any descriptive text already present in the version heading. If no codename field is present, the GitHub Release title falls back to the version tag; older changelog sections therefore remain valid without edits. Keep the release section's remaining contents as the release body.

After publication, retain that most recent release section in `CHANGELOG.md`. When a newer release supersedes it, move older release sections into `CHANGELOG-HISTORY.md` while preserving their headings and entries. Do not archive unreleased work or the current release notes before publication. Update `ROADMAP.md` separately to show the current development line and remaining priorities.


## GitHub Actions run cleanup\n\nThe `Cleanup workflow runs` workflow invokes `.github/scripts/cleanup-workflow-runs.sh` after every completed CI/CD run. The script performs cleanup only after a successful push to `main` or a version tag; other events exit without deleting anything. It retains the triggering CI/CD run, the newest completed run for each companion workflow, and all queued or in-progress runs. GitHub API requests use bounded exponential-backoff retries, and deletion failures are isolated so one transient error does not prevent cleanup of other runs. The script supports `--dry-run` for review and `--force` for intentional manual execution. Manual use requires `gh`, `jq`, `GH_TOKEN` with Actions write permission, and `GITHUB_REPOSITORY=owner/repository`.\n\n## Cross-platform release coordination

Tag-triggered desktop and Android workflows run independently. The CI/CD workflow therefore creates the GitHub Release as a draft immediately after validating the tag and extracting its title/body from `CHANGELOG.md`, before the desktop build matrix starts. The Android workflow keeps its signed APKs as normal workflow artifacts, then uploads them to that draft release with replacement enabled so a retry does not create duplicate assets.

The desktop release job uploads its five archives to the same draft, verifies that all five desktop archives and all three Android APKs are attached, and only then changes the release from draft to published. If either platform build or upload fails, the draft remains available for recovery and is not presented as a complete public release. Desktop and Android workflow artifacts remain independently available for their configured seven-day retention period; release assets are the durable cross-workflow hand-off.

The optional itch.io publication continues to wait for and download the complete eight-asset set from the GitHub Release before publishing platform channels.
