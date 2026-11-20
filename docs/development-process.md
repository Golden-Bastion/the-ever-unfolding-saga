# Development process

This document records ongoing engineering and documentation practices. It is not a product roadmap or a list of future features.

## Where information belongs

- [ROADMAP.md](../ROADMAP.md) is the authoritative overview of unfinished work, current priorities, and milestone sequencing. Do not rely on open issues or scattered documentation to discover the project's major remaining goals.
- [CHANGELOG.md](../CHANGELOG.md) records completed work in the current development line.
- [RELEASE-NOTES.md](../RELEASE-NOTES.md) is the short, manually authored summary used for the next GitHub Release; it is not a replacement for the development changelog.
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

Keep \`CHANGELOG.md\` as the detailed development record. It may contain implementation history, issue references, validation details, and incremental version entries; it is not the GitHub Release summary.

Before creating a version tag, manually replace the template in \`RELEASE-NOTES.md\` with a concise, player-facing summary for that exact release. Its first line must be \`# vMAJOR.MINOR.PATCH — Codename\`, with the version matching the tag. CI/CD rejects a mismatched or incomplete heading, uses that heading as the GitHub Release title, and publishes everything below it as the release description. Do not copy the full changelog into this file.

After a release is published, the GitHub Release retains its own title and description. Reset \`RELEASE-NOTES.md\` to its template for the next release when appropriate. Keep the current and historical development records in \`CHANGELOG.md\` and \`CHANGELOG-HISTORY.md\`; move older release sections into the history file when a newer development line supersedes them. Update \`ROADMAP.md\` separately to show the current development line and remaining priorities.

## Manual GitHub Actions run cleanup

Workflow-run cleanup is intentionally not part of CI/CD and is not triggered automatically. After a release has completed successfully, run \`.github/scripts/cleanup-workflow-runs.sh --release-tag vMAJOR.MINOR.PATCH\` from the repository checkout to preview cleanup for that release. The script retains the successful CI/CD and Android APK runs for the selected tag, the newest completed PR Checks and License check runs, and every active run. It retries transient GitHub API failures and isolates deletion failures.

The script defaults to a dry run. Review the proposed deletions, then add \`--apply\` to perform them. It requires \`gh\`, \`jq\`, and a GitHub CLI authentication session or \`GH_TOKEN\` with Actions write permission. It also accepts \`--repo owner/repository\` when run outside the repository checkout. Deleting an old workflow run also deletes artifacts attached to that run; release assets uploaded to the GitHub Release are separate and remain available.

## Cross-platform release coordination

Tag-triggered desktop and Android workflows run independently. The CI/CD workflow therefore creates the GitHub Release as a draft immediately after validating the tag and reading its title/body from the manually maintained `RELEASE-NOTES.md`, before the desktop build matrix starts. The Android workflow keeps its signed APKs as normal workflow artifacts, then uploads them to that draft release with replacement enabled so a retry does not create duplicate assets.

The desktop release job uploads its five archives to the same draft, verifies that all five desktop archives and all three Android APKs are attached, and only then changes the release from draft to published. If either platform build or upload fails, the draft remains available for recovery and is not presented as a complete public release. Desktop and Android workflow artifacts remain independently available for their configured seven-day retention period; release assets are the durable cross-workflow hand-off.

The optional itch.io publication continues to wait for and download the complete eight-asset set from the GitHub Release before publishing platform channels.
