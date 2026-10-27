use super::*;

fn identities(line: &str) -> Identities {
    serde_json::from_str(line).unwrap()
}

#[test]
fn given_prefix_and_count_when_listed_then_numbered_ids_sharing_the_password() {
    let spawn = identities(r#"{"prefix":"CP","count":2,"password":"shared"}"#);

    let list = spawn.list().unwrap();

    let ids: Vec<_> = list.iter().map(|identity| identity.id.as_str()).collect();
    assert_eq!(ids, ["CP1", "CP2"]);
    assert!(
        list.iter()
            .all(|identity| identity.password.as_deref() == Some("shared"))
    );
}

#[test]
fn given_listed_stations_when_listed_then_each_keeps_its_own_password() {
    let spawn = identities(r#"{"stations":[{"id":"A","password":"pa"},{"id":"B"}]}"#);

    let list = spawn.list().unwrap();

    let pairs: Vec<_> = list
        .iter()
        .map(|identity| (identity.id.as_str(), identity.password.as_deref()))
        .collect();
    assert_eq!(pairs, [("A", Some("pa")), ("B", None)]);
}

#[test]
fn given_stations_and_a_prefix_when_listed_then_refused() {
    let spawn = identities(r#"{"stations":[{"id":"A"}],"prefix":"CP","count":1}"#);

    let list = spawn.list();

    assert_eq!(
        list.err(),
        Some("stations takes no prefix, count or password")
    );
}

#[test]
fn given_stations_and_a_shared_password_when_listed_then_refused() {
    let spawn = identities(r#"{"stations":[{"id":"A"}],"password":"shared"}"#);

    let list = spawn.list();

    assert_eq!(
        list.err(),
        Some("stations takes no prefix, count or password")
    );
}

#[test]
fn given_a_prefix_without_count_when_listed_then_refused() {
    let spawn = identities(r#"{"prefix":"CP"}"#);

    let list = spawn.list();

    assert_eq!(list.err(), Some("needs stations, or prefix with count"));
}
