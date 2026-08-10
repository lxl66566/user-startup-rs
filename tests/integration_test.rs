use std::fs;

use user_startup::{
    AddOptions, Error, add_item, find_writable_path, get_items_list, remove_items, utils,
};

/// Run the whole flow in a single test: `CONFIG_PATH` is a process-global
/// lazy static, so all steps must share one temp directory anyway.
#[test]
fn integration_test() {
    let tmp = tempfile::TempDir::new().unwrap();
    // SAFETY: this is the only test in the binary and runs before any access
    // to CONFIG_PATH, so no other thread can observe a half-set environment.
    unsafe { std::env::set_var("USER_STARTUP_CONFIG_DIR", tmp.path()) };

    // Empty config dir means an empty list (and the dir is created on add).
    assert_eq!(get_items_list().unwrap(), Vec::<(String, String)>::new());

    // Add a new item; the file is created inside the temp dir.
    let p1 = add_item("myusrtest", AddOptions::default()).unwrap();
    assert!(p1.starts_with(tmp.path()));
    assert!(p1.exists());
    assert_eq!(get_items_list().unwrap(), vec![(
        "myusrtest".to_string(),
        "myusrtest".to_string()
    )]);

    // Adding the same command again is rejected unless forced.
    assert!(matches!(
        add_item("myusrtest", AddOptions::default()),
        Err(Error::AlreadyExists(_))
    ));

    // A forced duplicate gets a suffixed id.
    let p2 = add_item("myusrtest", AddOptions {
        force: true,
        ..AddOptions::default()
    })
    .unwrap();
    assert!(p2.exists());
    assert_eq!(get_items_list().unwrap(), vec![
        ("myusrtest".to_string(), "myusrtest".to_string()),
        ("myusrtest1".to_string(), "myusrtest".to_string()),
    ]);

    // find_writable_path skips existing files.
    let p3 = find_writable_path("myusrtest").unwrap();
    assert_eq!(p3.file_stem().unwrap().to_string_lossy(), "myusrtest2");

    // The shell option preserves the raw command in `list` output.
    let p4 = add_item("echo a && echo b", AddOptions {
        shell: true,
        ..AddOptions::default()
    })
    .unwrap();
    let content = fs::read_to_string(&p4).unwrap();
    assert_eq!(
        utils::extract_command(&content).as_deref(),
        Some("echo a && echo b")
    );

    // A name with spaces is sanitized for the file stem on platforms that
    // need it (systemd unit names cannot contain spaces).
    let p5 = add_item("somecmd", AddOptions {
        name: Some("my app"),
        ..AddOptions::default()
    })
    .unwrap();
    let stem5 = p5.file_stem().unwrap().to_string_lossy().into_owned();
    #[cfg(target_os = "linux")]
    assert_eq!(stem5, "my-app");
    #[cfg(not(target_os = "linux"))]
    assert_eq!(stem5, "my app");

    // Ids with path separators are rejected and cannot escape the config dir.
    let outside = tmp.path().join(format!("outside{}", utils::FILE_EXT));
    fs::write(&outside, "x").unwrap();
    assert!(matches!(
        remove_items(&["../outside".to_string()]),
        Err(Error::InvalidId(_))
    ));
    assert!(outside.exists());

    // Removing a mix of existing and missing ids removes what it can and
    // reports the missing one.
    assert!(matches!(
        remove_items(&["myusrtest".to_string(), "nonexistent".to_string()]),
        Err(Error::NotFound(_))
    ));
    assert!(!p1.exists());

    // Clean up the rest.
    remove_items(&[
        "myusrtest1".to_string(),
        p4.file_stem().unwrap().to_string_lossy().into_owned(),
        p5.file_stem().unwrap().to_string_lossy().into_owned(),
    ])
    .unwrap();
    assert_eq!(get_items_list().unwrap(), Vec::<(String, String)>::new());
}
