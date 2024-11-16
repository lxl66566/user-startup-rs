use std::{collections::HashSet, vec};

use user_startup::add_item;

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

#[test]
fn intergration_test() {
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
