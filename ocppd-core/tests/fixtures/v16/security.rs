use crate::wire::frame::Frame;
use serde_json::Value;

pub fn result(frame: Frame) -> Value {
    match frame {
        Frame::CallResult { payload, .. } => payload,
        other => panic!("expected a result, got {other:?}"),
    }
}
