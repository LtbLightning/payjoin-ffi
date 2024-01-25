use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use bdk::bitcoin::psbt::PartiallySignedTransaction;
use bdk::bitcoin::{Address, Script, Transaction};
use bdk::blockchain::EsploraBlockchain;
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex;
use bdk::{FeeRate, LocalUtxo, SignOptions, Wallet as BdkWallet};
use payjoin::bitcoin::consensus::encode::serialize_hex;
use payjoin_ffi::error::PayjoinError;
use payjoin_ffi::receive::{
    Headers, IsOutputKnown, IsScriptOwned, PayjoinProposal, ProcessPartiallySignedTransaction,
    UncheckedProposal,
};
use payjoin_ffi::types::{Network, OutPoint, Request, TxOut};
use payjoin_ffi::uri::Uri;
use uniffi::deps::log::debug;

pub struct EsploraClient(EsploraBlockchain);

impl EsploraClient {
    pub fn new(url: String) -> Self {
        let client = EsploraBlockchain::new(url.as_str(), 10);
        Self(client)
    }

    #[allow(dead_code)]
    pub fn broadcast(&self, transaction: bdk::bitcoin::Transaction) -> Result<(), Box<dyn Error>> {
        match self.0.broadcast(&transaction) {
            Ok(_) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
        }
    }
}
#[derive(Debug)]
pub struct Wallet {
    inner_mutex: Mutex<BdkWallet<MemoryDatabase>>,
}
impl Wallet {
    pub fn new_no_persist(descriptor: String, network: Network) -> Result<Self, Box<dyn Error>> {
        let wallet =
            BdkWallet::new(descriptor.as_str(), None, network.into(), MemoryDatabase::new())?;

        Ok(Wallet { inner_mutex: Mutex::new(wallet) })
    }

    pub(crate) fn get_wallet(&self) -> MutexGuard<BdkWallet<MemoryDatabase>> {
        self.inner_mutex.lock().expect("wallet")
    }

    pub fn get_address(&self, address_index: AddressIndex) -> Address {
        self.get_wallet().get_address(address_index.into()).unwrap().address
    }

    pub fn get_balance(&self) -> String {
        self.get_wallet().get_balance().unwrap().to_string()
    }

    pub fn is_mine(&self, script: &Script) -> Result<bool, bdk::Error> {
        self.get_wallet().is_mine(&script)
    }
    #[allow(dead_code)]
    pub fn list_unspent(&self) -> Vec<LocalUtxo> {
        self.get_wallet().list_unspent().unwrap()
    }
    pub fn sync(&self, client: &EsploraClient) {
        self.get_wallet().sync(&client.0, Default::default()).unwrap();
    }
    pub(crate) fn sign(
        &self,
        psbt: &mut PartiallySignedTransaction,
    ) -> Result<PartiallySignedTransaction, Box<dyn Error>> {
        match self
            .get_wallet()
            .sign(psbt, SignOptions { trust_witness_utxo: true, ..Default::default() })
        {
            Ok(_) => Ok(psbt.clone()),
            Err(e) => panic!("{}", e.to_string()),
        }
    }
}
fn get_sender_descriptor() -> String {
    "wpkh(tprv8ZgxMBicQKsPfE37eJfsg7gA1xMKgwXjgUnLKPTXEW3RsxJo8N9yFkwc29rjsnt1Vko4Px2xDd9fE1NRDvJMnMvyypiNRT9s27vmdFTwwJk)".to_string()
}
fn get_receiver_descriptor() -> String {
    "wpkh(tprv8ZgxMBicQKsPdKDr8NzmXxnSXTSH5YifkTnJfWURVvM9g8SjJbEYAwgo3spnZXTahWUjHtXptXDmx2oejhGVtmxvMU6sT3NdbvXNZPPmeRk)".to_string()
}
#[allow(dead_code)]
fn extract_pj_tx(sender_wallet: &Wallet, psbt: &str) -> Result<Transaction, Box<dyn Error>> {
    let mut psbt: PartiallySignedTransaction =
        PartiallySignedTransaction::from_str(psbt).expect("Invalid psbt");
    let signed_psbt = sender_wallet.sign(&mut psbt)?;
    debug!("Sender's Payjoin PSBT: {:#?}", signed_psbt);
    Ok(signed_psbt.extract_tx())
}

#[allow(dead_code)]
fn is_success(status: u16) -> bool {
    status >= 200 && status < 300
}

fn restore_wallet(descriptor: String) -> Result<Wallet, Box<dyn Error>> {
    match Wallet::new_no_persist(descriptor.to_string(), Network::Regtest) {
        Ok(e) => Ok(e),
        Err(e) => panic!("{}", e.to_string()),
    }
}

