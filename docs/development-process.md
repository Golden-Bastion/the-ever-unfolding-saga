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

Keep an `Unreleased` section for the current development line and retain the most recent published release section in `CHANGELOG.md`. The release workflow extracts the section whose heading exactly matches `## vMAJOR.MINOR.PATCH` and uses its contents as the GitHub Release body. Keep that exact section in place until its tag-triggered release has been published. When a newer release supersedes it, move older release sections into `CHANGELOG-HISTORY.md` while preserving their headings and entries. Do not archive unreleased work or the current release notes before publication. Update `ROADMAP.md` separately to show the current development line and remaining priorities.
