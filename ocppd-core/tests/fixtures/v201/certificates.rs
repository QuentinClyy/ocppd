use crate::fixtures::v201::station::Run;
use crate::fixtures::x509::authority::Authority;
use crate::x509::signing::signing_request;
use serde_json::{Value, json};

pub const CSMS_ROOT: &str = "CSMSRootCertificate";

pub fn install(run: &mut Run, kind: &str, pem: &str) -> Value {
    let request = json!({"certificateType": kind, "certificate": pem});
    run.result(0, "InstallCertificate", request)["status"].clone()
}

pub fn entries(run: &mut Run) -> Value {
    let data =
        json!({"component": {"name": "SecurityCtrlr"}, "variable": {"name": "CertificateEntries"}});
    run.result(0, "GetVariables", json!({"getVariableData": [data]}))["getVariableResult"][0]
        ["attributeValue"]
        .clone()
}

pub fn signed_by(authority: &Authority, name: &str) -> String {
    let (_, csr) = signing_request(name, "").unwrap();
    String::from_utf8(authority.sign(&csr).to_pem().unwrap()).unwrap()
}
