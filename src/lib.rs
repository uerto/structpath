#![allow(non_local_definitions)]

use pyo3::exceptions::{PyIndexError, PyKeyError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};

mod access;
mod format;
mod iter;
mod parse;
mod serialization;
mod types;
mod walk;
mod write;

pub use types::{Segment, SegmentKey, Structpath, StructpathError};

#[cfg(feature = "extension-module")]
#[pymodule]
pub fn _structpath(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStructpath>()?;
    m.add_class::<PyWalker>()?;
    m.add_class::<PyVariableIterator>()?;
    Ok(())
}

#[pyclass(name = "Structpath")]
#[derive(Clone)]
struct PyStructpath {
    inner: Structpath,
}

#[derive(Clone)]
struct WalkerState {
    path: Structpath,
    value: Value,
}

#[pyclass(name = "Walker")]
struct PyWalker {
    stack: VecDeque<WalkerState>,
}

#[pyclass(name = "VariableIterator")]
struct PyVariableIterator {
    results: Vec<(Value, HashMap<String, Value>)>,
    current_pos: usize,
}

#[pymethods]
impl PyVariableIterator {
    #[new]
    fn new() -> Self {
        PyVariableIterator {
            results: Vec::new(),
            current_pos: 0,
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'_>,
    ) -> PyResult<Option<PyObject>> {
        if slf.current_pos < slf.results.len() {
            let idx = slf.current_pos;
            slf.current_pos += 1;
            let (ref value, ref vars) = slf.results[idx];

            let py_vars = PyDict::new(py);
            for (k, v) in vars {
                py_vars.set_item(k, serialization::deserialize(v, py)?)?;
            }

            let result = PyTuple::new(
                py,
                &[
                    py_vars.to_object(py),
                    serialization::deserialize(value, py)?,
                ],
            );

            return Ok(Some(result.to_object(py)));
        }
        Ok(None)
    }
}

#[pymethods]
impl PyWalker {
    #[new]
    fn new(data: &PyAny) -> PyResult<Self> {
        let json_data = serialization::serialize(data)?;

        // Initialize stack with an empty result list
        let mut stack = VecDeque::new();

        // Set up our walker to mimic the Rust implementation
        let rust_walker = walk::new_walker(&json_data);

        // Collect all pairs from the walker
        let results: Vec<(Structpath, Value)> = rust_walker
            .map(|(path, value)| (path, value.clone()))
            .collect();

        // Prepare them for Python iteration
        for (path, value) in results {
            stack.push_back(WalkerState { path, value });
        }

        Ok(PyWalker { stack })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'_>,
    ) -> PyResult<Option<(PyObject, PyObject)>> {
        if let Some(state) = slf.stack.pop_front() {
            // Convert to Python objects
            let path_obj = PyStructpath { inner: state.path }.into_py(py);
            let value_obj = serialization::deserialize(&state.value, py)?;

            return Ok(Some((path_obj, value_obj)));
        }
        Ok(None)
    }
}

#[pymethods]
impl PyStructpath {
    #[new]
    fn new() -> Self {
        PyStructpath {
            inner: Structpath::new(),
        }
    }

    #[staticmethod]
    #[pyo3(name = "parse")]
    fn py_parse(path_str: &str) -> PyResult<Self> {
        match Structpath::parse(path_str) {
            Ok(inner) => Ok(PyStructpath { inner }),
            Err(err) => match err {
                StructpathError::DuplicateVariable(name) => {
                    Err(PyValueError::new_err(format!(
                        "Duplicate variable name: {}",
                        name
                    )))
                }
                _ => Err(PyValueError::new_err(err.to_string())),
            },
        }
    }

    fn push_key(&mut self, key: &PyAny) -> PyResult<()> {
        if let Ok(int_key) = key.extract::<i64>() {
            self.inner.push_int_key(int_key);
            Ok(())
        } else if let Ok(str_key) = key.extract::<String>() {
            self.inner.push_string_key(&str_key);
            Ok(())
        } else {
            Err(PyTypeError::new_err("Key must be a string or integer"))
        }
    }

    fn push_index(&mut self, index: usize) {
        self.inner.push_index(index);
    }

    fn push_key_variable(&mut self, name: &str) -> PyResult<()> {
        match self.inner.push_key_variable(name) {
            Ok(()) => Ok(()),
            Err(err) => match err {
                StructpathError::DuplicateVariable(name) => {
                    Err(PyValueError::new_err(format!(
                        "Duplicate variable name: {}",
                        name
                    )))
                }
                _ => Err(PyValueError::new_err(err.to_string())),
            },
        }
    }

