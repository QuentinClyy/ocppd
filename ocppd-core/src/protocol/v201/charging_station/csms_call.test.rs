use crate::fixtures::v201::station::Run;
use serde_json::{Value, json};

fn custom(mut payload: Value) -> Value {
    payload["customData"] = json!({"vendorId": "org.openchargealliance.Test", "note": 1});
    payload
}

fn offline_threshold() -> Value {
    custom(
        json!({"component": {"name": "OCPPCommCtrlr"}, "variable": {"name": "OfflineThreshold"}, "attributeType": "Actual"}),
    )
}

fn set_200(run: &mut Run) -> Value {
    let mut data = offline_threshold();
    data["attributeValue"] = json!("200");
    let request = custom(json!({"setVariableData": [data]}));
    let result = run.result(0, "SetVariables", request)["setVariableResult"][0].clone();
    run.answer_all(0);
    result
}

#[test]
fn given_no_vendor_extensions_when_data_transfer_received_then_unknown_vendor_id_per_p01_and_tc_p_01()
 {
    let mut run = Run::accepted(300);
    let request = json!({"vendorId": "com.example", "messageId": "Ping", "data": {"n": 1}});

    let response = run.result(0, "DataTransfer", request);

    assert_eq!(response, json!({"status": "UnknownVendorId"}));
}

#[test]
fn given_booted_station_when_set_variables_carries_custom_data_then_accepted_per_tc_p_03() {
    let mut run = Run::accepted(300);

    let result = set_200(&mut run);

    assert_eq!(result["attributeStatus"], "Accepted");
}

#[test]
fn given_value_set_with_custom_data_when_read_with_custom_data_then_reported_per_tc_p_03() {
    let mut run = Run::accepted(300);
    set_200(&mut run);
    let request = custom(json!({"getVariableData": [offline_threshold()]}));

    let result = run.result(0, "GetVariables", request)["getVariableResult"][0].clone();

    assert_eq!(result["attributeStatus"], "Accepted");
    assert_eq!(result["attributeValue"], "200");
}
