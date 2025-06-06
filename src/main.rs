use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
type Span = SimpleSpan<usize>;
use ariadne::{Color, Label, Report, ReportKind, sources};
use std::{env, fs};

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
    Type(JsonType),
}

#[derive(Debug, Clone, PartialEq)]
struct SpannedJson {
    span: Span,
    kind: SpannedKind,
}

#[derive(Debug, Clone, PartialEq)]
enum SpannedKind {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<SpannedJson>),
    Object(Vec<(String, SpannedJson, Span)>),
    Type(JsonType),
}

impl SpannedJson {
    fn to_json(&self) -> Json {
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

struct UnifyError {
    msg: String,
    span: Span,
    prev_span: Span,
}

#[derive(Debug, Clone, PartialEq)]
enum JsonType {
    Any,
    Nothing,
    Int,
    Number,
    Rational,
    Float,
    String,
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

fn unify(a: &Json, b: &Json) -> Result<Json, String> {
    unify_with_path(a, b, "")
}

fn add_path(path: &str, msg: String) -> String {
    if path.is_empty() {
        msg
    } else {
        format!("{}: {}", path, msg)
    }
}

fn unify_spanned(a: &SpannedJson, b: &SpannedJson, path: &str) -> Result<SpannedJson, UnifyError> {
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

fn unify_with_path(a: &Json, b: &Json, path: &str) -> Result<Json, String> {
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

fn parser<'a>() -> impl Parser<'a, &'a str, Json, extra::Err<Rich<'a, char>>> {
    spanned_value().map(|v| v.to_json())
}

fn spanned_value<'a>() -> impl Parser<'a, &'a str, SpannedJson, extra::Err<Rich<'a, char>>> {
    recursive(|value| {
        let comment = just('#')
            .then(none_of('\n').repeated())
            .then_ignore(text::newline().or_not())
            .ignored();
        let ws = choice((text::whitespace().at_least(1).ignored(), comment.clone()))
            .repeated()
            .ignored();
        let digits = text::digits(10);
        let int = text::int(10);

        let number = just('-')
            .or_not()
            .then(int)
            .then(just('.').then(digits.clone()).or_not())
            .then(
                one_of("eE")
                    .then(one_of("+-").or_not())
                    .then(digits.clone())
                    .or_not(),
            )
            .to_slice()
            .map_with(|s: &str, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Number(s.parse().unwrap()),
            });

        let escape = just('\\').ignore_then(choice((
            just('\\'),
            just('/'),
            just('"'),
            just('b').to('\x08'),
            just('f').to('\x0c'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('u').ignore_then(text::digits(16).exactly(4).to_slice().map(|digits: &str| {
                char::from_u32(u32::from_str_radix(digits, 16).unwrap()).unwrap()
            })),
        )));

        let string = none_of("\\\"")
            .or(escape)
            .repeated()
            .collect::<String>()
            .delimited_by(just('"'), just('"'))
            .map_with(|s: String, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::String(s),
            });

        let array = value
            .clone()
            .separated_by(just(',').padded_by(ws.clone()))
            .allow_trailing()
            .collect()
            .delimited_by(
                just('[').padded_by(ws.clone()),
                just(']').padded_by(ws.clone()),
            )
            .map_with(|vals, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Array(vals),
            });

        let key_string = string.clone().map(|j| {
            if let SpannedJson {
                kind: SpannedKind::String(s),
                ..
            } = j
            {
                s
            } else {
                unreachable!()
            }
        });
        let key = key_string.or(text::ident().map(|s: &str| s.to_string()));

        let key_span = key.clone().map_with(|k: String, e| (k, e.span()));

        let member = key_span
            .then_ignore(just(':').padded_by(ws.clone()))
            .then(value.clone())
            .map(|((k, k_span), mut v): ((String, Span), SpannedJson)| {
                let span = SimpleSpan::new((), k_span.start()..v.span.end());
                v.span = span;
                (k, v, span)
            });
        let object = member
            .separated_by(just(',').padded_by(ws.clone()))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just('{').padded_by(ws.clone()),
                just('}').padded_by(ws.clone()),
            )
            .validate(|members: Vec<(String, SpannedJson, Span)>, _extra, emit| {
                use chumsky::error::LabelError;
                use std::collections::hash_map::HashMap;
                let mut seen: HashMap<String, SpannedJson> = HashMap::new();
                let mut out: Vec<(String, SpannedJson, Span)> = Vec::new();
                for (k, v, span) in members {
                    match seen.get(k.as_str()) {
                        Some(prev) => match unify_spanned(prev, &v, &k) {
                            Ok(unified) => {
                                seen.insert(k.clone(), unified);
                                out.push((k, v, span));
                            }
                            Err(err) => {
                                let mut e = Rich::custom(
                                    err.span.clone(),
                                    format!(
                                        "duplicate key '{}' could not be unified: {}",
                                        k, err.msg
                                    ),
                                );
                                <Rich<_> as LabelError<&str, _>>::in_context(
                                    &mut e,
                                    "previous value here",
                                    err.prev_span.clone(),
                                );
                                emit.emit(e);
                            }
                        },
                        None => {
                            seen.insert(k.clone(), v.clone());
                            out.push((k, v, span));
                        }
                    }
                }
                out
            })
            .map_with(|members, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Object(members),
            });

        choice((
            just("null").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Null,
            }),
            just("true").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Bool(true),
            }),
            just("false").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Bool(false),
            }),
            just("Any").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Type(JsonType::Any),
            }),
            just("Nothing").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Type(JsonType::Nothing),
            }),
            just("Int").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Type(JsonType::Int),
            }),
            just("Number").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Type(JsonType::Number),
            }),
            just("Rational").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Type(JsonType::Rational),
            }),
            just("Float").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Type(JsonType::Float),
            }),
            just("String").map_with(|_, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Type(JsonType::String),
            }),
            number,
            string,
            array,
            object,
        ))
        .padded_by(ws)
    })
}

