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

Development is organized around coherent milestones rather than individual micro-tasks. A milestone groups related implementation, integration, tests, assets, and documentation into one meaningful delivery. Its scope should follow real dependencies and shared system boundaries, not an arbitrary PR size.

1. Inspect the current `main` branch, the authoritative roadmap, relevant design and architecture documents, existing implementation, and related issues.
2. Identify the milestone outcome, dependencies, acceptance criteria, and unresolved design decisions. Ask the project owner only for decisions that cannot be resolved from established project direction.
3. Create one dedicated branch for the milestone and implement its related work together. Use multiple commits as practical checkpoints, but normally open one PR for the integrated milestone rather than a series of PRs for each component.
4. Keep the milestone focused. Do not bundle unrelated cleanup or speculative features. Split work only when it is independently useful, has a separate dependency or release boundary, or would otherwise make the integrated change unsafe or impractical to review.
5. Review the full branch as a whole: check cross-system integration, consistency, regression risks, test coverage, platform implications, and documentation. Add or update tests for the changed behavior and run the checks available in the development environment.
6. Update `ROADMAP.md` as work is completed, deferred, re-scoped, or blocked. Update the changelog and relevant design/system documentation according to their responsibilities.
7. Open a substantial but reviewable PR that describes the milestone goal, the integrated changes, important decisions, validation, and remaining limitations. Hand it to the project owner for review and the established CI/merge process.

Prefer fewer, meaningful PRs over frequent small PRs. Do not inflate scope merely to make a PR larger, and do not fragment a coherent subsystem merely to make PRs smaller. The owner should be able to review a milestone as one understandable improvement.

The project owner remains responsible for product direction, final design decisions, local/device playtesting, subjective gameplay and audiovisual review, and merging. Agents should complete as much implementation and automated verification as practical before requesting that review. Do not claim automated checks prove physical-device behavior or subjective quality.

### Branch and merge policy

- Implementation work belongs on dedicated branches and is delivered through pull requests targeting `main`.
- Do not push implementation changes directly to `main`, merge PRs manually, or bypass the owner's review/merge process.
- Documentation-only workflow or policy updates may be committed directly to `main` when the project owner explicitly requests it.
- Keep commits descriptive and logically grouped. A milestone may contain multiple commits; commit count is not the unit of review.
- Avoid unrelated formatting changes and opportunistic refactoring.
- Validate the final PR head and report what was actually checked. Do not wait for CI to finish or repeatedly poll it when the owner's automation handles checks and merging.

Use Semantic Versioning for project releases. Documentation-only workflow changes do not by themselves require a version bump.

## Changelog and release-note rotation

Keep `CHANGELOG.md` as the detailed development record. It may contain implementation history, issue references, validation details, and incremental version entries; it is not the GitHub Release summary.

Before creating a version tag, manually replace the template in `RELEASE-NOTES.md` with a concise, player-facing summary for that exact release. Its first line must be `# vMAJOR.MINOR.PATCH — Codename`, with the version matching the tag. CI/CD rejects a mismatched or incomplete heading, uses that heading as the GitHub Release title, and publishes everything below it as the release description. Do not copy the full changelog into this file.

After a release is published, the GitHub Release retains its own title and description. `RELEASE-NOTES.md` may retain the concise notes for the just-published release; replace its heading and body with the next version's notes before creating another release tag. Keep the current development record in `CHANGELOG.md` and older completed development lines in `CHANGELOG-HISTORY.md`; move older release sections into the history file when a newer development line supersedes them. Update `ROADMAP.md` separately to show the current development line and remaining priorities.

## Manual GitHub Actions run cleanup

Workflow-run cleanup is intentionally not part of CI/CD and is not triggered automatically. After a release has completed successfully, run `.github/scripts/cleanup-workflow-runs.sh --release-tag vMAJOR.MINOR.PATCH` from the repository checkout to preview cleanup for that release. The script retains the successful CI/CD and Android APK runs for the selected tag, the newest completed PR Checks and License check runs, and every active run. It retries transient GitHub API failures and isolates deletion failures.

The script defaults to a dry run. Review the proposed deletions, then add `--apply` to perform them. It requires `gh`, `jq`, and a GitHub CLI authentication session or `GH_TOKEN` with Actions write permission. It also accepts `--repo owner/repository` when run outside the repository checkout. Deleting an old workflow run also deletes artifacts attached to that run; release assets uploaded to the GitHub Release are separate and remain available.

## Cross-platform release coordination

Tag-triggered desktop and Android workflows run independently. The CI/CD workflow therefore creates the GitHub Release as a draft immediately after validating the tag and reading its title/body from the manually maintained `RELEASE-NOTES.md`, before the desktop build matrix starts. The Android workflow keeps its signed APKs as normal workflow artifacts, then uploads them to that draft release with replacement enabled so a retry does not create duplicate assets.

The desktop release job uploads its five archives to the same draft, verifies that all five desktop archives and all three Android APKs are attached, and only then changes the release from draft to published. If either platform build or upload fails, the draft remains available for recovery and is not presented as a complete public release. Desktop and Android workflow artifacts remain independently available for their configured seven-day retention period; release assets are the durable cross-workflow hand-off for normal tag-triggered publication.

The CI/CD workflow also offers a manual `publish` operation for artifact-only promotion. It locates completed tag-triggered desktop and Android runs by the exact commit SHA, requires all five desktop archives and all three APK artifacts to be present and unexpired, then downloads them without invoking either build pipeline. The `publish_to` choice targets GitHub Releases, itch.io, or both. GitHub publication requires the tag's existing release and replaces matching assets; itch-only publication uploads directly from workflow artifacts. This path needs no Android signing secrets. It is limited by the seven-day workflow-artifact retention period; missing or expired artifacts must be rebuilt through the normal build/release operations.

For ordinary tag-triggered releases, the Android workflow's automatic itch.io mirror continues to wait for and download the complete eight-asset set from the GitHub Release before publishing platform channels.
