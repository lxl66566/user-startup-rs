use std::{path::PathBuf, sync::LazyLock as Lazy};

pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config")
        .join("autostart")
});

pub const COMMENT_PREFIX: &str = "# ";

pub fn comment(s: &str) -> String {
    format!("{}{}", COMMENT_PREFIX, s)
}

pub const OPEN_COMMAND: &str = "xdg-open";

pub const FILE_EXT: &str = ".desktop";

pub fn format(cmd: &str, name: Option<&str>, stdout: Option<&str>, stderr: Option<&str>) -> String {
    let name = name.unwrap_or_else(|| cmd.split_whitespace().next().unwrap());
    format!(
        r#"{prefixed_cmd}
[Desktop Entry]
Type=Application
Version=1.0
Name={name}
Comment={name} startup script
Exec={cmd}
StartupNotify=false
Terminal=false
"#,
        prefixed_cmd = comment(cmd),
        name = name,
        cmd = cmd,
    )
}