fn restore_esplora_client(url: String) -> EsploraClient {
    EsploraClient::new(url)
}
fn init_sender_receiver_wallet() -> (Wallet, Wallet) {
    let sender = restore_wallet(get_sender_descriptor()).expect("Wallet::new failed");
    let receiver = restore_wallet(get_receiver_descriptor()).expect("Wallet::new failed");
    (sender, receiver)
}

#[allow(dead_code)]
fn broadcast_tx(esplora_client: EsploraClient, tx: Transaction) -> Result<(), Box<dyn Error>> {
    esplora_client.broadcast(tx)
}
fn build_pj_uri<'a>(
    address: String,
    amount: u64,
    pj: &str,
    ohttp: Option<&str>,
) -> Result<Uri, Box<dyn Error>> {
    let pj_uri_string =
        format!("{}?amount={}&pj={}", address, (amount as f64 / 100_000_000.0), pj,);
    if let Some(ohttp) = ohttp {
        format!("{} {} {}", pj_uri_string, "&ohttp={}", ohttp);
    }
    debug!("PJ URI: {}", &pj_uri_string);
    match Uri::from_str(pj_uri_string) {
        Ok(e) => Ok(e),
        Err(e) => panic!("{}", e.to_string()),
    }
}

#[allow(dead_code)]
fn handle_proposal(proposal: UncheckedProposal, receiver: Wallet) -> Arc<PayjoinProposal> {
    let receiver = Arc::new(receiver);
    // in a payment processor where the sender could go offline, this is where you schedule to broadcast the original_tx
    let _to_broadcast_in_failure_case = proposal.extract_tx_to_schedule_broadcast();

    // Receive Check 1: Can Broadcast
    let proposal = proposal
        .check_broadcast_suitability(None, Box::new(MockCanBroadcast()))
        .expect("Payjoin proposal should be broadcastable");

    // Receive Check 2: receiver can't sign for proposal inputs
    let proposal = proposal
        .check_inputs_not_owned(Box::new(MockScriptOwned { 0: Arc::clone(&receiver) }))
        .expect("Receiver should not own any of the inputs");

    // Receive Check 3: receiver can't sign for proposal inputs
    let proposal = proposal.check_no_mixed_input_scripts().unwrap();

    // Receive Check 4: have we seen this input before? More of a check for non-interactive i.e. payment processor receivers.
    let payjoin = proposal
        .check_no_inputs_seen_before(Box::new(MockOutputOwned {}))
        .unwrap()
        .identify_receiver_outputs(Box::new(MockScriptOwned { 0: Arc::clone(&receiver) }))
        .expect("Receiver should have at least one output");

    // Select receiver payjoin inputs. TODO Lock them.
    let available_inputs = receiver.list_unspent();
    let candidate_inputs: HashMap<u64, OutPoint> = available_inputs
        .iter()
        .map(|i| {
            (i.txout.value, OutPoint { txid: i.outpoint.txid.to_string(), vout: i.outpoint.vout })
        })
        .collect();
    let selected_outpoint = payjoin.try_preserving_privacy(candidate_inputs).expect("gg");
    let selected_utxo = available_inputs
        .iter()
        .find(|i| {
            i.outpoint.txid.to_string() == selected_outpoint.txid
                && i.outpoint.vout == selected_outpoint.vout
        })
        .unwrap();

    //  calculate receiver payjoin outputs given receiver payjoin inputs and original_psbt,
    let txo_to_contribute = TxOut {
        value: selected_utxo.txout.value,
        script_pubkey: selected_utxo.txout.script_pubkey.clone().into_bytes(),
    };
    let outpoint_to_contribute = OutPoint {
        txid: selected_utxo.outpoint.txid.to_string(),
        vout: selected_utxo.outpoint.vout,
    };
    payjoin
        .contribute_witness_input(txo_to_contribute, outpoint_to_contribute)
        .expect("contribute_witness_input error");

    let receiver_substitute_address = receiver.get_address(AddressIndex::New).to_string();
    payjoin
        .substitute_output_address(receiver_substitute_address)
        .expect("substitute_output_address error");
    let payjoin_proposal = payjoin
        .finalize_proposal(
            Box::new(MockProcessPartiallySignedTransaction(Arc::clone(&receiver))),
            Some(10),
        )
        .expect("finalize error");
    payjoin_proposal
}

