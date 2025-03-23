use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Key(SegmentKey),
    Index(usize),
    KeyVariable(String),
    IndexVariable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SegmentKey {
    String(String),
    Int(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Structpath {
    segments: Vec<Segment>,
    variable_names: HashSet<String>,
}

#[derive(Error, Debug)]
pub enum StructpathError {
    #[error("Failed to parse path: {0}")]
    ParseError(String),
    #[error("Duplicate variable name: {0}")]
    DuplicateVariable(String),
    #[error("Value not found at path")]
    NotFound,
    #[error("Invalid path: expected {expected} but found {found}")]
    InvalidPath { expected: String, found: String },
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),
    #[error("Missing variable: {0}")]
    MissingVariable(String),
    #[error("Invalid variable value: expected number for index, got {0}")]
    InvalidVariableValue(String),
}

impl Structpath {
    pub fn new() -> Self {
        Structpath {
            segments: Vec::new(),
            variable_names: HashSet::new(),
        }
    }

    pub fn push_string_key(&mut self, key: &str) {
        self.segments
            .push(Segment::Key(SegmentKey::String(key.to_string())));
    }

    pub fn push_int_key(&mut self, key: i64) {
        self.segments.push(Segment::Key(SegmentKey::Int(key)));
    }

    pub fn push_index(&mut self, index: usize) {
        self.segments.push(Segment::Index(index));
    }

    pub fn push_key_variable(
        &mut self,
        name: &str,
    ) -> Result<(), StructpathError> {
        if !self.variable_names.insert(name.to_string()) {
            return Err(StructpathError::DuplicateVariable(name.to_string()));
        }
        self.segments.push(Segment::KeyVariable(name.to_string()));
        Ok(())
    }

    pub fn push_index_variable(
        &mut self,
        name: &str,
    ) -> Result<(), StructpathError> {
        if !self.variable_names.insert(name.to_string()) {
            return Err(StructpathError::DuplicateVariable(name.to_string()));
        }
        self.segments.push(Segment::IndexVariable(name.to_string()));
        Ok(())
    }

    pub fn parse(path_str: &str) -> Result<Self, StructpathError> {
        crate::parse::parse(path_str)
    }

    pub fn get<'a>(
        &self,
        data: &'a Value,
        vars: Option<&HashMap<String, String>>,
    ) -> Result<&'a Value, StructpathError> {
        crate::access::get(self, data, vars)
    }

    pub fn write(
        &self,
        data: Option<&mut Value>,
        value: Value,
        vars: Option<&HashMap<String, String>>,
    ) -> Result<Value, StructpathError> {
        crate::write::write(self, data, value, vars)
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn walk(data: &Value) -> impl Iterator<Item = (Structpath, &Value)> {
        crate::walk::new_walker(data)
    }
}

impl fmt::Display for Structpath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::format::to_string(self))
    }
}

impl Default for Structpath {
    fn default() -> Self {
        Self::new()
    }
}
