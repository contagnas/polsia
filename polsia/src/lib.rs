pub mod parser;
pub mod types;
pub mod unify;

pub use parser::{document, parser};
pub use types::{Annotation, Document, SpannedValue, ValType, Value, ValueKind};
pub use unify::{UnifyError, unify_spanned, unify_tree};

use crate::types::Span;
use ariadne::{Color, Config, Label, Report, ReportKind, sources};
use chumsky::prelude::*;

pub fn parse_to_json(src: &str) -> Result<String, String> {
    let filename = "input".to_string();
    let parse_result = document().parse(src).into_result();
    match parse_result {
        Ok(doc) => match unify_tree(&doc.value) {
            Ok(value) => match find_unresolved(&value) {
                Some((span, t)) => {
                    let msg = format!("value of type {} is unspecified", t);
                    let span_range = span.into_range();
                    let mut buf = Vec::new();
                    Report::build(ReportKind::Error, (filename.clone(), span_range.clone()))
                        .with_config(Config::default().with_color(false))
                        .with_message(&msg)
                        .with_label(
                            Label::new((filename.clone(), span_range))
                                .with_message(&msg)
                                .with_color(Color::Red),
                        )
                        .finish()
                        .write(sources([(filename.clone(), src)]), &mut buf)
                        .unwrap();
                    Err(String::from_utf8(buf).unwrap())
                }
                None => Ok(value.to_value().to_pretty_string()),
            },
            Err(err) => {
                use chumsky::error::LabelError;
                let mut e = Rich::custom(err.span, err.msg.clone());
                <Rich<_> as LabelError<&str, _>>::in_context(
                    &mut e,
                    "previous value here",
                    err.prev_span,
                );
                let span = (*e.span()).into_range();
                let msg = e.to_string();
                let mut buf = Vec::new();
                Report::build(ReportKind::Error, (filename.clone(), span.clone()))
                    .with_config(Config::default().with_color(false))
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
                    .write(sources([(filename.clone(), src)]), &mut buf)
                    .unwrap();
                Err(String::from_utf8(buf).unwrap())
            }
        },
        Err(errs) => {
            let mut out = String::new();
            for e in errs {
                let span = (*e.span()).into_range();
                let msg = e.to_string();
                let mut buf = Vec::new();
                Report::build(ReportKind::Error, (filename.clone(), span.clone()))
                    .with_config(Config::default().with_color(false))
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
                    .write(sources([(filename.clone(), src)]), &mut buf)
                    .unwrap();
                out.push_str(&String::from_utf8(buf).unwrap());
            }
            Err(out)
        }
    }
}

