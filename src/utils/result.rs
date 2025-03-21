use wasm_bindgen::JsError;

pub type JsResult<T> = Result<T, JsError>;
