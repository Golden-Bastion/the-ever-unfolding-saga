# Startup Recovery and Safe Mode

> Status: approved target design. This document defines startup diagnostics and recovery behavior. It does not claim that the current application already has a Bevy-independent startup supervisor or fallback interface.

## Purpose

The game must remain diagnosable when normal startup fails, including when Bevy or one of its startup systems cannot initialize. A developer or player should not need a terminal, Android Debug Bridge (ADB), or logcat access to retrieve a useful diagnostic report.

Safe mode is both a recovery path and a deliberately selectable test mode. It must be possible to enter it manually on a healthy installation and to be offered it automatically after a failed launch.

## Startup supervision

Startup should be divided into identifiable stages managed by a small supervisor that runs before normal Bevy application initialization.

The supervisor is responsible for:

- Recording the current startup stage and build/platform context.
- Establishing bounded diagnostic-log capture before risky initialization begins.
- Detecting handled startup errors and Rust panics where recovery is possible.
- Writing a durable failure marker/report before handing control to the normal application.
- Selecting normal startup, manual safe mode, or recovery startup.
- Presenting diagnostics through a fallback path that does not depend on Bevy's renderer or normal Bevy application startup.

The supervisor and fallback interface should be kept deliberately small. They are recovery infrastructure, not a second game frontend.

## Failure capture and report

Capture useful diagnostic context as early as practical. A report should include, where available:

- Game version, build identifier, and relevant build configuration.
- Operating system, device/platform, and runtime environment.
- The last recorded startup stage.
- The panic payload or handled error, including its error chain.
- A backtrace when the platform and build configuration make one available.
- A bounded tail of structured startup logs.
- Relevant graphics/backend and asset/extension context when available.
- Whether normal startup or safe mode was active, and which temporary overrides were applied.

The report must be readable and copyable in full from the fallback interface. On Android, copying should use the platform clipboard without requiring ADB. The report should be stored locally so it survives a restart, with bounded retention or rotation to avoid unbounded storage growth.

Diagnostics must avoid including credentials, secrets, or unrelated private user data. Reports are local by default; uploading or sharing them is a separate, explicit user action.

A Rust panic hook and explicit error propagation can capture many application-level failures. The implementation should avoid depending on complex or allocation-heavy recovery work inside low-level panic or signal handlers. Diagnostic capture must be best-effort and must not cause a second failure that destroys the original report.

## Bevy-independent fallback interface

If Bevy initialization fails in a recoverable way, the application should display a minimal diagnostic interface using a platform-host or other recovery path independent of Bevy's renderer and normal app initialization.

The fallback interface should provide:

- The complete captured report in a readable, scrollable view.
- A copy-all action.
- A way to enter safe mode or retry normal startup where appropriate.
- A clear indication of the failure stage and whether the report may be incomplete.

The fallback interface must not depend on the subsystem that failed. In particular, it must not require Bevy rendering, Bevy UI, normal game assets, or the ordinary content-loading pipeline.

## Safe-mode entry and behavior

Safe mode has two entry paths:

1. **Manual entry:** A startup option allows safe mode to be selected even when normal startup works. This is required for deliberate testing and troubleshooting.
2. **Automatic recovery:** After a failed launch, the next launch offers safe mode as an option alongside retrying normal startup. If repeated failures prevent reaching the ordinary startup menu, the supervisor should enter or offer the fallback recovery path directly.

Safe mode starts the game with a documented set of temporary overrides that disable or simplify optional systems likely to prevent startup. Candidate categories may include optional graphics features, nonessential asset packs, mods/extensions, or optional integrations. The exact list must be determined from the systems the game actually implements.

Safe mode must:

- Preserve essential authoritative game data and required base content.
- Clearly show which systems or features were disabled or changed.
- Avoid silently rewriting normal settings, content configuration, or saves.
- Apply temporary overrides that do not become permanent player preferences.
- Allow a normal launch to be retried after troubleshooting.
- Be testable on a healthy installation without requiring a previous crash.

Safe mode is not a reduced-difficulty mode and must not alter authoritative campaign outcomes merely to make startup easier.

## Crash classes and recovery limits

The recovery design must distinguish failures that the application can handle from failures that terminate the process outside its control.

- **Handled initialization errors:** The supervisor can record the error and switch to the fallback interface.
- **Rust panics:** A panic hook can usually capture the panic and a backtrace when unwinding is enabled and the process remains viable. Recovery to the fallback interface is possible only when the panic boundary and application state make it safe.
- **Native crashes, segmentation faults, driver aborts, process kills, and device-level failures:** These may terminate the process before in-process recovery can run. The application cannot guarantee that it will display a screen during the same failed process.

For hard process termination, write startup-stage markers and other durable diagnostics before risky initialization. On the next launch, detect an interrupted startup, preserve any available report, and offer safe mode. Platform crash facilities may provide additional information, but the in-app design must not claim it can catch every native or operating-system failure.

The goal is to keep failures diagnosable and provide a reliable next-launch recovery path, not to promise that software can recover from every way the operating system or hardware can terminate it.

## Persistence and report lifecycle

Crash reports and startup markers are diagnostic data, separate from campaign saves and player preferences.

- A failed launch must not corrupt or overwrite the original campaign save.
- Safe-mode overrides must not silently replace normal configuration.
- Reports should remain available after restart until explicitly cleared or superseded according to a documented retention policy.
- A successful normal launch may clear an interrupted-startup marker, but should not silently erase a report that may still be useful.
- Recovery actions and report cleanup should be explicit.

## Validation expectations

The implementation should eventually test:

- A handled failure at each startup stage.
- A Rust panic during startup where unwinding is supported.
- Bevy renderer or application initialization failure without relying on Bevy UI for diagnostics.
- Manual safe-mode entry on a healthy installation.
- Automatic safe-mode offer after an interrupted or failed launch.
- Copying the complete report on Android and desktop.
- Report persistence across restart.
- Safe-mode overrides leaving normal settings and saves unchanged.
- Missing optional assets/extensions and unavailable optional integrations.
- Bounded report/log storage and redaction of sensitive values.
- The behavior when a native crash cannot be recovered in-process, including next-launch detection where platform facilities permit it.

## Decisions intentionally left open

This specification does not select:

- The exact platform-host mechanism used to display the fallback interface on each target.
- The panic strategy and build-profile configuration needed for useful backtraces.
- The crash-marker format, log-buffer implementation, or report serialization format.
- The exact safe-mode options and how each optional subsystem is disabled.
- The clipboard integration details for each platform.
- The report retention limit and user-facing cleanup controls.
- Which platform crash-reporting facilities can be integrated safely.

These details require implementation research and platform validation. They must preserve the independence of the fallback path and the recovery guarantees described above.
