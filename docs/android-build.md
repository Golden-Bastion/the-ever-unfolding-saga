# Android CI builds

The Android CI workflow produces installable, signed APK artifacts for the supported Android ABIs:

- `android-aarch64` → `arm64-v8a`
- `android-armv7` → `armeabi-v7a`
- `android-x86_64` → `x86_64`

A `workflow_dispatch` run supports two operations. `build` builds the selected ABI and exposes the signed APK as a workflow artifact without touching a GitHub Release. `release` rebuilds all three APKs from an existing `release` tag and publishes them to that release. The `release` operation requires the persistent Android signing secrets and remains useful when Android APKs need to be rebuilt.

For artifact-only release promotion, use the main [CI/CD workflow](../.github/workflows/ci-cd.yml) with `operation=publish`. It downloads the three APKs from the original tag-triggered Android workflow run alongside the five desktop archives, then publishes the complete set to GitHub Releases, itch.io, or both. This does not run Android builds or signing and does not require the Android keystore secrets. The original tag-run artifacts must still be within their seven-day retention period.

For manual runs, the workflow can generate a short-lived CI-only signing key when the Android signing secrets are absent. This is intended for personal/device testing. Release-tag builds require the persistent repository signing secrets below.

## Required repository secrets

Release builds expect these GitHub Actions repository secrets:

- `ANDROID_KEYSTORE_BASE64`: the base64-encoded Java keystore containing the release signing key
- `ANDROID_KEYSTORE_PASSWORD`: keystore password
- `ANDROID_KEY_ALIAS`: alias of the signing key
- `ANDROID_KEY_PASSWORD`: signing key password

The private keystore is decoded only into the runner's temporary directory and is never committed to the repository.

A new local keystore can be created with the JDK's `keytool`, for example:

```text
keytool -genkeypair -v \
  -keystore ever-unfolding-saga-release.keystore \
  -alias ever-unfolding-saga \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000
```

Encode that keystore for the `ANDROID_KEYSTORE_BASE64` secret without adding the file to the repository. The remaining secret values must match the keystore created above.

## Android storage

The application ID is `com.rightward.everunfoldingsaga`. This is a new Android application identity, so Android installs it as a separate app rather than upgrading the previous package. Private files are not transferred between the two applications. Export saves from the previous installation and import them into the new app before uninstalling the older one; uninstalling it first removes its private app data.

Android stores all active game data in the application's private files directory. Rust resolves that single internal root through the Activity's `getNativeGameRootPath()` method. The game does not request a shared-folder tree permission and does not synchronize data to public storage.

Bundled base assets and packaged default mods are copied into a protected bundled area within the internal root. Default mods are seeded into the managed mods catalog once, using their manifest IDs as installation directory names. User-installed mods, saves, import staging files, and mod preferences remain private to the app.

The Options screen uses Android's document picker only for explicit file operations:

- Save import copies the selected document into private staging storage, then Rust validates and installs it without replacing an existing save.
- Mod import copies the selected ZIP into private staging storage, then Rust validates and installs it into the internal mod catalog.
- Save export creates a destination document and copies the selected internal save to it.

The selected document's temporary URI is used for the operation only; no persistent SAF folder permission or background synchronization is retained.

## Local Android project

The Gradle project is under `android/`. It uses Bevy's `GameActivity` integration and expects the Rust `cdylib` to be copied into the ABI-specific `jniLibs` directory before Gradle packages the APK.

The CI workflow performs that native-library build and packaging automatically. APK packaging deliberately compresses the native `.so` library so the directly distributed APK is substantially smaller; the release variant explicitly sets `packaging.jniLibs.useLegacyPackaging` through the Android Components variant API. Local release signing is optional; when the signing environment variables are absent, the Gradle release build remains unsigned.