#[allow(dead_code)]
fn handle_pj_request(req: Request, headers: Headers, receiver: Wallet) -> String {
    let proposal = UncheckedProposal::from_request(
        req.body,
        req.url.query().unwrap_or("".to_string()),
        Arc::new(headers),
    )
    .unwrap();
    let proposal = handle_proposal(proposal, receiver);

    let psbt = proposal.psbt();
    psbt
}
fn build_original_psbt(
    sender_wallet: &Wallet,
    pj_uri: &Uri,
) -> Result<PartiallySignedTransaction, Box<dyn Error>> {
    let wallet_mutex = sender_wallet.get_wallet();
    let mut builder = wallet_mutex.build_tx();
    let script = bdk::bitcoin::Address::from_str(pj_uri.address().as_str())?
        .assume_checked()
        .script_pubkey();
    builder
        .fee_rate(FeeRate::from_sat_per_kwu(2000.0))
        .add_recipient(script, pj_uri.amount().unwrap())
        .only_witness_utxo();
    let (mut psbt, _) = builder.finish()?;
    wallet_mutex
        .sign(&mut psbt, SignOptions { trust_witness_utxo: true, ..Default::default() })
        .unwrap();
    Ok(psbt)
}
mod v1 {
    use bdk::wallet::AddressIndex;
    use payjoin_ffi::send::RequestBuilder;

    use super::*;

    const EXAMPLE_URL: &str = "https://example.com";

    #[test]
    fn v1_to_v1() -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = init_sender_receiver_wallet();
        let esplora_client = restore_esplora_client("http://0.0.0.0:30000".to_string());
        let pj_receiver_address = receiver.get_address(AddressIndex::New);

        println!("\nReceiver address: {:?}", pj_receiver_address.to_string());
        println!("\nSender address: {:?}", sender.get_address(AddressIndex::New).to_string());
        let _ = sender.sync(&esplora_client);
        let _ = receiver.sync(&esplora_client);

        println!("\nSender balance: {:?}", sender.get_balance().to_string());
        println!("\nReceiver balance: {:?}", receiver.get_balance().to_string());

        let pj_uri =
            build_pj_uri(pj_receiver_address.to_qr_uri(), 500000, EXAMPLE_URL, None).unwrap();
        let psbt = build_original_psbt(&sender, &pj_uri)?;
        println!("\nOriginal sender psbt: {:#?}", psbt.to_string());

        let req_ctx = RequestBuilder::from_psbt_and_uri(psbt.to_string(), Arc::new(pj_uri))?
            .build_with_additional_fee(10000, None, 0, false)?
            .extract_v1()?;
        let headers = Headers::from_vec(req_ctx.request.body.clone());
        let response = handle_pj_request(req_ctx.request, headers, receiver);
        println!("\nOriginal receiver psbt: {:#?}", response);
        let checked_payjoin_proposal_psbt = req_ctx
            .context_v1
            .process_response(response.as_bytes().to_vec())
            .expect("process res error");
        let payjoin_tx = extract_pj_tx(&sender, checked_payjoin_proposal_psbt.as_str())?;
        // esplora_client.broadcast(payjoin_tx.clone()).expect("Broadcast error");
        println!("Broadcast success: {}", payjoin_tx.txid().to_string());
        Ok(())
    }
}

struct MockCanBroadcast();

impl payjoin_ffi::receive::CanBroadcast for MockCanBroadcast {
    fn callback(&self, tx: Vec<u8>) -> Result<bool, PayjoinError> {
        debug!("{}", serialize_hex(&tx));
        Ok(true)
    }
}

struct MockProcessPartiallySignedTransaction(Arc<Wallet>);

impl ProcessPartiallySignedTransaction for MockProcessPartiallySignedTransaction {
    fn callback(&self, psbt: String) -> Result<String, PayjoinError> {
        match self.0.sign(&mut PartiallySignedTransaction::from_str(&*psbt.as_str()).unwrap()) {
            Ok(e) => Ok(e.to_string()),
            Err(e) => Err(PayjoinError::UnexpectedError { message: e.to_string() }),
        }
    }
}

struct MockOutputOwned {}
impl IsOutputKnown for MockOutputOwned {
    fn callback(&self, outpoint: OutPoint) -> Result<bool, PayjoinError> {
        debug!("{:?}", outpoint);
        Ok(false)
    }
}

struct MockScriptOwned(Arc<Wallet>);

impl IsScriptOwned for MockScriptOwned {
    fn callback(&self, script: Vec<u8>) -> Result<bool, PayjoinError> {
        self.0
            .clone()
            .is_mine(Script::from_bytes(script.as_slice()))
            .map_err(|x| PayjoinError::UnexpectedError { message: x.to_string() })
    }
}
