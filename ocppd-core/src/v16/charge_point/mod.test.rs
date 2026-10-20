use crate::fixtures::v16::fake_csms::Csms;
use crate::fixtures::v16::security::result;
use serde_json::json;

fn reset(kind: &str) -> Csms {
    let mut csms = Csms::booted();
    result(csms.call(0, "Reset", json!({"type": kind})));
    csms
}

#[test]
fn given_booted_charge_point_when_reset_then_closed() {
    for kind in ["Soft", "Hard"] {
        let csms = reset(kind);

        let closed = csms.closed;

        assert!(closed, "{kind}");
    }
}
