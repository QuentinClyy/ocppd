use crate::protocol::v201::charging_station::ChargingStation;
use crate::protocol::v201::charging_station::device_model::table::OCPP_COMM_CTRLR;
use crate::station::jitter::fraction;

pub const RETRY_BACK_OFF_WAIT_MINIMUM: &str = "RetryBackOffWaitMinimum";
pub const RETRY_BACK_OFF_REPEAT_TIMES: &str = "RetryBackOffRepeatTimes";
pub const RETRY_BACK_OFF_RANDOM_RANGE: &str = "RetryBackOffRandomRange";

impl ChargingStation {
    pub fn reconnect_delay_ms(&self, failures: u32) -> u64 {
        let setting = |name| self.device_model.number(OCPP_COMM_CTRLR, name).unwrap_or(0);
        let doublings =
            u64::from(failures.saturating_sub(1)).min(setting(RETRY_BACK_OFF_REPEAT_TIMES));
        let wait_ms = setting(RETRY_BACK_OFF_WAIT_MINIMUM) * 1000;
        let backed_off = wait_ms.saturating_mul(2_u64.saturating_pow(doublings as u32));
        let random = match self.station.jitter {
            true => fraction(&self.station.id, u64::from(failures)),
            false => 0.0,
        };
        let random_ms = (random * (setting(RETRY_BACK_OFF_RANDOM_RANGE) * 1000) as f64) as u64;
        backed_off.saturating_add(random_ms)
    }
}

#[cfg(test)]
#[path = "back_off.test.rs"]
mod tests;