    fn push_index_variable(&mut self, name: &str) -> PyResult<()> {
        match self.inner.push_index_variable(name) {
            Ok(()) => Ok(()),
            Err(err) => match err {
                StructpathError::DuplicateVariable(name) => {
                    Err(PyValueError::new_err(format!(
                        "Duplicate variable name: {}",
                        name
                    )))
                }
                _ => Err(PyValueError::new_err(err.to_string())),
            },
        }
    }

    #[pyo3(signature = (data, vars = None))]
    fn get(&self, data: &PyAny, vars: Option<&PyDict>) -> PyResult<PyObject> {
        let value = serialization::serialize(data)?;

        let rust_vars = match vars {
            Some(dict) => {
                let mut vars_map = HashMap::new();
                for (key, value) in dict.iter() {
                    let key_str = key.extract::<String>()?;
                    let value_str = value.extract::<String>()?;
                    vars_map.insert(key_str, value_str);
                }
                Some(vars_map)
            }
            None => None,
        };

        let vars_ref =
            rust_vars.as_ref().map(|v| v as &HashMap<String, String>);

        match self.inner.get(&value, vars_ref) {
            Ok(result) => serialization::deserialize(result, data.py()),
            Err(err) => match err {
                StructpathError::NotFound => Err(PyKeyError::new_err(format!(
                    "Path not found: {}",
                    self.inner
                ))),
                StructpathError::InvalidPath { expected, found } => {
                    Err(PyTypeError::new_err(format!(
                        "Invalid path: expected {}, found {}",
                        expected, found
                    )))
                }
                StructpathError::IndexOutOfBounds(msg) => {
                    Err(PyIndexError::new_err(msg))
                }
                StructpathError::MissingVariable(var_name) => {
                    Err(PyValueError::new_err(format!(
                        "Missing variable in context: {}",
                        var_name
                    )))
                }
                _ => Err(PyValueError::new_err(err.to_string())),
            },
        }
    }

    fn iter(&self, data: &PyAny) -> PyResult<PyVariableIterator> {
        let json_data = serialization::serialize(data)?;

        let rust_iter = iter::iter_variables(&self.inner, &json_data);
        let mut results = Vec::new();

        for (value, vars) in rust_iter {
            results.push((value.clone(), vars));
        }

        Ok(PyVariableIterator {
            results,
            current_pos: 0,
        })
    }

    #[pyo3(signature = (data = None, value = None, vars = None))]
    fn write(
        &self,
        data: Option<&PyAny>,
        value: Option<&PyAny>,
        vars: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let mut json_data = match &data {
            Some(py_data) => serialization::serialize(py_data)?,
            None => Value::Null,
        };

        let json_value = match value {
            Some(val) => serialization::serialize(val)?,
            None => Value::Null,
        };

        let rust_vars = match vars {
            Some(dict) => {
                let mut vars_map = HashMap::new();
                for (key, val) in dict.iter() {
                    let key_str = key.extract::<String>()?;
                    let value_str = val.extract::<String>()?;
                    vars_map.insert(key_str, value_str);
                }
                Some(vars_map)
            }
            None => None,
        };

        let vars_ref =
            rust_vars.as_ref().map(|v| v as &HashMap<String, String>);

        match self.inner.write(Some(&mut json_data), json_value, vars_ref) {
            Ok(result) => {
                let py = match value {
                    Some(val) => val.py(),
                    None => unsafe { Python::assume_gil_acquired() },
                };

                // Update the original Python object if provided (for side effects)
                if let Some(py_data) = data {
                    if !py_data.is_none() {
                        // Check if it's a dictionary that can be modified
                        if let Ok(dict) = py_data.downcast::<PyDict>() {
                            // Clear the original dict
                            dict.clear();

                            // Deserialize the result to a Python object
                            let result_obj =
                                serialization::deserialize(&result, py)?;

                            // Try to get it as a dictionary
                            if let Ok(result_dict) =
                                result_obj.extract::<&PyDict>(py)
                            {
                                // Copy all items from result_dict to the original dict
                                for (key, value) in result_dict.iter() {
                                    let _ = dict.set_item(key, value);
                                }
                            }
                        }
                    }
                }

                serialization::deserialize(&result, py)
            }
            Err(err) => match err {
                StructpathError::InvalidPath { expected, found } => {
                    Err(PyTypeError::new_err(format!(
                        "Invalid path: expected {}, found {}",
                        expected, found
                    )))
                }
                StructpathError::MissingVariable(var_name) => {
                    Err(PyValueError::new_err(format!(
                        "Missing variable in context: {}",
                        var_name
                    )))
                }
                _ => Err(PyValueError::new_err(err.to_string())),
            },
        }
    }

    #[staticmethod]
    #[pyo3(name = "walk")]
    fn py_walk(data: &PyAny) -> PyResult<PyWalker> {
        PyWalker::new(data)
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        format!("Structpath('{}')", self.inner)
    }
}
