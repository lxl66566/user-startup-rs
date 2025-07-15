use std::{path::PathBuf, sync::LazyLock as Lazy};

use super::parse_command;

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

pub const FILE_EXT: &str = ".ps1";

fn escape_quotes(s: impl AsRef<str>) -> String {
    s.as_ref().replace("\"", "\"\"").replace("'", "''")
}

pub fn format(cmd: &str, _: Option<&str>, stdout: Option<&str>, stderr: Option<&str>) -> String {
    let (bin, rest) = parse_command(cmd);
    format!(
        r#"{prefixed_cmd}
Start-Process "{bin}" -ArgumentList "{rest}" -WindowStyle Hidden {stdout} {stderr}
"#,
        prefixed_cmd = comment(cmd),
        bin = bin,
        rest = escape_quotes(rest),
        stdout = stdout.map_or(String::new(), |s| format!("-RedirectStandardOutput {s}")),
        stderr = stderr.map_or(String::new(), |s| format!("-RedirectStandardError {s}"))
    )
}
