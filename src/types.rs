use chumsky::span::SimpleSpan;
use serde_json::{Map, Number, Value as JsValue};

pub type Span = SimpleSpan<usize>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Reference(String),
    Type(ValType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedValue {
    pub span: Span,
    pub kind: ValueKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<SpannedValue>),
    Object(Vec<(String, SpannedValue, Span)>),
    Reference(String),
    Type(ValType),
}

impl SpannedValue {
    pub fn to_value(&self) -> Value {
        match &self.kind {
            ValueKind::Null => Value::Null,
            ValueKind::Bool(b) => Value::Bool(*b),
            ValueKind::Number(n) => Value::Number(*n),
            ValueKind::String(s) => Value::String(s.clone()),
            ValueKind::Array(a) => Value::Array(a.iter().map(|j| j.to_value()).collect()),
            ValueKind::Object(m) => Value::Object(
                m.iter()
                    .map(|(k, v, _)| (k.clone(), v.to_value()))
                    .collect(),
            ),
            ValueKind::Reference(r) => Value::Reference(r.clone()),
            ValueKind::Type(t) => Value::Type(t.clone()),
        }
    }
}

impl Value {
    pub fn to_value(&self) -> JsValue {
        match self {
            Value::Null => JsValue::Null,
            Value::Bool(b) => JsValue::Bool(*b),
            Value::Number(n) => JsValue::Number(Number::from_f64(*n).unwrap()),
            Value::String(s) => JsValue::String(s.clone()),
            Value::Array(arr) => JsValue::Array(arr.iter().map(|v| v.to_value()).collect()),
            Value::Object(obj) => {
                let map: Map<String, JsValue> =
                    obj.iter().map(|(k, v)| (k.clone(), v.to_value())).collect();
                JsValue::Object(map)
            }
            Value::Reference(r) => JsValue::String(r.clone()),
            Value::Type(t) => panic!("unresolved type {:?}", t),
        }
    }

    pub fn to_pretty_string(&self) -> String {
        serde_json::to_string_pretty(&self.to_value()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValType {
    Any,
    Nothing,
    Int,
    Number,
    Rational,
    Float,
    String,
}
