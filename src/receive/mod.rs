pub mod v2;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use payjoin::bitcoin::psbt::Psbt;
use payjoin::bitcoin::FeeRate;
use payjoin::receive::{
    MaybeInputsOwned as PdkMaybeInputsOwned, MaybeInputsSeen as PdkMaybeInputsSeen,
    MaybeMixedInputScripts as PdkMaybeMixedInputScripts, OutputsUnknown as PdkOutputsUnknown,
    PayjoinProposal as PdkPayjoinProposal, ProvisionalProposal as PdkProvisionalProposal,
    UncheckedProposal as PdkUncheckedProposal,
};
use payjoin::Error as PdkError;

use crate::error::PayjoinError;
use crate::types::{OutPoint, TxOut};

pub trait CanBroadcast {
    fn callback(&self, tx: Vec<u8>) -> Result<bool, PayjoinError>;
}

#[derive(Clone)]
pub struct Headers(HashMap<String, String>);

impl Headers {
    pub fn from_vec(body: Vec<u8>) -> Self {
        let mut h = HashMap::new();
        h.insert("content-type".to_string(), "text/plain".to_string());
        h.insert("content-length".to_string(), body.len().to_string());
        Headers(h)
    }
    pub fn get_map(&self) -> HashMap<String, String> {
        self.0.clone()
    }
}

impl payjoin::receive::Headers for Headers {
    fn get_header(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|e| e.as_str())
    }
}

/// The sender’s original PSBT and optional parameters
///
/// This type is used to proces the request. It is returned by UncheckedProposal::from_request().
///
/// If you are implementing an interactive payment processor, you should get extract the original transaction with get_transaction_to_schedule_broadcast() and schedule, followed by checking that the transaction can be broadcast with check_can_broadcast. Otherwise it is safe to call assume_interactive_receive to proceed with validation.
#[derive(Clone)]
pub struct UncheckedProposal(PdkUncheckedProposal);

impl From<PdkUncheckedProposal> for UncheckedProposal {
    fn from(value: PdkUncheckedProposal) -> Self {
        Self(value)
    }
}

impl UncheckedProposal {
    pub fn from_request(
        body: Vec<u8>,
        query: String,
        headers: Arc<Headers>,
    ) -> Result<Self, PayjoinError> {
        match PdkUncheckedProposal::from_request(
            body.as_slice(),
            query.as_str(),
            (*headers).clone(),
        ) {
            Ok(e) => Ok(e.into()),
            Err(e) => Err(e.into()),
        }
    }

    /// The Sender’s Original PSBT
    pub fn extract_tx_to_schedule_broadcast(&self) -> Vec<u8> {
        payjoin::bitcoin::consensus::encode::serialize(
            &self.0.clone().extract_tx_to_schedule_broadcast(),
        )
    }

    /// Call after checking that the Original PSBT can be broadcast.
    ///
    /// Receiver MUST check that the Original PSBT from the sender can be broadcast, i.e. testmempoolaccept bitcoind rpc returns { “allowed”: true,.. } for get_transaction_to_check_broadcast() before calling this method.
    ///
    /// Do this check if you generate bitcoin uri to receive Payjoin on sender request without manual human approval, like a payment processor. Such so called “non-interactive” receivers are otherwise vulnerable to probing attacks. If a sender can make requests at will, they can learn which bitcoin the receiver owns at no cost. Broadcasting the Original PSBT after some time in the failure case makes incurs sender cost and prevents probing.
    ///
    /// Call this after checking downstream.
    pub fn check_broadcast_suitability(
        &self,
        min_fee_rate: Option<u64>,
        can_broadcast: Box<dyn CanBroadcast>,
    ) -> Result<Arc<MaybeInputsOwned>, PayjoinError> {
        let res = self.0.clone().check_broadcast_suitability(
            min_fee_rate.map(|x| payjoin::bitcoin::FeeRate::from_sat_per_kwu(x)),
            |transaction| {
                match can_broadcast
                    .callback(payjoin::bitcoin::consensus::encode::serialize(transaction))
                {
                    Ok(e) => Ok(e),
                    Err(e) => Err(PdkError::Server(e.into())),
                }
            },
        );
        match res {
            Ok(e) => Ok(Arc::new(e.into())),
            Err(e) => Err(e.into()),
        }
    }

