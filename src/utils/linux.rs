//! use systemd to manage startup.

use std::{path::PathBuf, sync::LazyLock as Lazy};

use super::parse_command;

pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config")
        .join("systemd")
        .join("user")
});

pub const COMMENT_PREFIX: &str = "# ";

pub fn comment(s: &str) -> String {
    format!("{}{}", COMMENT_PREFIX, s)
}

pub const OPEN_COMMAND: &str = "xdg-open";

pub const FILE_EXT: &str = ".service";

pub fn format(cmd: &str, name: Option<&str>, stdout: Option<&str>, stderr: Option<&str>) -> String {
    let name = name
        .map(|s| s.to_string())
        .unwrap_or_else(|| parse_command(cmd).0);
    format!(
        r#"{prefixed_cmd}
[Unit]
Description={name}
After=network.target

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
WorkingDirectory=/tmp

[Install]
WantedBy=default.target
"#,
        prefixed_cmd = comment(cmd),
        name = name,
        cmd = cmd,
        stdout = stdout.unwrap_or("journal"),
        stderr = stderr.unwrap_or("journal"),
    )
}
