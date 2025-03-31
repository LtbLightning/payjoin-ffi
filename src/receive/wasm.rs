use std::str::FromStr;
use std::time::Duration;
use std::sync::Arc;

// use payjoin::bitcoin::psbt::Psbt;
// use payjoin::bitcoin::FeeRate;
// use payjoin::receive as pdk;

use crate::bitcoin_ffi::Network;
// use crate::error::PayjoinError;

use crate::ohttp::{ClientResponse, OhttpKeys};
use crate::request_wasm::Request;
use crate::Url;
use crate::error::PayjoinError;

use {
  crate::utils::result::JsResult,
  wasm_bindgen::prelude::*,
  wasm_bindgen::JsValue,
  web_sys::console,
  web_sys::js_sys,
  // web_sys::js_sys::Date,
  serde_wasm_bindgen,
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

  pub fn extract_req(&self, ohttp_relay: String) -> JsResult<RequestResponse> {
      self.0
          .extract_req(ohttp_relay)
          .map(|(request, ctx)| RequestResponse::new(request.into(), ctx))
          .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
  }

  ///The response can either be an UncheckedProposal or an ACCEPTED message indicating no UncheckedProposal is available yet.
  pub fn process_res(
      &self,
      body: &[u8],
      context: ClientResponse,
  ) -> JsResult<Option<UncheckedProposal>> {
      self.0
          .process_res(body, &context)
          .map(|e| e.map(|x| x.into()))
          .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
  }

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

#[wasm_bindgen]
pub struct RequestResponse(Request, ClientResponse);

#[wasm_bindgen]
impl RequestResponse {
    #[wasm_bindgen(constructor)]
    pub fn new(request: Request, client_response: ClientResponse) -> Self {
        Self(request, client_response)
    }

    #[wasm_bindgen(getter)]
    pub fn request(&self) -> Request {
        self.0.clone()
    }

    // consumes self, so RequestResponse won't be available in js after getting client_response
    #[wasm_bindgen(getter)]
    pub fn client_response(self) -> ClientResponse {
        self.1
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct UncheckedProposal(super::UncheckedProposal);

impl From<super::UncheckedProposal> for UncheckedProposal {
    fn from(value: super::UncheckedProposal) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl UncheckedProposal {
    pub fn check_broadcast_suitability(
        &self,
        min_fee_rate: Option<u64>,
        can_broadcast: Option<bool>,//Box<dyn CanBroadcast>,//fn to check tx can broadcast
    ) -> JsResult<MaybeInputsOwned> {
        self.0
            .clone()
            .check_broadcast_suitability(min_fee_rate, |transaction| {
                // should actually check if the transaction can be broadcast
                Ok(can_broadcast.unwrap_or(false))
            })
            .map(|e| e.into())
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct MaybeInputsOwned(super::MaybeInputsOwned);

impl From<super::MaybeInputsOwned> for MaybeInputsOwned {
    fn from(value: super::MaybeInputsOwned) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl MaybeInputsOwned {
    ///Check that the Original PSBT has no receiver-owned inputs. Return original-psbt-rejected error or otherwise refuse to sign undesirable inputs.
    /// An attacker could try to spend receiver's own inputs. This check prevents that.
    pub fn check_inputs_not_owned(
        &self,
        is_owned: js_sys::Function,
    ) -> JsResult<MaybeInputsSeen> {
        self.0
            .check_inputs_not_owned(|input| {
                let result = is_owned.call1(&JsValue::NULL, &js_sys::Uint8Array::from(&input[..]))
                    .map_err(|e| PayjoinError::UnexpectedError { 
                        message: e.as_string().unwrap_or_else(|| "Unknown JS error".to_string())
                    })?;
                result.as_bool()
                    .ok_or_else(|| PayjoinError::UnexpectedError { 
                        message: "Function must return boolean".to_string() 
                    })
            })
            .map(|t| t.into())
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct MaybeInputsSeen(super::MaybeInputsSeen);

impl From<super::MaybeInputsSeen> for MaybeInputsSeen {
    fn from(value: super::MaybeInputsSeen) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl MaybeInputsSeen {
    /// Make sure that the original transaction inputs have never been seen before. This prevents probing attacks. This prevents reentrant Payjoin, where a sender proposes a Payjoin PSBT as a new Original PSBT for a new Payjoin.
    pub fn check_no_inputs_seen_before(
        &self,
        is_known: js_sys::Function,
    ) -> JsResult<OutputsUnknown> {
        self.0
            .clone()
            .check_no_inputs_seen_before(|outpoint| {
                // Convert the outpoint to a JsValue and call the callback
                is_known.call1(&JsValue::null(), &serde_wasm_bindgen::to_value(outpoint).map_err(|e| PayjoinError::UnexpectedError {
                    message: e.to_string(),
                })?)
                    .map(|result| result.as_bool().unwrap_or(false))
                    .map(Ok)
                    .unwrap_or(Ok(false))
            })
            .map(|t| t.into())
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct OutputsUnknown(super::OutputsUnknown);

impl From<super::OutputsUnknown> for OutputsUnknown {
    fn from(value: super::OutputsUnknown) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl OutputsUnknown {
    /// Find which outputs belong to the receiver
    pub fn identify_receiver_outputs(
        &self,
        is_receiver_output: js_sys::Function,
    ) -> JsResult<WantsOutputs> {
        self.0
            .clone()
            .identify_receiver_outputs(|output_script| {
                is_receiver_output.call1(&JsValue::null(), &js_sys::Uint8Array::from(&output_script[..]))
                    .map(|result| result.as_bool().unwrap_or(false))
                    .map_err(|e| PayjoinError::UnexpectedError {
                        message: e.as_string().unwrap_or_else(|| "Unknown JS error".to_string())
                    })
            })
            .map(|t| t.into())
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    }
}

#[wasm_bindgen]
pub struct WantsOutputs(super::WantsOutputs);

impl From<super::WantsOutputs> for WantsOutputs {
    fn from(value: super::WantsOutputs) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WantsOutputs {
    pub fn commit_outputs(&self) -> WantsInputs {
        self.0.commit_outputs().into()
    }
}

#[wasm_bindgen]
pub struct WantsInputs(super::WantsInputs);

impl From<super::WantsInputs> for WantsInputs {
    fn from(value: super::WantsInputs) -> Self {
        Self(value)
    }
}
#[wasm_bindgen]
impl WantsInputs {

    pub fn contribute_inputs(
        &self,
        replacement_inputs: Vec<InputPair>,
    ) -> JsResult<WantsInputs> {
        let replacement_inputs: Vec<super::InputPair> = replacement_inputs
            .into_iter()
            .map(|pair| pair.0)
            .collect();
        self.0.contribute_inputs(replacement_inputs)
            .map(|t| t.into())
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    }

    pub fn commit_inputs(&self) -> ProvisionalProposal {
        self.0.commit_inputs().into()
    }
}

#[wasm_bindgen]
pub struct InputPair(super::InputPair);

impl From<super::InputPair> for InputPair {
    fn from(value: super::InputPair) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl InputPair {
  pub fn new(
    txid: String,
    vout: u32,
    value: u64,
    script_pubkey: Vec<u8>,
  ) -> Self {
    let txin = bitcoin_ffi::TxIn {
      previous_output: bitcoin_ffi::OutPoint {
        txid: bitcoin_ffi::Txid::from_str(&txid).unwrap(),
        vout,
      },
      script_sig: Arc::new(bitcoin_ffi::Script::new(Vec::new())),
      sequence: 0xffffffff,
      witness: Vec::new(),
    };
    // console::log_1(&JsValue::from_str(&format!("InputPair::new: txid={}, vout={}, value={}, script_pubkey={}", txid, vout, value, script_pubkey)));
    let psbtin = crate::bitcoin_ffi::PsbtInput {
      witness_utxo: Some(bitcoin_ffi::TxOut {
        value: Arc::new(bitcoin_ffi::Amount::from_sat(value)),
        script_pubkey: Arc::new(bitcoin_ffi::Script::new(script_pubkey)),
      }),
      redeem_script: None,
      witness_script: None,
    };
    Self(super::InputPair::new(txin, psbtin).unwrap())
  }
}

#[wasm_bindgen]
pub struct ProvisionalProposal(super::ProvisionalProposal);

impl From<super::ProvisionalProposal> for ProvisionalProposal {
    fn from(value: super::ProvisionalProposal) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl ProvisionalProposal {
    pub fn finalize_proposal(
        &self,
        process_psbt: js_sys::Function,
        min_feerate_sat_per_vb: Option<u64>,
        max_effective_fee_rate_sat_per_vb: Option<u64>,
    ) -> JsResult<PayjoinProposal> {
        self.0
            .finalize_proposal(
                |psbt| {
                    process_psbt.call1(&JsValue::null(), &JsValue::from_str(&psbt.to_string()))
                        .map_err(|e| PayjoinError::UnexpectedError { 
                            message: e.as_string().unwrap_or_else(|| "Unknown JS error".to_string())
                        })?
                        .as_string()
                        .ok_or_else(|| PayjoinError::UnexpectedError {
                            message: "Process PSBT must return string".to_string() 
                        })
                },
                min_feerate_sat_per_vb,
                max_effective_fee_rate_sat_per_vb,
            )
            .map(|e| e.into())
            .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct PayjoinProposal(super::PayjoinProposal);

impl From<PayjoinProposal> for super::PayjoinProposal {
    fn from(value: PayjoinProposal) -> Self {
        value.0
    }
}

impl From<super::PayjoinProposal> for PayjoinProposal {
    fn from(value: super::PayjoinProposal) -> Self {
        Self(value)
    }
}