use chumsky::span::SimpleSpan;
use serde_json::{Map, Number, Value as JsValue};

pub type Span = SimpleSpan<usize>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Reference(String),
    Type(ValType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    NoExport(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub value: SpannedValue,
    pub directives: Vec<Directive>,
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
    Int(i64),
    Float(f64),
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
            ValueKind::Int(n) => Value::Int(*n),
            ValueKind::Float(n) => Value::Float(*n),
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
            Value::Int(n) => JsValue::Number(Number::from(*n)),
            Value::Float(n) => JsValue::Number(Number::from_f64(*n).unwrap()),
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
    Boolean,
}

fn remove_path(value: &mut Value, parts: &[&str]) {
    if parts.is_empty() {
        return;
    }
    if let Value::Object(members) = value {
        if let Some(pos) = members.iter().position(|(k, _)| k == parts[0]) {
            if parts.len() == 1 {
                members.remove(pos);
            } else {
                remove_path(&mut members[pos].1, &parts[1..]);
            }
        }
    }
}

pub fn apply_directives(mut value: Value, directives: &[Directive]) -> Value {
    for d in directives {
        match d {
            Directive::NoExport(path) => {
                let parts: Vec<&str> = path.split('.').collect();
                remove_path(&mut value, &parts);
            }
        }
    }
    value
}

fn remove_path_spanned(value: &mut SpannedValue, parts: &[&str]) {
    if parts.is_empty() {
        return;
    }
    if let ValueKind::Object(members) = &mut value.kind {
        if let Some(pos) = members.iter().position(|(k, _, _)| k == parts[0]) {
            if parts.len() == 1 {
                members.remove(pos);
            } else {
                remove_path_spanned(&mut members[pos].1, &parts[1..]);
            }
        }
    }
}

pub fn apply_directives_spanned(value: &mut SpannedValue, directives: &[Directive]) {
    for d in directives {
        match d {
            Directive::NoExport(path) => {
                let parts: Vec<&str> = path.split('.').collect();
                remove_path_spanned(value, &parts);
            }
        }
    }
}