    /// Call this method if the only way to initiate a Payjoin with this receiver requires manual intervention, as in most consumer wallets.
    ///
    /// So-called “non-interactive” receivers, like payment processors, that allow arbitrary requests are otherwise vulnerable to probing attacks. Those receivers call get_transaction_to_check_broadcast() and attest_tested_and_scheduled_broadcast() after making those checks downstream.
    pub fn assume_interactive_receiver(&self) -> Arc<MaybeInputsOwned> {
        Arc::new(self.0.clone().assume_interactive_receiver().into())
    }
}

/// Type state to validate that the Original PSBT has no receiver-owned inputs.

/// Call check_no_receiver_owned_inputs() to proceed.
#[derive(Clone)]
pub struct MaybeInputsOwned(PdkMaybeInputsOwned);

impl From<PdkMaybeInputsOwned> for MaybeInputsOwned {
    fn from(value: PdkMaybeInputsOwned) -> Self {
        Self(value)
    }
}

pub trait IsScriptOwned {
    fn callback(&self, script: Vec<u8>) -> Result<bool, PayjoinError>;
}

impl MaybeInputsOwned {
    pub fn check_inputs_not_owned(
        &self,
        is_owned: Box<dyn IsScriptOwned>,
    ) -> Result<Arc<MaybeMixedInputScripts>, PayjoinError> {
        let owned_inputs = self.0.clone();
        match owned_inputs.check_inputs_not_owned(|input| {
            let res = is_owned.callback(input.to_bytes());
            match res {
                Ok(e) => Ok(e),
                Err(e) => Err(PdkError::Server(e.into())),
            }
        }) {
            Ok(e) => Ok(Arc::new(e.into())),
            Err(e) => Err(PayjoinError::ServerError { message: e.to_string() }),
        }
    }
}

/// Typestate to validate that the Original PSBT has no inputs that have been seen before.
///
/// Call check_no_inputs_seen to proceed.
#[derive(Clone)]
pub struct MaybeMixedInputScripts(PdkMaybeMixedInputScripts);

impl From<PdkMaybeMixedInputScripts> for MaybeMixedInputScripts {
    fn from(value: PdkMaybeMixedInputScripts) -> Self {
        Self(value)
    }
}

impl MaybeMixedInputScripts {
    /// Verify the original transaction did not have mixed input types Call this after checking downstream.
    ///
    /// Note: mixed spends do not necessarily indicate distinct wallet fingerprints. This check is intended to prevent some types of wallet fingerprinting.
    pub fn check_no_mixed_input_scripts(&self) -> Result<Arc<MaybeInputsSeen>, PayjoinError> {
        match self.0.clone().check_no_mixed_input_scripts() {
            Ok(e) => Ok(Arc::new(e.into())),
            Err(e) => Err(e.into()),
        }
    }
}

pub trait IsOutputKnown {
    fn callback(&self, outpoint: OutPoint) -> Result<bool, PayjoinError>;
}

/// Typestate to validate that the Original PSBT has no inputs that have been seen before.
///
/// Call check_no_inputs_seen to proceed.
#[derive(Clone)]
pub struct MaybeInputsSeen(PdkMaybeInputsSeen);

impl From<PdkMaybeInputsSeen> for MaybeInputsSeen {
    fn from(value: PdkMaybeInputsSeen) -> Self {
        Self(value)
    }
}

impl MaybeInputsSeen {
    /// Make sure that the original transaction inputs have never been seen before. This prevents probing attacks. This prevents reentrant Payjoin, where a sender proposes a Payjoin PSBT as a new Original PSBT for a new Payjoin.
    pub fn check_no_inputs_seen_before(
        &self,
        is_known: Box<dyn IsOutputKnown>,
    ) -> Result<Arc<OutputsUnknown>, PayjoinError> {
        match self.0.clone().check_no_inputs_seen_before(|outpoint| {
            let res = is_known.callback(outpoint.clone().into());
            match res {
                Ok(e) => Ok(e),
                Err(e) => Err(PdkError::Server(e.into())),
            }
        }) {
            Ok(e) => Ok(Arc::new(e.into())),
            Err(e) => Err(e.into()),
        }
    }
}

/// The receiver has not yet identified which outputs belong to the receiver.
///
/// Only accept PSBTs that send us money. Identify those outputs with identify_receiver_outputs() to proceed
#[derive(Clone)]
pub struct OutputsUnknown(PdkOutputsUnknown);

impl From<PdkOutputsUnknown> for OutputsUnknown {
    fn from(value: PdkOutputsUnknown) -> Self {
        Self(value)
    }
}

