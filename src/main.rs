use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::prelude::*;
use polsia::{parser, unify_tree};
use std::{env, fs};

fn main() {
    let filename = env::args().nth(1).expect("expected file argument");
    let src = fs::read_to_string(&filename).expect("failed to read file");
    let parse_result = parser().parse(&src).into_result();
    match parse_result {
        Ok(ast) => match unify_tree(&ast) {
            Ok(value) => println!("{:#?}", value.to_value()),
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
