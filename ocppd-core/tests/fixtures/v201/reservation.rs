use crate::api::intent::Intent;
use crate::api::io::Input;
use crate::fixtures::v201::station::{MINUTE, Run};
use serde_json::{Value, json};

pub const EXPIRY: &str = "1970-01-01T00:10:00Z";
pub const EXPIRY_MS: u64 = 10 * MINUTE;

pub fn reserve(run: &mut Run, id: i32, evse: Option<u32>, id_tag: &str) -> Value {
    let mut request = json!({
        "id": id,
        "expiryDateTime": EXPIRY,
        "idToken": {"idToken": id_tag, "type": "ISO14443"},
    });
    if let Some(evse) = evse {
        request["evseId"] = json!(evse);
    }
    let status = run.result(0, "ReserveNow", request)["status"].clone();
    run.answer_all(0);
    status
}

pub fn sent(run: &Run, action: &str) -> Vec<Value> {
    run.calls
        .iter()
        .filter(|(_, sent, _)| sent == action)
        .map(|(_, _, payload)| payload.clone())
        .collect()
}

pub fn statuses(run: &Run) -> Vec<(u64, String)> {
    let notifications = sent(run, "StatusNotification");
    notifications
        .iter()
        .map(|status| {
            let evse = status["evseId"].as_u64().unwrap();
            (evse, status["connectorStatus"].as_str().unwrap().to_owned())
        })
        .collect()
}

pub fn authorize(run: &mut Run, connector: u32, id_tag: &str, answer: Value) {
    let intent = Intent::Authorize {
        connector: Some(connector),
        id_tag: id_tag.into(),
    };
    run.input(0, Input::Intent(intent));
    run.answer(0, answer);
    run.answer_all(0);
}

pub fn accepted() -> Value {
    json!({"idTokenInfo": {"status": "Accepted"}})
}
