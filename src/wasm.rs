use crate::{parser, unify_tree};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn polsia_to_json(src: &str) -> Result<String, String> {
    match parser().parse(src).into_result() {
        Ok(ast) => match unify_tree(&ast) {
            Ok(v) => Ok(v.to_value().to_pretty_string()),
            Err(err) => Err(err.msg),
        },
        Err(errs) => {
            let msg = errs
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            Err(msg)
        }
    }
}
