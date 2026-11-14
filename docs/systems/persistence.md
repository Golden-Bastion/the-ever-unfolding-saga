# Persistence System

The persistence system stores world and character state while keeping inherited worlds compatible across character deaths and game versions.

## Internal storage

Each installation has one authoritative, application-managed game-data root.

- Desktop uses the operating system's local application-data directory.
- Android uses private application files storage.
- The root contains the save collection, installed mods, import staging folders, and game-managed configuration.
- The active root is not configurable from the Options screen. The process working directory is not used to locate saves, mods, or assets.
- External storage is accessed only when the player explicitly imports or exports a file through the Load screen.

Bundled base content and base-game art remain protected package resources. Bundled default mods are seeded into the internal mod catalog on first initialization. User-installed mods and mod preferences are managed in that same catalog.

## Save format

Saves use the existing JSON payload format compressed with gzip and are stored under the internal root's `saves/` directory. Character-specific filenames follow the form `ever_unfolding_saga_save_<character>.json.gz` after filename sanitization.

The save envelope records the active mod dependencies at save time. Each dependency includes its stable mod ID, display name, and version. When loading, the game checks those dependencies against the active mod snapshot captured at process startup. Missing, disabled, or version-mismatched dependencies are reported before the player continues; the player may proceed at their own responsibility.

There is no automatic mod upgrade, save migration, or compatibility layer. Mod authors are responsible for maintaining compatibility between their releases and saves that depend on them.

Legacy uncompressed `ever_unfolding_saga_save.json` payloads remain readable when encountered inside the current internal root. The new application identity uses a new private data directory, so saves from previous installations are not discovered automatically; export and import them explicitly before removing an older installation.

## Load screen: save management

The Load screen lists saved lives and owns the complete save-management workflow. Each save row offers Load, Export, and Delete; Import and Back remain in the fixed footer. The Load screen remains available even when the catalogue is empty so a fresh installation can import a save.

- Imported saves are parsed and validated before installation.
- Import never overwrites an existing save slot. A collision receives a distinct imported filename.
- Export validates the selected save and refuses to overwrite an existing destination.
- Delete permanently removes the selected save only after the player confirms in the shared destructive-action dialog.
- Desktop uses native file dialogs. Android uses the system document picker and copies the selected file into a private import staging directory before validation. Import staging is polled by the lifecycle Load flow, not by Options.
- Readable save entries in the Load screen show the character's full display name, level, world name and day, save-format version, and saved mod-dependency count. This metadata is read without reloading campaign definitions; unreadable files fall back to their filename.
- The current save envelope does not record the game release version or a creation timestamp. The displayed version is the save-format version, not the application version.

Collection-level backup and cloud synchronization are outside the current scope.

## World and character boundaries

Character-specific state includes progression, conditions, and the personal quest log. World-persistent state includes faction memories, completed quest deeds, corpses, history, event cooldowns, and world time.

When a character dies, the next character can inherit the world without inheriting the dead character's personal quest log or faction reputation. Corpse contents and other persistent traces remain available to later lives.

## Saving and loading

Saving is tied to safe meditation rather than quitting. The start and load flows explicitly select compatible saves instead of silently loading a save. Loading preserves the selected save slot as the active path, including imported slots with collision-safe filenames.

Loading validates runtime references and reports broken or inconsistent save data clearly.

## Compatibility

New save-envelope fields use defaults so saves created before mod dependency metadata remain loadable. Malformed compressed data produces a stable invalid-data error rather than crashing through an unrelated failure.

## Design direction

Keep the save payload focused on persistent game state. Runtime-only campaign content is rehydrated from the current content definitions and active mod set instead of duplicated inside save files.
