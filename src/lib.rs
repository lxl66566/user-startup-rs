#![warn(clippy::cargo)]

pub mod utils;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use log::{debug, error, info, warn};

/// Read the first not-empty line of a file.
pub fn read_first_line(path: &Path) -> std::io::Result<String> {
    debug!("Reading first line of `{}`", path.display());
    let content = fs::read_to_string(path)?;
    for line in content.lines() {
        let stripped = line.trim();
        if !stripped.is_empty() && !stripped.starts_with("#!") {
            return Ok(stripped.to_string());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "File is empty",
    ))
}

/// Find a writable path for a startup command. Because the command may be not
/// unique, it will try to find the first available filename like this:
///
/// test, test1, test2, test3, test4, test5 ... test1000.
pub fn find_writable_path(name: &str) -> PathBuf {
    debug!("Finding writable path for `{}`", name);
    let base_path = &utils::CONFIG_PATH;
    let ext = utils::FILE_EXT;

    let initial_path = base_path.join(format!("{}{}", name, ext));
    if !initial_path.exists() {
        return initial_path;
    }

    for i in 1..1000 {
        let path = base_path.join(format!("{}{}{}", name, i, ext));
        if !path.exists() {
            debug!(
                "Found writable path `{}`",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            return path;
        }
    }
    panic!("TOO MANY ITEMS OF SAME NAME!");
}

/// Add a new startup command.
pub fn add_item(cmd: &str, name: Option<&str>, stdout: Option<&str>, stderr: Option<&str>) {
    if cfg!(target_os = "linux") && (stdout.is_some() || stderr.is_some()) {
        warn!("--stdout and --stderr are not supported for linux startup scripts");
    }

    let name = name.unwrap_or_else(|| cmd.split_whitespace().next().unwrap());
    let path = find_writable_path(name);

    fs::write(&path, utils::format(cmd, None, stdout, stderr))
        .expect("Failed to write config file");

    info!("Added `{}` to `{}`", cmd, path.display());
}

/// Get a list of startup commands.
///
/// # Returns
///
/// A vector of tuples, where the first element is the id of the command and the
/// second element is the command itself.
pub fn get_items_list() -> Vec<(String, String)> {
    let config_path = utils::CONFIG_PATH.as_os_str();
    debug!(
        "Finding config files in `{}` with extension `{}`",
        config_path.to_string_lossy(),
        utils::FILE_EXT
    );
    let mut res = vec![];

    for entry in fs::read_dir(config_path)
        .expect("Failed to read config directory")
        .flatten()
    {
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |ext| ext == utils::FILE_EXT.trim_start_matches('.'))
        {
            if let Ok(first_line) = read_first_line(&path) {
                let id = path.file_stem().unwrap().to_string_lossy().into_owned();
                if first_line.starts_with(utils::COMMENT_PREFIX) {
                    let command = first_line.trim_start_matches(utils::COMMENT_PREFIX).trim();
                    res.push((id, command.to_string()));
                }
            }
        }
    }
    res
}

/// Remove startup commands.
pub fn remove_items(ids: Vec<String>) {
    for id in ids {
        let path = utils::CONFIG_PATH.join(format!("{}{}", id, utils::FILE_EXT));
        if path.exists() {
            fs::remove_file(&path)
                .unwrap_or_else(|e| panic!("Failed to remove file `{}`: {}", path.display(), e));
            info!("Removed `{}`", id);
        } else {
            error!("Config file id `{}` not found", id);
        }
    }
}

/// Open the startup folder.
pub fn open_config_folder() {
    Command::new(utils::OPEN_COMMAND)
        .arg(utils::CONFIG_PATH.as_os_str())
        .spawn()
        .expect("Failed to open config folder")
        .wait()
        .expect("Failed to open config folder");
}
