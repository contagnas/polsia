use crate::types::{Json, JsonType, Span, SpannedJson, SpannedKind};
use chumsky::span::{SimpleSpan, Span as ChumSpan};

#[derive(Debug)]
pub struct UnifyError {
    pub msg: String,
    pub span: Span,
    pub prev_span: Span,
}

fn type_name(t: &JsonType) -> &'static str {
    match t {
        JsonType::Any => "Any",
        JsonType::Nothing => "Nothing",
        JsonType::Int => "Int",
        JsonType::Number => "Number",
        JsonType::Rational => "Rational",
        JsonType::Float => "Float",
        JsonType::String => "String",
    }
}

fn unify_types(a: &JsonType, b: &JsonType) -> Result<JsonType, String> {
    if a == b {
        return Ok(a.clone());
    }
    if matches!(a, JsonType::Any) {
        return Ok(b.clone());
    }
    if matches!(b, JsonType::Any) {
        return Ok(a.clone());
    }
    if matches!(a, JsonType::Nothing) || matches!(b, JsonType::Nothing) {
        return Err("cannot unify Nothing".into());
    }
    let rank = |t: &JsonType| match t {
        JsonType::Int => Some(0),
        JsonType::Rational => Some(1),
        JsonType::Float => Some(2),
        JsonType::Number => Some(3),
        _ => None,
    };
    match (rank(a), rank(b)) {
        (Some(ra), Some(rb)) => {
            let r = ra.max(rb);
            Ok(match r {
                0 => JsonType::Int,
                1 => JsonType::Rational,
                2 => JsonType::Float,
                _ => JsonType::Number,
            })
        }
        _ => Err(format!(
            "{} cannot be unified with {}",
            type_name(a),
            type_name(b)
        )),
    }
}

fn unify_type_value(t: &JsonType, val: &Json) -> Result<Json, String> {
    match t {
        JsonType::Any => Ok(val.clone()),
        JsonType::Nothing => Err("cannot unify Nothing".into()),
        JsonType::Int => match val {
            Json::Number(n) if n.fract() == 0.0 => Ok(Json::Number(*n)),
            Json::Type(other) => unify_types(t, other).map(Json::Type),
            _ => Err("expected integer".into()),
        },
        JsonType::Rational | JsonType::Float | JsonType::Number => match val {
            Json::Number(n) => Ok(Json::Number(*n)),
            Json::Type(other) => unify_types(t, other).map(Json::Type),
            _ => Err("expected number".into()),
        },
        JsonType::String => match val {
            Json::String(s) => Ok(Json::String(s.clone())),
            Json::Type(other) => unify_types(t, other).map(Json::Type),
            _ => Err("expected string".into()),
        },
    }
}

pub fn unify(a: &Json, b: &Json) -> Result<Json, String> {
    unify_with_path(a, b, "")
}

fn add_path(path: &str, msg: String) -> String {
    if path.is_empty() {
        msg
    } else {
        format!("{}: {}", path, msg)
    }
}

pub fn unify_spanned(
    a: &SpannedJson,
    b: &SpannedJson,
    path: &str,
) -> Result<SpannedJson, UnifyError> {
    if a.to_json() == b.to_json() {
        return Ok(b.clone());
    }
    match (&a.kind, &b.kind) {
        (SpannedKind::Type(ta), SpannedKind::Type(tb)) => match unify_types(ta, tb) {
            Ok(t) => Ok(SpannedJson {
                span: b.span,
                kind: SpannedKind::Type(t),
            }),
            Err(e) => Err(UnifyError {
                msg: add_path(path, e),
                span: b.span,
                prev_span: a.span,
            }),
        },
        (SpannedKind::Type(t), other) => match unify_type_value(t, &spkind_to_json(other)) {
            Ok(j) => Ok(SpannedJson {
                span: b.span,
                kind: json_to_spkind(j),
            }),
            Err(e) => Err(UnifyError {
                msg: add_path(path, e),
                span: b.span,
                prev_span: a.span,
            }),
        },
        (other, SpannedKind::Type(t)) => match unify_type_value(t, &spkind_to_json(other)) {
            Ok(j) => Ok(SpannedJson {
                span: a.span,
                kind: json_to_spkind(j),
            }),
            Err(e) => Err(UnifyError {
                msg: add_path(path, e),
                span: b.span,
                prev_span: a.span,
            }),
        },
        (SpannedKind::Object(a_members), SpannedKind::Object(b_members)) => {
            use std::collections::BTreeMap;
            let mut map: BTreeMap<String, SpannedJson> = BTreeMap::new();
            for (k, v, _) in a_members {
                map.insert(k.clone(), v.clone());
            }
            for (k, v, _) in b_members {
                match map.get(k) {
                    Some(prev) => {
                        let new_path = if path.is_empty() {
                            k.clone()
                        } else {
                            format!("{}.{}", path, k)
                        };
                        let unified = unify_spanned(prev, v, &new_path)?;
                        map.insert(k.clone(), unified);
                    }
                    None => {
                        map.insert(k.clone(), v.clone());
                    }
                }
            }
            let members = map.into_iter().collect::<Vec<_>>();
            Ok(SpannedJson {
                span: b.span,
                kind: SpannedKind::Object(
                    members
                        .into_iter()
                        .map(|(k, v)| {
                            let span = v.span;
                            (k, v, span)
                        })
                        .collect(),
                ),
            })
        }
        _ => Err(UnifyError {
            msg: add_path(path, "values do not unify".into()),
            span: b.span,
            prev_span: a.span,
        }),
    }
}

