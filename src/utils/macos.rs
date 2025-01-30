use std::{path::PathBuf, sync::LazyLock as Lazy};

use super::parse_command;

pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
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

pub fn format(cmd: &str, name: Option<&str>, stdout: Option<&str>, stderr: Option<&str>) -> String {
    let name = name
        .map(|s| s.to_string())
        .unwrap_or_else(|| parse_command(cmd).0);
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
