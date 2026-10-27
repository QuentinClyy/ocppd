use super::*;

fn names(text: &str) -> Option<(String, Option<String>, String, Option<String>)> {
    let (component, variable) = parse(text)?;
    let text = |value: Option<CiString<50>>| value.map(String::from);
    Some((
        component.name.into(),
        text(component.instance),
        variable.name.into(),
        text(variable.instance),
    ))
}

#[test]
fn given_variable_instance_shortcut_when_parsed_then_component_variable_and_instance() {
    let shortcut = "TariffCostCtrlr.Available[Tariff]";

    let parsed = names(shortcut);

    assert_eq!(
        parsed,
        Some((
            "TariffCostCtrlr".into(),
            None,
            "Available".into(),
            Some("Tariff".into())
        ))
    );
}

#[test]
fn given_component_instance_shortcut_when_parsed_then_component_instance_and_variable() {
    let shortcut = "A[x].B";

    let parsed = names(shortcut);

    assert_eq!(
        parsed,
        Some(("A".into(), Some("x".into()), "B".into(), None))
    );
}

#[test]
fn given_malformed_shortcuts_when_parsed_then_none() {
    for invalid in ["NoDot", ".B", "A.", "A[x.B", "A[].B"] {
        let parsed = names(invalid);

        assert_eq!(parsed, None, "{invalid}");
    }
}

#[test]
fn given_instances_on_both_sides_when_named_then_the_name_parses_back_to_them() {
    let shortcut = "EVSE[1].Power[Max]";
    let (component, variable) = parse(shortcut).unwrap();

    let named = name(&component, &variable);

    assert_eq!(named, shortcut);
}
