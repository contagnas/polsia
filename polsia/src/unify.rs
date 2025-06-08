use crate::types::{Span, SpannedValue, ValType, Value, ValueKind};
use chumsky::span::{SimpleSpan, Span as ChumSpan};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct UnifyError {
    pub msg: String,
    pub span: Span,
    pub prev_span: Span,
}

fn type_name(t: &ValType) -> &'static str {
    match t {
        ValType::Any => "Any",
        ValType::Nothing => "Nothing",
        ValType::Int => "Int",
        ValType::Number => "Number",
        ValType::Rational => "Rational",
        ValType::Float => "Float",
        ValType::String => "String",
        ValType::Boolean => "Boolean",
    }
}

fn unify_types(a: &ValType, b: &ValType) -> Result<ValType, String> {
    if a == b {
        return Ok(a.clone());
    }
    if matches!(a, ValType::Any) {
        return Ok(b.clone());
    }
    if matches!(b, ValType::Any) {
        return Ok(a.clone());
    }
    if matches!(a, ValType::Nothing) || matches!(b, ValType::Nothing) {
        return Err("cannot unify Nothing".into());
    }
    let rank = |t: &ValType| match t {
        ValType::Int => Some(0),
        ValType::Rational => Some(1),
        ValType::Float => Some(2),
        ValType::Number => Some(3),
        _ => None,
    };
    match (rank(a), rank(b)) {
        (Some(ra), Some(rb)) => {
            let r = ra.max(rb);
            Ok(match r {
                0 => ValType::Int,
                1 => ValType::Rational,
                2 => ValType::Float,
                _ => ValType::Number,
            })
        }
        _ => Err(format!(
            "{} cannot be unified with {}",
            type_name(a),
            type_name(b)
        )),
    }
}

fn unify_type_value(t: &ValType, val: &Value) -> Result<Value, String> {
    match t {
        ValType::Any => Ok(val.clone()),
        ValType::Nothing => Err("cannot unify Nothing".into()),
        ValType::Int => match val {
            Value::Int(n) => Ok(Value::Int(*n)),
            Value::Float(n) if n.fract() == 0.0 => Ok(Value::Int(*n as i64)),
            Value::Type(other) => unify_types(t, other).map(Value::Type),
            _ => Err("expected integer".into()),
        },
        ValType::Rational | ValType::Float | ValType::Number => match val {
            Value::Int(n) => Ok(Value::Int(*n)),
            Value::Float(n) => Ok(Value::Float(*n)),
            Value::Type(other) => unify_types(t, other).map(Value::Type),
            _ => Err("expected number".into()),
        },
        ValType::String => match val {
            Value::String(s) => Ok(Value::String(s.clone())),
            Value::Type(other) => unify_types(t, other).map(Value::Type),
            _ => Err("expected string".into()),
        },
        ValType::Boolean => match val {
            Value::Bool(b) => Ok(Value::Bool(*b)),
            Value::Type(other) => unify_types(t, other).map(Value::Type),
            _ => Err("expected boolean".into()),
        },
    }
}

pub fn unify(a: &Value, b: &Value) -> Result<Value, String> {
    unify_with_path(a, b, "")
}

fn add_path(path: &str, msg: String) -> String {
    if path.is_empty() {
        msg
    } else {
        format!("{}: {}", path, msg)
    }
}

fn lookup<'a>(root: &'a BTreeMap<String, SpannedValue>, path: &str) -> Option<&'a SpannedValue> {
    let mut segments = path.split('.');
    let first = segments.next()?;
    let mut current = root.get(first)?;
    for seg in segments {
        match &current.kind {
            ValueKind::Object(members) => match members.iter().find(|(k, _, _)| k == seg) {
                Some((_, v, _)) => current = v,
                None => return None,
            },
            _ => return None,
        }
    }
    Some(current)
}

