use super::Entry;
use crate::protocol::v201::messages::datatypes::component::{Component, Evse, Variable};
use crate::protocol::v201::messages::provisioning::report::{
    DataType, Mutability, VariableCharacteristics,
};
use crate::wire::ci_string::{CiString, TooLong};

pub struct Row<'a> {
    component: &'a str,
    evse: Option<Evse>,
    variable: &'a str,
    instance: Option<&'a str>,
    value: String,
    mutability: Mutability,
    data_type: DataType,
    unit: Option<&'a str>,
    min: Option<f64>,
    max: Option<f64>,
    values: Option<&'a str>,
}

pub fn row<'a>(
    component: &'a str,
    variable: &'a str,
    value: &str,
    mutability: Mutability,
    data_type: DataType,
) -> Row<'a> {
    Row {
        component,
        evse: None,
        variable,
        instance: None,
        value: value.to_owned(),
        mutability,
        data_type,
        unit: None,
        min: None,
        max: None,
        values: None,
    }
}

impl<'a> Row<'a> {
    pub fn instance(mut self, instance: &'a str) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn evse(mut self, id: u32, connector_id: Option<u32>) -> Self {
        self.evse = Some(Evse { id, connector_id });
        self
    }

    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn values(mut self, values: &'a str) -> Self {
        self.values = Some(values);
        self
    }

    pub fn entry(self) -> Result<Entry, TooLong> {
        Ok(Entry {
            component: Component {
                evse: self.evse,
                name: CiString::new(self.component)?,
                instance: None,
            },
            variable: Variable {
                name: CiString::new(self.variable)?,
                instance: text(self.instance)?,
            },
            value: self.value,
            mutability: self.mutability,
            characteristics: VariableCharacteristics {
                unit: text(self.unit)?,
                data_type: self.data_type,
                min_limit: self.min,
                max_limit: self.max,
                values_list: text(self.values)?,
                supports_monitoring: true,
            },
        })
    }
}

fn text<const N: usize>(value: Option<&str>) -> Result<Option<CiString<N>>, TooLong> {
    value.map(CiString::new).transpose()
}
