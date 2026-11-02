use crate::api::io::Input;
use crate::fixtures::v16::fake_csms::{Csms, MINUTE};
use crate::fixtures::v16::raw::station;
use crate::protocol::v16::charge_point::ChargePoint;

fn ping_every(csms: &mut Csms, seconds: &str) {
    assert_eq!(
        csms.configure(0, "WebSocketPingInterval", seconds),
        "Accepted"
    );
}

#[test]
fn given_a_new_charge_point_when_the_ping_interval_is_read_then_it_is_30_seconds() {
    let charge_point = ChargePoint::new(station()).unwrap();

    assert_eq!(
        charge_point.configuration.number("WebSocketPingInterval"),
        Some(30)
    );
}

#[test]
fn given_ping_interval_set_when_it_passes_then_pinged_and_next_ping_scheduled() {
    let mut csms = Csms::booted();
    ping_every(&mut csms, "45");
    assert_eq!(csms.charge_point.next_deadline(), Some(45_000));

    csms.tick(45_000);

    assert_eq!(csms.pings, 1);
    assert_eq!(csms.charge_point.next_deadline(), Some(90_000));
}

#[test]
fn given_ping_disabled_when_time_passes_then_no_ping() {
    let mut csms = Csms::booted();
    ping_every(&mut csms, "45");
    ping_every(&mut csms, "0");

    csms.tick(MINUTE);

    assert_eq!(csms.pings, 0);
}

#[test]
fn given_ping_interval_while_offline_when_time_passes_then_no_ping() {
    let mut csms = Csms::booted();
    ping_every(&mut csms, "45");
    csms.disconnect(MINUTE);

    csms.tick(2 * MINUTE);

    assert_eq!(csms.pings, 0);
}

#[test]
fn given_a_booted_charge_point_when_a_ping_interval_is_set_then_input_other_than_tick_also_schedules_it()
 {
    let mut csms = Csms::booted();

    ping_every(&mut csms, "45");
    csms.input(10_000, Input::Tick);

    assert_eq!(csms.charge_point.next_deadline(), Some(45_000));
}
