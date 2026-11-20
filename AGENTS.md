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
- `RELEASE-NOTES.md`: manually authored, concise, player-facing summary for the next GitHub Release. Never derive the release description by extracting a section from `CHANGELOG.md`.
- Relevant documents under `docs/`: detailed design and implementation context.

Do not assume an old issue, conversation, or plan overrides the current repository state.

When documentation conflicts, investigate the discrepancy rather than silently choosing whichever interpretation is easiest.

## 3. Development workflow

For every development task:

1. Inspect the latest repository state, relevant documentation, and related issues or existing implementation.
2. Understand the current implementation and its architectural constraints before changing it.
3. Identify the intended outcome and outline a focused implementation plan.
4. Implement the requested work without unnecessarily changing unrelated systems.
5. Review the modified files for consistency, correctness, and unintended effects.
6. Update relevant documentation and `ROADMAP.md` to reflect what was actually completed.
7. Prepare a focused branch and pull request.

Prefer small, independently understandable changes and commits over large, difficult-to-review batches.

Build solid foundations for future development. Do not rush to declare a system complete merely because its first usable implementation exists.

Preserve existing functionality unless the requested change intentionally modifies or removes it.

If an important design decision or requirement is missing, ask the project owner rather than inventing an answer.

## 4. Git and pull request workflow

- Work on dedicated feature or fix branches.
- Prefer pull requests over direct changes to `main`.
- Keep commits focused and descriptive.
- Avoid unrelated formatting changes or opportunistic refactoring.
- Write pull request descriptions that explain the problem, implementation, and relevant design decisions.
- Link related issues where appropriate.
- Do not merge pull requests manually.

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

Saved games are part of the project's user-facing functionality.

When modifying persistent data:

- Consider existing save compatibility.
- Prefer explicit, versioned migrations when a format change requires them.
- Avoid silently discarding data.
- Keep transient presentation state separate from persistent world and character state.
- Ensure stable identifiers remain meaningful across save/load cycles.

Do not make breaking save-format changes casually.

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

When a task is complete, provide a concise report covering:

- What changed.
- Important implementation or design decisions.
- Relevant commits and pull request.
- Documentation and roadmap updates.
- Any remaining work, known limitations, or decisions requiring the owner's input.

Keep the report factual. Distinguish completed implementation from future work and do not overstate what the change accomplishes.
