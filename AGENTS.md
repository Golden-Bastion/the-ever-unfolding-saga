# AGENTS.md

Instructions for AI agents contributing to **The Ever-Unfolding Saga (TEUS)**.

## 1. Project identity and priorities

The Ever-Unfolding Saga is a graphics-first, character-centred dark fantasy RPG built with Rust and Bevy.

Its core priorities are:

- A memorable player journey built around exploration, adventure, relationships, and meaningful decisions.
- A persistent, evolving world with characters and events that continue beyond the player's immediate involvement.
- A spatial, three-quarter-view presentation with pixel-art visuals.
- Desktop and Android support, with both platforms treated as first-class targets.
- Offline-first gameplay.
- A foundation for moddable content alongside procedural generation.

Do not reduce the project to a generic survival game, life simulator, or death-and-legacy simulator. Those systems may contribute to the experience, but they are not its central identity.

## 2. Documentation authority

Before making changes, inspect the relevant project documentation.

Use these sources according to their responsibilities:

- `ROADMAP.md`: authoritative record of milestone goals, progress, and remaining work.
- `docs/architecture.md`: architectural boundaries, responsibilities, and design constraints.
- `docs/design-principles.md`: game identity and high-level design decisions.
- `docs/development-process.md`: contribution and development workflow.
- `CHANGELOG.md`: record of completed project changes in the current development line.
- `CHANGELOG-HISTORY.md`: detailed archive of completed work from older development lines and releases.
- `RELEASE-NOTES.md`: manually authored, concise, player-facing summary for the release being prepared or most recently published. Before creating a new release tag, its version heading must match that tag. Never derive the release description by extracting a section from `CHANGELOG.md`.
- Relevant documents under `docs/`: detailed design and implementation context.

Do not assume an old issue, conversation, or plan overrides the current repository state.

When documentation conflicts, investigate the discrepancy rather than silently choosing whichever interpretation is easiest.

## 3. Development workflow

For every development task:

1. Inspect the latest repository state, relevant documentation, related issues, and existing implementation.
2. Understand the current implementation and its architectural constraints before changing it.
3. Identify a coherent outcome and outline a milestone-level implementation plan. Group related work that shares dependencies, state, interfaces, or validation into one delivery.
4. Implement the milestone as an integrated whole, including necessary supporting code, tests, assets, and documentation. Do not split naturally connected work into artificial micro-tasks or PRs.
5. Review the complete change set for consistency, correctness, integration gaps, regressions, and unintended scope.
6. Update relevant documentation and `ROADMAP.md` to reflect what was actually completed and what remains.
7. Prepare one substantial, reviewable pull request for the milestone.

Prefer fewer, meaningful milestone PRs over a stream of small PRs. A milestone may span multiple development sessions, and its branch should remain the working place for that coherent body of work. Use intermediate commits as useful checkpoints; commits do not each need to represent a separately reviewable feature.

Milestone size is governed by coherence and reviewability, not by an arbitrary file, line, or task count. A milestone should deliver a meaningful integrated improvement, but must not become an unrelated collection of changes or an unreviewable rewrite. Split work when independent parts can be reviewed and delivered safely on their own, when a dependency requires an earlier landing, or when the scope becomes too broad to validate reliably.

Build solid foundations for future development. Do not rush to declare a system complete merely because its first usable implementation exists.

Preserve existing functionality unless the requested change intentionally modifies or removes it.

If an important design decision or requirement is missing, ask the project owner rather than inventing an answer. Do not interrupt the owner for routine implementation details already established by project documentation or prior decisions.

## 4. Git and pull request workflow

- Work on dedicated feature or fix branches for implementation work.
- Use milestone-level pull requests targeting `main`; avoid opening separate PRs for every small component of the same coherent milestone.
- Keep commits descriptive and logically grouped, but do not create a PR for each commit.
- Avoid unrelated formatting changes or opportunistic refactoring.
- Write pull request descriptions that explain the milestone goal, integrated implementation, important design decisions, validation, and remaining limitations.
- Link related issues where appropriate.
- Do not merge pull requests manually.

Documentation-only workflow or policy updates may be committed directly to `main` when explicitly requested by the project owner. This exception does not authorize direct pushes of implementation changes.

The project owner has a local automation script that monitors pull request checks, merges a PR when its checks pass, and notifies the owner when checks fail.

Therefore:

- Do not wait for CI checks to finish.
- Do not repeatedly poll or monitor CI status.
- Once implementation and the PR are ready, report the PR and hand control back to the owner.
- Do not attempt to duplicate the owner's merge automation.

## 5. Build and test responsibilities

The project's CI/CD workflows handle building and executing tests. The development environment used by AI agents does not necessarily have the Rust toolchain or the dependencies required to build the project.

Accordingly:

- Do not attempt to build or compile the project locally.
- Do not attempt to execute the test suite locally.
- Add or update appropriate automated tests when implementing functionality.
- Review test coverage and test code for consistency with the implementation.
- Use the repository's CI/CD configuration as the source of truth for automated validation.
- Do not add unnecessary statements to PR descriptions or reports about builds or tests not being run locally.

