use super::*;
use std::fs;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ocppd-tui-test-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn given_the_tui_beside_ocppd_and_on_the_path_when_found_then_the_one_beside() {
    let beside = scratch("beside");
    let on_path = scratch("on-path");
    fs::write(beside.join(NAME), "").unwrap();
    fs::write(on_path.join(NAME), "").unwrap();

    let found = find(Some(&beside), Some(on_path.as_os_str()));

    assert_eq!(found, Some(beside.join(NAME)));
}

#[test]
fn given_the_tui_only_on_the_path_when_found_then_the_one_on_the_path() {
    let beside = scratch("empty");
    let on_path = scratch("path-only");
    fs::write(on_path.join(NAME), "").unwrap();
    let path = std::env::join_paths([scratch("other"), on_path.clone()]).unwrap();

    let found = find(Some(&beside), Some(&path));

    assert_eq!(found, Some(on_path.join(NAME)));
}

#[test]
fn given_no_tui_anywhere_when_found_then_none() {
    let beside = scratch("nothing");

    let found = find(Some(&beside), None);

    assert_eq!(found, None);
}
