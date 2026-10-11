use super::*;
#[test]
fn numeric_edits_validate_bounds_and_nonfinite_values() {
    let integer = SettingDescriptor::new(
        "spins",
        "Spins",
        SettingKind::Integer { min: 1, max: 100 },
        json!(10),
    );
    assert_eq!(parse_edit(&integer, "100").unwrap(), json!(100));
    for value in ["0", "101", "1.5", ""] {
        assert!(parse_edit(&integer, value).is_err());
    }
    let number = SettingDescriptor::new(
        "radius",
        "Radius",
        SettingKind::Number {
            min: 0.0,
            max: 10.0,
        },
        json!(2.0),
    );
    assert_eq!(parse_edit(&number, "1.25").unwrap(), json!(1.25));
    for value in ["NaN", "inf", "-1", "11"] {
        assert!(parse_edit(&number, value).is_err());
    }
}
#[test]
fn dictionary_and_json_edits_do_not_accept_partial_or_wrong_types() {
    let map = SettingDescriptor::new(
        "targets",
        "Targets",
        SettingKind::BooleanMap { discover: None },
        json!({}),
    );
    assert_eq!(
        parse_edit(&map, r#"{"boss":false}"#).unwrap(),
        json!({"boss": false})
    );
    for value in [r#"{"boss":1}"#, "[]", "{", "null"] {
        assert!(parse_edit(&map, value).is_err());
    }
    let text = SettingDescriptor::new("event", "Event", SettingKind::Text, json!(""));
    assert_eq!(
        parse_edit(&text, "winter event").unwrap(),
        json!("winter event")
    );
}
