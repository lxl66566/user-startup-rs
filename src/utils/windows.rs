use std::{
    fs::File,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock as Lazy,
};

use super::parse_command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
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

pub const COMMENT_PREFIX: &str = "# ";

pub fn comment(s: &str) -> String {
    format!("{COMMENT_PREFIX}{s}")
}

pub const OPEN_COMMAND: &str = "explorer";

pub const FILE_EXT: &str = ".cmd";

fn escape_quotes(s: impl AsRef<str>) -> String {
    s.as_ref().replace("\"", "^\"")
}

pub fn format(cmd: &str, _: Option<&str>, stdout: Option<&str>, stderr: Option<&str>) -> String {
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
    let (bin, rest) = parse_command(cmd);
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
