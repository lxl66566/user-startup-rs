use std::{collections::HashSet, fs, vec};

use log::{LevelFilter, warn};
use user_startup::{add_item, utils};

/// a - b
fn vec_diff<T: Eq + std::hash::Hash + Clone>(a: &[T], b: &[T]) -> HashSet<T> {
    let mut set = HashSet::new();
    for item in a {
        set.insert(item.clone());
    }
    for item in b {
        set.remove(item);
    }
    set
}

/// log and path init
fn test_init() {
    _ = pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Debug)
        .try_init();
    let config_path = &utils::CONFIG_PATH;
    if !config_path.exists() {
        warn!("Config path not found. Creating it...");
        fs::create_dir_all(config_path.as_os_str()).expect("Failed to create config directory");
    }
}

#[test]
fn intergration_test() {
    test_init();
    let start = user_startup::get_items_list();
    add_item("myusrtest", None, None, None);
    add_item("myusrtest", None, None, None);
    let items = user_startup::get_items_list();
    assert_eq!(items.len(), start.len() + 2);
    assert_eq!(
        vec_diff(&items, &start),
        HashSet::from([
            ("myusrtest".to_string(), "myusrtest".to_string()),
            ("myusrtest1".to_string(), "myusrtest".to_string())
        ])
    );
    user_startup::remove_items(vec!["myusrtest".to_string(), "myusrtest1".to_string()]);
}
