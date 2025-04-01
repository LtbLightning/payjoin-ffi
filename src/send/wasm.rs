use crate::ohttp::ClientResponse;

use crate::Url;

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
    pub fn extract_v2(&self, ohttp_relay: String) -> JsResult<RequestV2PostContext> {
        let url = Url::parse(ohttp_relay)?;
        match self.0.extract_v2(url.into()) {
            Ok((req, ctx)) => Ok(RequestV2PostContext::new(req.into(), ctx.into())),
            Err(e) => Err(e.into()),
        }
    }
}

#[wasm_bindgen]
pub struct V2PostContext(payjoin::send::v2::V2PostContext);

impl From<payjoin::send::v2::V2PostContext> for V2PostContext {
    fn from(ctx: payjoin::send::v2::V2PostContext) -> Self {
        Self(ctx)
    }
}

#[wasm_bindgen]
impl V2PostContext {
    // consumes self, so V2PostContext won't be available in js after calling this
    pub fn process_response(self, response: &[u8]) -> JsResult<V2GetContext> {
        self.0.process_response(response)
            .map(Into::into)
            .map_err(Into::into)
    }
}

#[wasm_bindgen]
pub struct V2GetContext(payjoin::send::v2::V2GetContext);

impl From<payjoin::send::v2::V2GetContext> for V2GetContext {
    fn from(ctx: payjoin::send::v2::V2GetContext) -> Self {
        Self(ctx)
    }
}

#[wasm_bindgen]
impl V2GetContext {
    pub fn extract_req(&self, ohttp_relay: String) -> JsResult<RequestOhttpContext> {
        self.0.extract_req(payjoin::Url::parse(&ohttp_relay)?)
            .map(|(request, ctx)| RequestOhttpContext::new(request.into(), ctx.into()))
            .map_err(Into::into)
    }

    pub fn process_response(&self, response: &[u8], ohttp_ctx: &ClientResponse) -> JsResult<Option<String>> {
        self.0.process_response(response, ohttp_ctx.into())
            .map(|opt_psbt| opt_psbt.map(|psbt| psbt.to_string()))
            .map_err(Into::into)
    }
}

#[wasm_bindgen]
pub struct RequestOhttpContext (Request, ClientResponse);

#[wasm_bindgen]
impl RequestOhttpContext {
    pub fn new(request: Request, ohttp_ctx: ClientResponse) -> Self {
        Self(request, ohttp_ctx)
    }

    #[wasm_bindgen(getter)]
    pub fn request(&self) -> Request {
        self.0.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn ohttp_ctx(self) -> ClientResponse {
        self.1
    }
}

#[wasm_bindgen]
pub struct RequestV2PostContext (Request, V2PostContext);

#[wasm_bindgen]
impl RequestV2PostContext {
    #[wasm_bindgen(constructor)]
    pub fn new(request: Request, context: V2PostContext) -> Self {
        Self(request, context)
    }

    #[wasm_bindgen(getter)]
    pub fn request(&self) -> Request {
        self.0.clone() // Assuming Request implements Clone
    }

    // consumes self, so RequestV2PostContext won't be available in js after getting context, however, using destructuring on the js end makes this seemless.
    #[wasm_bindgen(getter)]
    pub fn context(self) -> V2PostContext {
        self.1
    }
}
