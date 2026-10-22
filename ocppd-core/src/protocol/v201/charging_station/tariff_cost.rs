use super::ChargingStation;
use crate::api::io::Output;
use crate::protocol::v201::event::EventKind;
use crate::protocol::v201::messages::datatypes::id_token::MessageContent;
use crate::protocol::v201::messages::tariff_cost::{CostUpdatedRequest, CostUpdatedResponse};

pub const TARIFF_COST_CTRLR: &str = "TariffCostCtrlr";
pub const TARIFF_FALLBACK_MESSAGE: &str = "TariffFallbackMessage";
pub const TOTAL_COST_FALLBACK_MESSAGE: &str = "TotalCostFallbackMessage";

impl ChargingStation {
    pub(super) fn cost_updated(
        &mut self,
        out: &mut Vec<Output>,
        now: u64,
        request: CostUpdatedRequest,
    ) -> CostUpdatedResponse {
        let running = self.sessions.iter().position(|session| {
            let transaction = session.transaction.as_ref();
            transaction.is_some_and(|transaction| transaction.id.matches(&request.transaction_id))
        });
        if let Some(index) = running {
            self.show_cost(out, now, index, request.total_cost, false);
        }
        CostUpdatedResponse {}
    }

    pub(super) fn show_cost(
        &self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        total_cost: f64,
        final_cost: bool,
    ) {
        let kind = EventKind::CostShown {
            evse: index as u32 + 1,
            total_cost,
            final_cost,
        };
        self.emit(out, now, kind);
    }

    pub(super) fn show_tariff(
        &self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        personal: Option<&MessageContent>,
    ) {
        match personal {
            Some(message) => self.show_driver(out, now, index, message.content.as_str()),
            None => self.show_fallback(out, now, index, TARIFF_FALLBACK_MESSAGE),
        }
    }

    pub(super) fn show_fallback(
        &self,
        out: &mut Vec<Output>,
        now: u64,
        index: usize,
        variable: &str,
    ) {
        let text = self.device_model.value(TARIFF_COST_CTRLR, variable);
        if let Some(text) = text.filter(|text| !text.is_empty()) {
            self.show_driver(out, now, index, text);
        }
    }

    pub(super) fn show_driver(&self, out: &mut Vec<Output>, now: u64, index: usize, text: &str) {
        let kind = EventKind::DriverMessage {
            evse: index as u32 + 1,
            text: text.to_owned(),
        };
        self.emit(out, now, kind);
    }
}

#[cfg(test)]
#[path = "tariff_cost.test.rs"]
mod tests;
