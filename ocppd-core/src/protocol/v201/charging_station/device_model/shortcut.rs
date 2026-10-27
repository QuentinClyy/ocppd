use crate::protocol::v201::messages::datatypes::component::{Component, Variable};
use crate::wire::ci_string::CiString;

pub fn parse(text: &str) -> Option<(Component, Variable)> {
    let (component, variable) = text.split_once('.')?;
    let (component_name, component_instance) = named(component)?;
    let (variable_name, variable_instance) = named(variable)?;
    let component = Component {
        evse: None,
        name: CiString::new(component_name).ok()?,
        instance: component_instance.map(CiString::new).transpose().ok()?,
    };
    let variable = Variable {
        name: CiString::new(variable_name).ok()?,
        instance: variable_instance.map(CiString::new).transpose().ok()?,
    };
    Some((component, variable))
}

pub fn name(component: &Component, variable: &Variable) -> String {
    let named = |name: &str, instance: Option<&CiString<50>>| match instance {
        Some(instance) => format!("{name}[{}]", instance.as_str()),
        None => name.to_owned(),
    };
    format!(
        "{}.{}",
        named(component.name.as_str(), component.instance.as_ref()),
        named(variable.name.as_str(), variable.instance.as_ref())
    )
}

fn named(text: &str) -> Option<(&str, Option<&str>)> {
    let (name, instance) = match text.split_once('[') {
        Some((name, rest)) => (name, Some(rest.strip_suffix(']')?)),
        None => (text, None),
    };
    let valid = !name.is_empty() && instance.is_none_or(|instance| !instance.is_empty());
    valid.then_some((name, instance))
}

#[cfg(test)]
#[path = "shortcut.test.rs"]
mod tests;
