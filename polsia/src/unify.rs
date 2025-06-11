use crate::types::{Span, SpannedValue, ValType, Value, ValueKind};
use chumsky::span::{SimpleSpan, Span as ChumSpan};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct UnifyError {
    pub msg: String,
    pub span: Span,
    pub prev_span: Span,
}

fn branch_matches(
    branch: &SpannedValue,
    value: &SpannedValue,
    root: &BTreeMap<String, SpannedValue>,
) -> bool {
    let branch_kind = match &branch.kind {
        ValueKind::Reference(p) => lookup(root, p).map(|v| &v.kind),
        _ => Some(&branch.kind),
    };
    let value_kind = match &value.kind {
        ValueKind::Reference(p) => lookup(root, p).map(|v| &v.kind),
        _ => Some(&value.kind),
    };
    match (branch_kind, value_kind) {
        (Some(ValueKind::Object(bm)), Some(ValueKind::Object(vm))) => vm
            .iter()
            .all(|(vk, _, _)| bm.iter().any(|(k, _, _)| k == vk)),
        (Some(ValueKind::Type(bt)), Some(ValueKind::Type(vt))) => bt == vt,
        _ => true,
    }
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

// Helper utilities used by both spanned and plain unification implementations

fn unify_array_spanned(
    a_items: &[SpannedValue],
    b_items: &[SpannedValue],
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
    seen: &mut std::collections::HashSet<String>,
    span: Span,
) -> Result<SpannedValue, UnifyError> {
    if a_items.len() != b_items.len() {
        return Err(UnifyError {
            msg: add_path(path, "array lengths differ".into()),
            span,
            prev_span: span,
        });
    }
    let mut out = Vec::new();
    for (i, (av, bv)) in a_items.iter().zip(b_items.iter()).enumerate() {
        let new_path = if path.is_empty() {
            format!("[{}]", i)
        } else {
            format!("{}[{}]", path, i)
        };
        out.push(unify_spanned_inner(av, bv, &new_path, root, seen)?);
    }
    Ok(SpannedValue {
        span,
        kind: ValueKind::Array(out),
    })
}

fn unify_array_plain(a_items: &[Value], b_items: &[Value], path: &str) -> Result<Value, String> {
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
        out.push(unify_with_path(av, bv, &new_path)?);
    }
    Ok(Value::Array(out))
}

fn unify_object_spanned(
    a_members: &[(String, SpannedValue, Span)],
    b_members: &[(String, SpannedValue, Span)],
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
    seen: &mut std::collections::HashSet<String>,
    span: Span,
) -> Result<SpannedValue, UnifyError> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, SpannedValue> = BTreeMap::new();
    for (k, v, _) in a_members {
        map.insert(k.clone(), v.clone());
    }
    for (k, v, _) in b_members {
        let new = if let Some(prev) = map.get(k) {
            let new_path = if path.is_empty() {
                k.clone()
            } else {
                format!("{}.{}", path, k)
            };
            unify_spanned_inner(prev, v, &new_path, root, seen)?
        } else {
            v.clone()
        };
        map.insert(k.clone(), new);
    }
    let members = map
        .into_iter()
        .map(|(k, v)| {
            let span = v.span;
            (k, v, span)
        })
        .collect();
    Ok(SpannedValue {
        span,
        kind: ValueKind::Object(members),
    })
}

fn unify_object_plain(
    a_members: &[(String, Value)],
    b_members: &[(String, Value)],
    path: &str,
) -> Result<Value, String> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, Value> = BTreeMap::new();
    for (k, v) in a_members {
        map.insert(k.clone(), v.clone());
    }
    for (k, v) in b_members {
        let new = if let Some(prev) = map.get(k) {
            let new_path = if path.is_empty() {
                k.clone()
            } else {
                format!("{}.{}", path, k)
            };
            unify_with_path(prev, v, &new_path)?
        } else {
            v.clone()
        };
        map.insert(k.clone(), new);
    }
    Ok(Value::Object(map.into_iter().collect()))
}

fn unify_union_pairs_spanned(
    a_opts: &[SpannedValue],
    b_opts: &[SpannedValue],
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
    seen: &mut std::collections::HashSet<String>,
    span: Span,
    prev_span: Span,
) -> Result<SpannedValue, UnifyError> {
    let mut results = Vec::new();
    for ao in a_opts {
        for bo in b_opts {
            if branch_matches(ao, bo, root) && branch_matches(bo, ao, root) {
                if let Ok(res) = unify_spanned_inner(ao, bo, path, root, seen) {
                    results.push(res);
                }
            }
        }
    }
    if results.is_empty() {
        Err(UnifyError {
            msg: add_path(path, "values do not unify".into()),
            span,
            prev_span,
        })
    } else if results.len() == 1 {
        Ok(results.pop().unwrap())
    } else {
        Ok(SpannedValue {
            span,
            kind: ValueKind::Union(results),
        })
    }
}

