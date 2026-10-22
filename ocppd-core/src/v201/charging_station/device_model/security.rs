use super::row::{Row, row};
use crate::v201::charging_station::security::{
    BASIC_AUTH_PASSWORD, SECURITY_CTRLR, SECURITY_PROFILE,
};
use crate::v201::messages::provisioning::report::{DataType, Mutability};

const PASSWORD_LENGTH: (f64, f64) = (16.0, 40.0);

pub fn rows() -> Vec<Row<'static>> {
    use DataType::Integer;
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
        row(SECURITY_CTRLR, "CertificateEntries", "0", ReadOnly, Integer),
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
