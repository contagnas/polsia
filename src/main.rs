use chumsky::prelude::*;
use std::{env, fs};

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
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
            .then(value.clone());
        let object = member
            .separated_by(just(',').padded_by(ws.clone()))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just('{').padded_by(ws.clone()),
                just('}').padded_by(ws.clone()),
            )
            .try_map(|members: Vec<(String, Json)>, span| {
                use std::collections::hash_map::{Entry, HashMap};
                let mut seen: HashMap<&str, &Json> = HashMap::new();
                for (k, v) in &members {
                    match seen.entry(k.as_str()) {
                        Entry::Occupied(entry) => {
                            if *entry.get() != v {
                                return Err(Rich::custom(
                                    span,
                                    format!("duplicate key '{}' with differing values", k),
                                ));
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(v);
                        }
                    }
                }
                Ok(Json::Object(members))
            });

        choice((
            just("null").to(Json::Null),
            just("true").to(Json::Bool(true)),
            just("false").to(Json::Bool(false)),
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
    let result = parser().parse(src.trim()).into_result();
    match result {
        Ok(json) => println!("{:#?}", json),
        Err(errs) => {
            for e in errs {
                eprintln!("Error: {}", e);
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
}
