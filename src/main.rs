use polsia::types::Span;
use polsia::{ValType, SpannedValue, ValueKind, parser, unify_tree};
use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::prelude::*;
use std::{env, fs};

fn find_unresolved(value: &SpannedValue) -> Option<(Span, ValType)> {
    match &value.kind {
        ValueKind::Type(t) => Some((value.span, t.clone())),
        ValueKind::Array(items) => {
            for item in items {
                if let Some(res) = find_unresolved(item) {
                    return Some(res);
                }
            }
            None
        }
        ValueKind::Object(members) => {
            for (_, v, _) in members {
                if let Some(res) = find_unresolved(v) {
                    return Some(res);
                }
            }
            None
        }
        _ => None,
    }
}

fn main() {
    let filename = env::args().nth(1).expect("expected file argument");
    let src = fs::read_to_string(&filename).expect("failed to read file");
    let parse_result = parser().parse(&src).into_result();
    match parse_result {
        Ok(ast) => match unify_tree(&ast) {
            Ok(value) => {
                if let Some((span, t)) = find_unresolved(&value) {
                    let msg = format!("value of type {:?} is unspecified", t);
                    let span_range = span.into_range();
                    Report::build(ReportKind::Error, (filename.clone(), span_range.clone()))
                        .with_message(&msg)
                        .with_label(
                            Label::new((filename.clone(), span_range))
                                .with_message(&msg)
                                .with_color(Color::Red),
                        )
                        .finish()
                        .print(sources([(filename.clone(), src.as_str())]))
                        .unwrap();
                } else {
                    println!("{}", value.to_value().to_pretty_string());
                }
            }
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
        },
        Err(errs) => {
            for e in errs {
                let span = (*e.span()).into_range();
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
