mod authorization;
mod display;
mod monitoring;
mod physical;
mod reservation;
mod row;
mod security;
pub(super) mod shortcut;
mod smart_charging;
pub(super) mod table;

use crate::protocol::v201::messages::datatypes::component::{Component, Evse, Variable};
use crate::protocol::v201::messages::provisioning::report::{
    DataType, Mutability, VariableCharacteristics,
};
use crate::protocol::v201::messages::provisioning::variables::{
    GetVariableStatus, SetVariableStatus,
};
use crate::wire::ci_string::CiString;
use crate::wire::date_time::DateTime;

pub struct DeviceModel {
    entries: Vec<Entry>,
    changed: bool,
}

pub struct Entry {
    pub component: Component,
    pub variable: Variable,
    pub value: String,
    pub mutability: Mutability,
    pub characteristics: VariableCharacteristics,
}

enum Found<'a> {
    Entry(&'a mut Entry),
    UnknownComponent,
    UnknownVariable,
}

pub enum Missing {
    Component,
    Variable,
}

impl DeviceModel {
    pub fn new(entries: Vec<Entry>) -> DeviceModel {
        DeviceModel {
            entries,
            changed: false,
        }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn get(
        &mut self,
        component: &Component,
        variable: &Variable,
    ) -> (GetVariableStatus, Option<String>) {
        match self.find(component, variable) {
            Found::Entry(entry) if entry.mutability == Mutability::WriteOnly => {
                (GetVariableStatus::Rejected, None)
            }
            Found::Entry(entry) => (GetVariableStatus::Accepted, Some(entry.value.clone())),
            Found::UnknownComponent => (GetVariableStatus::UnknownComponent, None),
            Found::UnknownVariable => (GetVariableStatus::UnknownVariable, None),
        }
    }

    pub fn provision(&mut self, component: &Component, variable: &Variable, value: &str) -> bool {
        let Found::Entry(entry) = self.find(component, variable) else {
            return false;
        };
        if !fits(&entry.characteristics, value) {
            return false;
        }
        let changed = entry.value != value;
        entry.value = value.to_owned();
        self.changed |= changed;
        true
    }

    pub fn set(
        &mut self,
        component: &Component,
        variable: &Variable,
        value: &str,
    ) -> SetVariableStatus {
        let entry = match self.find(component, variable) {
            Found::Entry(entry) => entry,
            Found::UnknownComponent => return SetVariableStatus::UnknownComponent,
            Found::UnknownVariable => return SetVariableStatus::UnknownVariable,
        };
        if entry.mutability == Mutability::ReadOnly || !fits(&entry.characteristics, value) {
            return SetVariableStatus::Rejected;
        }
        let changed = entry.value != value;
        entry.value = value.to_owned();
        self.changed |= changed;
        SetVariableStatus::Accepted
    }

    pub fn value(&self, component: &str, variable: &str) -> Option<&str> {
        self.instance_value(component, variable, None)
    }

    pub fn instance_value(
        &self,
        component: &str,
        variable: &str,
        instance: Option<&str>,
    ) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| {
                entry.component.evse.is_none()
                    && entry.component.name.as_str() == component
                    && entry.variable.name.as_str() == variable
                    && entry.variable.instance.as_ref().map(CiString::as_str) == instance
            })
            .map(|entry| entry.value.as_str())
    }

    pub fn set_value(
        &mut self,
        component: &str,
        evse: Option<Evse>,
        variable: &str,
        value: String,
    ) {
        self.assign(value, |entry| {
            entry.component.evse == evse
                && entry.component.name.as_str() == component
                && entry.variable.name.as_str() == variable
                && entry.variable.instance.is_none()
        });
    }

    pub fn set_instance_value(
        &mut self,
        component: &str,
        variable: &str,
        instance: &str,
        value: String,
    ) {
        self.assign(value, |entry| {
            entry.component.evse.is_none()
                && entry.component.name.as_str() == component
                && entry.variable.name.as_str() == variable
                && entry.variable.instance.as_ref().map(CiString::as_str) == Some(instance)
        });
    }

    fn assign(&mut self, value: String, matches: impl Fn(&Entry) -> bool) {
        let found = self.entries.iter_mut().find(|entry| matches(entry));
        if let Some(entry) = found
            && entry.value != value
        {
            entry.value = value;
            self.changed = true;
        }
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    pub fn number(&self, component: &str, variable: &str) -> Option<u64> {
        self.value(component, variable)?.parse().ok()
    }

    pub fn entry(&self, component: &Component, variable: &Variable) -> Result<&Entry, Missing> {
        let index = self.locate(component, variable)?;
        Ok(&self.entries[index])
    }

    fn find(&mut self, component: &Component, variable: &Variable) -> Found<'_> {
        match self.locate(component, variable) {
            Ok(index) => Found::Entry(&mut self.entries[index]),
            Err(Missing::Component) => Found::UnknownComponent,
            Err(Missing::Variable) => Found::UnknownVariable,
        }
    }

    fn locate(&self, component: &Component, variable: &Variable) -> Result<usize, Missing> {
        let same_component = |entry: &Entry| {
            entry.component.name.matches(&component.name)
                && same_instance(&entry.component.instance, &component.instance)
                && entry.component.evse == component.evse
        };
        if !self.entries.iter().any(same_component) {
            return Err(Missing::Component);
        }
        self.entries
            .iter()
            .position(|entry| {
                same_component(entry)
                    && entry.variable.name.matches(&variable.name)
                    && same_instance(&entry.variable.instance, &variable.instance)
            })
            .ok_or(Missing::Variable)
    }
}

fn same_instance<const N: usize>(left: &Option<CiString<N>>, right: &Option<CiString<N>>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.matches(right),
        (None, None) => true,
        _ => false,
    }
}

fn fits(characteristics: &VariableCharacteristics, value: &str) -> bool {
    let in_range = |number: f64| {
        characteristics.min_limit.is_none_or(|min| number >= min)
            && characteristics.max_limit.is_none_or(|max| number <= max)
    };
    let allowed = |item: &str| {
        characteristics
            .values_list
            .as_ref()
            .is_some_and(|list| list.as_str().split(',').any(|allowed| allowed == item))
    };
    match characteristics.data_type {
        DataType::Integer => value
            .parse::<i64>()
            .is_ok_and(|number| in_range(number as f64)),
        DataType::Decimal => value.parse::<f64>().is_ok_and(in_range),
        DataType::Boolean => value == "true" || value == "false",
        DataType::DateTime => value.parse::<DateTime>().is_ok(),
        DataType::String => in_range(value.chars().count() as f64),
        DataType::OptionList => allowed(value),
        DataType::MemberList | DataType::SequenceList => {
            value.is_empty() || value.split(',').all(allowed)
        }
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod tests;
