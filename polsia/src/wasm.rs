use crate::{apply_annotations_spanned, document, unify_tree};
use ariadne::{Color, Config, Label, Report, ReportKind, sources};
use chumsky::prelude::*;
use wasm_bindgen::prelude::*;

use crate::{SpannedValue, ValueKind, types::Span};

fn find_unresolved(value: &SpannedValue) -> Option<(Span, String)> {
    match &value.kind {
        ValueKind::Reference(p) => Some((value.span, format!("reference {}", p))),
        ValueKind::Type(t) => Some((value.span, format!("{:?}", t))),
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

#[wasm_bindgen]
pub fn polsia_to_json(src: &str) -> Result<String, String> {
    let filename = "input".to_string();
    let parse_result = document().parse(src).into_result();
    match parse_result {
        Ok(doc) => match unify_tree(&doc.value) {
            Ok(mut value) => {
                apply_annotations_spanned(&mut value, &doc.annotations);
                if let Some((span, t)) = find_unresolved(&value) {
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
                } else {
                    Ok(value.to_value().to_pretty_string())
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
