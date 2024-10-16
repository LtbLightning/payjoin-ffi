#![crate_name = "payjoin_ffi"]

pub mod error;
pub mod io;
pub mod ohttp;
pub mod receive;
pub mod send;
pub mod types;
pub mod uri;

use crate::error::PayjoinError;
#[allow(unused_imports)]
use crate::ohttp::ClientResponse;
#[cfg(feature = "uniffi")]
use crate::receive::v1::{
    CanBroadcast, GenerateScript, IsOutputKnown, IsScriptOwned, ProcessPartiallySignedTransaction,
};
#[allow(unused_imports)]
use crate::receive::v1::{
    Headers, MaybeInputsOwned, MaybeInputsSeen, MaybeMixedInputScripts, OutputsUnknown,
    PayjoinProposal, ProvisionalProposal, UncheckedProposal,
};
#[allow(unused_imports)]
use crate::receive::v2::{
    Receiver, RequestResponse, V2MaybeInputsOwned, V2MaybeInputsSeen, V2MaybeMixedInputScripts,
    V2OutputsUnknown, V2PayjoinProposal, V2ProvisionalProposal, V2UncheckedProposal,
};
#[allow(unused_imports)]
use crate::send::*;
#[allow(unused_imports)]
use crate::types::{Network, OhttpKeys, OutPoint, Request, TxOut};
#[allow(unused_imports)]
use crate::uri::{PjUri, PjUriBuilder, Uri, Url};

#[cfg(feature = "uniffi")]
uniffi::include_scaffolding!("payjoin_ffi");
