//! Cross-platform snapshot tests for the startup file generators. All
//! platform modules are compiled on every target, so these run everywhere.

use user_startup::utils::{linux, macos, windows};

#[test]
fn test_linux_format() {
    let expected = r"# echo hi
[Unit]
Description=test

[Service]
ExecStart=echo hi
Restart=on-failure
RestartSec=5
LimitNOFILE=4096
StandardOutput=journal
StandardError=journal
SyslogIdentifier=test
LogLevelMax=info
TimeoutStartSec=60
TimeoutStopSec=30

[Install]
WantedBy=default.target
";
    assert_eq!(
        linux::format("echo hi", "test", None, None, false),
        expected
    );
}

#[test]
fn test_linux_format_with_redirection() {
    let content = linux::format(
        "echo hi",
        "test",
        Some("/tmp/o.log"),
        Some("/tmp/e.log"),
        false,
    );
    assert!(content.contains("StandardOutput=append:/tmp/o.log"));
    assert!(content.contains("StandardError=append:/tmp/e.log"));
}

#[test]
fn test_linux_format_shell_escaping() {
    // Quotes and backslashes must survive systemd's double-quote layer and
    // sh's ANSI-C quoting layer.
    let content = linux::format("echo \"a'b\\c\"", "test", None, None, true);
    assert!(content.contains(r#"ExecStart=/bin/sh -c "$'echo \"a\\'b\\\\c\"'""#));
    assert_eq!(
        linux::extract_command(&content).as_deref(),
        Some("echo \"a'b\\c\"")
    );
}

#[test]
fn test_linux_format_shell_multiline() {
    let content = linux::format("echo a\necho b", "test", None, None, true);
    // Newlines must not appear literally inside a unit value.
    let exec_line = content
        .lines()
        .find(|l| l.starts_with("ExecStart="))
        .unwrap();
    assert_eq!(exec_line, r#"ExecStart=/bin/sh -c "$'echo a\\necho b'""#);
    assert_eq!(
        linux::extract_command(&content).as_deref(),
        Some("echo a\necho b")
    );
}

#[test]
fn test_linux_extract_command() {
    let content = linux::format("echo hi", "test", None, None, false);
    assert_eq!(linux::extract_command(&content).as_deref(), Some("echo hi"));
}

#[test]
fn test_macos_format() {
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>user-startup.test</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/sh</string>
        <string>-c</string>
        <string>echo hi</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#;
    assert_eq!(
        macos::format("echo hi", "test", None, None, false),
        expected
    );
}

#[test]
fn test_macos_format_escaping_and_redirection() {
    // XML-special characters must be escaped, and redirection keys emitted.
    let content = macos::format(
        "echo 'a<b>' & bye",
        "test",
        Some("/tmp/o.log"),
        Some("/tmp/e.log"),
        false,
    );
    assert!(content.contains("<string>echo &apos;a&lt;b&gt;&apos; &amp; bye</string>"));
    assert!(content.contains("    <key>StandardOutPath</key>\n    <string>/tmp/o.log</string>\n"));
    assert!(
        content.contains("    <key>StandardErrorPath</key>\n    <string>/tmp/e.log</string>\n")
    );
    // The original command must round-trip.
    assert_eq!(
        macos::extract_command(&content).as_deref(),
        Some("echo 'a<b>' & bye")
    );
}

#[test]
fn test_macos_format_valid_plist() {
    // No dangling <string> elements outside ProgramArguments: every <string>
    // must either follow a <key> or live inside the <array>.
    let content = macos::format("cmd", "test", None, None, false);
    assert!(!content.contains("KeepAlive"));
    let dict_body = content
        .split("<array>")
        .nth(1)
        .unwrap()
        .split("</array>")
        .nth(1)
        .unwrap();
    assert!(!dict_body.contains("<string>"));
}

#[test]
fn test_windows_format() {
    let self_bin = std::env::current_exe().unwrap();
    let expected = format!(":: echo hi\n\"{}\" run \"echo hi\"\n", self_bin.display());
    assert_eq!(
        windows::format("echo hi", "test", None, None, false),
        expected
    );
}

#[test]
fn test_windows_format_escaping_and_redirection() {
    let self_bin = std::env::current_exe().unwrap();
    let content = windows::format(
        "echo 100% & say \"hi\"",
        "test",
        Some("D:\\my logs\\o.log"),
        Some("D:\\my logs\\e.log"),
        false,
    );
    let expected = format!(
        ":: echo 100% & say \"hi\"\n\"{}\" run \"echo 100%% & say \\\"hi\\\"\" --stdout \"D:\\my \
         logs\\o.log\" --stderr \"D:\\my logs\\e.log\"\n",
        self_bin.display()
    );
    assert_eq!(content, expected);
    assert_eq!(
        windows::extract_command(&content).as_deref(),
        Some("echo 100% & say \"hi\"")
    );
}

#[test]
fn test_windows_format_shell_multiline() {
    let content = windows::format("echo a\necho b", "test", None, None, true);
    assert_eq!(content, ":: echo a\n:: echo b\necho a\necho b\n");
    assert_eq!(
        windows::extract_command(&content).as_deref(),
        Some("echo a\necho b")
    );
}
