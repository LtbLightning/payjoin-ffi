#[cfg(not(feature = "wasm"))]
use crate::error::PayjoinError;

#[cfg(feature = "wasm")]
use {
    crate::utils::result::JsResult,
    wasm_bindgen::prelude::*,
};

use std::str::FromStr;

impl From<payjoin::OhttpKeys> for OhttpKeys {
    fn from(value: payjoin::OhttpKeys) -> Self {
        Self(value)
    }
}
impl From<OhttpKeys> for payjoin::OhttpKeys {
    fn from(value: OhttpKeys) -> Self {
        value.0
    }
}
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct OhttpKeys(
    #[wasm_bindgen(skip)]
    pub payjoin::OhttpKeys
);

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl OhttpKeys {
    /// Decode an OHTTP KeyConfig
    #[cfg(not(feature = "wasm"))]
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn decode(bytes: Vec<u8>) -> Result<Self, PayjoinError> {
        payjoin::OhttpKeys::decode(bytes.as_slice()).map(|e| e.into()).map_err(|e| e.into())
    }

    // #[cfg(feature = "wasm")]
    // #[wasm_bindgen(constructor)]
    // pub fn decode(bytes: Vec<u8>) -> JsResult<OhttpKeys> {
    //     payjoin::OhttpKeys::decode(bytes.as_slice())
    //         .map(|e| e.into())
    //         .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    // }

    #[cfg(feature = "wasm")]
    #[wasm_bindgen(constructor)]
    pub fn parse(s: &str) -> JsResult<OhttpKeys> {
        payjoin::OhttpKeys::from_str(s)
            .map(|e| e.into())
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    }
}


use std::sync::Mutex;

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ClientResponse(Mutex<Option<ohttp::ClientResponse>>);

impl From<&ClientResponse> for ohttp::ClientResponse {
    fn from(value: &ClientResponse) -> Self {
        let mut data_guard = value.0.lock().unwrap();
        Option::take(&mut *data_guard).expect("ClientResponse moved out of memory")
    }
}

impl From<ohttp::ClientResponse> for ClientResponse {
    fn from(value: ohttp::ClientResponse) -> Self {
        Self(Mutex::new(Some(value)))
    }
}
