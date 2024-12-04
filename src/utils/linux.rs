//! use systemd to manage startup.

use std::{path::PathBuf, sync::LazyLock as Lazy};

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
    let name = name.unwrap_or_else(|| cmd.split_whitespace().next().unwrap());
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
JournalMaxFileSize=10M
JournalMaxRetention=5
TimeoutStartSec=60
TimeoutStopSec=30
WorkingDirectory=/tmp
User=%i

[Install]
WantedBy=default.target
"#,
        prefixed_cmd = comment(cmd),
        name = name,
        cmd = cmd,
        stdout = stdout.unwrap_or("syslog"),
        stderr = stderr.unwrap_or("syslog"),
    )
}
