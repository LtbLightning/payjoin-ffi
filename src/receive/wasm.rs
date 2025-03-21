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


#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Receiver(super::Receiver);

impl From<payjoin::receive::v2::Receiver> for Receiver {
    fn from(receiver: payjoin::receive::v2::Receiver) -> Self {
        Self(receiver.into())
    }
}

#[wasm_bindgen]
impl Receiver {
  pub fn new(
      address: String,
      network: String,
      directory: String,
      ohttp_keys: String,
      ohttp_relay: String,
      expire_after: Option<u64>,
  ) -> JsResult<Receiver> {
      // Log inputs to console
      console::log_1(&JsValue::from_str(&format!(
          "Receiver::new inputs: address={}, network={}, directory={}, ohttp_keys={}, ohttp_relay={}, expire_after={:?}",
          address, network, directory, ohttp_keys, ohttp_relay, expire_after
      )));

      // Parse network string
      let network = Network::from_str(&network)
          .map_err(|_| wasm_bindgen::JsError::new("Invalid network"))?;

      // Parse URLs
      // let directory = Url::parse(directory)
      //     .map_err(|_| wasm_bindgen::JsError::new("Invalid directory URL"))?;
      let ohttp_relay = Url::parse(ohttp_relay)
          .map_err(|_| wasm_bindgen::JsError::new("Invalid relay URL"))?;

      // Parse OHTTP keys from JSON string
      let ohttp_keys: OhttpKeys = OhttpKeys::parse(&ohttp_keys)
          .map_err(|_| wasm_bindgen::JsError::new("Invalid OHTTP keys"))?;

      // Parse Bitcoin address and verify network
      let address = payjoin::bitcoin::Address::from_str(&address)
          .map_err(|_| wasm_bindgen::JsError::new("Invalid Bitcoin address"))?
          .require_network(network)
          .map_err(|_| wasm_bindgen::JsError::new("Address network mismatch"))?;

      Ok(payjoin::receive::v2::Receiver::new(
          address,
          directory,
          ohttp_keys.into(),
          expire_after.map(Duration::from_secs)
      )
      .map_err(PayjoinError::from)?
      .into())
  }

  pub fn pj_uri(&self) -> crate::PjUri {
      self.0.pj_uri().into()
  }

  // pub fn extract_req(&self) -> Result<RequestResponse, PayjoinError> {
  //     self.0
  //         .extract_req()
  //         .map(|(request, ctx)| RequestResponse { request, client_response: Arc::new(ctx) })
  // }

  // ///The response can either be an UncheckedProposal or an ACCEPTED message indicating no UncheckedProposal is available yet.
  // pub fn process_res(
  //     &self,
  //     body: &[u8],
  //     context: Arc<ClientResponse>,
  // ) -> Result<Option<Arc<UncheckedProposal>>, PayjoinError> {
  //     <Self as Into<super::Receiver>>::into(self.clone())
  //         .process_res(body, context.as_ref())
  //         .map(|e| e.map(|x| Arc::new(x.into())))
  // }

  // /// The contents of the `&pj=` query parameter including the base64url-encoded public key receiver subdirectory.
  // /// This identifies a session at the payjoin directory server.
  // #[cfg(feature = "uniffi")]
  // pub fn pj_url(&self) -> Arc<Url> {
  //     Arc::new(self.0.pj_url())
  // }
  ///The per-session public key to use as an identifier
  pub fn id(&self) -> String {
      self.0.id()
  }

  pub fn to_json(&self) -> JsResult<String> {
      self.0.to_json()
          .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
  }

  // pub fn from_json(json: &str) -> JsResult<Self> {
  //     super::Receiver::from_json(json)
  //         .map(Into::into)
  //         .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
  // }
}
