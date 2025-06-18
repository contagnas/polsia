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
    Call(String, Box<Value>),
    OpCall(String, Box<Value>, Box<Value>),
    Union(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    NoExport,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub value: SpannedValue,
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
    Object(Vec<(String, SpannedValue, Span, Vec<Annotation>)>),
    Reference(String),
    Type(ValType),
    Call(String, Box<SpannedValue>),
    OpCall(String, Box<SpannedValue>, Box<SpannedValue>),
    Union(Vec<SpannedValue>),
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
                    .filter(|(_, _, _, anns)| !anns.contains(&Annotation::NoExport))
                    .map(|(k, v, _, _)| (k.clone(), v.to_value()))
                    .collect(),
            ),
            ValueKind::Reference(r) => Value::Reference(r.clone()),
            ValueKind::Type(t) => Value::Type(t.clone()),
            ValueKind::Call(name, arg) => Value::Call(name.clone(), Box::new(arg.to_value())),
            ValueKind::OpCall(op, left, right) => Value::OpCall(
                op.clone(),
                Box::new(left.to_value()),
                Box::new(right.to_value()),
            ),
            ValueKind::Union(items) => Value::Union(items.iter().map(|v| v.to_value()).collect()),
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
            Value::Call(name, _) => panic!("unresolved call {}", name),
            Value::OpCall(op, _, _) => panic!("unresolved op {}", op),
            Value::Union(_) => panic!("unresolved union"),
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
