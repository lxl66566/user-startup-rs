#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)] // windows-sys

pub mod utils;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use log::{debug, error, info, warn};

/// Execute a command and log itself.
pub fn exec(cmd: &str) -> std::io::Result<()> {
    let mut parts = cmd.split_whitespace();
    let true_cmd = parts.next().expect("Command cannot be empty");
    let args: Vec<&str> = parts.collect();
    debug!("Executing `{}`", cmd);
    Command::new(true_cmd).args(args).status()?;
    Ok(())
}

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

/// Extract the script name from a command.
///
/// # Examples
///
/// ```rust
/// use user_startup::extract_name_from_cmd;
/// assert_eq!(extract_name_from_cmd("test.ps1"), "test");
/// assert_eq!(extract_name_from_cmd("D:\\no_install_software\\syncthing\\syncthing.exe"), "syncthing");
/// ```
pub fn extract_name_from_cmd(cmd: &str) -> String {
    cmd.split_whitespace()
        .find(|x| !x.is_empty())
        .expect("Command is empty!")
        .split(['/', '\\'])
        .rev()
        .find(|x| !x.is_empty())
        .expect("cannot extract name from invalid command")
        .split('.')
        .find(|x| !x.is_empty())
        .expect("cannot extract name from invalid command")
        .into()
}

/// Find a writable path for a startup command. Because the command may be not
/// unique, it will try to find the first available filename like this:
///
/// test, test1, test2, test3, test4, test5 ... test1000.
pub fn find_writable_path(name: impl AsRef<str>) -> PathBuf {
    let name = name.as_ref();
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

    let path = if let Some(name) = name {
        find_writable_path(name)
    } else {
        find_writable_path(extract_name_from_cmd(cmd))
    };

    fs::write(&path, utils::format(cmd, None, stdout, stderr))
        .expect("Failed to write config file");

    info!("Added `{}` to `{}`", cmd, path.display());

    // Reload the daemon and enable the service
    #[cfg(target_os = "linux")]
    {
        exec("systemctl daemon-reload --user").expect("daemon reloading error");
        exec(
            format!(
                "systemctl enable {} --user",
                path.file_name().unwrap().to_string_lossy()
            )
            .as_str(),
        )
        .expect("daemon enabling error");
    }
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
            .is_some_and(|ext| ext == utils::FILE_EXT.trim_start_matches('.'))
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
            // Disable the service
            #[cfg(target_os = "linux")]
            {
                exec(
                    format!(
                        "systemctl disable {} --user",
                        path.file_name().unwrap().to_string_lossy()
                    )
                    .as_str(),
                )
                .expect("daemon disabling error");
            }
            fs::remove_file(&path)
                .unwrap_or_else(|e| panic!("Failed to remove file `{}`: {}", path.display(), e));
            info!("Removed id `{}`", id);
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

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_find_writable_path() {
        let path = find_writable_path("test");
        assert_eq!(path, utils::CONFIG_PATH.join("test.ps1"));
        let path = find_writable_path(r#"D:\no_install_software\syncthing\syncthing.exe"#);
        assert_eq!(path, utils::CONFIG_PATH.join("syncthing.ps1"));
    }
}
