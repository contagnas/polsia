pub mod parser;
pub mod types;
pub mod unify;

pub use parser::{document, parser};
pub use types::{
    Directive, Document, SpannedValue, ValType, Value, ValueKind, apply_directives,
    apply_directives_spanned,
};
pub use unify::{UnifyError, unify, unify_spanned, unify_tree, unify_with_path};

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::polsia_to_json;

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::prelude::*;

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
        let unified = unify_tree(&parsed).unwrap();
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
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
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
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
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
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![("a".into(), Value::Int(1))])
        );
    }

    #[test]
    fn object_with_duplicate_keys_different() {
        let src = r#"{ "a": 1, "a": 2 }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_err());
    }

    #[test]
    fn unify_type_with_value() {
        let src = r#"{ "a": Int, "a": 1 }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn unify_type_with_incompatible_value() {
        let src = r#"{ "a": Int, "a": 1.1 }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_err());
    }

    #[test]
    fn unify_any_with_value() {
        let src = r#"{ "a": Any, "a": 1 }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn unify_nothing_with_value() {
        let src = r#"{ "a": Nothing, "a": 1 }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_err());
    }

    #[test]
    fn unify_number_hierarchy() {
        let src = r#"{ "a": Int, "a": Float }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn unify_string_with_value() {
        let src = r#"{ "a": String, "a": "hi" }"#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn unify_recursive_object() {
        let src = r#"
            {
                foo: { bar: Int },
                foo: { bar: 3 },
            }
        "#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn object_union_of_keys_parses() {
        let src = r#"
            {
                foo: { bar: 1 },
                foo: { baz: 2 },
            }
        "#;
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn unify_object_union_of_keys() {
        let a = Value::Object(vec![(
            "foo".into(),
            Value::Object(vec![("bar".into(), Value::Int(1))]),
        )]);
        let b = Value::Object(vec![(
            "foo".into(),
            Value::Object(vec![("baz".into(), Value::Int(2))]),
        )]);
        let unified = unify(&a, &b).unwrap();
        let expected = Value::Object(vec![(
            "foo".into(),
            Value::Object(vec![
                ("bar".into(), Value::Int(1)),
                ("baz".into(), Value::Int(2)),
            ]),
        )]);
        assert_eq!(unified, expected);
    }

    #[test]
    fn duplicate_key_error_details() {
        let src = "{\n    hello: Int,\n    hello: String,\n}";
        let parsed = parser().parse(src).into_result().unwrap();
        match unify_tree(&parsed) {
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
        let parsed = parser().parse(src).into_result().unwrap();
        match unify_tree(&parsed) {
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
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_err());
    }

    #[test]
    fn top_level_braces_optional() {
        let with_braces = "{ foo: 1, bar: 2, }";
        let without_braces = "foo: 1,\nbar: 2,\n";
        let a = parser().parse(with_braces).into_result().unwrap();
        let a = unify_tree(&a).unwrap();
        let b = parser().parse(without_braces).into_result().unwrap();
        let b = unify_tree(&b).unwrap();
        assert_eq!(a.to_value(), b.to_value());
    }

    #[test]
    fn single_key_chain_without_braces() {
        let src = "foo: bar: baz: 1";
        let expected = parser()
            .parse("foo: { bar: { baz: 1 } }")
            .into_result()
            .unwrap();
        let expected = unify_tree(&expected).unwrap();
        let parsed = parser().parse(src).into_result().unwrap();
        let parsed = unify_tree(&parsed).unwrap();
        assert_eq!(parsed.to_value(), expected.to_value());
    }

    #[test]
    fn object_commas_optional() {
        let with_commas = "foo: 1,\nbar: 2,\n";
        let without_commas = "foo: 1\nbar: 2\n";
        let a = parser().parse(with_commas).into_result().unwrap();
        let a = unify_tree(&a).unwrap();
        let b = parser().parse(without_commas).into_result().unwrap();
        let b = unify_tree(&b).unwrap();
        assert_eq!(a.to_value(), b.to_value());
    }

    #[test]
    fn chain_with_duplicate_keys_no_commas() {
        let src = "foo: bar: 1\nfoo: baz: 2\n";
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
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
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
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
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
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
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_err());
    }

    #[test]
    fn unresolved_reference_fails() {
        let src = "hello: world";
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_err());
    }

    #[test]
    fn reference_to_value_resolves() {
        let src = r#"
            greet: "world"
            hello: greet
        "#;
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
        assert_eq!(
            unified.to_value(),
            Value::Object(vec![
                ("greet".into(), Value::String("world".into())),
                ("hello".into(), Value::String("world".into())),
            ])
        );
    }

    #[test]
    fn parse_int_and_float_values() {
        let src = "my_int: 1\nmy_float: 3.1415";
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
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
        let src = "my_int: Float\nmy_int: 1";
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn int_type_chain_with_number_unifies() {
        let src = "my_int: Number\nmy_int: Float\nmy_int: 1";
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn float_type_chain_unifies() {
        let src = "my_float: Number\nmy_float: Float\nmy_float: 3.1415";
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }

    #[test]
    fn int_exports_without_decimal() {
        let src = "1";
        let parsed = parser().parse(src).into_result().unwrap();
        let unified = unify_tree(&parsed).unwrap();
        assert_eq!(unified.to_value().to_pretty_string(), "1");
    }

    #[test]
    fn demo_file_parses_to_json() {
        let src = std::fs::read_to_string("../examples/demo.pls").unwrap();
        let doc = document().parse(&src).into_result().unwrap();
        let mut unified = unify_tree(&doc.value).unwrap();
        apply_directives_spanned(&mut unified, &doc.directives);
        let json = unified.to_value().to_pretty_string();
        assert!(!json.is_empty());
    }

    #[test]
    fn noexport_removes_field() {
        let src = "foo: 1\nbar: 2\nnoexport bar";
        let doc = document().parse(src).into_result().unwrap();
        let unified = unify_tree(&doc.value).unwrap();
        let val = apply_directives(unified.to_value(), &doc.directives);
        assert_eq!(val, Value::Object(vec![("foo".into(), Value::Int(1))]));
    }

    #[test]
    fn noexport_with_type_allows_export() {
        let src = r#"
            # unexported types don't break json
            noexport creature
            creature: {
              name: String
              age: Int
            }

            forest: creature
            forest: name: "forest"
            forest: age: 4
        "#;
        let doc = document().parse(src).into_result().unwrap();
        let mut unified = unify_tree(&doc.value).unwrap();
        apply_directives_spanned(&mut unified, &doc.directives);
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
}
