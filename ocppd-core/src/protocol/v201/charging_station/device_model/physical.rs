use super::row::{Row, row};
use super::table::{AVAILABILITY_STATE, CONNECTOR_STATUSES, POWER};
use crate::protocol::v201::charging_station::diagnostics::monitoring::{LOCK, PROBLEM};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};
use crate::station::Station;

pub fn rows(station: &Station) -> Vec<Row<'_>> {
    use DataType::{Boolean, Decimal, Integer, OptionList};
    use Mutability::ReadOnly;
    let phases = station.phases.to_string();
    let mut rows = vec![
        row("ChargingStation", "Available", "true", ReadOnly, Boolean),
        row(
            "ChargingStation",
            AVAILABILITY_STATE,
            "Available",
            ReadOnly,
            OptionList,
        )
        .values(CONNECTOR_STATUSES),
        row(
            "ChargingStation",
            "SupplyPhases",
            &phases,
            ReadOnly,
            Integer,
        ),
    ];
    for id in 1..=station.connectors.len() as u32 {
        rows.extend([
            row("EVSE", "Available", "true", ReadOnly, Boolean).evse(id, None),
            row(
                "EVSE",
                AVAILABILITY_STATE,
                "Available",
                ReadOnly,
                OptionList,
            )
            .evse(id, None)
            .values(CONNECTOR_STATUSES),
            row("EVSE", POWER, "0", ReadOnly, Decimal)
                .evse(id, None)
                .unit("W")
                .max(station.limit_w()),
            row("EVSE", "SupplyPhases", &phases, ReadOnly, Integer).evse(id, None),
            row("Connector", "Available", "true", ReadOnly, Boolean).evse(id, Some(1)),
            row(
                "Connector",
                AVAILABILITY_STATE,
                "Available",
                ReadOnly,
                OptionList,
            )
            .evse(id, Some(1))
            .values(CONNECTOR_STATUSES),
            row(
                "Connector",
                "ConnectorType",
                "cType2",
                ReadOnly,
                DataType::String,
            )
            .evse(id, Some(1)),
            row("Connector", "SupplyPhases", &phases, ReadOnly, Integer).evse(id, Some(1)),
            row(LOCK, PROBLEM, "false", ReadOnly, Boolean).evse(id, Some(1)),
        ]);
    }
    rows
}