fn unify_union_against_spanned(
    opts: &[SpannedValue],
    other: &SpannedValue,
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
    seen: &mut std::collections::HashSet<String>,
    span: Span,
    prev_span: Span,
) -> Result<SpannedValue, UnifyError> {
    let mut results = Vec::new();
    for o in opts {
        if branch_matches(o, other, root) {
            let res = unify_spanned_inner(o, other, path, root, seen);
            if let Ok(res) = res {
                results.push(res);
            }
        }
    }
    if results.is_empty() {
        Err(UnifyError {
            msg: add_path(path, "values do not unify".into()),
            span,
            prev_span,
        })
    } else if results.len() == 1 {
        Ok(results.pop().unwrap())
    } else {
        Ok(SpannedValue {
            span,
            kind: ValueKind::Union(results),
        })
    }
}

fn unify_union_pairs_plain(
    a_opts: &[Value],
    b_opts: &[Value],
    path: &str,
) -> Result<Value, String> {
    let mut results = Vec::new();
    for ao in a_opts {
        for bo in b_opts {
            if let Ok(res) = unify_with_path(ao, bo, path) {
                results.push(res);
            }
        }
    }
    if results.is_empty() {
        Err(add_path(path, "values do not unify".into()))
    } else if results.len() == 1 {
        Ok(results.pop().unwrap())
    } else {
        Ok(Value::Union(results))
    }
}

