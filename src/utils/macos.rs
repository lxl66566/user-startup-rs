use std::{path::PathBuf, sync::LazyLock as Lazy};

/// The LaunchAgents directory. Can be overridden by the
/// `USER_STARTUP_CONFIG_DIR` environment variable (useful for testing).
pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    if let Some(path) = std::env::var_os("USER_STARTUP_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    dirs::home_dir()
        .expect("Could not find home directory")
        .join("Library")
        .join("LaunchAgents")
});

pub const COMMENT_PREFIX: &str = "<!--";

pub fn comment(s: &str) -> String {
    format!("{}{}\n-->", COMMENT_PREFIX, s)
}

pub const OPEN_COMMAND: &str = "open";

pub const FILE_EXT: &str = ".plist";

/// Make a name safe to use as a file stem on macOS.
#[must_use]
pub fn sanitize_file_stem(name: &str) -> String {
    name.replace(['/', ':'], "-")
}

/// Extract the original command from a plist file's content.
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
    format!(
        r#"{prefixed_cmd}
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{name}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{cmd}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <string>{stdout}</string>
    <string>{stderr}</string>
    <key>StandardOutPath</key>
    <string>{stdout_path}</string>
    <key>StandardErrorPath</key>
    <string>{stderr_path}</string>
</dict>
</plist>
"#,
        prefixed_cmd = comment(cmd),
        name = name,
        cmd = cmd,
        stdout = stdout.unwrap_or_default(),
        stderr = stderr.unwrap_or_default(),
        stdout_path = stdout.map_or(String::new(), |s| format!("{}.out", s)),
        stderr_path = stderr.map_or(String::new(), |s| format!("{}.err", s)),
    )
}
