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
    s.lines()
        .map(|l| format!("{COMMENT_PREFIX}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
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
    let mut parts = vec![];
    for line in content.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(COMMENT_PREFIX) {
            parts.push(rest.to_string());
        } else {
            break;
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Escape a string for sh's ANSI-C quoting (`$'...'`).
fn sh_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Escape a string inside systemd's double quotes.
fn systemd_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn format(
    cmd: &str,
    id: &str,
    stdout: Option<&str>,
    stderr: Option<&str>,
    shell: bool,
) -> String {
    // systemd requires an explicit `append:` prefix to redirect output to a
    // file; a bare path is invalid and would prevent the unit from starting.
    let stdout = stdout.map_or_else(|| "journal".to_string(), |s| format!("append:{s}"));
    let stderr = stderr.map_or_else(|| "journal".to_string(), |s| format!("append:{s}"));
    // Unit values cannot span lines and systemd only groups tokens with
    // double quotes, so pass the script through sh's ANSI-C quoting; this
    // keeps quotes, newlines and tabs intact through both escape layers.
    let exec_start = if shell {
        format!("/bin/sh -c \"$'{}'\"", systemd_escape(&sh_escape(cmd)))
    } else {
        cmd.to_string()
    };
    format!(
        r#"{prefixed_cmd}
[Unit]
Description={name}

[Service]
ExecStart={exec_start}
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
        name = id,
        exec_start = exec_start,
        stdout = stdout,
        stderr = stderr,
    )
}
