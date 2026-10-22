use crate::api::io::Output;
use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::id_token::IdToken;
use crate::protocol::v201::messages::diagnostics::customer_information::{
    CustomerInformationRequest, CustomerInformationResponse, CustomerInformationStatus,
    NotifyCustomerInformationRequest,
};
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;
use serde_json::{Map, Value, json};

const PART_CHARS: usize = 512;
const CLEARED: &str = "Cleared";

impl ChargingStation {
    pub(in crate::protocol::v201::charging_station) fn customer_information(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: CustomerInformationRequest,
    ) -> CustomerInformationResponse {
        let references = [
            request.id_token.is_some(),
            request.customer_certificate.is_some(),
            request.customer_identifier.is_some(),
        ];
        if references.iter().filter(|&&given| given).count() != 1 {
            return response(CustomerInformationStatus::Invalid);
        }
        if !request.report && !request.clear {
            return response(CustomerInformationStatus::Rejected);
        }
        let data = match &request.id_token {
            Some(id_token) => self.customer_data(id_token),
            None => String::new(),
        };
        if request.clear
            && let Some(id_token) = &request.id_token
            && self.cache.tokens.find(id_token).is_some()
        {
            self.cache.tokens.remove(id_token);
            let id_tag = id_token.id_token.as_str().to_owned();
            self.emit(out, now, EventKind::CustomerDataCleared { id_tag });
        }
        let data = match request.report {
            true => data,
            false => CLEARED.to_owned(),
        };
        self.notify_customer_information(now, request.request_id, &data);
        response(CustomerInformationStatus::Accepted)
    }

    fn customer_data(&self, id_token: &IdToken) -> String {
        let stores = [
            ("authorizationCache", self.cache.tokens.find(id_token)),
            ("localList", self.local_list.tokens.find(id_token)),
        ];
        let found: Map<String, Value> = stores
            .into_iter()
            .filter_map(|(store, entry)| Some((store.to_owned(), json!(entry?.info))))
            .collect();
        match found.is_empty() {
            true => String::new(),
            false => Value::Object(found).to_string(),
        }
    }

    fn notify_customer_information(&mut self, now: u64, request_id: i32, data: &str) {
        let chars: Vec<char> = data.chars().collect();
        let mut parts: Vec<String> = chars
            .chunks(PART_CHARS)
            .map(|part| part.iter().collect())
            .collect();
        if parts.is_empty() {
            parts.push(String::new());
        }
        let count = parts.len();
        for (seq_no, part) in parts.into_iter().enumerate() {
            let data = CiString::new(&part)
                .unwrap_or_else(|_| unreachable!("a part holds at most 512 characters"));
            let request = NotifyCustomerInformationRequest {
                data,
                tbc: seq_no + 1 < count,
                seq_no: seq_no as u32,
                generated_at: DateTime { unix_ms: now },
                request_id,
            };
            self.outbox
                .push("NotifyCustomerInformation", 0, json!(request));
        }
    }
}

fn response(status: CustomerInformationStatus) -> CustomerInformationResponse {
    CustomerInformationResponse {
        status,
        status_info: None,
    }
}

#[cfg(test)]
#[path = "customer_information.test.rs"]
mod tests;
