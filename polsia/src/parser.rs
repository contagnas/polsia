use crate::types::{Annotation, Document, Span, SpannedValue, ValType, ValueKind};
use chumsky::prelude::*;
use chumsky::span::{SimpleSpan, Span as ChumSpan};

pub fn document<'a>() -> impl Parser<'a, &'a str, Document, extra::Err<Rich<'a, char>>> {
    let value = spanned_value();

    let comment = just('#')
        .then(none_of('\n').repeated())
        .then_ignore(text::newline().or_not())
        .ignored();
    let ws = choice((text::whitespace().at_least(1).ignored(), comment))
        .repeated()
        .ignored();
    let ws1 = choice((text::whitespace().at_least(1).ignored(), comment))
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

    let key_string = string;
    let key = key_string.or(text::ident().map(|s: &str| s.to_string()));
    let key_span = key.map_with(|k: String, e| (k, e.span()));

    let member = key_span
        .then_ignore(just(':').padded_by(ws))
        .then(spanned_value_no_pad())
        .map(
            |((k, k_span), (mut v, anns)): ((String, Span), (SpannedValue, Vec<Annotation>))| {
                let span = SimpleSpan::new((), k_span.start()..v.span.end());
                v.span = span;
                (k, v, span, anns)
            },
        );

    #[derive(Debug)]
    enum Item {
        Member((String, SpannedValue, Span, Vec<Annotation>)),
        Object(Vec<(String, SpannedValue, Span, Vec<Annotation>)>),
    }

    let comma = just(',').then_ignore(ws).ignored();
    let inline_object = spanned_value_no_pad()
        .map(|(v, _)| v)
        .filter(|v: &SpannedValue| matches!(v.kind, ValueKind::Object(_)))
        .map(|v| {
            if let ValueKind::Object(m) = v.kind {
                Item::Object(m)
            } else {
                unreachable!()
            }
        });
    let item = choice((member.map(Item::Member), inline_object));
    let top_object = item
        .separated_by(choice((comma, ws1)))
        .allow_trailing()
        .at_least(1)
        .collect::<Vec<_>>()
        .map_with(|items, e| {
            let mut members = Vec::new();
            for i in items {
                match i {
                    Item::Member(m) => members.push(m),
                    Item::Object(mut objs) => members.append(&mut objs),
                }
            }
            Document {
                value: SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Object(members),
                },
            }
        });

    choice((top_object, value.map(|v| Document { value: v })))
        .padded_by(ws)
        .map(|d| d)
}

pub fn parser<'a>() -> impl Parser<'a, &'a str, SpannedValue, extra::Err<Rich<'a, char>>> {
    document().map(|d| d.value)
}