pub fn unify_spanned(
    a: &SpannedValue,
    b: &SpannedValue,
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
) -> Result<SpannedValue, UnifyError> {
    if a.to_value() == b.to_value() {
        return Ok(b.clone());
    }
    match (&a.kind, &b.kind) {
        (ValueKind::Reference(pa), _) => {
            if let Some(val) = lookup(root, pa) {
                unify_spanned(val, b, path, root)
            } else {
                Err(UnifyError {
                    msg: add_path(path, format!("unresolved reference {}", pa)),
                    span: b.span,
                    prev_span: a.span,
                })
            }
        }
        (_, ValueKind::Reference(pb)) => {
            if let Some(val) = lookup(root, pb) {
                unify_spanned(a, val, path, root)
            } else {
                Err(UnifyError {
                    msg: add_path(path, format!("unresolved reference {}", pb)),
                    span: b.span,
                    prev_span: a.span,
                })
            }
        }
        (ValueKind::Type(ta), ValueKind::Type(tb)) => match unify_types(ta, tb) {
            Ok(t) => Ok(SpannedValue {
                span: b.span,
                kind: ValueKind::Type(t),
            }),
            Err(e) => Err(UnifyError {
                msg: add_path(path, e),
                span: b.span,
                prev_span: a.span,
            }),
        },
        (ValueKind::Type(t), other) => match unify_type_value(t, &kind_to_value(other)) {
            Ok(j) => Ok(SpannedValue {
                span: b.span,
                kind: value_to_kind(j),
            }),
            Err(e) => Err(UnifyError {
                msg: add_path(path, e),
                span: b.span,
                prev_span: a.span,
            }),
        },
        (other, ValueKind::Type(t)) => match unify_type_value(t, &kind_to_value(other)) {
            Ok(j) => Ok(SpannedValue {
                span: a.span,
                kind: value_to_kind(j),
            }),
            Err(e) => Err(UnifyError {
                msg: add_path(path, e),
                span: b.span,
                prev_span: a.span,
            }),
        },
        (ValueKind::Array(a_items), ValueKind::Array(b_items)) => {
            if a_items.len() != b_items.len() {
                return Err(UnifyError {
                    msg: add_path(path, "array lengths differ".into()),
                    span: b.span,
                    prev_span: a.span,
                });
            }
            let mut out = Vec::new();
            for (i, (av, bv)) in a_items.iter().zip(b_items.iter()).enumerate() {
                let new_path = if path.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", path, i)
                };
                let unified = unify_spanned(av, bv, &new_path, root)?;
                out.push(unified);
            }
            Ok(SpannedValue {
                span: b.span,
                kind: ValueKind::Array(out),
            })
        }
        (ValueKind::Object(a_members), ValueKind::Object(b_members)) => {
            use std::collections::BTreeMap;
            let mut map: BTreeMap<String, SpannedValue> = BTreeMap::new();
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
                        let unified = unify_spanned(prev, v, &new_path, root)?;
                        map.insert(k.clone(), unified);
                    }
                    None => {
                        map.insert(k.clone(), v.clone());
                    }
                }
            }
            let members = map.into_iter().collect::<Vec<_>>();
            Ok(SpannedValue {
                span: b.span,
                kind: ValueKind::Object(
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

fn unify_tree_inner(
    value: &SpannedValue,
    path: &str,
    root: &mut BTreeMap<String, SpannedValue>,
    is_root: bool,
) -> Result<SpannedValue, UnifyError> {
    match &value.kind {
        ValueKind::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                out.push(unify_tree_inner(item, path, root, false)?);
            }
            Ok(SpannedValue {
                span: value.span,
                kind: ValueKind::Array(out),
            })
        }
        ValueKind::Object(members) => {
            use std::collections::HashMap;
            // Preserve the order of first appearance while merging duplicates
            let mut indices: HashMap<String, usize> = HashMap::new();
            let mut out: Vec<(String, SpannedValue, Span)> = Vec::new();
            for (k, v, span) in members {
                let new_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", path, k)
                };
                let unified_v = unify_tree_inner(v, &new_path, root, false)?;
                if let Some(i) = indices.get(k).copied() {
                    let prev = out[i].1.clone();
                    let merged = unify_spanned(&prev, &unified_v, &new_path, root)?;
                    out[i].1 = merged.clone();
                    if is_root {
                        root.insert(k.clone(), merged);
                    }
                } else {
                    indices.insert(k.clone(), out.len());
                    out.push((k.clone(), unified_v.clone(), *span));
                    if is_root {
                        root.insert(k.clone(), unified_v);
                    }
                }
            }
            Ok(SpannedValue {
                span: value.span,
                kind: ValueKind::Object(out),
            })
        }
        _ => Ok(value.clone()),
    }
}

