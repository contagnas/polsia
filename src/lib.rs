pub mod parser;
pub mod types;
pub mod unify;

pub use parser::parser;
pub use types::{Json, JsonType, SpannedJson, SpannedKind};
pub use unify::{UnifyError, unify, unify_spanned, unify_tree, unify_with_path};

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
            unified.to_json(),
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
        let unified = unify_tree(&parsed).unwrap();
        assert_eq!(
            unified.to_json(),
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
        let unified = unify_tree(&parsed).unwrap();
        assert_eq!(
            unified.to_json(),
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
        let unified = unify_tree(&parsed).unwrap();
        assert_eq!(
            unified.to_json(),
            Json::Object(vec![
                ("a".into(), Json::Number(1.0)),
                ("a".into(), Json::Number(1.0)),
            ])
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
        assert_eq!(a.to_json(), b.to_json());
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
        assert_eq!(parsed.to_json(), expected.to_json());
    }

    #[test]
    fn object_commas_optional() {
        let with_commas = "foo: 1,\nbar: 2,\n";
        let without_commas = "foo: 1\nbar: 2\n";
        let a = parser().parse(with_commas).into_result().unwrap();
        let a = unify_tree(&a).unwrap();
        let b = parser().parse(without_commas).into_result().unwrap();
        let b = unify_tree(&b).unwrap();
        assert_eq!(a.to_json(), b.to_json());
    }

    #[test]
    fn chain_with_duplicate_keys_no_commas() {
        let src = "foo: bar: 1\nfoo: baz: 2\n";
        let parsed = parser().parse(src).into_result().unwrap();
        assert!(unify_tree(&parsed).is_ok());
    }
}