fn spanned_value<'a>() -> impl Parser<'a, &'a str, SpannedValue, extra::Err<Rich<'a, char>>> {
    spanned_value_no_pad().map(|(v, _)| v).padded_by(
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

fn spanned_value_no_pad<'a>()
-> impl Parser<'a, &'a str, (SpannedValue, Vec<Annotation>), extra::Err<Rich<'a, char>>> {
    recursive(|value| {
        let comment = just('#')
            .then(none_of('\n').repeated())
            .then_ignore(text::newline().or_not())
            .ignored();
        let ws = choice((text::whitespace().at_least(1).ignored(), comment))
            .repeated()
            .ignored();
        let ws1 = choice((text::whitespace().at_least(1).ignored(), comment))
            .repeated()
            .at_least(1)
            .ignored();
        let digits = text::digits(10);
        let int = text::int(10);

        let number = just('-')
            .or_not()
            .then(int)
            .then(just('.').then(digits).or_not())
            .then(
                one_of("eE")
                    .then(one_of("+-").or_not())
                    .then(digits)
                    .or_not(),
            )
            .to_slice()
            .map_with(|s: &str, e| {
                let kind = if s.contains('.') || s.contains('e') || s.contains('E') {
                    ValueKind::Float(s.parse().unwrap())
                } else {
                    ValueKind::Int(s.parse().unwrap())
                };
                (
                    SpannedValue {
                        span: e.span(),
                        kind,
                    },
                    Vec::new(),
                )
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
            .map_with(|s: String, e| {
                (
                    SpannedValue {
                        span: e.span(),
                        kind: ValueKind::String(s),
                    },
                    Vec::new(),
                )
            });

        let array = value
            .clone()
            .separated_by(just(',').padded_by(ws))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded_by(ws), ws.then_ignore(just(']')))
            .map_with(|vals, e| {
                (
                    SpannedValue {
                        span: e.span(),
                        kind: ValueKind::Array(vals.into_iter().map(|(v, _)| v).collect()),
                    },
                    Vec::new(),
                )
            });

        let key_string = string.map(|(j, _)| {
            if let SpannedValue {
                kind: ValueKind::String(s),
                ..
            } = j
            {
                s
            } else {
                unreachable!()
            }
        });
        let key = key_string.or(text::ident().map(|s: &str| s.to_string()));

        let key_span = key.map_with(|k: String, e| (k, e.span()));

        let member = key_span
            .then_ignore(just(':').padded_by(ws))
            .then(value.clone())
            .map(
                |((k, k_span), (mut v, anns)): (
                    (String, Span),
                    (SpannedValue, Vec<Annotation>),
                )| {
                    let span = SimpleSpan::new((), k_span.start()..v.span.end());
                    v.span = span;
                    (k, v, span, anns)
                },
            );
        let comma = just(',').then_ignore(ws).ignored();
        let object = member
            .separated_by(choice((comma, ws1)))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded_by(ws), ws.then_ignore(just('}')))
            .map_with(|members, e| {
                (
                    SpannedValue {
                        span: e.span(),
                        kind: ValueKind::Object(members),
                    },
                    Vec::new(),
                )
            });

        let chain = key_span
            .then_ignore(just(':').padded_by(ws))
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .then(value.clone())
            .map(|(keys, (mut v, anns))| {
                for (k, k_span) in keys.into_iter().rev() {
                    let span = SimpleSpan::new((), k_span.start()..v.span.end());
                    v = SpannedValue {
                        span,
                        kind: ValueKind::Object(vec![(k, v, span, anns.clone())]),
                    };
                }
                (v, Vec::new())
            });

        let reference = text::ident()
            .separated_by(just('.'))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|parts: Vec<&str>| parts.join("."))
            .filter(|s: &String| {
                !matches!(
                    s.as_str(),
                    "null"
                        | "true"
                        | "false"
                        | "Any"
                        | "Nothing"
                        | "Int"
                        | "Number"
                        | "Rational"
                        | "Float"
                        | "String"
                        | "Boolean"
                )
            })
            .map_with(|s: String, e| {
                (
                    SpannedValue {
                        span: e.span(),
                        kind: ValueKind::Reference(s),
                    },
                    Vec::new(),
                )
            });

        let hspace = one_of(" \t").repeated().at_least(1).ignored();

        let call = reference.then_ignore(hspace).then(value.clone()).map_with(
            |((func, _), (arg, _)), e| {
                let name = if let ValueKind::Reference(n) = func.kind {
                    n
                } else {
                    unreachable!()
                };
                (
                    SpannedValue {
                        span: e.span(),
                        kind: ValueKind::Call(name, Box::new(arg)),
                    },
                    Vec::new(),
                )
            },
        );

        let annotation = just('@')
            .ignore_then(text::keyword("NoExport"))
            .map_with(|_, e| {
                (
                    SpannedValue {
                        span: e.span(),
                        kind: ValueKind::Type(ValType::Any),
                    },
                    vec![Annotation::NoExport],
                )
            });

        let atom = choice((
            text::keyword("null")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Null,
                })
                .map(|v| (v, Vec::new())),
            text::keyword("true")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Bool(true),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("false")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Bool(false),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("Any")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::Any),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("Nothing")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::Nothing),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("Int")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::Int),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("Number")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::Number),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("Rational")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::Rational),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("Float")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::Float),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("String")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::String),
                })
                .map(|v| (v, Vec::new())),
            text::keyword("Boolean")
                .map_with(|_, e| SpannedValue {
                    span: e.span(),
                    kind: ValueKind::Type(ValType::Boolean),
                })
                .map(|v| (v, Vec::new())),
            annotation,
            number,
            string,
            array,
            object,
            call,
            chain,
            reference,
        ));

        let union = atom
            .clone()
            .separated_by(just('|').padded_by(ws))
            .at_least(2)
            .collect::<Vec<_>>()
            .map_with(|vals: Vec<(SpannedValue, Vec<Annotation>)>, e| {
                (
                    SpannedValue {
                        span: e.span(),
                        kind: ValueKind::Union(vals.into_iter().map(|(v, _)| v).collect()),
                    },
                    Vec::new(),
                )
            });

        choice((union, atom))
    })
}
