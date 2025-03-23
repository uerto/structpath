use crate::types::{Segment, SegmentKey, Structpath, StructpathError};
use serde_json::Value;
use std::collections::HashMap;

pub fn get<'a>(
    path: &Structpath,
    data: &'a Value,
    vars: Option<&HashMap<String, String>>,
) -> Result<&'a Value, StructpathError> {
    // Check if path contains variables
    let has_variables = path.segments().iter().any(|segment| {
        matches!(segment, Segment::KeyVariable(_) | Segment::IndexVariable(_))
    });

    // If path has variables but no vars provided, that's an error
    if has_variables && vars.is_none() {
        return Err(StructpathError::ParseError(
            "Path contains variables, but no variable context was provided."
                .to_string(),
        ));
    }

    let mut current = data;

    for segment in path.segments() {
        match segment {
            Segment::Key(key) => {
                current = get_by_key(current, key)?;
            }
            Segment::Index(idx) => {
                current = get_by_index(current, *idx)?;
            }
            Segment::KeyVariable(var_name) => {
                // Safe to unwrap here because we already checked that vars is Some if path has variables
                let variables = vars.unwrap();

                // Resolve the variable value from the provided context
                let var_value = variables.get(var_name).ok_or_else(|| {
                    StructpathError::MissingVariable(var_name.clone())
                })?;

                // Use it as a string key - this is a key variable
                current = get_by_string_key(current, var_value)?;
            }
            Segment::IndexVariable(var_name) => {
                // Safe to unwrap here because we already checked that vars is Some if path has variables
                let variables = vars.unwrap();

                // Resolve the variable value from the provided context
                let var_value = variables.get(var_name).ok_or_else(|| {
                    StructpathError::MissingVariable(var_name.clone())
                })?;

                // Parse as index - this is an index variable
                let idx = var_value.parse::<usize>().map_err(|_| {
                    StructpathError::InvalidVariableValue(var_value.clone())
                })?;

                current = get_by_index(current, idx)?;
            }
        }
    }

    Ok(current)
}

fn get_by_key<'a>(
    data: &'a Value,
    key: &SegmentKey,
) -> Result<&'a Value, StructpathError> {
    if let Value::Object(map) = data {
        let lookup_key = match key {
            SegmentKey::String(s) => s.clone(),
            SegmentKey::Int(i) => i.to_string(),
        };

        if let Some(value) = map.get(&lookup_key) {
            Ok(value)
        } else {
            Err(StructpathError::NotFound)
        }
    } else {
        Err(StructpathError::InvalidPath {
            expected: "object".to_string(),
            found: format!("{:?}", data),
        })
    }
}

fn get_by_string_key<'a>(
    data: &'a Value,
    key: &str,
) -> Result<&'a Value, StructpathError> {
    if let Value::Object(map) = data {
        if let Some(value) = map.get(key) {
            Ok(value)
        } else {
            Err(StructpathError::NotFound)
        }
    } else {
        Err(StructpathError::InvalidPath {
            expected: "object".to_string(),
            found: format!("{:?}", data),
        })
    }
}

fn get_by_index(data: &Value, idx: usize) -> Result<&Value, StructpathError> {
    if let Value::Array(arr) = data {
        if let Some(value) = arr.get(idx) {
            Ok(value)
        } else {
            Err(StructpathError::IndexOutOfBounds(format!(
                "Index {} out of bounds for array of length {}",
                idx,
                arr.len()
            )))
        }
    } else {
        Err(StructpathError::InvalidPath {
            expected: "array".to_string(),
            found: format!("{:?}", data),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use serde_json::json;

    #[test]
    fn test_get_value() {
        let data = json!({
            "a": [
                {"b": {"c": 42}},
                {"d": "hello"}
            ],
            "e.f": "escaped key",
            "123": "integer key"
        });

        let path = parse("$a[0].b.c").unwrap();
        let value = get(&path, &data, None).unwrap();
        assert_eq!(*value, json!(42));

        let path = parse("$a[1].d").unwrap();
        let value = get(&path, &data, None).unwrap();
        assert_eq!(*value, json!("hello"));

        let path = parse(r"$e\.f").unwrap();
        let value = get(&path, &data, None).unwrap();
        assert_eq!(*value, json!("escaped key"));

        let path = parse("$123").unwrap();
        let value = get(&path, &data, None).unwrap();
        assert_eq!(*value, json!("integer key"));
    }

    #[test]
    fn test_get_not_found() {
        let data = json!({"a": {"b": 1}});
        let path = parse("$a.c").unwrap();
        let result = get(&path, &data, None);
        assert!(matches!(result, Err(StructpathError::NotFound)));
    }

    #[test]
    fn test_get_invalid_type() {
        let data = json!({"a": 1});
        let path = parse("$a.b").unwrap();
        let result = get(&path, &data, None);
        assert!(matches!(result, Err(StructpathError::InvalidPath { .. })));
    }

    #[test]
    fn test_get_index_out_of_bounds() {
        let data = json!({"a": [1, 2]});
        let path = parse("$a[5]").unwrap();
        let result = get(&path, &data, None);
        assert!(matches!(result, Err(StructpathError::IndexOutOfBounds(_))));
    }
}
