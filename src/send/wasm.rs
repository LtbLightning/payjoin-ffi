use std::str::FromStr;
use std::time::Duration;

// use payjoin::bitcoin::psbt::Psbt;
// use payjoin::bitcoin::FeeRate;
// use payjoin::receive as pdk;

use crate::bitcoin_ffi::Network;
// use crate::error::PayjoinError;
use crate::ohttp::OhttpKeys;
use crate::Url;
use crate::error::PayjoinError;

use {
  crate::utils::result::JsResult,
  wasm_bindgen::prelude::*,
  wasm_bindgen::JsValue,
  web_sys::console,
  // web_sys::js_sys::Date,
};
use crate::uri::PjUri;
use crate::request_wasm::Request;

#[wasm_bindgen]
#[derive(Clone)]
pub struct SenderBuilder(super::SenderBuilder);

impl From<payjoin::send::v2::SenderBuilder<'static>> for SenderBuilder {
    fn from(sender: payjoin::send::v2::SenderBuilder<'static>) -> Self {
        Self(super::SenderBuilder::from(sender))
    }
}

impl From<super::SenderBuilder> for SenderBuilder {
    fn from(sender: super::SenderBuilder) -> Self {
        Self(sender)
    }
}

#[wasm_bindgen]
impl SenderBuilder {
  pub fn from_psbt_and_uri(psbt: String, uri: PjUri) -> JsResult<SenderBuilder> {
    console::log_1(&JsValue::from_str(&format!("SenderBuilder::from_psbt_and_uri: psbt={}, uri={}", psbt, uri.as_string())));
    super::SenderBuilder::from_psbt_and_uri(psbt, uri)
        .map(Into::into)
        .map_err(Into::into)
  }

  pub fn build_recommended(&self, min_fee_rate: u64) -> JsResult<Sender> {
    self.0
        .clone()
        .build_recommended(min_fee_rate)
        .map(Into::into)
        .map_err(Into::into)
  }
}


#[wasm_bindgen]
#[derive(Clone)]
pub struct Sender(payjoin::send::v2::Sender);

impl From<payjoin::send::v2::Sender> for Sender {
    fn from(value: payjoin::send::v2::Sender) -> Self {
        Self(value)
    }
}

impl From<super::Sender> for Sender {
    fn from(sender: super::Sender) -> Self {
        Self(sender.0)
    }
}
#[wasm_bindgen]
impl Sender {
    /// Extract serialized Request and Context from a Payjoin Proposal.
    pub fn extract_v2(&self, ohttp_relay: String) -> JsResult<Request> {//(Request, V2PostContext)> {
      let url = Url::parse(ohttp_relay)?;
        match self.0.extract_v2(url.into()) {
            Ok((req, _ctx)) => Ok(req.into()),//(req.into(), ctx.into())),
            Err(e) => Err(e.into()),
        }
    }
}