fn unify_tree_inner(value: &SpannedJson, path: &str) -> Result<SpannedJson, UnifyError> {
    match &value.kind {
        SpannedKind::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                out.push(unify_tree_inner(item, path)?);
            }
            Ok(SpannedJson { span: value.span, kind: SpannedKind::Array(out) })
        }
        SpannedKind::Object(members) => {
            use std::collections::HashMap;
            let mut seen: HashMap<String, SpannedJson> = HashMap::new();
            let mut out: Vec<(String, SpannedJson, Span)> = Vec::new();
            for (k, v, span) in members {
                let new_path = if path.is_empty() { k.clone() } else { format!("{}.{}", path, k) };
                let unified_v = unify_tree_inner(v, &new_path)?;
                if let Some(prev) = seen.get(k) {
                    let merged = unify_spanned(prev, &unified_v, &new_path)?;
                    seen.insert(k.clone(), merged);
                } else {
                    seen.insert(k.clone(), unified_v.clone());
                }
                out.push((k.clone(), unified_v, *span));
            }
            Ok(SpannedJson { span: value.span, kind: SpannedKind::Object(out) })
        }
        _ => Ok(value.clone()),
    }
}

pub fn unify_tree(value: &SpannedJson) -> Result<SpannedJson, UnifyError> {
    unify_tree_inner(value, "")
}

fn json_to_spkind(j: Json) -> SpannedKind {
    match j {
        Json::Null => SpannedKind::Null,
        Json::Bool(b) => SpannedKind::Bool(b),
        Json::Number(n) => SpannedKind::Number(n),
        Json::String(s) => SpannedKind::String(s),
        Json::Array(arr) => SpannedKind::Array(
            arr.into_iter()
                .map(|v| SpannedJson {
                    span: SimpleSpan::new((), 0..0),
                    kind: json_to_spkind(v),
                })
                .collect(),
        ),
        Json::Object(obj) => SpannedKind::Object(
            obj.into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        SpannedJson {
                            span: SimpleSpan::new((), 0..0),
                            kind: json_to_spkind(v),
                        },
                        SimpleSpan::new((), 0..0),
                    )
                })
                .collect(),
        ),
        Json::Type(t) => SpannedKind::Type(t),
    }
}

fn spkind_to_json(k: &SpannedKind) -> Json {
    match k {
        SpannedKind::Null => Json::Null,
        SpannedKind::Bool(b) => Json::Bool(*b),
        SpannedKind::Number(n) => Json::Number(*n),
        SpannedKind::String(s) => Json::String(s.clone()),
        SpannedKind::Array(arr) => Json::Array(arr.iter().map(|v| v.to_json()).collect()),
        SpannedKind::Object(obj) => Json::Object(
            obj.iter()
                .map(|(k, v, _)| (k.clone(), v.to_json()))
                .collect(),
        ),
        SpannedKind::Type(t) => Json::Type(t.clone()),
    }
}

pub fn unify_with_path(a: &Json, b: &Json, path: &str) -> Result<Json, String> {
    if a == b {
        return Ok(a.clone());
    }
    match (a, b) {
        (Json::Type(ta), Json::Type(tb)) => unify_types(ta, tb)
            .map(Json::Type)
            .map_err(|e| add_path(path, e)),
        (Json::Type(t), val) | (val, Json::Type(t)) => {
            unify_type_value(t, val).map_err(|e| add_path(path, e))
        }
        (Json::Object(a_members), Json::Object(b_members)) => {
            use std::collections::BTreeMap;
            let mut map: BTreeMap<String, Json> = BTreeMap::new();
            for (k, v) in a_members {
                map.insert(k.clone(), v.clone());
            }
            for (k, v) in b_members {
                match map.get(k) {
                    Some(existing) => {
                        let new_path = if path.is_empty() {
                            k.clone()
                        } else {
                            format!("{}.{}", path, k)
                        };
                        let unified = unify_with_path(existing, v, &new_path)?;
                        map.insert(k.clone(), unified);
                    }
                    None => {
                        map.insert(k.clone(), v.clone());
                    }
                }
            }
            Ok(Json::Object(map.into_iter().collect()))
        }
        _ => Err(add_path(path, "values do not unify".into())),
    }
}
