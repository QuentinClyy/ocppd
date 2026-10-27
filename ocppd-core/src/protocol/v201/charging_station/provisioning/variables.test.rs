use crate::api::event::LinkEvent;
use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::station::Run;
use crate::protocol::v201::event::EventKind;

fn configure(run: &mut Run, key: &str, value: &str) {
    let intent = Intent::Configure {
        key: key.into(),
        value: value.into(),
    };
    run.input(0, Input::Intent(intent));
}

fn rejected(reason: &str) -> EventKind {
    EventKind::Link(LinkEvent::IntentRejected {
        connector: None,
        reason: reason.into(),
    })
}

#[test]
fn given_a_station_when_settings_listed_then_read_only_is_flagged_and_the_password_hidden() {
    let mut run = Run::new(1);

    let settings = run.station.settings(0);

    let key = |name: &str| settings.iter().find(|setting| setting.key == name).unwrap();
    assert!(!key("OCPPCommCtrlr.HeartbeatInterval").readonly);
    assert!(key("SecurityCtrlr.SecurityProfile").readonly);
    assert_eq!(key("SecurityCtrlr.BasicAuthPassword").value, None);
}

#[test]
fn given_a_station_when_settings_listed_then_configure_takes_a_listed_writable_key() {
    let mut run = Run::new(1);
    let settings = run.station.settings(0);
    let writable = settings
        .iter()
        .find(|setting| setting.key == "OCPPCommCtrlr.HeartbeatInterval")
        .unwrap();

    let accepted = run.station.configure(&writable.key, "120");

    assert!(accepted);
}

#[test]
fn given_an_accepted_station_when_configure_intent_then_the_variable_changes_and_variable_changed_is_emitted()
 {
    let mut run = Run::accepted(300);

    configure(&mut run, "OCPPCommCtrlr.HeartbeatInterval", "120");

    let settings = run.station.settings(0);
    let heartbeat = settings
        .iter()
        .find(|setting| setting.key == "OCPPCommCtrlr.HeartbeatInterval")
        .unwrap();
    assert_eq!(heartbeat.value.as_deref(), Some("120"));
    assert!(run.events.contains(&EventKind::VariableChanged {
        component: "OCPPCommCtrlr".into(),
        variable: "HeartbeatInterval".into(),
        value: "120".into(),
    }));
}

#[test]
fn given_an_accepted_station_when_configure_intent_on_a_read_only_variable_then_intent_rejected() {
    let mut run = Run::accepted(300);

    configure(&mut run, "SecurityCtrlr.SecurityProfile", "2");

    assert!(run.events.contains(&rejected(
        "SecurityCtrlr.SecurityProfile refused (Rejected)"
    )));
}

#[test]
fn given_an_accepted_station_when_configure_intent_without_a_component_then_intent_rejected() {
    let mut run = Run::accepted(300);

    configure(&mut run, "HeartbeatInterval", "120");

    assert!(run.events.contains(&rejected(
        "HeartbeatInterval refused: not Component.Variable"
    )));
}
