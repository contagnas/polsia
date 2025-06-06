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

fn parser<'a>() -> impl Parser<'a, &'a str, Json, extra::Err<Simple<'a, char>>> {
    recursive(|value| {
        let digits = text::digits(10);
        let int = text::int(10);

        let number = just('-')
            .or_not()
            .then(int)
            .then(just('.').then(digits.clone()).or_not())
            .then(one_of("eE").then(one_of("+-").or_not()).then(digits.clone()).or_not())
            .to_slice()
            .map(|s: &str| Json::Number(s.parse().unwrap()));

        let escape = just('\\').ignore_then(
            choice((
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
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect()
            .delimited_by(just('[').padded(), just(']').padded())
            .map(Json::Array);

        let member = string
            .clone()
            .map(|j| if let Json::String(s) = j { s } else { unreachable!() })
            .then_ignore(just(':').padded())
            .then(value.clone());
        let object = member
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded())
            .map(Json::Object);

        choice((
            just("null").to(Json::Null),
            just("true").to(Json::Bool(true)),
            just("false").to(Json::Bool(false)),
            number,
            string,
            array,
            object,
        ))
        .padded()
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

