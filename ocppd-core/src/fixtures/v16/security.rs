use crate::fixtures::v16::fake_csms::Csms;
use crate::wire::frame::Frame;
use serde_json::Value;

pub fn result(frame: Frame) -> Value {
    match frame {
        Frame::CallResult { payload, .. } => payload,
        other => panic!("expected a result, got {other:?}"),
    }
}

pub const KEY: &str = "30313233343536373839616263646566";

pub fn security_events(csms: &Csms) -> Vec<Value> {
    let sent = csms.sent("SecurityEventNotification");
    sent.iter().map(|sent| sent["type"].clone()).collect()
}
