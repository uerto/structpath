use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use serde_json::Value;

pub fn serialize(obj: &PyAny) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Null);
    }

    if let Ok(val) = obj.extract::<bool>() {
        return Ok(Value::Bool(val));
    }

    if let Ok(val) = obj.extract::<i64>() {
        return Ok(Value::Number(val.into()));
    }

    if let Ok(val) = obj.extract::<f64>() {
        return Ok(serde_json::Number::from_f64(val)
            .map(Value::Number)
            .unwrap_or(Value::Null));
    }

    if let Ok(val) = obj.extract::<String>() {
        return Ok(Value::String(val));
    }

    if let Ok(list) = obj.downcast::<PyList>() {
        let mut values = Vec::new();
        for item in list.iter() {
            values.push(serialize(item)?);
        }
        return Ok(Value::Array(values));
    }

    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = if let Ok(key_str) = key.extract::<String>() {
                key_str
            } else if let Ok(key_int) = key.extract::<i64>() {
                key_int.to_string()
            } else {
                return Err(PyTypeError::new_err(
                    "Dictionary keys must be strings or integers",
                ));
            };
            map.insert(key_str, serialize(value)?);
        }
        return Ok(Value::Object(map));
    }

    let py = obj.py();
    let datetime_module = py.import("datetime")?;
    let datetime_class = datetime_module.getattr("datetime")?;
    let date_class = datetime_module.getattr("date")?;
    let time_class = datetime_module.getattr("time")?;

    if obj.is_instance(datetime_class)? {
        let mut map = serde_json::Map::new();
        map.insert(
            "__type__".to_string(),
            Value::String("datetime".to_string()),
        );
        map.insert(
            "iso".to_string(),
            Value::String(obj.call_method0("isoformat")?.extract::<String>()?),
        );
        return Ok(Value::Object(map));
    }

    if obj.is_instance(date_class)? && !obj.is_instance(datetime_class)? {
        let mut map = serde_json::Map::new();
        map.insert("__type__".to_string(), Value::String("date".to_string()));
        map.insert(
            "iso".to_string(),
            Value::String(obj.call_method0("isoformat")?.extract::<String>()?),
        );
        return Ok(Value::Object(map));
    }

    if obj.is_instance(time_class)? {
        let mut map = serde_json::Map::new();
        map.insert("__type__".to_string(), Value::String("time".to_string()));
        map.insert(
            "iso".to_string(),
            Value::String(obj.call_method0("isoformat")?.extract::<String>()?),
        );
        return Ok(Value::Object(map));
    }

    Err(PyTypeError::new_err(format!(
        "Invalid type {}",
        obj.get_type().name()?
    )))
}

pub fn deserialize(value: &Value, py: Python) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.to_object(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Err(PyValueError::new_err("Invalid JSON number"))
            }
        }
        Value::String(s) => Ok(s.to_object(py)),
        Value::Array(a) => {
            let list = PyList::empty(py);
            for item in a {
                list.append(deserialize(item, py)?)?;
            }
            Ok(list.to_object(py))
        }
        Value::Object(o) => {
            if let (
                Some(Value::String(type_val)),
                Some(Value::String(iso_val)),
            ) = (o.get("__type__"), o.get("iso"))
            {
                let datetime = py.import("datetime")?;

                match type_val.as_str() {
                    "datetime" => {
                        let args = PyTuple::new(py, &[iso_val.to_object(py)]);
                        let dt = datetime
                            .getattr("datetime")?
                            .getattr("fromisoformat")?
                            .call1(args)?;
                        return Ok(dt.to_object(py));
                    }
                    "date" => {
                        let args = PyTuple::new(py, &[iso_val.to_object(py)]);
                        let date = datetime
                            .getattr("date")?
                            .getattr("fromisoformat")?
                            .call1(args)?;
                        return Ok(date.to_object(py));
                    }
                    "time" => {
                        let args = PyTuple::new(py, &[iso_val.to_object(py)]);
                        let time = datetime
                            .getattr("time")?
                            .getattr("fromisoformat")?
                            .call1(args)?;
                        return Ok(time.to_object(py));
                    }
                    _ => {}
                }
            }

            let dict = PyDict::new(py);
            for (key, value) in o {
                dict.set_item(key, deserialize(value, py)?)?;
            }
            Ok(dict.to_object(py))
        }
    }
}
