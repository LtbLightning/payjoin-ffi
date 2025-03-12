#![crate_name = "payjoin_ffi"]

pub mod bitcoin_ffi;
pub mod error;
#[cfg(not(feature = "wasm"))]
pub mod io;
pub mod ohttp;
pub mod receive;
pub mod request;
pub mod send;
pub mod uri;
mod utils;

pub use utils::*;
pub use crate::bitcoin_ffi::*;
pub use crate::error::PayjoinError;
pub use crate::ohttp::*;
#[cfg(feature = "uniffi")]
pub use crate::receive::uni::*;
pub use crate::request::Request;
#[cfg(feature = "uniffi")]
pub use crate::send::uni::*;
pub use crate::uri::{PjUri, Uri, Url};
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
// Initialize WASM logging (optional but helpful for debugging)
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}