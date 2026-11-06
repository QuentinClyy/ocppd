use super::*;

#[test]
fn given_no_schedule_when_connected_then_first_ping_scheduled_one_interval_away() {
    assert_eq!(next(None, 1_000, 30, true), (Some(31_000), false));
}

#[test]
fn given_a_due_schedule_when_connected_then_pings_and_reschedules() {
    assert_eq!(next(Some(31_000), 31_000, 30, true), (Some(61_000), true));
}

#[test]
fn given_a_future_schedule_when_connected_then_nothing_changes() {
    assert_eq!(next(Some(31_000), 20_000, 30, true), (Some(31_000), false));
}

#[test]
fn given_a_schedule_when_the_interval_is_zero_then_it_is_cleared() {
    assert_eq!(next(Some(31_000), 31_000, 0, true), (None, false));
}

#[test]
fn given_a_schedule_when_disconnected_then_it_is_cleared() {
    assert_eq!(next(Some(31_000), 31_000, 30, false), (None, false));
}
