use super::row::{Row, row};
use crate::protocol::v201::charging_station::reservation::{NON_EVSE_SPECIFIC, RESERVATION_CTRLR};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

pub fn rows() -> Vec<Row<'static>> {
    use DataType::Boolean;
    use Mutability::{ReadOnly, ReadWrite};
    vec![
        row(RESERVATION_CTRLR, "Available", "true", ReadOnly, Boolean),
        row(RESERVATION_CTRLR, "Enabled", "true", ReadWrite, Boolean),
        row(
            RESERVATION_CTRLR,
            NON_EVSE_SPECIFIC,
            "true",
            ReadWrite,
            Boolean,
        ),
    ]
}
