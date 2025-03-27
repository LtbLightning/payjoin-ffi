use wasm_bindgen::prelude::*;
use crate::request::Request as PdkRequest;

/// Represents data that needs to be transmitted to the receiver.
/// You need to send this request over HTTP(S) to the receiver.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Request {
    url: String,
    content_type: String,
    body: Vec<u8>
}

#[wasm_bindgen]
impl Request {
    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn content_type(&self) -> String {
        self.content_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> Vec<u8> {
        self.body.clone()
    }
}

impl From<payjoin::Request> for Request {
    fn from(value: payjoin::Request) -> Self {
        Self {
            url: value.url.to_string(),
            content_type: value.content_type.to_string(),
            body: value.body,
        }
    }
}

impl From<PdkRequest> for Request {
    fn from(value: PdkRequest) -> Self {
        Self {
            url: value.url.as_ref().as_string(),
            content_type: value.content_type.to_string(),
            body: value.body,
        }
    }
}