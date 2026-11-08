use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[cfg(not(target_os = "android"))]
pub const GAME_DIRECTORY_NAME: &str = "The Ever-Unfolding Saga";
pub const SAVES_DIRECTORY_NAME: &str = "saves";
pub const MODS_DIRECTORY_NAME: &str = "mods";
pub const IMPORTS_DIRECTORY_NAME: &str = "imports";
pub const DEVELOPER_FIXTURES_DIRECTORY_NAME: &str = "developer_fixtures";

static GAME_PATHS: OnceLock<GamePaths> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct GamePaths {
    pub root: PathBuf,
    pub saves_dir: PathBuf,
    pub mods_dir: PathBuf,
    pub imports_dir: PathBuf,
    pub developer_fixtures_dir: PathBuf,
    pub spatial_fixture_imports_dir: PathBuf,
    #[cfg(target_os = "android")]
    pub save_imports_dir: PathBuf,
    #[cfg(target_os = "android")]
    pub mod_imports_dir: PathBuf,
    pub config_dir: PathBuf,
    pub mod_settings_path: PathBuf,
    pub bundled_assets_dir: PathBuf,
    pub bundled_mods_dir: PathBuf,
}

impl GamePaths {
    pub fn initialize() -> io::Result<()> {
        if GAME_PATHS.get().is_some() {
            return Ok(());
        }

        let root = resolve_internal_root()?;
        let saves_dir = root.join(SAVES_DIRECTORY_NAME);
        let mods_dir = root.join(MODS_DIRECTORY_NAME);
        let imports_dir = root.join(IMPORTS_DIRECTORY_NAME);
        let developer_fixtures_dir = root.join(DEVELOPER_FIXTURES_DIRECTORY_NAME);
        let spatial_fixture_imports_dir = imports_dir.join("spatial_fixtures");
        #[cfg(target_os = "android")]
        let save_imports_dir = imports_dir.join("saves");
        #[cfg(target_os = "android")]
        let mod_imports_dir = imports_dir.join("mods");
        let config_dir = root.join("config");

        for directory in [
            root.as_path(),
            saves_dir.as_path(),
            mods_dir.as_path(),
            developer_fixtures_dir.as_path(),
            spatial_fixture_imports_dir.as_path(),
            #[cfg(target_os = "android")]
            save_imports_dir.as_path(),
            #[cfg(target_os = "android")]
            mod_imports_dir.as_path(),
            config_dir.as_path(),
        ] {
            fs::create_dir_all(directory)?;
        }

        #[cfg(target_os = "android")]
        let (bundled_assets_dir, bundled_mods_dir) = {
            let bundled = root.join("bundled");
            (bundled.join("assets"), bundled.join("mods"))
        };

        #[cfg(not(target_os = "android"))]
        let (bundled_assets_dir, bundled_mods_dir) = packaged_data_directories();

        let paths = Self {
            mod_settings_path: config_dir.join("mod_settings.json"),
            root,
            saves_dir,
            mods_dir,
            imports_dir,
            developer_fixtures_dir,
            spatial_fixture_imports_dir,
            #[cfg(target_os = "android")]
            save_imports_dir,
            #[cfg(target_os = "android")]
            mod_imports_dir,
            config_dir,
            bundled_assets_dir,
            bundled_mods_dir,
        };

        seed_bundled_mods(&paths)?;
        GAME_PATHS.set(paths).map_err(|_| {
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                "game storage paths were initialized concurrently",
            )
        })?;
        Ok(())
    }

    pub fn get() -> &'static Self {
        GAME_PATHS
            .get()
            .expect("GamePaths::initialize() must run before storage is accessed")
    }
}

fn resolve_internal_root() -> io::Result<PathBuf> {
    #[cfg(target_os = "android")]
    {
        let app = bevy::android::ANDROID_APP
            .get()
            .ok_or_else(|| io::Error::other("Android Activity is unavailable"))?;
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) };
        let root = vm
            .attach_current_thread(|env| -> jni::errors::Result<Option<String>> {
                let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
                let activity = unsafe {
                    env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
                };
                let value = env.call_method(
                    activity.as_ref(),
                    jni::jni_str!("getNativeGameRootPath"),
                    jni::jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?;
                let object = value.l()?;
                if object.as_raw().is_null() {
                    return Ok(None);
                }
                let string = jni::objects::JString::cast_local(env, object)?;
                Ok(Some(string.try_to_string(env)?))
            })
            .map_err(|error| {
                io::Error::other(format!(
                    "could not resolve Android private storage: {error}"
                ))
            })?
            .ok_or_else(|| io::Error::other("Android Activity returned an empty storage path"))?;
        Ok(PathBuf::from(root))
    }

    #[cfg(not(target_os = "android"))]
    {
        let data_root = dirs::data_local_dir()
            .ok_or_else(|| io::Error::other("no local application data directory is available"))?;
        Ok(data_root.join(GAME_DIRECTORY_NAME))
    }
}

#[cfg(not(target_os = "android"))]
fn packaged_data_directories() -> (PathBuf, PathBuf) {
    let executable_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));

    let packaged = executable_dir
        .map(|directory| directory.join("data"))
        .unwrap_or_default();
    let source_data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    let packaged_assets = packaged.join("assets");
    let packaged_mods = packaged.join("mods");

    let assets = if packaged_assets.is_dir() {
        packaged_assets
    } else {
        source_data.join("assets")
    };
    let mods = if packaged_mods.is_dir() {
        packaged_mods
    } else {
        source_data.join("mods")
    };
    (assets, mods)
}

fn seed_bundled_mods(paths: &GamePaths) -> io::Result<()> {
    let marker = paths.config_dir.join("bundled_mods_seeded");
    if marker.exists() || !paths.bundled_mods_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(&paths.bundled_mods_dir)? {
        let entry = entry?;
        let source = entry.path();
        if !source.is_dir() {
            continue;
        }
        let manifest_path = source.join("manifest.json");
        let manifest_data = match fs::read(&manifest_path) {
            Ok(data) => data,
            Err(error) => {
                bevy::log::warn!(
                    "Skipping bundled mod without a readable manifest at {}: {error}",
                    manifest_path.display()
                );
                continue;
            }
        };
        let manifest: crate::content::ModManifest = match serde_json::from_slice(&manifest_data) {
            Ok(manifest) => manifest,
            Err(error) => {
                bevy::log::warn!(
                    "Skipping bundled mod with invalid manifest {}: {error}",
                    manifest_path.display()
                );
                continue;
            }
        };
        crate::mod_manager::validate_manifest(&manifest)?;
        let destination = paths.mods_dir.join(&manifest.id);
        copy_directory_without_overwriting(&source, &destination)?;
    }

    fs::write(marker, b"1\n")
}

fn copy_directory_without_overwriting(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let metadata = fs::symlink_metadata(&source_path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let target = destination.join(entry.file_name());
        if metadata.is_dir() {
            copy_directory_without_overwriting(&source_path, &target)?;
        } else if metadata.is_file() && !target.exists() {
            fs::copy(&source_path, &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/game_paths.rs"]
mod tests;
