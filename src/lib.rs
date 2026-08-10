#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

mod error;
pub mod utils;

use std::{fs, path::PathBuf, process::Command};

pub use error::{Error, Result};
use log::{debug, error, info, warn};

/// Options for [`add_item`].
#[derive(Debug, Default, Clone, Copy)]
pub struct AddOptions<'a> {
    /// The name of the command. If `None`, it is extracted from the command.
    pub name: Option<&'a str>,
    /// Redirect the command's stdout to a file.
    pub stdout: Option<&'a str>,
    /// Redirect the command's stderr to a file.
    pub stderr: Option<&'a str>,
}

/// Run a program and return an error if it exits with a non-success status.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn run_status(program: &str, args: &[&str]) -> std::io::Result<()> {
    debug!("Executing `{program} {}`", args.join(" "));
    let status = Command::new(program).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "`{program}` exited with {status}"
        )))
    }
}

/// Run a post-add hook for the new startup file (e.g. enable a systemd
/// service). Failures are only logged, since the file itself is already in
/// place and will take effect on next login.
#[cfg(target_os = "linux")]
fn on_item_added(path: &std::path::Path) {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if let Err(e) = run_status("systemctl", &["daemon-reload", "--user"]) {
        warn!("Failed to reload systemd daemon: {e}");
    }
    if let Err(e) = run_status("systemctl", &["enable", &name, "--user"]) {
        warn!("Failed to enable service `{name}`: {e}");
    }
}

/// Run a pre-remove hook for the startup file (e.g. disable a systemd
/// service). Failures are only logged.
#[cfg(target_os = "linux")]
fn on_item_removed(path: &std::path::Path) {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if let Err(e) = run_status("systemctl", &["disable", &name, "--user"]) {
        warn!("Failed to disable service `{name}`: {e}");
    }
}

/// Load the LaunchAgent so that it takes effect immediately. Failures are
/// only logged, since the plist will be loaded automatically on next login.
#[cfg(target_os = "macos")]
fn on_item_added(path: &std::path::Path) {
    let path_str = path.to_string_lossy();
    if let Err(e) = run_status("launchctl", &["load", "-w", &path_str]) {
        warn!("Failed to load LaunchAgent `{}`: {e}", path.display());
    }
}

/// Unload the LaunchAgent before removing its plist. Failures are only
/// logged.
#[cfg(target_os = "macos")]
fn on_item_removed(path: &std::path::Path) {
    let path_str = path.to_string_lossy();
    if let Err(e) = run_status("launchctl", &["unload", "-w", &path_str]) {
        warn!("Failed to unload LaunchAgent `{}`: {e}", path.display());
    }
}

/// Extract the script name from a command.
///
/// # Examples
///
/// ```rust
/// use user_startup::extract_name_from_cmd;
/// assert_eq!(extract_name_from_cmd("test.cmd").unwrap(), "test");
/// assert_eq!(
///     extract_name_from_cmd("D:\\no_install_software\\syncthing\\syncthing.exe").unwrap(),
///     "syncthing"
/// );
/// assert_eq!(
///     extract_name_from_cmd("'C:\\Program Files\\My App\\myapp.exe' --arg").unwrap(),
///     "myapp"
/// );
/// ```
pub fn extract_name_from_cmd(cmd: &str) -> Result<String> {
    let (executable, _) = utils::parse_command(cmd).ok_or(Error::EmptyCommand)?;
    executable
        .rsplit(['/', '\\'])
        .find(|x| !x.is_empty())
        .and_then(|file| file.split('.').find(|x| !x.is_empty()))
        .map(Into::into)
        .ok_or_else(|| Error::InvalidCommand(cmd.to_string()))
}

/// Find a writable path for a startup command. Because the command may be not
/// unique, it will try to find the first available filename like this:
///
/// test, test1, test2, test3, test4, test5 ... test1000.
pub fn find_writable_path(name: impl AsRef<str>) -> Result<PathBuf> {
    let name = name.as_ref();
    debug!("Finding writable path for `{name}`");
    let base_path = &utils::CONFIG_PATH;
    let ext = utils::FILE_EXT;

    let initial_path = base_path.join(format!("{name}{ext}"));
    if !initial_path.exists() {
        return Ok(initial_path);
    }

    for i in 1..1000 {
        let path = base_path.join(format!("{name}{i}{ext}"));
        if !path.exists() {
            debug!(
                "Found writable path `{}`",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            return Ok(path);
        }
    }
    Err(Error::TooManyItems(name.to_string()))
}

/// Add a new startup command.
///
/// Returns the path of the created startup file.
pub fn add_item(cmd: &str, options: AddOptions) -> Result<PathBuf> {
    if cmd.trim().is_empty() {
        return Err(Error::EmptyCommand);
    }

    let name = match options.name {
        Some(name) => name.to_string(),
        None => extract_name_from_cmd(cmd)?,
    };

    fs::create_dir_all(&*utils::CONFIG_PATH)?;

    let path = find_writable_path(utils::sanitize_file_stem(&name))?;
    // Use the final file stem as the item id so that generated ids (e.g.
    // systemd unit names, launchd labels) are always unique and valid.
    let id = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    fs::write(
        &path,
        utils::format(cmd, &id, options.stdout, options.stderr),
    )?;

    info!("Added `{}` to `{}`", cmd, path.display());

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    on_item_added(&path);

    Ok(path)
}

/// Get a list of startup commands, sorted by id.
///
/// # Returns
///
/// A vector of tuples, where the first element is the id of the command and
/// the second element is the command itself.
pub fn get_items_list() -> Result<Vec<(String, String)>> {
    let config_path = utils::CONFIG_PATH.as_os_str();
    debug!(
        "Finding config files in `{}` with extension `{}`",
        config_path.to_string_lossy(),
        utils::FILE_EXT
    );
    let mut res = vec![];

    let entries = match fs::read_dir(config_path) {
        Ok(entries) => entries,
        // No config directory yet means no items.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(res),
        Err(e) => return Err(e.into()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == utils::FILE_EXT.trim_start_matches('.'))
            && let Ok(content) = fs::read_to_string(&path)
            && let Some(command) = utils::extract_command(&content)
        {
            let id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            res.push((id, command));
        }
    }
    res.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(res)
}

/// Check that an id cannot escape the config directory.
fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") {
        return Err(Error::InvalidId(id.to_string()));
    }
    Ok(())
}

/// Remove startup commands.
///
/// Tries to remove all given ids; returns an error if any of them does not
/// exist.
pub fn remove_items(ids: &[String]) -> Result<()> {
    let mut missing = vec![];
    for id in ids {
        validate_id(id)?;
        let path = utils::CONFIG_PATH.join(format!("{}{}", id, utils::FILE_EXT));
        if path.exists() {
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            on_item_removed(&path);
            fs::remove_file(&path)?;
            info!("Removed id `{id}`");
        } else {
            error!("Config file id `{id}` not found");
            missing.push(id.clone());
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(Error::NotFound(missing.join(", ")))
    }
}

/// Open the startup folder.
pub fn open_config_folder() -> Result<()> {
    fs::create_dir_all(&*utils::CONFIG_PATH)?;
    Command::new(utils::OPEN_COMMAND)
        .arg(utils::CONFIG_PATH.as_os_str())
        .spawn()?
        .wait()?;
    Ok(())
}
