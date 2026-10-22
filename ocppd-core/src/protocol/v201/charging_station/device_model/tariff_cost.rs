use super::row::{Row, row};
use crate::protocol::v201::charging_station::tariff_cost::{
    TARIFF_COST_CTRLR, TARIFF_FALLBACK_MESSAGE, TOTAL_COST_FALLBACK_MESSAGE,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

pub fn rows() -> Vec<Row<'static>> {
    use DataType::Boolean;
    use Mutability::{ReadOnly, ReadWrite};
    let text = DataType::String;
    vec![
        row(TARIFF_COST_CTRLR, "Available", "true", ReadOnly, Boolean).instance("Cost"),
        row(TARIFF_COST_CTRLR, "Available", "true", ReadOnly, Boolean).instance("Tariff"),
        row(TARIFF_COST_CTRLR, "Enabled", "true", ReadOnly, Boolean).instance("Cost"),
        row(TARIFF_COST_CTRLR, "Enabled", "true", ReadOnly, Boolean).instance("Tariff"),
        row(
            TARIFF_COST_CTRLR,
            TARIFF_FALLBACK_MESSAGE,
            "",
            ReadWrite,
            text,
        )
        .max(255.0),
        row(
            TARIFF_COST_CTRLR,
            TOTAL_COST_FALLBACK_MESSAGE,
            "",
            ReadWrite,
            text,
        )
        .max(255.0),
        row(TARIFF_COST_CTRLR, "Currency", "EUR", ReadWrite, text)
            .min(3.0)
            .max(3.0),
    ]
}
