use super::row::{Row, row};
use super::table::BYTES_PER_MESSAGE;
use crate::protocol::v201::charging_station::diagnostics::monitoring::{
    ACTIVE_MONITORING_LEVEL, HIGHEST_SEVERITY, ITEMS_PER_MONITORING, MONITORING_CTRLR,
    OFFLINE_QUEUING_SEVERITY,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

pub fn rows() -> Vec<Row<'static>> {
    use DataType::{Boolean, Integer};
    use Mutability::{ReadOnly, ReadWrite};
    let items = ITEMS_PER_MONITORING.to_string();
    let highest = HIGHEST_SEVERITY.to_string();
    vec![
        row(MONITORING_CTRLR, "Available", "true", ReadOnly, Boolean),
        row(MONITORING_CTRLR, "Enabled", "true", ReadOnly, Boolean),
        row(
            MONITORING_CTRLR,
            "ItemsPerMessage",
            &items,
            ReadOnly,
            Integer,
        )
        .instance("SetVariableMonitoring"),
        row(
            MONITORING_CTRLR,
            "ItemsPerMessage",
            &items,
            ReadOnly,
            Integer,
        )
        .instance("ClearVariableMonitoring"),
        row(
            MONITORING_CTRLR,
            "BytesPerMessage",
            BYTES_PER_MESSAGE,
            ReadOnly,
            Integer,
        )
        .instance("SetVariableMonitoring"),
        row(
            MONITORING_CTRLR,
            OFFLINE_QUEUING_SEVERITY,
            "4",
            ReadWrite,
            Integer,
        )
        .min(0.0)
        .max(9.0),
        row(
            MONITORING_CTRLR,
            ACTIVE_MONITORING_LEVEL,
            &highest,
            ReadOnly,
            Integer,
        )
        .min(0.0)
        .max(9.0),
    ]
}
