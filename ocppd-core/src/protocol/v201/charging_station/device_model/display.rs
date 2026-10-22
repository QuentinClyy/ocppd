use super::row::{Row, row};
use crate::protocol::v201::charging_station::display::{
    DISPLAY_MESSAGE_CTRLR, DISPLAY_MESSAGES, MAX_DISPLAY_MESSAGES, SUPPORTED_FORMATS,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

const PRIORITIES: &str = "AlwaysFront,InFront,NormalCycle";
const STATES: &str = "Charging,Faulted,Idle,Unavailable";

pub fn rows() -> Vec<Row<'static>> {
    use DataType::{Boolean, Integer, MemberList};
    use Mutability::ReadOnly;
    vec![
        row(
            DISPLAY_MESSAGE_CTRLR,
            "Available",
            "true",
            ReadOnly,
            Boolean,
        ),
        row(DISPLAY_MESSAGE_CTRLR, "Enabled", "true", ReadOnly, Boolean),
        row(
            DISPLAY_MESSAGE_CTRLR,
            DISPLAY_MESSAGES,
            "0",
            ReadOnly,
            Integer,
        )
        .max(MAX_DISPLAY_MESSAGES as f64),
        row(
            DISPLAY_MESSAGE_CTRLR,
            SUPPORTED_FORMATS,
            "ASCII,UTF8",
            ReadOnly,
            MemberList,
        )
        .values("ASCII,HTML,URI,UTF8"),
        row(
            DISPLAY_MESSAGE_CTRLR,
            "SupportedPriorities",
            PRIORITIES,
            ReadOnly,
            MemberList,
        )
        .values(PRIORITIES),
        row(
            DISPLAY_MESSAGE_CTRLR,
            "SupportedStates",
            STATES,
            ReadOnly,
            MemberList,
        )
        .values(STATES),
    ]
}
