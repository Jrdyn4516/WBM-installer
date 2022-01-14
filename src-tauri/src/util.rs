use futures_util::StreamExt;
use std::cmp::min;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use tauri::api::path::home_dir;

/// build client with header
pub fn build_client() -> reqwest::Client {
    return reqwest::Client::builder()
        .user_agent("WBM-Installer")
        .build()
        .unwrap();
}

/// gets path to WB game files.
pub fn get_default_game_path() -> Option<String> {
    let home = buf2str(home_dir());
    if home.is_none() {
        return None;
    }
    let home = home.unwrap();

    let game_path = match std::env::consts::OS {
        "linux" => format!("{}/.steam/steam/steamapps/common/WarBrokers", home),
        "macos" => format!(
            "{}/Library/Application Support/Steam/steamapps/common/WarBrokers",
            home
        ),
        "windows" => String::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\WarBrokers"),

        _ => return None,
    };

    if fs::metadata(game_path.as_str()).is_ok() {
        return Some(String::from(game_path));
    }

    return None;
}

pub fn buf2str(input: Option<PathBuf>) -> Option<String> {
    if input.is_none() {
        return None;
    }

    return Some(String::from(input.unwrap().to_str().unwrap()));
}

/// get the latest WBM release version.
/// data is not converted to a json object because it'll be done in the front-end
pub async fn get_latest_release() -> String {
    let client = build_client();

    // todo: handle error
    let res = client
        .get("https://api.github.com/repos/War-Brokers-Mods/WBM/releases")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    return res;
}

/// download a specific release version
pub async fn download_release_zip(url: &str, path: &str) -> Result<(), String> {
    let client = build_client();

    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;

    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    // download chunks

    let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
    let mut stream = res.bytes_stream();

    let mut downloaded: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;

        file.write(&chunk)
            .or(Err(format!("Error while writing to file")))?;

        let new = min(downloaded + (chunk.len() as u64), total_size);

        downloaded = new;
    }

    return Ok(());
}
