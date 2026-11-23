# itch.io publishing

## GitHub Actions publishing

Tag-based releases are mirrored to itch.io by the Android workflow after all five desktop archives and three Android APKs are available on the GitHub Release. This prevents the two tag-triggered workflows from racing each other.

The following repository-level Actions settings control itch.io publishing:

- **Variable `ITCHIO_TARGET`** — the itch.io target in `username/game-slug` form. For a page such as `https://username.itch.io/game-slug`, the value is `username/game-slug`.
- **Secret `BUTLER_API_KEY`** — an itch.io API key supplied to Butler through the `BUTLER_API_KEY` environment variable. The secret remains in GitHub Actions and is not stored in repository files or workflow output.

For automatic tag-based mirroring, the Android workflow skips only the itch.io step when either setting is absent; APKs are still uploaded to the GitHub Release. For a manual publish operation that explicitly selects itch.io, both settings are required and the workflow fails clearly when either is missing.

## itch.io channels

Each release asset is published to its own channel:

| Release asset | Butler channel |
| --- | --- |
| Linux x86_64 archive | `linux-x86_64` |
| Linux aarch64 archive | `linux-aarch64` |
| macOS aarch64 archive | `macos-aarch64` |
| Windows x86_64 archive | `windows-x86_64` |
| Windows aarch64 archive | `windows-aarch64` |
| Android x86_64 APK | `android-x86_64` |
| Android aarch64 APK | `android-aarch64` |
| Android ARMv7 APK | `android-armv7` |

Butler uses channel names to infer platform tags. It does not currently provide architecture tags, so architectures are kept in separate channel names for clarity. The Android builds are uploaded as APK files rather than renamed archives.

The GitHub release tag is passed to Butler as the user version for every channel.

## Re-publishing an existing release

The main **CI/CD** workflow exposes three manual operations:

- `build` builds one selected desktop target and stores its archive as a workflow artifact.
- `release` rebuilds all five desktop targets and publishes them to the existing or newly prepared GitHub Release. The separate Android workflow must still build and upload its three APKs.
- `publish` downloads the five desktop archives and three Android APKs from their original tag-triggered workflow runs and republishes them without rebuilding. Select `github`, `itch`, or `both` as the destination.

The artifact-only operation matches source runs to the exact commit referenced by the selected release tag and verifies that all eight expected artifacts are present and unexpired before publishing. It does not run Rust, Gradle, or Android signing. The Android signing secrets are therefore not required.

For GitHub publishing, the GitHub Release for the selected tag must already exist; the operation replaces matching assets and publishes the release if it is still a draft. An itch-only publish does not require a GitHub Release because it uploads directly from the downloaded workflow artifacts.

Workflow artifacts are retained for seven days. The artifact-only operation cannot recover expired or missing artifacts; use the build/release operations to rebuild them first.

## Butler reference

- [Butler manual](https://itch.io/docs/butler/)
- [Running Butler in CI](https://itch.io/docs/butler/login.html)
- [Pushing builds](https://itch.io/docs/butler/pushing.html)

The workflow downloads the stable Linux x86_64 Butler build from itch.io's permanent [Broth](https://broth.itch.zone/butler) endpoint.
