use crate::fixtures::v201::station::Run;

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