fn unify_union_against_plain(opts: &[Value], other: &Value, path: &str) -> Result<Value, String> {
    let mut results = Vec::new();
    for o in opts {
        if matches!((o, other), (Value::Type(bt), Value::Type(vt)) if bt != vt) {
            continue;
        }
        let res = unify_with_path(o, other, path);
        if let Ok(res) = res {
            results.push(res);
        }
    }
    if results.is_empty() {
        Err(add_path(path, "values do not unify".into()))
    } else if results.len() == 1 {
        Ok(results.pop().unwrap())
    } else {
        Ok(Value::Union(results))
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

use std::collections::HashSet;

pub fn unify_spanned(
    a: &SpannedValue,
    b: &SpannedValue,
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
) -> Result<SpannedValue, UnifyError> {
    let mut seen = HashSet::new();
    unify_spanned_inner(a, b, path, root, &mut seen)
}

fn unify_spanned_inner(
    a: &SpannedValue,
    b: &SpannedValue,
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
    seen: &mut HashSet<String>,
) -> Result<SpannedValue, UnifyError> {
    if a.to_value() == b.to_value() {
        return Ok(b.clone());
    }
    match (&a.kind, &b.kind) {
        (ValueKind::Reference(pa), _) => {
            if !seen.insert(pa.clone()) {
                return Ok(b.clone());
            }
            let res = match lookup(root, pa) {
                Some(val) => unify_spanned_inner(val, b, path, root, seen),
                None => Err(UnifyError {
                    msg: add_path(path, format!("unresolved reference {}", pa)),
                    span: b.span,
                    prev_span: a.span,
                }),
            };
            seen.remove(pa);
            res
        }
        (_, ValueKind::Reference(pb)) => {
            if !seen.insert(pb.clone()) {
                return Ok(a.clone());
            }
            let res = match lookup(root, pb) {
                Some(val) => unify_spanned_inner(a, val, path, root, seen),
                None => Err(UnifyError {
                    msg: add_path(path, format!("unresolved reference {}", pb)),
                    span: b.span,
                    prev_span: a.span,
                }),
            };
            seen.remove(pb);
            res
        }
        (ValueKind::Union(a_opts), ValueKind::Union(b_opts)) => {
            unify_union_pairs_spanned(a_opts, b_opts, path, root, seen, b.span, a.span)
        }
        (ValueKind::Union(opts), _) | (_, ValueKind::Union(opts)) => {
            let other = if matches!(&a.kind, ValueKind::Union(_)) {
                b
            } else {
                a
            };
            unify_union_against_spanned(opts, other, path, root, seen, b.span, a.span)
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
        (ValueKind::Type(t), other) | (other, ValueKind::Type(t)) => {
            let is_a_type = matches!(&a.kind, ValueKind::Type(_));
            let span = if is_a_type { b.span } else { a.span };
            match unify_type_value(t, &kind_to_value(other)) {
                Ok(j) => Ok(SpannedValue {
                    span,
                    kind: value_to_kind(j),
                }),
                Err(e) => Err(UnifyError {
                    msg: add_path(path, e),
                    span,
                    prev_span: a.span,
                }),
            }
        }
        (ValueKind::Array(a_items), ValueKind::Array(b_items)) => {
            unify_array_spanned(a_items, b_items, path, root, seen, b.span)
        }
        (ValueKind::Object(a_members), ValueKind::Object(b_members)) => {
            unify_object_spanned(a_members, b_members, path, root, seen, b.span)
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
        ValueKind::Union(items) => {
            let mut out = Vec::new();
            for item in items {
                out.push(unify_tree_inner(item, path, root, false)?);
            }
            Ok(SpannedValue {
                span: value.span,
                kind: ValueKind::Union(out),
            })
        }
        ValueKind::Object(members) => {
            use std::collections::HashMap;
            // Preserve the order keys first appear for stable output. Unification
            // itself must not depend on field order.
            let mut indices: HashMap<String, usize> = HashMap::new();
            let mut out: Vec<(String, SpannedValue, Span)> = Vec::new();
            let mut all_values: HashMap<String, Vec<SpannedValue>> = HashMap::new();

            for (k, v, span) in members {
                let new_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", path, k)
                };
                let unified_v = unify_tree_inner(v, &new_path, root, false)?;
                all_values
                    .entry(k.clone())
                    .or_default()
                    .push(unified_v.clone());
                if let Some(&i) = indices.get(k) {
                    // already recorded first occurrence
                    let _ = i; // suppress unused warning in some compilers
                } else {
                    indices.insert(k.clone(), out.len());
                    out.push((k.clone(), unified_v.clone(), *span));
                    if is_root {
                        root.insert(k.clone(), unified_v.clone());
                    }
                }
            }

            // Repeatedly unify duplicates until results stabilize so that
            // reference resolution does not depend on ordering.
            let mut changed = true;
            while changed {
                changed = false;
                for (k, values) in &all_values {
                    let i = indices[k];
                    let entry_path = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", path, k)
                    };
                    let mut current = values[0].clone();
                    for v in &values[1..] {
                        current = unify_spanned(&current, v, &entry_path, root)?;
                    }
                    if current.to_value() != out[i].1.to_value() {
                        out[i].1 = current.clone();
                        changed = true;
                    }
                    if is_root {
                        root.insert(k.clone(), current);
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
    let mut seen = HashSet::new();
    resolve_refs_inner(value, path, root, &mut seen)
}

fn resolve_refs_inner(
    value: &SpannedValue,
    path: &str,
    root: &BTreeMap<String, SpannedValue>,
    seen: &mut HashSet<String>,
) -> Result<SpannedValue, UnifyError> {
    match &value.kind {
        ValueKind::Reference(p) => match lookup(root, p) {
            Some(v) => {
                if !seen.insert(p.clone()) {
                    if !matches!(v.kind, ValueKind::Reference(_)) {
                        return Err(UnifyError {
                            msg: add_path(path, "infinite structural cycle".into()),
                            span: value.span,
                            prev_span: value.span,
                        });
                    }
                    return Ok(value.clone());
                }
                let res = resolve_refs_inner(v, path, root, seen);
                seen.remove(p);
                res
            }
            None => Err(UnifyError {
                msg: add_path(path, format!("unresolved reference {}", p)),
                span: value.span,
                prev_span: value.span,
            }),
        },
        ValueKind::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                out.push(resolve_refs_inner(item, path, root, seen)?);
            }
            Ok(SpannedValue {
                span: value.span,
                kind: ValueKind::Array(out),
            })
        }
        ValueKind::Union(items) => {
            let mut out = Vec::new();
            for item in items {
                out.push(resolve_refs_inner(item, path, root, seen)?);
            }
            Ok(SpannedValue {
                span: value.span,
                kind: ValueKind::Union(out),
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
                let resolved = resolve_refs_inner(v, &new_path, root, seen)?;
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
        Value::Union(items) => ValueKind::Union(
            items
                .into_iter()
                .map(|v| SpannedValue {
                    span: SimpleSpan::new((), 0..0),
                    kind: value_to_kind(v),
                })
                .collect(),
        ),
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
        ValueKind::Union(items) => Value::Union(items.iter().map(|v| v.to_value()).collect()),
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
        (Value::Array(a_items), Value::Array(b_items)) => unify_array_plain(a_items, b_items, path),
        (Value::Object(a_members), Value::Object(b_members)) => {
            unify_object_plain(a_members, b_members, path)
        }
        (Value::Union(a_opts), Value::Union(b_opts)) => {
            unify_union_pairs_plain(a_opts, b_opts, path)
        }
        (Value::Union(opts), other) | (other, Value::Union(opts)) => {
            unify_union_against_plain(opts, other, path)
        }
        _ => Err(add_path(path, "values do not unify".into())),
    }
}
