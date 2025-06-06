use crate::types::{JsonType, Span, SpannedJson, SpannedKind};
use chumsky::prelude::*;
use chumsky::span::{SimpleSpan, Span as ChumSpan};

pub fn parser<'a>() -> impl Parser<'a, &'a str, SpannedJson, extra::Err<Rich<'a, char>>> {
    let value = spanned_value();

    let comment = just('#')
        .then(none_of('\n').repeated())
        .then_ignore(text::newline().or_not())
        .ignored();
    let ws = choice((text::whitespace().at_least(1).ignored(), comment.clone()))
        .repeated()
        .ignored();
    let ws1 = choice((text::whitespace().at_least(1).ignored(), comment.clone()))
        .repeated()
        .at_least(1)
        .ignored();

    let escape =
        just('\\').ignore_then(choice((
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
        .map(|s| s);

    let key_string = string.clone();
    let key = key_string.or(text::ident().map(|s: &str| s.to_string()));
    let key_span = key.clone().map_with(|k: String, e| (k, e.span()));

    let member = key_span
        .then_ignore(just(':').padded_by(ws.clone()))
        .then(spanned_value_no_pad())
        .map(|((k, k_span), mut v): ((String, Span), SpannedJson)| {
            let span = SimpleSpan::new((), k_span.start()..v.span.end());
            v.span = span;
            (k, v, span)
        });

    let comma = just(',').then_ignore(ws.clone()).ignored();
    let top_object = member
        .separated_by(choice((comma, ws1.clone())))
        .allow_trailing()
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|members| members)
        .map_with(|members, e| SpannedJson {
            span: e.span(),
            kind: SpannedKind::Object(members),
        });

    choice((top_object, value))
        .padded_by(ws)
        .map(|v| v)
}

fn spanned_value<'a>() -> impl Parser<'a, &'a str, SpannedJson, extra::Err<Rich<'a, char>>> {
    spanned_value_no_pad().padded_by(
        choice((
            text::whitespace().at_least(1).ignored(),
            just('#')
                .then(none_of('\n').repeated())
                .then_ignore(text::newline().or_not())
                .ignored(),
        ))
        .repeated()
        .ignored(),
    )
}

fn spanned_value_no_pad<'a>() -> impl Parser<'a, &'a str, SpannedJson, extra::Err<Rich<'a, char>>> {
    recursive(|value| {
        let comment = just('#')
            .then(none_of('\n').repeated())
            .then_ignore(text::newline().or_not())
            .ignored();
        let ws = choice((text::whitespace().at_least(1).ignored(), comment.clone()))
            .repeated()
            .ignored();
        let ws1 = choice((text::whitespace().at_least(1).ignored(), comment.clone()))
            .repeated()
            .at_least(1)
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
                ws.clone().then_ignore(just(']')),
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
        let comma = just(',').then_ignore(ws.clone()).ignored();
        let object = member
            .separated_by(choice((comma, ws1.clone())))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just('{').padded_by(ws.clone()),
                ws.clone().then_ignore(just('}')),
            )
            .map_with(|members, e| SpannedJson {
                span: e.span(),
                kind: SpannedKind::Object(members),
            });

        let chain = key_span
            .clone()
            .then_ignore(just(':').padded_by(ws.clone()))
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .then(value.clone())
            .map(|(keys, mut v)| {
                for (k, k_span) in keys.into_iter().rev() {
                    let span = SimpleSpan::new((), k_span.start()..v.span.end());
                    v = SpannedJson {
                        span,
                        kind: SpannedKind::Object(vec![(k, v, span)]),
                    };
                }
                v
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
            chain,
        ))
    })
}