fn find_unresolved(value: &SpannedValue) -> Option<(Span, String)> {
    match &value.kind {
        ValueKind::Reference(p) => Some((value.span, format!("reference {}", p))),
        ValueKind::Type(t) => Some((value.span, format!("{:?}", t))),
        ValueKind::Call(name, _) => Some((value.span, format!("call {}", name))),
        ValueKind::Union(items) => {
            for item in items {
                if let Some(res) = find_unresolved(item) {
                    return Some(res);
                }
            }
            Some((value.span, "union".into()))
        }
        ValueKind::Array(items) => {
            for item in items {
                if let Some(res) = find_unresolved(item) {
                    return Some(res);
                }
            }
            None
        }
        ValueKind::Object(members) => {
            for (_, v, _, anns) in members {
                if anns.contains(&Annotation::NoExport) {
                    continue;
                }
                if let Some(res) = find_unresolved(v) {
                    return Some(res);
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::polsia_to_json;

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_unify(src: &str) -> Result<SpannedValue, UnifyError> {
        let parsed = parser().parse(src).into_result().unwrap();
        unify_tree(&parsed)
    }

    fn must_unify(src: &str) -> SpannedValue {
        parse_unify(src).unwrap()
    }

    fn must_err(src: &str) {
        assert!(parse_unify(src).is_err());
    }

    fn span_value(value: Value) -> SpannedValue {
        use Value::*;
        let span = SimpleSpan::new((), 0..0);
        SpannedValue {
            span,
            kind: match value {
                Null => ValueKind::Null,
                Bool(b) => ValueKind::Bool(b),
                Int(n) => ValueKind::Int(n),
                Float(n) => ValueKind::Float(n),
                String(s) => ValueKind::String(s),
                Array(items) => ValueKind::Array(items.into_iter().map(span_value).collect()),
                Object(members) => ValueKind::Object(
                    members
                        .into_iter()
                        .map(|(k, v)| (k, span_value(v), span, Vec::new()))
                        .collect(),
                ),
                Reference(r) => ValueKind::Reference(r),
                Type(t) => ValueKind::Type(t),
                Call(name, arg) => ValueKind::Call(name, Box::new(span_value(*arg))),
                Union(items) => ValueKind::Union(items.into_iter().map(span_value).collect()),
            },
        }
    }

    #[test]
    fn array_with_trailing_comma_and_comments() {
        let src = r#"
            [
                1,
                2,
                3,# comment
            ]
        "#;
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
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
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("a".into(), Value::Bool(true)),
                ("b".into(), Value::Array(vec![Value::Bool(false)])),
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
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("a".into(), Value::Int(1)),
                ("b".into(), Value::Object(vec![("c".into(), Value::Int(2))]),),
            ])
        );
    }

    #[test]
    fn object_with_duplicate_keys_equal() {
        let src = r#"{ "a": 1, "a": 1 }"#;
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![("a".into(), Value::Int(1))])
        );
    }

    #[test]
    fn object_with_duplicate_keys_different() {
        let src = r#"{ "a": 1, "a": 2 }"#;
        must_err(src);
    }

    #[test]
    fn unify_type_with_value() {
        let src = r#"{ "a": Int, "a": 1 }"#;
        must_unify(src);
    }

    #[test]
    fn unify_type_with_incompatible_value() {
        let src = r#"{ "a": Int, "a": 1.1 }"#;
        must_err(src);
    }

    #[test]
    fn unify_any_with_value() {
        let src = r#"{ "a": Any, "a": 1 }"#;
        must_unify(src);
    }

    #[test]
    fn unify_nothing_with_value() {
        let src = r#"{ "a": Nothing, "a": 1 }"#;
        must_err(src);
    }

    #[test]
    fn unify_number_hierarchy() {
        let src = r#"{ "a": Int, "a": Float }"#;
        must_unify(src);
    }

    #[test]
    fn unify_string_with_value() {
        let src = r#"{ "a": String, "a": "hi" }"#;
        must_unify(src);
    }

    #[test]
    fn unify_recursive_object() {
        let src = r#"
            {
                foo: { bar: Int },
                foo: { bar: 3 },
            }
        "#;
        must_unify(src);
    }

    #[test]
    fn object_union_of_keys_parses() {
        let src = r#"
            {
                foo: { bar: 1 },
                foo: { baz: 2 },
            }
        "#;
        must_unify(src);
    }

    #[test]
    fn unify_object_union_of_keys() {
        use std::collections::BTreeMap;

        let a = Value::Object(vec![(
            "foo".into(),
            Value::Object(vec![("bar".into(), Value::Int(1))]),
        )]);
        let b = Value::Object(vec![(
            "foo".into(),
            Value::Object(vec![("baz".into(), Value::Int(2))]),
        )]);

        let a_sp = span_value(a);
        let b_sp = span_value(b);
        let root = BTreeMap::new();
        let unified = unify_spanned(&a_sp, &b_sp, "", &root).unwrap();
        let expected = Value::Object(vec![(
            "foo".into(),
            Value::Object(vec![
                ("bar".into(), Value::Int(1)),
                ("baz".into(), Value::Int(2)),
            ]),
        )]);
        assert_eq!(unified.to_value(), expected);
    }

    #[test]
    fn duplicate_key_error_details() {
        let src = "{\n    hello: Int,\n    hello: String,\n}";
        match parse_unify(src) {
            Ok(_) => panic!("expected error"),
            Err(err) => {
                use chumsky::error::LabelError;
                let mut e = Rich::custom(err.span, err.msg.clone());
                <Rich<_> as LabelError<&str, _>>::in_context(
                    &mut e,
                    "previous value here",
                    err.prev_span,
                );
                let msg = e.to_string();
                assert!(msg.contains("Int"));
                assert!(msg.contains("String"));
                let span = (*e.span()).into_range();
                let prev_span = e.contexts().next().unwrap().1.into_range();
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
        match parse_unify(src) {
            Ok(_) => panic!("expected error"),
            Err(err) => {
                use chumsky::error::LabelError;
                let mut e = Rich::custom(err.span, err.msg.clone());
                <Rich<_> as LabelError<&str, _>>::in_context(
                    &mut e,
                    "previous value here",
                    err.prev_span,
                );
                let msg = e.to_string();
                assert!(msg.contains("bar"));
                let span = (*e.span()).into_range();
                let prev_span = e.contexts().next().unwrap().1.into_range();
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
        must_err(src);
    }

    #[test]
    fn top_level_braces_optional() {
        let with_braces = "{ foo: 1, bar: 2, }";
        let without_braces = "foo: 1,\nbar: 2,\n";
        let a = must_unify(with_braces);
        let b = must_unify(without_braces);
        assert_eq!(a.to_value(), b.to_value());
    }

    #[test]
    fn parses_multiple_top_objects() {
        let src = "hello: \"world\"\n{hello: \"world\"}";
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![("hello".into(), Value::String("world".into()))])
        );
    }

    #[test]
    fn single_key_chain_without_braces() {
        let src = "foo: bar: baz: 1";
        let expected = parser()
            .parse("foo: { bar: { baz: 1 } }")
            .into_result()
            .unwrap();
        let expected = unify_tree(&expected).unwrap();
        let parsed = must_unify(src);
        assert_eq!(parsed.to_value(), expected.to_value());
    }

    #[test]
    fn object_commas_optional() {
        let with_commas = "foo: 1,\nbar: 2,\n";
        let without_commas = "foo: 1\nbar: 2\n";
        let a = must_unify(with_commas);
        let b = must_unify(without_commas);
        assert_eq!(a.to_value(), b.to_value());
    }

    #[test]
    fn chain_with_duplicate_keys_no_commas() {
        let src = "foo: bar: 1\nfoo: baz: 2\n";
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![(
                "foo".into(),
                Value::Object(vec![
                    ("bar".into(), Value::Int(1)),
                    ("baz".into(), Value::Int(2)),
                ]),
            ),])
        );
    }

    #[test]
    fn unify_type_is_overwritten_by_value() {
        let src = "company: founded: Int\ncompany: founded: 1985\n";
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![(
                "company".into(),
                Value::Object(vec![("founded".into(), Value::Int(1985))]),
            ),])
        );
    }

    #[test]
    fn reference_unifies_successfully() {
        let src = r#"
            person: {
              name: String
              age: Int
            }

            meadow: person
            meadow: name: "meadow"
            meadow: age: 4
        "#;
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                (
                    "person".into(),
                    Value::Object(vec![
                        ("name".into(), Value::Type(ValType::String)),
                        ("age".into(), Value::Type(ValType::Int)),
                    ]),
                ),
                (
                    "meadow".into(),
                    Value::Object(vec![
                        ("age".into(), Value::Int(4)),
                        ("name".into(), Value::String("meadow".into())),
                    ]),
                ),
            ])
        );
    }

    #[test]
    fn reference_unify_type_mismatch() {
        let src = r#"
            person: name: String
            person: age: Int

            forest: person
            forest: name: "forest"
            forest: age: "old"
        "#;
        must_err(src);
    }

    #[test]
    fn reference_unify_type_mismatch_reordered() {
        let src = r#"
            person: @NoExport
            forest: person
            forest: name: "forest"
            forest: age: "old"

            person: name: String
            person: age: Int
        "#;
        must_err(src);
    }

    #[test]
    fn unresolved_reference_fails() {
        let src = "hello: world";
        must_err(src);
    }

    #[test]
    fn reference_to_value_resolves() {
        let src = r#"
            greet: "world"
            hello: greet
        "#;
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("greet".into(), Value::String("world".into())),
                ("hello".into(), Value::String("world".into())),
            ])
        );
    }

    #[test]
    fn reference_cycle_not_exportable() {
        let src = "foo: bar\nbar: foo";
        let unified = must_unify(src);
        match &unified.kind {
            ValueKind::Object(members) => {
                let foo = members
                    .iter()
                    .find(|(k, _, _, _)| k == "foo")
                    .unwrap()
                    .1
                    .clone();
                let bar = members
                    .iter()
                    .find(|(k, _, _, _)| k == "bar")
                    .unwrap()
                    .1
                    .clone();
                assert!(matches!(foo.kind, ValueKind::Reference(_)));
                assert!(matches!(bar.kind, ValueKind::Reference(_)));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn reference_cycle_resolves_with_value() {
        let src = "foo: bar\nbar: foo\nfoo: 3";
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("foo".into(), Value::Int(3)),
                ("bar".into(), Value::Int(3)),
            ])
        );
    }

    #[test]
    fn reference_cycle_resolves_regardless_of_order() {
        let src = "foo: 3\n\nfoo: bar\nbar: baz\nbaz: foo";
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("foo".into(), Value::Int(3)),
                ("bar".into(), Value::Int(3)),
                ("baz".into(), Value::Int(3)),
            ])
        );
    }

    #[test]
    fn reference_cycle_conflict_fails() {
        let src = "foo: bar\nbar: foo\nfoo: 3\nbar: 4";
        must_err(src);
    }

    #[test]
    fn structural_reference_cycle_reports_error() {
        let src = r#"
            meadow: {
                color: "black"
                bestfriend: forest
            }

            forest: {
                color: "grey"
                bestfriend: meadow
            }
        "#;
        match parse_unify(src) {
            Ok(_) => panic!("expected error"),
            Err(err) => assert!(err.msg.contains("infinite structural cycle")),
        }
    }

    #[test]
    fn parse_int_and_float_values() {
        let src = "my_int: 1\nmy_float: 3.1415";
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("my_int".into(), Value::Int(1)),
                ("my_float".into(), Value::Float(3.1415)),
            ])
        );
    }

    #[test]
    fn int_type_chain_parses() {
        let src = r"my_int: Float
my_int: 1";
        must_unify(src);
    }

    #[test]
    fn int_type_chain_with_number_unifies() {
        let src = r"my_int: Number
my_int: Float
my_int: 1";
        must_unify(src);
    }

    #[test]
    fn float_type_chain_unifies() {
        let src = r"my_float: Number
my_float: Float
my_float: 3.1415";
        must_unify(src);
    }

    #[test]
    fn int_exports_without_decimal() {
        let src = "1";
        let unified = must_unify(src);
        assert_eq!(unified.to_value().to_pretty_string(), "1");
    }

    #[test]
    fn examples_parse_to_json() {
        for entry in std::fs::read_dir("../examples").unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) != Some("pls") {
                continue;
            }
            let src = std::fs::read_to_string(&path).unwrap();
            let json = parse_to_json(&src).unwrap();
        }
    }

    #[test]
    fn noexport_removes_field() {
        let src = "foo: 1\nbar: 2\nbar: @NoExport";
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![("foo".into(), Value::Int(1))])
        );
    }

    #[test]
    fn noexport_with_type_allows_export() {
        let src = r#"
            # unexported types don't break json
            creature: @NoExport
            creature: {
              name: String
              age: Int
            }

            forest: creature
            forest: name: "forest"
            forest: age: 4
        "#;
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![(
                "forest".into(),
                Value::Object(vec![
                    ("age".into(), Value::Int(4)),
                    ("name".into(), Value::String("forest".into())),
                ]),
            ),])
        );
    }

    #[test]
    fn noexport_sum_type_exportable() {
        let src = r#"
            Foo: @NoExport
            Bar: @NoExport
            FooOrBar: @NoExport

            Foo: foo: Int
            Bar: bar: String
            FooOrBar: Foo | Bar

            baz: FooOrBar
            baz: bar: ""
        "#;
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![(
                "baz".into(),
                Value::Object(vec![("bar".into(), Value::String("".into()))]),
            )])
        );
    }

    #[test]
    fn nested_noexport_in_object() {
        let src = r#"
credentials: {
  username: "root"
  password: @NoExport
  password: Nothing
}
"#;
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![(
                "credentials".into(),
                Value::Object(vec![("username".into(), Value::String("root".into()))]),
            )])
        );
    }

    #[test]
    fn chain_noexport() {
        let src = r#"
credentials: password: @NoExport
credentials: {
  username: "root"
  password: Nothing
}
"#;
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![(
                "credentials".into(),
                Value::Object(vec![("username".into(), Value::String("root".into()))]),
            )])
        );
    }

    #[test]
    fn list_unify_type_value() {
        let src = r#"
anInt: [Int]
anInt: [3]
"#;
        must_unify(src);
    }

    #[test]
    fn list_unify_any() {
        let src = r#"
anInt: [Any]
anInt: [3]
"#;
        must_unify(src);
    }

    #[test]
    fn list_unify_with_refs() {
        let src = r#"
            myType: Int
            myInts: [Int, myType]
            myInts: [Int, Int]
            myInts: [3, Int]
            myInts: [myType, 3]
            myInts: [3, 3]
        "#;
        must_unify(src);
    }

    #[test]
    fn list_reference_unifies() {
        let src = r#"
            twoints: [Int, Int]
            couple3s: twoints
            couple3s: [3, 3]
        "#;
        must_unify(src);
    }

    #[test]
    fn list_reference_values() {
        let src = r#"
            foo: "bar"
            bar: "bar"
            baz: [bar]
            baz: [foo]
        "#;
        must_unify(src);
    }

    #[test]
    fn list_type_mismatch_fails() {
        let src = r#"
foo: [Int]
foo: ["hello"]
"#;
        must_err(src);
    }

    #[test]
    fn list_length_mismatch_fails() {
        let src = r#"
foo: [Int]
foo: [1, 2]
"#;
        must_err(src);
    }

    #[test]
    fn sum_type_string_or_int_string() {
        let src = r#"
stringOrInt: String | Int
stringOrInt: "hello"
"#;
        must_unify(src);
    }

    #[test]
    fn sum_type_string_or_int_int() {
        let src = r#"
stringOrInt: String | Int
stringOrInt: 3
"#;
        must_unify(src);
    }

    #[test]
    fn sum_type_string_or_int_bool_fails() {
        let src = r#"
stringOrInt: String | Int
stringOrInt: false
"#;
        must_err(src);
    }

    #[test]
    fn sum_type_string_or_int_float_type_fails() {
        let src = r#"
StringOrInt: String | Int
StringOrInt: Float
StringOrInt: 3.4
"#;
        must_err(src);
    }

    #[test]
    fn sum_type_string_or_int_float_type_no_value_fails() {
        let src = r#"
StringOrInt: String | Int
StringOrInt: Float
"#;
        must_err(src);
    }

    #[test]
    fn sum_type_nested_bool() {
        let src = r#"
stringOrInt: String | Int
stringOrIntOrBool: stringOrInt | Boolean
stringOrIntOrBool: false
"#;
        must_unify(src);
    }

    #[test]
    fn sum_type_nested_int() {
        let src = r#"
stringOrInt: String | Int
stringOrIntOrBool: stringOrInt | Boolean
stringOrIntOrBool: 3
"#;
        must_unify(src);
    }

    #[test]
    fn sum_type_object_union_unifies() {
        let src = r#"
FooOrBar: { foo: Any } | { bar: Any }
foo: FooOrBar
foo: { foo: 3 }

bar: FooOrBar
bar: { bar: 3 }
"#;
        must_unify(src);
    }

    #[test]
    fn sum_type_object_union_fails() {
        let src = r#"
FooOrBar: { foo: Any } | { bar: Any }
baz: FooOrBar
baz: { baz: 3 }
"#;
        must_err(src);
    }

    #[test]
    fn reference_name_starts_with_type() {
        let src = r#"
StringCheese: "stringy"
snack: StringCheese
"#;
        let unified = must_unify(src);
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("StringCheese".into(), Value::String("stringy".into())),
                ("snack".into(), Value::String("stringy".into())),
            ])
        );
    }

    #[test]
    fn unresolved_sum_type_not_exportable() {
        let src = r#"
Dog: @NoExport
Dog: {
  species: "dog"
  says: "bark"
}

Cat: @NoExport
Cat: {
  species: "cat"
  says: "meow"
}

Pet: @NoExport
Pet: Cat | Dog
Pet: {
  species: String
  says: String
}

pet: Pet
"#;
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        match &unified.kind {
            ValueKind::Object(members) => {
                let pet = members
                    .iter()
                    .find(|(k, _, _, _)| k == "pet")
                    .unwrap()
                    .1
                    .clone();
                assert!(matches!(pet.kind, ValueKind::Union(_)));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn sum_type_resolves_when_fields_specified() {
        let src = r#"
Dog: @NoExport
Dog: {
  species: "dog"
  says: "bark"
}

Cat: @NoExport
Cat: {
  species: "cat"
  says: "meow"
}

Pet: @NoExport
Pet: Cat | Dog
Pet: {
  species: String
  says: String
}

pet: Pet
pet: species: "cat"
pet: says: "meow"
"#;
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![(
                "pet".into(),
                Value::Object(vec![
                    ("says".into(), Value::String("meow".into())),
                    ("species".into(), Value::String("cat".into())),
                ]),
            )])
        );
    }

    #[test]
    fn duplicate_union_branch_unifies() {
        let src = "foo: true | true\nfoo: true";
        must_unify(src);
    }

    #[test]
    fn export_error_for_unresolved_type() {
        let src = "foo: Int";
        let err = parse_to_json(src).unwrap_err();
        assert!(err.contains("Int"));
    }

    #[test]
    fn call_increment_literal() {
        let src = "foo: increment 2";
        let unified = must_unify(src);
        match &unified.kind {
            ValueKind::Object(members) => {
                let foo = members
                    .iter()
                    .find(|(k, _, _, _)| k == "foo")
                    .unwrap()
                    .1
                    .clone();
                assert_eq!(foo.to_value(), Value::Int(3));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn call_increment_reference() {
        let src = "two: 2\nfoo: increment two";
        let unified = must_unify(src);
        match &unified.kind {
            ValueKind::Object(members) => {
                let foo = members
                    .iter()
                    .find(|(k, _, _, _)| k == "foo")
                    .unwrap()
                    .1
                    .clone();
                assert_eq!(foo.to_value(), Value::Int(3));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn call_increment_with_type() {
        let src = "foo: Int\nfoo: increment 2";
        let unified = must_unify(src);
        match &unified.kind {
            ValueKind::Object(members) => {
                let foo = members
                    .iter()
                    .find(|(k, _, _, _)| k == "foo")
                    .unwrap()
                    .1
                    .clone();
                assert_eq!(foo.to_value(), Value::Int(3));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn call_increment_chain() {
        let src = r#"
one: Int
one: 1

two: Int
two: increment one

foo: Int
foo: increment two
"#;
        let unified = must_unify(src);
        match &unified.kind {
            ValueKind::Object(members) => {
                let foo = members
                    .iter()
                    .find(|(k, _, _, _)| k == "foo")
                    .unwrap()
                    .1
                    .clone();
                assert_eq!(foo.to_value(), Value::Int(3));
            }
            _ => panic!("expected object"),
        }
    }
}
