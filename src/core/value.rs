use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Value {
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Self::Null | Self::Bool(_) | Self::Integer(_) | Self::Float(_) | Self::String(_)
        )
    }

    pub fn scalar_text(&self) -> Option<String> {
        match self {
            Self::Null => Some(String::new()),
            Self::Bool(value) => Some(value.to_string()),
            Self::Integer(value) => Some(value.to_string()),
            Self::Float(value) => Some(value.to_string()),
            Self::String(value) => Some(value.clone()),
            Self::List(_) | Self::Object(_) => None,
        }
    }
}
