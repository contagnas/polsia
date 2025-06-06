use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan<usize>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
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
            ValueKind::Type(t) => Value::Type(t.clone()),
        }
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
