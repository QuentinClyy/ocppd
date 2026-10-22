use super::row::{Row, row};
use crate::protocol::v201::charging_station::smart_charging::{
    ENTRIES, ENTRIES_INSTANCE, MAX_PROFILES, PERIODS_PER_SCHEDULE, PROFILE_STACK_LEVEL,
    SMART_CHARGING_CTRLR,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

const UNITS: &str = "A,W";

pub fn rows() -> Vec<Row<'static>> {
    use DataType::{Boolean, Decimal, Integer, MemberList};
    use Mutability::{ReadOnly, ReadWrite};
    vec![
        row(SMART_CHARGING_CTRLR, "Enabled", "true", ReadOnly, Boolean),
        row(SMART_CHARGING_CTRLR, ENTRIES, "0", ReadOnly, Integer)
            .instance(ENTRIES_INSTANCE)
            .max(MAX_PROFILES as f64),
        row(
            SMART_CHARGING_CTRLR,
            "LimitChangeSignificance",
            "0",
            ReadWrite,
            Decimal,
        )
        .unit("Percent")
        .min(0.0),
        row(
            SMART_CHARGING_CTRLR,
            PROFILE_STACK_LEVEL,
            "10",
            ReadOnly,
            Integer,
        ),
        row(
            SMART_CHARGING_CTRLR,
            PERIODS_PER_SCHEDULE,
            "24",
            ReadOnly,
            Integer,
        ),
        row(
            SMART_CHARGING_CTRLR,
            "RateUnit",
            UNITS,
            ReadOnly,
            MemberList,
        )
        .values(UNITS),
    ]
}
