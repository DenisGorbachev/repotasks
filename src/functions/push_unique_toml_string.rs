use toml_edit::{Array, Value};

pub fn push_unique_toml_string(array: &mut Array, string: &str) -> bool {
    if array.iter().any(|value| value.as_str() == Some(string)) {
        return false;
    }
    let value = match array.iter_mut().last() {
        Some(last) => {
            let decor = last.decor_mut();
            let prefix = decor
                .prefix()
                .filter(|prefix| prefix.as_str() != Some(""))
                .cloned()
                .unwrap_or_else(|| " ".into());
            let suffix = decor.suffix().cloned().unwrap_or_default();
            decor.set_suffix("");
            Value::from(string).decorated(prefix, suffix)
        }
        None => Value::from(string),
    };
    array.push_formatted(value);
    true
}
