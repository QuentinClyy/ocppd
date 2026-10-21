use super::ChargingStation;
use crate::api::io::Output;
use crate::wire::frame::Frame;
use serde_json::json;

impl ChargingStation {
    pub(super) fn csms_call(&mut self, out: &mut Vec<Output>, id: String, action: &str) {
        out.push(Output::Send(Frame::CallError {
            id,
            code: "NotImplemented".into(),
            description: format!("{action} is not implemented"),
            details: json!({}),
        }));
    }
}