The responsibility is to implement the change correctly and provide appropriate automated test coverage. CI/CD is responsible for executing the validation.

## 6. Architecture and code quality

Respect the existing architectural boundaries.

In particular:

- Keep gameplay rules and domain logic independent of rendering and presentation wherever the architecture requires it.
- Treat Bevy as the application and presentation framework, not as a reason to couple every gameplay system directly to engine-specific types.
- Keep input handling and platform-specific adapters separate from gameplay rules.
- Give state a clear owner. Avoid storing redundant copies of the same information.
- Use stable identifiers for persistent entities and references.
- Keep module responsibilities clear and avoid unnecessary cross-module dependencies.
- Prefer explicit data flow and maintainable abstractions over premature generalisation.

Do not introduce a new framework, dependency, architectural pattern, or abstraction without a concrete reason.

A foundation should support foreseeable development without attempting to implement every future possibility prematurely.

## 7. Persistence and compatibility

Compatibility protects player trust; it is not an obligation to preserve prototype history.

When modifying persistent data:

- Consider save compatibility in the context of the project's release stage and the value of the data.
- During prototyping and pre-release development, project-owned test saves, generated maps and worlds, placeholder content, and internal data may be reset, regenerated, or redesigned when that better serves the intended game.
- Do not let prototype layouts, generated content, or old implementation details constrain the intended architecture or player experience.
- Preserve or migrate meaningful player progress when proportionate and compatible with the intended design. Released and long-running campaigns deserve more care than disposable test data.
- Prefer explicit, versioned migrations when migration is warranted. Use backups and clear warnings for intentional breaking changes when feasible.
- Do not silently discard valuable player-owned data. When an intentional change invalidates data, state that clearly in documentation and the PR, and provide safe recovery or regeneration when practical.
- Keep transient presentation state separate from persistent world and character state.
- Ensure stable identifiers remain meaningful where the corresponding content remains part of the intended game.

Backward compatibility is a means of protecting players, not an end in itself. A deliberate redesign or breaking change is acceptable when it is the soundest way to achieve the current product direction.

## 8. Gameplay and design constraints

Keep the established design direction in mind when implementing gameplay systems:

- The player experiences the world through a character, not through a detached management dashboard.
- Exploration should be spatial and rewarding.
- The world should feel coherent rather than like disconnected collections of encounters.
- NPCs should eventually have meaningful autonomy, relationships, and motivations.
- Simulation should be deep enough to create interesting outcomes without becoming tedious.
- Combat is currently transitional: existing tactical or turn-based foundations should be respected while the project develops toward spatial, real-time action.
- Desktop and Android controls must both remain viable.
- Do not introduce separate screens or abstractions that undermine the intended spatial experience without a clear design reason.

When implementing a system, consider how it will interact with existing and future systems. Avoid isolated mechanics that cannot meaningfully connect to the rest of the game.

## 9. Content, assets, and modding

The project is intended to support authored content and procedural content.

- Keep content data-driven where appropriate.
- Avoid hardcoding content that should reasonably be configurable or moddable.
- Preserve a clear distinction between engine code, game logic, and content assets.
- Follow existing asset organisation conventions.
- Respect the project's licensing requirements.
- Do not introduce assets with unclear provenance or incompatible licenses.

Do not create placeholder content that accidentally becomes a permanent architectural dependency.

## 10. Roadmap, documentation, and versioning

After completing work:

- Update `ROADMAP.md` to reflect goals actually completed in the iteration.
- Do not mark partially implemented goals as complete.
- Update relevant design or architecture documentation when implementation changes affect documented behaviour or boundaries.
- Update `CHANGELOG.md` according to the project's existing conventions.
- Maintain `RELEASE-NOTES.md` separately as the concise, manually authored GitHub Release title and description. Its version heading must match the tag exactly; do not copy the full development changelog into it.
- Use Semantic Versioning.
- Increment the version when the scope of the work warrants it; do not bump it mechanically for every commit.

Keep milestone scope coherent. Do not pull unrelated future-milestone work into the current milestone without a clear reason.

## 11. Collaboration with the project owner

The project owner is the final authority on game design and project direction.

- Follow explicit decisions already made.
- Do not silently reverse established design choices.
- Explain meaningful trade-offs when proposing a different approach.
- Ask when requirements are genuinely ambiguous or when implementation would require an unapproved design decision.
- Do not repeatedly ask for confirmation on routine implementation details that are already covered by the project direction.
- Be transparent about limitations, incomplete work, and unresolved issues.
- Do not claim that functionality is complete when only part of the intended behaviour exists.

The owner values sound architecture and maintainability over the fastest possible implementation.

## 12. Final reporting

When a milestone is complete, provide a concise report covering:

- The milestone goal and what changed across the integrated work.
- Important implementation or design decisions.
- The pull request and relevant commits.
- Documentation and roadmap updates.
- Validation performed and its results when available.
- Remaining work, known limitations, or decisions requiring the owner's input.

Keep the report factual. Distinguish completed implementation from future work and do not overstate what the change accomplishes.