fn main() {
    let filename = env::args().nth(1).expect("expected file argument");
    let src = fs::read_to_string(&filename).expect("failed to read file");
    let result = parser().parse(&src).into_result();
    match result {
        Ok(json) => println!("{:#?}", json),
        Err(errs) => {
            for e in errs {
                let span = e.span().clone().into_range();
                let msg = e.to_string();
                Report::build(ReportKind::Error, (filename.clone(), span.clone()))
                    .with_message(&msg)
                    .with_label(
                        Label::new((filename.clone(), span.clone()))
                            .with_message(&msg)
                            .with_color(Color::Red),
                    )
                    .with_labels(e.contexts().map(|(label, span)| {
                        Label::new((filename.clone(), span.into_range()))
                            .with_message(label.to_string())
                            .with_color(Color::Yellow)
                    }))
                    .finish()
                    .print(sources([(filename.clone(), src.as_str())]))
                    .unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_with_trailing_comma_and_comments() {
        let src = r#"
            [
                1,
                2,
                3,# comment
            ]
        "#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert_eq!(
            parsed,
            Json::Array(vec![
                Json::Number(1.0),
                Json::Number(2.0),
                Json::Number(3.0)
            ])
        );
    }

    #[test]
    fn object_with_trailing_comma_and_comments() {
        let src = r#"
            {
                # leading comment
                "a": true,
                "b": [false,],
            }
        "#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert_eq!(
            parsed,
            Json::Object(vec![
                ("a".into(), Json::Bool(true)),
                ("b".into(), Json::Array(vec![Json::Bool(false)])),
            ])
        );
    }

    #[test]
    fn object_with_unquoted_keys() {
        let src = r#"
            {
                a: 1,
                b: { c: 2, },
            }
        "#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert_eq!(
            parsed,
            Json::Object(vec![
                ("a".into(), Json::Number(1.0)),
                (
                    "b".into(),
                    Json::Object(vec![("c".into(), Json::Number(2.0))]),
                ),
            ])
        );
    }

    #[test]
    fn object_with_duplicate_keys_equal() {
        let src = r#"{ "a": 1, "a": 1 }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert_eq!(
            parsed,
            Json::Object(vec![
                ("a".into(), Json::Number(1.0)),
                ("a".into(), Json::Number(1.0)),
            ])
        );
    }

    #[test]
    fn object_with_duplicate_keys_different() {
        let src = r#"{ "a": 1, "a": 2 }"#;
        assert!(parser().parse(src).into_result().is_err());
    }

    #[test]
    fn unify_type_with_value() {
        let src = r#"{ "a": Int, "a": 1 }"#;
        assert!(parser().parse(src).into_result().is_ok());
    }

    #[test]
    fn unify_type_with_incompatible_value() {
        let src = r#"{ "a": Int, "a": 1.1 }"#;
        assert!(parser().parse(src).into_result().is_err());
    }

    #[test]
    fn unify_any_with_value() {
        let src = r#"{ "a": Any, "a": 1 }"#;
        assert!(parser().parse(src).into_result().is_ok());
    }

    #[test]
    fn unify_nothing_with_value() {
        let src = r#"{ "a": Nothing, "a": 1 }"#;
        assert!(parser().parse(src).into_result().is_err());
    }

    #[test]
    fn unify_number_hierarchy() {
        let src = r#"{ "a": Int, "a": Float }"#;
        assert!(parser().parse(src).into_result().is_ok());
    }

    #[test]
    fn unify_string_with_value() {
        let src = r#"{ "a": String, "a": "hi" }"#;
        assert!(parser().parse(src).into_result().is_ok());
    }

    #[test]
    fn unify_recursive_object() {
        let src = r#"
            {
                foo: { bar: Int },
                foo: { bar: 3 },
            }
        "#;
        assert!(parser().parse(src).into_result().is_ok());
    }

    #[test]
    fn object_union_of_keys_parses() {
        let src = r#"
            {
                foo: { bar: 1 },
                foo: { baz: 2 },
            }
        "#;
        assert!(parser().parse(src).into_result().is_ok());
    }

    #[test]
    fn unify_object_union_of_keys() {
        let a = Json::Object(vec![(
            "foo".into(),
            Json::Object(vec![("bar".into(), Json::Number(1.0))]),
        )]);
        let b = Json::Object(vec![(
            "foo".into(),
            Json::Object(vec![("baz".into(), Json::Number(2.0))]),
        )]);
        let unified = unify(&a, &b).unwrap();
        let expected = Json::Object(vec![(
            "foo".into(),
            Json::Object(vec![
                ("bar".into(), Json::Number(1.0)),
                ("baz".into(), Json::Number(2.0)),
            ]),
        )]);
        assert_eq!(unified, expected);
    }

    #[test]
    fn duplicate_key_error_details() {
        let src = "{\n    hello: Int,\n    hello: String,\n}";
        let result = parser().parse(src).into_result();
        match result {
            Ok(_) => panic!("expected error"),
            Err(errs) => {
                assert!(!errs.is_empty());
                let err = &errs[0];
                let msg = err.to_string();
                assert!(msg.contains("Int"));
                assert!(msg.contains("String"));
                let span = err.span().clone().into_range();
                let prev_span = err.contexts().next().unwrap().1.into_range();
                let line_for = |i: usize| src[..i].chars().filter(|&c| c == '\n').count() + 1;
                assert_eq!(line_for(span.start), 3);
                assert_eq!(line_for(prev_span.start), 2);
            }
        }
    }

    #[test]
    fn nested_unify_error_mentions_bar() {
        let src = r#"
            {
                foo: { bar: String },
                foo: { bar: Int },
            }
        "#;
        let result = parser().parse(src).into_result();
        match result {
            Ok(_) => panic!("expected error"),
            Err(errs) => {
                assert!(!errs.is_empty());
                let err = &errs[0];
                let msg = err.to_string();
                assert!(msg.contains("bar"));
                let span = err.span().clone().into_range();
                let prev_span = err.contexts().next().unwrap().1.into_range();
                assert!(src[span.start..span.end].contains("bar"));
                assert!(src[prev_span.start..prev_span.end].contains("bar"));
            }
        }
    }

    #[test]
    fn conflicting_nested_duplicates_fail() {
        let src = r#"
            {
                foo: { bar: 1 },
                foo: { baz: 2 },
                foo: { baz: String },
            }
        "#;
        assert!(parser().parse(src).into_result().is_err());
    }
}
