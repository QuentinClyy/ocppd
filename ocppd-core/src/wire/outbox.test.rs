use super::Outbox;
use crate::wire::frame::Frame;
use serde_json::json;

fn ids(outbox: &mut Outbox, now: u64, count: usize) -> Vec<String> {
    let mut ids = Vec::new();
    for _ in 0..count {
        outbox.push("Heartbeat", 0, json!({}));
        let Some(Frame::Call { id, .. }) = outbox.next_frame(now, true) else {
            panic!("no call sent");
        };
        outbox.resolve(&id);
        ids.push(id);
    }
    ids
}

#[test]
fn given_calls_sent_in_the_same_millisecond_when_ids_are_read_then_each_is_different() {
    let mut outbox = Outbox::new(&[]);

    let sent = ids(&mut outbox, 1_792_893_141_250, 3);

    assert_eq!(sent, ["17928931412501", "17928931412502", "17928931412503"]);
}

#[test]
fn given_a_station_recreated_later_when_it_sends_then_no_id_of_the_earlier_run_comes_back() {
    let mut earlier = Outbox::new(&[]);
    let first_run = ids(&mut earlier, 1_792_893_141_250, 3);
    let mut later = Outbox::new(&[]);

    let second_run = ids(&mut later, 1_792_893_141_251, 3);

    assert!(
        second_run.iter().all(|id| !first_run.contains(id)),
        "{first_run:?} {second_run:?}"
    );
}
