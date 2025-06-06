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
            .map(|s: &str| Json::Number(s.parse().unwrap()));

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
            .map(Json::String);

        let array = value
            .clone()
            .separated_by(just(',').padded_by(ws.clone()))
            .allow_trailing()
            .collect()
            .delimited_by(
                just('[').padded_by(ws.clone()),
                just(']').padded_by(ws.clone()),
            )
            .map(Json::Array);

        let key = string
            .clone()
            .map(|j| {
                if let Json::String(s) = j {
                    s
                } else {
                    unreachable!()
                }
            })
            .or(text::ident().map(|s: &str| s.to_string()));

        let member = key
            .then_ignore(just(':').padded_by(ws.clone()))
            .then(value.clone().map_with(|v, e| (v, e.span())))
            .map(|(k, (v, span))| (k, v, span));
        let object = member
            .separated_by(just(',').padded_by(ws.clone()))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just('{').padded_by(ws.clone()),
                just('}').padded_by(ws.clone()),
            )
            .validate(|members: Vec<(String, Json, Span)>, _extra, emit| {
                use chumsky::error::LabelError;
                use std::collections::hash_map::HashMap;
                let mut seen: HashMap<&str, (Json, Span)> = HashMap::new();
                for (k, v, span) in &members {
                    match seen.get(k.as_str()) {
                        Some((prev, prev_span)) => {
                            if let Err(msg) = unify_with_path(prev, v, k) {
                                let mut err = Rich::custom(
                                    span.clone(),
                                    format!("duplicate key '{}' could not be unified: {}", k, msg),
                                );
                                <Rich<_> as LabelError<&str, _>>::in_context(
                                    &mut err,
                                    "previous value here",
                                    prev_span.clone(),
                                );
                                emit.emit(err);
                            }
                        }
                        None => {
                            seen.insert(k.as_str(), (v.clone(), span.clone()));
                        }
                    }
                }
                Json::Object(members.into_iter().map(|(k, v, _)| (k, v)).collect())
            });

        choice((
            just("null").to(Json::Null),
            just("true").to(Json::Bool(true)),
            just("false").to(Json::Bool(false)),
            just("Any").to(Json::Type(JsonType::Any)),
            just("Nothing").to(Json::Type(JsonType::Nothing)),
            just("Int").to(Json::Type(JsonType::Int)),
            just("Number").to(Json::Type(JsonType::Number)),
            just("Rational").to(Json::Type(JsonType::Rational)),
            just("Float").to(Json::Type(JsonType::Float)),
            just("String").to(Json::Type(JsonType::String)),
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
                let msg = errs[0].to_string();
                assert!(msg.contains("bar"));
            }
        }
    }
}
