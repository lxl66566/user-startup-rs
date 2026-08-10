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

pub const OPEN_COMMAND: &str = "open";

pub const FILE_EXT: &str = ".plist";

/// Make a name safe to use as a file stem on macOS.
#[must_use]
pub fn sanitize_file_stem(name: &str) -> String {
    name.replace(['/', ':'], "-")
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        // `&amp;` must be unescaped last
        .replace("&amp;", "&")
}

/// Extract the original command from a plist file's content.
#[must_use]
pub fn extract_command(content: &str) -> Option<String> {
    let start = content.find("<key>ProgramArguments</key>")?;
    let rest = &content[start..];
    let end = rest.find("</array>")?;
    let mut args = Vec::new();
    let mut cur = &rest[..end];
    while let Some(s) = cur.find("<string>") {
        let after = &cur[s + "<string>".len()..];
        let e = after.find("</string>")?;
        args.push(xml_unescape(&after[..e]));
        cur = &after[e + "</string>".len()..];
    }
    match args.as_slice() {
        [] => None,
        [sh, flag, cmd] if sh == "/bin/sh" && flag == "-c" => Some(cmd.clone()),
        _ => Some(args.join(" ")),
    }
}

// `_shell` is unused: commands always run through `/bin/sh -c`, see below.
#[must_use]
pub fn format(
    cmd: &str,
    id: &str,
    stdout: Option<&str>,
    stderr: Option<&str>,
    _shell: bool,
) -> String {
    // launchd does not invoke a shell, so run the command through `sh -c` to
    // support arguments, quoting, pipes and other shell syntax.
    let argv = ["/bin/sh", "-c", cmd]
        .iter()
        .map(|a| format!("        <string>{}</string>", xml_escape(a)))
        .collect::<Vec<_>>()
        .join("\n");
    let stdout_block = stdout.map_or(String::new(), |s| {
        format!(
            "    <key>StandardOutPath</key>\n    <string>{}</string>\n",
            xml_escape(s)
        )
    });
    let stderr_block = stderr.map_or(String::new(), |s| {
        format!(
            "    <key>StandardErrorPath</key>\n    <string>{}</string>\n",
            xml_escape(s)
        )
    });
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>user-startup.{id}</string>
    <key>ProgramArguments</key>
    <array>
{argv}
    </array>
    <key>RunAtLoad</key>
    <true/>
{stdout_block}{stderr_block}</dict>
</plist>
"#,
        id = xml_escape(id),
        argv = argv,
        stdout_block = stdout_block,
        stderr_block = stderr_block,
    )
}