impl OutputsUnknown {
    /// Find which outputs belong to the receiver
    pub fn identify_receiver_outputs(
        &self,
        is_receiver_output: Box<dyn IsScriptOwned>,
    ) -> Result<Arc<ProvisionalProposal>, PayjoinError> {
        match self.0.clone().identify_receiver_outputs(|output_script| {
            let res = is_receiver_output.callback(output_script.to_bytes());
            match res {
                Ok(e) => Ok(e),
                Err(e) => Err(PdkError::Server(e.into())),
            }
        }) {
            Ok(e) => Ok(Arc::new(e.into())),
            Err(e) => Err(e.into()),
        }
    }
}

///A mutable checked proposal that the receiver may contribute inputs to make a payjoin.
pub struct ProvisionalProposal(Mutex<PdkProvisionalProposal>);

impl From<PdkProvisionalProposal> for ProvisionalProposal {
    fn from(value: PdkProvisionalProposal) -> Self {
        Self(Mutex::new(value))
    }
}

pub trait ProcessPartiallySignedTransaction {
    fn callback(&self, psbt: String) -> Result<String, PayjoinError>;
}

impl ProvisionalProposal {
    fn mutex_guard(&self) -> MutexGuard<'_, payjoin::receive::ProvisionalProposal> {
        self.0.lock().unwrap()
    }
    pub fn substitute_output_address(
        &self,
        substitute_address: String,
    ) -> Result<(), PayjoinError> {
        let address =
            payjoin::bitcoin::Address::from_str(substitute_address.as_str())?.assume_checked();
        Ok(self.mutex_guard().substitute_output_address(address))
    }
    pub fn contribute_witness_input(
        &self,
        txo: TxOut,
        outpoint: OutPoint,
    ) -> Result<(), PayjoinError> {
        let txo: payjoin::bitcoin::blockdata::transaction::TxOut = txo.into();
        Ok(self.mutex_guard().contribute_witness_input(txo, outpoint.into()))
    }

    pub fn contribute_non_witness_input(
        &self,
        tx: Vec<u8>,
        outpoint: OutPoint,
    ) -> Result<(), PayjoinError> {
        let tx: payjoin::bitcoin::Transaction =
            payjoin::bitcoin::consensus::encode::deserialize(&*tx)?;
        Ok(self.mutex_guard().contribute_non_witness_input(tx, outpoint.into()))
    }

    /// Select receiver input such that the payjoin avoids surveillance. Return the input chosen that has been applied to the Proposal.
    ///
    /// Proper coin selection allows payjoin to resemble ordinary transactions. To ensure the resemblance, a number of heuristics must be avoided.
    ///
    /// UIH “Unnecessary input heuristic” is one class of them to avoid. We define UIH1 and UIH2 according to the BlockSci practice BlockSci UIH1 and UIH2:
    pub fn try_preserving_privacy(
        &self,
        candidate_inputs: HashMap<u64, OutPoint>,
    ) -> Result<OutPoint, PayjoinError> {
        let mut _candidate_inputs: HashMap<payjoin::bitcoin::Amount, payjoin::bitcoin::OutPoint> =
            HashMap::new();
        for (key, value) in candidate_inputs.iter() {
            _candidate_inputs.insert(
                payjoin::bitcoin::Amount::from_sat(key.to_owned()),
                value.to_owned().into(),
            );
        }

        match self.mutex_guard().try_preserving_privacy(_candidate_inputs) {
            Ok(e) => Ok(OutPoint { txid: e.txid.to_string(), vout: e.vout }),
            Err(e) => Err(PayjoinError::SelectionError { message: format!("{:?}", e) }),
        }
    }

    pub fn finalize_proposal(
        &self,
        process_psbt: Box<dyn ProcessPartiallySignedTransaction>,
        min_feerate_sat_per_vb: Option<u64>,
    ) -> Result<Arc<PayjoinProposal>, PayjoinError> {
        match self.mutex_guard().clone().finalize_proposal(
            |psbt| {
                match process_psbt.callback(psbt.to_string()) {
                    Ok(e) => Ok(Psbt::from_str(e.as_str()).expect("Invalid process_psbt ")),
                    Err(e) => Err(PdkError::Server(e.into())),
                }
            },
            min_feerate_sat_per_vb.and_then(|x| FeeRate::from_sat_per_vb(x)),
        ) {
            Ok(e) => Ok(Arc::new(PayjoinProposal(e))),
            Err(e) => Err(e.into()),
        }
    }
}

/// A mutable checked proposal that the receiver may contribute inputs to to make a payjoin.
#[derive(Clone)]
pub struct PayjoinProposal(PdkPayjoinProposal);

