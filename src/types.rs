use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan<usize>;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
    Type(JsonType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedJson {
    pub span: Span,
    pub kind: SpannedKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpannedKind {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<SpannedJson>),
    Object(Vec<(String, SpannedJson, Span)>),
    Type(JsonType),
}

impl SpannedJson {
    pub fn to_json(&self) -> Json {
        match &self.kind {
            SpannedKind::Null => Json::Null,
            SpannedKind::Bool(b) => Json::Bool(*b),
            SpannedKind::Number(n) => Json::Number(*n),
            SpannedKind::String(s) => Json::String(s.clone()),
            SpannedKind::Array(a) => Json::Array(a.iter().map(|j| j.to_json()).collect()),
            SpannedKind::Object(m) => {
                Json::Object(m.iter().map(|(k, v, _)| (k.clone(), v.to_json())).collect())
            }
            SpannedKind::Type(t) => Json::Type(t.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonType {
    Any,
    Nothing,
    Int,
    Number,
    Rational,
    Float,
    String,
}
