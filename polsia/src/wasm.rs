use crate::parse_to_json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "polsiaToJson")]
pub fn polsia_to_json(src: &str) -> Result<String, String> {
    parse_to_json(src)
}
