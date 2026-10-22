use super::row::{Row, row};
use crate::protocol::v201::charging_station::security::certificates::{
    ADDITIONAL_ROOT_CERTIFICATE_CHECK, CAPACITY,
};
use crate::protocol::v201::charging_station::security::{
    BASIC_AUTH_PASSWORD, CERTIFICATE_ENTRIES, SECURITY_CTRLR, SECURITY_PROFILE,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

const PASSWORD_LENGTH: (f64, f64) = (16.0, 40.0);

pub fn rows() -> Vec<Row<'static>> {
    use DataType::{Boolean, Integer};
    use Mutability::{ReadOnly, ReadWrite, WriteOnly};
    let (shortest, longest) = PASSWORD_LENGTH;
    vec![
        row(
            SECURITY_CTRLR,
            "OrganizationName",
            "",
            ReadWrite,
            DataType::String,
        ),
        row(SECURITY_CTRLR, CERTIFICATE_ENTRIES, "0", ReadOnly, Integer).max(CAPACITY as f64),
        row(
            SECURITY_CTRLR,
            ADDITIONAL_ROOT_CERTIFICATE_CHECK,
            "false",
            ReadOnly,
            Boolean,
        ),
        row(SECURITY_CTRLR, SECURITY_PROFILE, "0", ReadOnly, Integer)
            .min(0.0)
            .max(3.0),
        row(
            SECURITY_CTRLR,
            BASIC_AUTH_PASSWORD,
            "",
            WriteOnly,
            DataType::String,
        )
        .min(shortest)
        .max(longest),
    ]
}
