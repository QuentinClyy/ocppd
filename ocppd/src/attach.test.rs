use super::*;
use std::collections::HashMap;
use std::os::unix::net::UnixListener;

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let values: HashMap<String, String> = pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    move |key| values.get(key).cloned()
}

#[test]
fn given_the_socket_variables_when_the_default_socket_is_chosen_then_in_their_order() {
    let cases = [
        (
            vec![
                ("OCPPD_SOCKET", "/s.sock"),
                ("XDG_RUNTIME_DIR", "/run/user/1"),
                ("HOME", "/home/q"),
            ],
            "/s.sock",
        ),
        (
            vec![("XDG_RUNTIME_DIR", "/run/user/1"), ("HOME", "/home/q")],
            "/run/user/1/ocppd.sock",
        ),
        (
            vec![("XDG_RUNTIME_DIR", ""), ("HOME", "/home/q")],
            "/home/q/.ocppd.sock",
        ),
    ];

    for (pairs, expected) in cases {
        let socket = default_socket(env(&pairs));

        assert_eq!(socket, PathBuf::from(expected), "{pairs:?}");
    }
}

#[test]
fn given_the_state_variables_when_the_log_file_is_chosen_then_xdg_state_first() {
    let with_state = log_file(env(&[("XDG_STATE_HOME", "/state"), ("HOME", "/home/q")]));
    let without = log_file(env(&[("HOME", "/home/q")]));

    assert_eq!(with_state, PathBuf::from("/state/ocppd/ocppd.log"));
    assert_eq!(
        without,
        PathBuf::from("/home/q/.local/state/ocppd/ocppd.log")
    );
}

#[test]
fn given_a_daemon_already_listening_when_ensured_then_nothing_is_started() {
    let dir = std::env::temp_dir().join(format!("ocppd-attach-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let socket = dir.join("ocppd.sock");
    let _ = fs::remove_file(&socket);
    let _listening = UnixListener::bind(&socket).unwrap();

    let ensured = ensure(
        &socket,
        "ws://x",
        Path::new("/does/not/exist"),
        &dir.join("log"),
    );

    assert_eq!(ensured, Ok(()));
    assert!(!dir.join("log").exists());
}
