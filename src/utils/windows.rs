use std::{
    fs::File,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock as Lazy,
};

use super::parse_command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The startup folder. Can be overridden by the `USER_STARTUP_CONFIG_DIR`
/// environment variable (useful for testing).
pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    if let Some(path) = std::env::var_os("USER_STARTUP_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    dirs::home_dir()
        .expect("Could not find home directory")
        .join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
});

pub const COMMENT_PREFIX: &str = ":: ";

#[must_use]
pub fn comment(s: &str) -> String {
    format!("{COMMENT_PREFIX}{s}")
}

pub const OPEN_COMMAND: &str = "explorer";

pub const FILE_EXT: &str = ".cmd";

/// Make a name safe to use as a file stem on Windows.
#[must_use]
pub fn sanitize_file_stem(name: &str) -> String {
    name.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "-")
}

/// Extract the original command from a startup file's content.
#[must_use]
pub fn extract_command(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .and_then(|line| {
            line.strip_prefix(COMMENT_PREFIX)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(Into::into)
        })
}

fn escape_quotes(s: impl AsRef<str>) -> String {
    s.as_ref().replace('"', "^\"")
}

#[must_use]
pub fn format(cmd: &str, _id: &str, stdout: Option<&str>, stderr: Option<&str>) -> String {
    format!(
        r#"{prefixed_cmd}
"{self_bin}" run "{cmd}" {stdout} {stderr}
"#,
        self_bin = std::env::current_exe()
            .expect("Failed to get current executable path")
            .display(),
        prefixed_cmd = comment(cmd),
        cmd = escape_quotes(cmd),
        stdout = stdout.map_or(String::new(), |s| format!("--stdout {s}")),
        stderr = stderr.map_or(String::new(), |s| format!("--stderr {s}"))
    )
}

/// Run a command with NO_WINDOW.
pub fn run_no_window(
    cmd: impl AsRef<str>,
    stdout: Option<impl AsRef<Path>>,
    stderr: Option<impl AsRef<Path>>,
) -> std::io::Result<()> {
    let (bin, rest) =
        parse_command(cmd).ok_or_else(|| std::io::Error::other("command is empty"))?;
    let mut command = Command::new(bin);
    command.creation_flags(CREATE_NO_WINDOW).raw_arg(rest);
    if let Some(stdout) = stdout {
        let f = File::create(stdout)?;
        command.stdout(f);
    }
    if let Some(stderr) = stderr {
        let f = File::create(stderr)?;
        command.stderr(f);
    }
    command.spawn()?;
    Ok(())
}
