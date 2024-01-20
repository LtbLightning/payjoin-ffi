#![crate_name = "payjoin_ffi"]

pub mod error;
pub mod receive;
pub mod send;
pub mod transaction;
mod types;
pub mod uri;





#[allow(unused_imports)]
use crate::receive::v2::{
    ClientResponse,
    ExtractReq,
    Enrolled, Enroller,  V2MaybeInputsOwned, V2MaybeInputsSeen,
    V2MaybeMixedInputScripts, V2OutputsUnknown, V2PayjoinProposal, V2ProvisionalProposal,
    V2UncheckedProposal,
};
use crate::error::PayjoinError;
use crate::receive::{
    CanBroadcast, Headers, IsOutputKnown, IsScriptOwned, MaybeInputsOwned, MaybeInputsSeen,
    MaybeMixedInputScripts, OutputsUnknown, PayjoinProposal, ProcessPartiallySignedTransaction,
    ProvisionalProposal, UncheckedProposal,
};
use crate::send::v2::{ContextV2};
use crate::send::{ContextV1, RequestBuilder, RequestContext, RequestContextV1, RequestContextV2};
use crate::transaction::{PartiallySignedTransaction, Transaction};
use crate::uri::{PjUri, Uri, Url};
use crate::types::{Request, Amount, Txid, OutPoint, Address, ScriptBuf, TxOut, Network, FeeRate };

uniffi::include_scaffolding!("payjoin_ffi");