pub fn unify_tree(value: &SpannedValue) -> Result<SpannedValue, UnifyError> {
    let mut root: BTreeMap<String, SpannedValue> = BTreeMap::new();
    let unified = unify_tree_inner(value, "", &mut root, true)?;
    resolve_refs(&unified, "", &root)
}

fn resolve_refs(
    value: &SpannedValue,
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
) -> Result<SpannedValue, UnifyError> {
    match &value.kind {
        ValueKind::Reference(p) => match lookup(root, p) {
            Some(v) => resolve_refs(v, path, root),
            None => Err(UnifyError {
                msg: add_path(path, format!("unresolved reference {}", p)),
                span: value.span,
                prev_span: value.span,
            }),
        },
        ValueKind::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                out.push(resolve_refs(item, path, root)?);
            }
            Ok(SpannedValue {
                span: value.span,
                kind: ValueKind::Array(out),
            })
        }
        ValueKind::Object(members) => {
            let mut out = Vec::new();
            for (k, v, span) in members {
                let new_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", path, k)
                };
                let resolved = resolve_refs(v, &new_path, root)?;
                out.push((k.clone(), resolved, *span));
            }
            Ok(SpannedValue {
                span: value.span,
                kind: ValueKind::Object(out),
            })
        }
        _ => Ok(value.clone()),
    }
}

fn value_to_kind(j: Value) -> ValueKind {
    match j {
        Value::Null => ValueKind::Null,
        Value::Bool(b) => ValueKind::Bool(b),
        Value::Int(n) => ValueKind::Int(n),
        Value::Float(n) => ValueKind::Float(n),
        Value::String(s) => ValueKind::String(s),
        Value::Array(arr) => ValueKind::Array(
            arr.into_iter()
                .map(|v| SpannedValue {
                    span: SimpleSpan::new((), 0..0),
                    kind: value_to_kind(v),
                })
                .collect(),
        ),
        Value::Object(obj) => ValueKind::Object(
            obj.into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        SpannedValue {
                            span: SimpleSpan::new((), 0..0),
                            kind: value_to_kind(v),
                        },
                        SimpleSpan::new((), 0..0),
                    )
                })
                .collect(),
        ),
        Value::Reference(r) => ValueKind::Reference(r),
        Value::Type(t) => ValueKind::Type(t),
    }
}

fn kind_to_value(k: &ValueKind) -> Value {
    match k {
        ValueKind::Null => Value::Null,
        ValueKind::Bool(b) => Value::Bool(*b),
        ValueKind::Int(n) => Value::Int(*n),
        ValueKind::Float(n) => Value::Float(*n),
        ValueKind::String(s) => Value::String(s.clone()),
        ValueKind::Array(arr) => Value::Array(arr.iter().map(|v| v.to_value()).collect()),
        ValueKind::Object(obj) => Value::Object(
            obj.iter()
                .map(|(k, v, _)| (k.clone(), v.to_value()))
                .collect(),
        ),
        ValueKind::Reference(r) => Value::Reference(r.clone()),
        ValueKind::Type(t) => Value::Type(t.clone()),
    }
}

pub fn unify_with_path(a: &Value, b: &Value, path: &str) -> Result<Value, String> {
    if a == b {
        return Ok(a.clone());
    }
    match (a, b) {
        (Value::Type(ta), Value::Type(tb)) => unify_types(ta, tb)
            .map(Value::Type)
            .map_err(|e| add_path(path, e)),
        (Value::Type(t), val) | (val, Value::Type(t)) => {
            unify_type_value(t, val).map_err(|e| add_path(path, e))
        }
        (Value::Array(a_items), Value::Array(b_items)) => {
            if a_items.len() != b_items.len() {
                return Err(add_path(path, "array lengths differ".into()));
            }
            let mut out = Vec::new();
            for (i, (av, bv)) in a_items.iter().zip(b_items.iter()).enumerate() {
                let new_path = if path.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", path, i)
                };
                let unified = unify_with_path(av, bv, &new_path)?;
                out.push(unified);
            }
            Ok(Value::Array(out))
        }
        (Value::Object(a_members), Value::Object(b_members)) => {
            use std::collections::BTreeMap;
            let mut map: BTreeMap<String, Value> = BTreeMap::new();
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
            Ok(Value::Object(map.into_iter().collect()))
        }
        _ => Err(add_path(path, "values do not unify".into())),
    }
}
