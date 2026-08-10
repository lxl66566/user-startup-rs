//! use systemd to manage startup.

use std::{path::PathBuf, sync::LazyLock as Lazy};

/// The systemd user unit directory. Can be overridden by the
/// `USER_STARTUP_CONFIG_DIR` environment variable (useful for testing).
pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    if let Some(path) = std::env::var_os("USER_STARTUP_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config")
        .join("systemd")
        .join("user")
});

pub const COMMENT_PREFIX: &str = "# ";

pub fn comment(s: &str) -> String {
    format!("{COMMENT_PREFIX}{s}")
}

pub const OPEN_COMMAND: &str = "xdg-open";

pub const FILE_EXT: &str = ".service";

/// Make a name safe to use as a systemd unit file stem.
#[must_use]
pub fn sanitize_file_stem(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Extract the original command from a unit file's content.
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

pub fn format(cmd: &str, name: &str, stdout: Option<&str>, stderr: Option<&str>) -> String {
    // systemd requires an explicit `append:` prefix to redirect output to a
    // file; a bare path is invalid and would prevent the unit from starting.
    let stdout = stdout.map_or_else(|| "journal".to_string(), |s| format!("append:{s}"));
    let stderr = stderr.map_or_else(|| "journal".to_string(), |s| format!("append:{s}"));
    format!(
        r#"{prefixed_cmd}
[Unit]
Description={name}

[Service]
ExecStart={cmd}
Restart=on-failure
RestartSec=5
LimitNOFILE=4096
StandardOutput={stdout}
StandardError={stderr}
SyslogIdentifier={name}
LogLevelMax=info
TimeoutStartSec=60
TimeoutStopSec=30

[Install]
WantedBy=default.target
"#,
        prefixed_cmd = comment(cmd),
        name = name,
        cmd = cmd,
        stdout = stdout,
        stderr = stderr,
    )
}
