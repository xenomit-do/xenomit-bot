use super::*;

pub(super) fn parse_edit(field: &SettingDescriptor, text: &str) -> Result<Value, String> {
    let value = match field.kind {
        SettingKind::Text => json!(text),
        SettingKind::Integer { .. } => json!(
            text.parse::<i64>()
                .map_err(|_| "Enter a whole number".to_owned())?
        ),
        SettingKind::Number { .. } => json!(
            text.parse::<f64>()
                .map_err(|_| "Enter a number".to_owned())?
        ),
        _ => serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?,
    };
    field.validate(&value)?;
    Ok(value)
}
