use wasm_bindgen::prelude::*;
use payjoin::Request as PdkRequest;

/// Represents data that needs to be transmitted to the receiver.
/// You need to send this request over HTTP(S) to the receiver.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Request(PdkRequest);

#[wasm_bindgen]
impl Request {
    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.0.url.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn content_type(&self) -> String {
        self.0.content_type.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> Vec<u8> {
        self.0.body.clone()
    }
}

impl From<payjoin::Request> for Request {
    fn from(value: payjoin::Request) -> Self {
        Self(value)
    }
}