impl From<PdkPayjoinProposal> for PayjoinProposal {
    fn from(value: PdkPayjoinProposal) -> Self {
        Self(value)
    }
}
impl From<PayjoinProposal> for PdkPayjoinProposal {
    fn from(value: PayjoinProposal) -> Self {
        value.0
    }
}

impl PayjoinProposal {
    pub fn utxos_to_be_locked(&self) -> Vec<OutPoint> {
        let mut outpoints: Vec<OutPoint> = Vec::new();
        for e in self.0.utxos_to_be_locked() {
            outpoints.push((e.to_owned()).into())
        }
        outpoints
    }
    pub fn is_output_substitution_disabled(&self) -> bool {
        self.0.is_output_substitution_disabled()
    }
    pub fn owned_vouts(&self) -> Vec<u64> {
        self.0.owned_vouts().iter().map(|x| *x as u64).collect()
    }
    pub fn psbt(&self) -> String {
        self.0.psbt().to_string()
    }
    pub fn psbt2(&self) -> Psbt {
        self.0.psbt().clone()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    fn get_proposal_from_test_vector() -> Result<UncheckedProposal, PayjoinError> {
        // OriginalPSBT Test Vector from BIP
        // | InputScriptType | Orginal PSBT Fee rate | maxadditionalfeecontribution | additionalfeeoutputindex|
        // |-----------------|-----------------------|------------------------------|-------------------------|
        // | P2SH-P2WPKH     |  2 sat/vbyte          | 0.00000182                   | 0                       |
        let original_psbt =
            "cHNidP8BAHMCAAAAAY8nutGgJdyYGXWiBEb45Hoe9lWGbkxh/6bNiOJdCDuDAAAAAAD+////AtyVuAUAAAAAF6kUHehJ8GnSdBUOOv6ujXLrWmsJRDCHgIQeAAAAAAAXqRR3QJbbz0hnQ8IvQ0fptGn+votneofTAAAAAAEBIKgb1wUAAAAAF6kU3k4ekGHKWRNbA1rV5tR5kEVDVNCHAQcXFgAUx4pFclNVgo1WWAdN1SYNX8tphTABCGsCRzBEAiB8Q+A6dep+Rz92vhy26lT0AjZn4PRLi8Bf9qoB/CMk0wIgP/Rj2PWZ3gEjUkTlhDRNAQ0gXwTO7t9n+V14pZ6oljUBIQMVmsAaoNWHVMS02LfTSe0e388LNitPa1UQZyOihY+FFgABABYAFEb2Giu6c4KO5YW0pfw3lGp9jMUUAAA=";
        let body = original_psbt.as_bytes();

        let headers = Headers::from_vec(body.to_vec());
        UncheckedProposal::from_request(
            body.to_vec(),
            "?maxadditionalfeecontribution=182?additionalfeeoutputindex=0".to_string(),
            Arc::new(headers),
        )
    }

    #[test]
    fn can_get_proposal_from_request() {
        let proposal = get_proposal_from_test_vector();
        assert!(proposal.is_ok(), "OriginalPSBT should be a valid request");
    }

    struct MockScriptOwned {}

    struct MockOutputOwned {}

    impl IsOutputKnown for MockOutputOwned {
        fn callback(&self, outpoint: OutPoint) -> Result<bool, PayjoinError> {
            println!("{:?}", outpoint);
            Ok(true)
        }
    }

    impl IsScriptOwned for MockScriptOwned {
        fn callback(&self, script: Vec<u8>) -> Result<bool, PayjoinError> {
            let network = payjoin::bitcoin::Network::Bitcoin;
            let script = payjoin::bitcoin::ScriptBuf::from_bytes(script);
            Ok(payjoin::bitcoin::Address::from_script(&script, network)
                == payjoin::bitcoin::Address::from_str("3CZZi7aWFugaCdUCS15dgrUUViupmB8bVM")
                    .map(|x| x.require_network(network).expect("Invalid address")))
        }
    }

    #[test]
    fn unchecked_proposal_unlocks_after_checks() {
        let proposal = get_proposal_from_test_vector().unwrap();
        let _payjoin = proposal
            .assume_interactive_receiver()
            .clone()
            .check_inputs_not_owned(Box::new(MockScriptOwned {}))
            .expect("No inputs should be owned")
            .check_no_mixed_input_scripts()
            .expect("No mixed input scripts")
            .check_no_inputs_seen_before(Box::new(MockOutputOwned {}))
            .expect("No inputs should be seen before")
            .identify_receiver_outputs(Box::new(MockScriptOwned {}))
            .expect("Receiver output should be identified");
    }
}
