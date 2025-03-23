use crate::types::{Segment, SegmentKey, Structpath, StructpathError};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub fn write(
    path: &Structpath,
    data: Option<&mut Value>,
    value: Value,
    vars: Option<&HashMap<String, String>>,
) -> Result<Value, StructpathError> {
    let mut root_value = match &data {
        Some(d) => (*d).clone(),
        None => Value::Null,
    };
    let mut_ref = &mut root_value;

    let has_variables = path.segments().iter().any(|segment| {
        matches!(segment, Segment::KeyVariable(_) | Segment::IndexVariable(_))
    });

    if has_variables && vars.is_none() {
        return Err(StructpathError::ParseError(
            "Path contains variables, but no variable context was provided."
                .to_string(),
        ));
    }

    if path.segments().is_empty() {
        *mut_ref = value;
        return Ok(root_value);
    }

    let segments = path.segments().to_vec();
    let segments_len = segments.len();

    let mut current = mut_ref;
    for (i, segment) in segments.iter().enumerate() {
        if i == segments_len - 1 {
            match segment {
                Segment::Key(key) => {
                    write_by_key(current, key, value)?;
                }
                Segment::Index(idx) => {
                    write_by_index(current, *idx, value)?;
                }
                Segment::KeyVariable(var_name) => {
                    let variables = vars.unwrap();
                    let var_value =
                        variables.get(var_name).ok_or_else(|| {
                            StructpathError::MissingVariable(var_name.clone())
                        })?;

                    write_by_key(
                        current,
                        &SegmentKey::String(var_value.clone()),
                        value,
                    )?;
                }
                Segment::IndexVariable(var_name) => {
                    let variables = vars.unwrap();
                    let var_value =
                        variables.get(var_name).ok_or_else(|| {
                            StructpathError::MissingVariable(var_name.clone())
                        })?;

                    let idx = var_value.parse::<usize>().map_err(|_| {
                        StructpathError::InvalidVariableValue(var_value.clone())
                    })?;

                    write_by_index(current, idx, value)?;
                }
            }
            break;
        }

        match segment {
            Segment::Key(key) => {
                current =
                    ensure_next_segment_exists(current, key, &segments[i + 1])?;
            }
            Segment::Index(idx) => {
                current =
                    ensure_array_index_exists(current, *idx, &segments[i + 1])?;
            }
            Segment::KeyVariable(var_name) => {
                let variables = vars.unwrap();
                let var_value = variables.get(var_name).ok_or_else(|| {
                    StructpathError::MissingVariable(var_name.clone())
                })?;

                current = ensure_next_segment_exists(
                    current,
                    &SegmentKey::String(var_value.clone()),
                    &segments[i + 1],
                )?;
            }
            Segment::IndexVariable(var_name) => {
                let variables = vars.unwrap();
                let var_value = variables.get(var_name).ok_or_else(|| {
                    StructpathError::MissingVariable(var_name.clone())
                })?;

                let idx = var_value.parse::<usize>().map_err(|_| {
                    StructpathError::InvalidVariableValue(var_value.clone())
                })?;

                current =
                    ensure_array_index_exists(current, idx, &segments[i + 1])?;
            }
        }
    }

    if let Some(original_data) = data {
        *original_data = root_value.clone();
    }

    Ok(root_value)
}

fn ensure_next_segment_exists<'a>(
    data: &'a mut Value,
    key: &SegmentKey,
    _next_segment: &Segment,
) -> Result<&'a mut Value, StructpathError> {
    let key_str = match key {
        SegmentKey::String(s) => s.clone(),
        SegmentKey::Int(i) => i.to_string(),
    };

    match data {
        Value::Object(map) => {
            // Create key if it doesn't exist
            if !map.contains_key(&key_str) {
                // Insert null instead of empty object/array
                map.insert(key_str.clone(), Value::Null);
            }

            // Now get the value to return
            let value = map.get_mut(&key_str).unwrap();

            // Based on the next segment, ensure the correct container type
            match _next_segment {
                Segment::Key(_) | Segment::KeyVariable(_) => {
                    // Need an object for the next segment
                    if !value.is_object() {
                        *value = Value::Object(Map::new());
                    }
                }
                Segment::Index(_) | Segment::IndexVariable(_) => {
                    // Need an array for the next segment
                    if !value.is_array() {
                        *value = Value::Array(Vec::new());
                    }
                }
            }

            Ok(map.get_mut(&key_str).unwrap())
        }
        _ => {
            // Convert to an object if it's not one already
            let mut map = Map::new();

            // Create the appropriate container based on the next segment
            match _next_segment {
                Segment::Key(_) | Segment::KeyVariable(_) => {
                    map.insert(key_str.clone(), Value::Object(Map::new()));
                }
                Segment::Index(_) | Segment::IndexVariable(_) => {
                    map.insert(key_str.clone(), Value::Array(Vec::new()));
                }
            }

            *data = Value::Object(map);

            if let Value::Object(map) = data {
                Ok(map.get_mut(&key_str).unwrap())
            } else {
                Err(StructpathError::InvalidPath {
                    expected: "object".to_string(),
                    found: format!("{:?}", data),
                })
            }
        }
    }
}

fn ensure_array_index_exists<'a>(
    data: &'a mut Value,
    idx: usize,
    _next_segment: &Segment,
) -> Result<&'a mut Value, StructpathError> {
    match data {
        Value::Array(arr) => {
            while arr.len() <= idx {
                arr.push(Value::Null);
            }

            Ok(&mut arr[idx])
        }
        _ => {
            let mut new_arr = Vec::new();

            for _ in 0..=idx {
                new_arr.push(Value::Null);
            }

            *data = Value::Array(new_arr);

            if let Value::Array(arr) = data {
                Ok(&mut arr[idx])
            } else {
                Err(StructpathError::InvalidPath {
                    expected: "array".to_string(),
                    found: format!("{:?}", data),
                })
            }
        }
    }
}

fn write_by_key(
    data: &mut Value,
    key: &SegmentKey,
    value: Value,
) -> Result<(), StructpathError> {
    let key_str = match key {
        SegmentKey::String(s) => s.clone(),
        SegmentKey::Int(i) => i.to_string(),
    };

    match data {
        Value::Object(map) => {
            map.insert(key_str, value);
            Ok(())
        }
        _ => {
            let mut map = Map::new();
            map.insert(key_str, value);
            *data = Value::Object(map);
            Ok(())
        }
    }
}

fn write_by_index(
    data: &mut Value,
    idx: usize,
    value: Value,
) -> Result<(), StructpathError> {
    match data {
        Value::Array(arr) => {
            while arr.len() <= idx {
                arr.push(Value::Null);
            }
            arr[idx] = value;
            Ok(())
        }
        _ => {
            let mut new_arr = Vec::new();

            for _ in 0..idx {
                new_arr.push(Value::Null);
            }
            new_arr.push(value);

            *data = Value::Array(new_arr);
            Ok(())
        }
    }
}
