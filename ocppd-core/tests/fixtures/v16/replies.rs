use crate::wire::frame::Frame;
use serde_json::Value;

pub fn status(frame: Frame) -> Value {
    match frame {
        Frame::CallResult { payload, .. } => payload["status"].clone(),
        other => panic!("expected a result, got {other:?}"),
    }
}
