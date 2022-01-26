// [Sync]: must be synced with `src/pages/Install/types.ts`
use tauri::Window;

use crate::constants;
use crate::util;

mod install_bepinex;
mod install_mod;
mod launch_game;
mod launch_options;

// [Sync]
#[derive(Clone)]
pub enum InstallSteps {
    DownloadBepInEx,
    InstallBepInEx,
    LaunchOption,
    LaunchGame,
    DownloadWbmZip,
    InstallWbm,
    Done,
}

// [Sync]
pub enum InstallResult {
    NoErr,
    FailedToGetGamePath,
    UnsupportedOS,
    BepInExDownloadFailed,
    BepInExUnzipFailed,
    SetLaunchOption,
    LaunchGame,
    WBMDownloadFailed,
    WBMRemoveFailed,
    WBMDirectoryCreationFailed,
    WBMUnzipFailed,
}

#[derive(Clone, serde::Serialize)]
struct InstallPayload(i64);

// todo: show current step in the frontend

/// automated version of the [manual installation](https://github.com/War-Brokers-Mods/WBM#installation).
///
/// This function exits if it requires a user input and is called again with the user feedback as its arguments.
///
/// ## Installation procedure
///
/// This function exits at the end of each step.
///
/// 1. BepInEx installation
/// 2. Steam launch option setup (only on Linux and MacOS)
/// 3. Launch game for plugins folder generation
/// 4. Mod installation
///
/// Some part of the function are unnecessary executed each time the function is called,
/// but the time loss is negligible and it's a sacrifice worth for code readability.
///
/// ## Arguments
///
/// All arguments except `windows` are empty by default.
///
/// * `window` - standard tauri argument. See [docs](https://tauri.studio/docs/guides/command#accessing-the-window-in-commands) for more info.
/// * `game_path` - absolute path to the game folder/directory.
/// * `is_launch_option_set` - whether if the steam launch option for the game is set or not.
/// * `was_game_launched` - whether if the game was launched once after installing BepInEx to generate the plugins folder.
#[tauri::command]
pub async fn install(
    window: Window,
    game_path: String,
    is_launch_option_set: bool,
    was_game_launched: bool,
) -> i64 {
    println!("install command called");

    //
    // Test if OS is compatible
    //

    match std::env::consts::OS {
        "linux" | "macos" | "windows" => {}

        _ => {
            println!("Unsupported OS!");
            return InstallResult::UnsupportedOS as i64;
        }
    }

    //
    // Resolve game path
    //

    let game_path = if game_path.is_empty() {
        let default_game_path = match util::get_default_game_path() {
            Some(path) => path,

            // failed to find game install location.
            // Prompt user to manually choose the game location.
            None => return InstallResult::FailedToGetGamePath as i64,
        };

        default_game_path
    } else {
        // todo: check if game path is valid and tell the user
        game_path
    };
    let game_path = game_path.as_str();

    //
    // Install BepInEx
    //

    if !is_launch_option_set {
        match install_bepinex::install_bepinex(&window, game_path).await {
            Ok(()) => {}
            Err(err) => return err as i64,
        }
    }

    //
    // Setup steam launch option if OS is linux or macOS
    //

    if !was_game_launched {
        match launch_options::unix_launch_option_setup(&window).await {
            Ok(()) => {}
            Err(err) => return err as i64,
        }
    }

    //
    // Run the game once to generate the plugins directory
    //

    match launch_game::launch_game_once(&window).await {
        Ok(()) => {}
        Err(err) => return err as i64,
    }

    //
    // Install the mod
    //

    match install_mod::install_wbm_mod(&window, game_path).await {
        Ok(()) => {}
        Err(err) => return err as i64,
    }

    //
    // Tell the frontend that the installation was successful
    //

    emit(&window, InstallSteps::Done);
    println!("Install complete!");

    return InstallResult::NoErr as i64;
}

pub fn emit(window: &Window, payload: InstallSteps) {
    util::emit(&window, constants::EVENT_INSTALL, payload as i64);
}
