use super::row::{Row, row};
use super::table::OCPP_COMM_CTRLR;
use crate::protocol::v201::charging_station::provisioning::back_off::{
    RETRY_BACK_OFF_RANDOM_RANGE, RETRY_BACK_OFF_REPEAT_TIMES, RETRY_BACK_OFF_WAIT_MINIMUM,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

pub fn rows() -> Vec<Row<'static>> {
    use DataType::Integer;
    use Mutability::ReadWrite;
    vec![
        row(
            OCPP_COMM_CTRLR,
            RETRY_BACK_OFF_WAIT_MINIMUM,
            "5",
            ReadWrite,
            Integer,
        )
        .unit("s")
        .min(0.0),
        row(
            OCPP_COMM_CTRLR,
            RETRY_BACK_OFF_REPEAT_TIMES,
            "6",
            ReadWrite,
            Integer,
        )
        .min(0.0),
        row(
            OCPP_COMM_CTRLR,
            RETRY_BACK_OFF_RANDOM_RANGE,
            "5",
            ReadWrite,
            Integer,
        )
        .unit("s")
        .min(0.0),
    ]
}
