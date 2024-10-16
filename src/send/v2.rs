use std::io::Cursor;
use std::sync::Mutex;

use crate::error::PayjoinError;
use crate::ohttp::ClientResponse;

pub struct Context(Mutex<Option<payjoin::send::Context>>);

impl From<&Context> for payjoin::send::Context {
    fn from(value: &Context) -> Self {
        let mut data_guard = value.0.lock().unwrap();
        Option::take(&mut *data_guard).expect("Context moved out of memory")
    }
}

impl Context {
    pub fn is_v1(&self) -> bool {
        matches!(
            <&Context as Into<payjoin::send::Context>>::into(self),
            payjoin::send::Context::V1(_)
        )
    }

    pub fn is_v2(&self) -> bool {
        matches!(
            <&Context as Into<payjoin::send::Context>>::into(self),
            payjoin::send::Context::V2(_)
        )
    }

    pub fn as_v1(&self) -> Option<crate::v1::V1Context> {
        match <&Context as Into<payjoin::send::Context>>::into(self) {
            payjoin::send::Context::V1(ctx) => Some(ctx.into()),
            _ => None,
        }
    }

    pub fn as_v2(&self) -> Option<V2PostContext> {
        match <&Context as Into<payjoin::send::Context>>::into(self) {
            payjoin::send::Context::V2(ctx) => Some(ctx.into()),
            _ => None,
        }
    }
}

pub struct V2PostContext(payjoin::send::V2PostContext);

impl V2PostContext {
    ///Decodes and validates the response.
    /// Call this method with response from receiver to continue BIP-??? flow. A successful response can either be None if the relay has not response yet or Some(Psbt).
    /// If the response is some valid PSBT you should sign and broadcast.
    pub fn process_response(&self, response: Vec<u8>) -> Result<V2GetContext, PayjoinError> {
        let mut decoder = Cursor::new(response);
        match self.0.process_response(&mut decoder) {
            Ok(ctx) => Ok(V2GetContext(ctx)),
            Err(e) => Err(e.into()),
        }
    }
}

impl From<payjoin::send::V2PostContext> for V2PostContext {
    fn from(value: payjoin::send::V2PostContext) -> Self {
        Self(value)
    }
}

pub struct V2GetContext(payjoin::send::V2GetContext);

impl V2GetContext {
    pub fn extract_req(
        &self,
        ohttp_relay: payjoin::Url,
    ) -> Result<(crate::types::Request, crate::ClientResponse), PayjoinError> {
        self.0
            .extract_req(ohttp_relay)
            .map(|(req, resp)| (req.into(), resp.into()))
            .map_err(|e| e.into())
    }

    /// Decodes and validates the response.
    /// Call this method with response from receiver to continue BIP-??? flow. A successful response can either be None if the relay has not response yet or Some(Psbt).
    /// If the response is some valid PSBT you should sign and broadcast.
    pub fn process_response(
        &self,
        response: Vec<u8>,
        ohttp_ctx: &ClientResponse,
    ) -> Result<Option<String>, PayjoinError> {
        let mut decoder = Cursor::new(response);
        match self.0.process_response(&mut decoder, ohttp_ctx.into()) {
            Ok(Some(psbt)) => Ok(Some(psbt.to_string())),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
