use std::{str::FromStr, sync::Arc};
use bitcoin::address::NetworkValidation;

use payjoin::{bitcoin::address::NetworkChecked};
use payjoin::bitcoin::address::NetworkUnchecked;

use crate::{
	error::PayjoinError,
	send::{RequestBuilder, Context, Request},
	transaction::PartiallySignedTransaction,
	Address,
};

pub struct PrjUriRequest {
	pub request: Request,
	pub context: Arc<Context>,
}

impl From<payjoin::Uri<'static, NetworkChecked>> for Uri {
	fn from(value: payjoin::Uri<'static, NetworkChecked>) -> Self {
		Uri(PayjoinUriWrapper::Checked(value))
	}
}
impl From<payjoin::Uri<'static, NetworkUnchecked>> for Uri {
	fn from(value: payjoin::Uri<'static, NetworkUnchecked>) -> Self {
		Uri(PayjoinUriWrapper::UnChecked(value))
	}
}
#[derive(Clone)]
pub struct Uri (PayjoinUriWrapper);
#[derive(Clone)]
enum PayjoinUriWrapper {
	Checked(payjoin::Uri<'static, NetworkChecked>),
	UnChecked(payjoin::Uri<'static, NetworkUnchecked>)
}

impl From<Uri> for payjoin::Uri<'static, NetworkChecked> {
	fn from(uri: Uri) -> Self {
		match uri.0 {
			PayjoinUriWrapper::Checked(e) => e,
			PayjoinUriWrapper::UnChecked(e) => e.assume_checked()
		}
	}
}
impl From<Uri> for payjoin::Uri<'static, NetworkUnchecked> {
	fn from(uri: Uri) -> Self {
		match uri.0 {
			PayjoinUriWrapper::UnChecked(e) => e,
			PayjoinUriWrapper::Checked(e) => panic!("Network already validated!"),
		}
	}
}

impl Uri {
	pub fn new(uri: String) -> Result<Self, PayjoinError> {
		match payjoin::Uri::from_str(uri.as_str()) {
			Ok(e) => Ok(e.into()),
			Err(e) => Err(PayjoinError::PjParseError { message: e.to_string() }),
		}
	}

	pub fn assume_checked(&self) -> Result<Self, PayjoinError> {
		match self.clone().0 {
			PayjoinUriWrapper::Checked(e) => Ok(e.into()),
			PayjoinUriWrapper::UnChecked(e) => Ok(e.assume_checked().into())
		}
	}
}

pub struct PjUri(payjoin::PjUri<'static>);






#[derive(Clone, Debug, PartialEq)]
pub struct Amount {
	internal: u64,
}

impl From<Amount> for payjoin::bitcoin::Amount {
	fn from(value: Amount) -> Self {
		payjoin::bitcoin::Amount::from_sat(value.internal)
	}
}

impl Amount {
	pub fn from_sat(sats: u64) -> Self {
		Self { internal: sats }
	}
	pub fn from_btc(btc: f64) -> Self {
		Self { internal: (btc as u64) * 100000000 }
	}
	pub fn to_sat(&self) -> u64 {
		self.internal
	}
	pub fn to_btc(&self) -> f64 {
		(self.internal as f64) / (100000000f64)
	}
}

pub struct Url {
	internal: url::Url,
}

impl Url {
	pub fn new(input: String) -> Result<Url, PayjoinError> {
		match url::Url::from_str(input.as_str()) {
			Ok(e) => Ok(Self { internal: e }),
			Err(e) => Err(PayjoinError::UnexpectedError { message: e.to_string() }),
		}
	}
	pub fn query(&self) -> Option<String> {
		self.internal.query().map(|x| x.to_string())
	}
}

#[cfg(test)]
mod tests {
	use std::convert::TryFrom;

	use payjoin::Uri;

	#[test]
	fn test_short() {
		assert!(Uri::try_from("").is_err());
		assert!(Uri::try_from("bitcoin").is_err());
		assert!(Uri::try_from("bitcoin:").is_err());
	}

	#[ignore]
	#[test]
	fn test_todo_url_encoded() {
		let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=https://example.com?ciao";
		assert!(Uri::try_from(uri).is_err(), "pj url should be url encoded");
	}

	#[test]
	fn test_valid_url() {
		let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=this_is_NOT_a_validURL";
		assert!(Uri::try_from(uri).is_err(), "pj is not a valid url");
	}

	#[test]
	fn test_missing_amount() {
		let uri =
            "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?pj=https://testnet.demo.btcpayserver.org/BTC/pj";
		assert!(Uri::try_from(uri).is_ok(), "missing amount should be ok");
	}

	#[test]
	fn test_unencrypted() {
		let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=http://example.com";
		assert!(Uri::try_from(uri).is_err(), "unencrypted connection");

		let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=ftp://foo.onion";
		assert!(Uri::try_from(uri).is_err(), "unencrypted connection");
	}

	#[test]
	fn test_valid_uris() {
		let https = "https://example.com";
		let onion = "http://vjdpwgybvubne5hda6v4c5iaeeevhge6jvo3w2cl6eocbwwvwxp7b7qd.onion";

		let base58 = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX";
		let bech32_upper = "BITCOIN:TB1Q6D3A2W975YNY0ASUVD9A67NER4NKS58FF0Q8G4";
		let bech32_lower = "bitcoin:tb1q6d3a2w975yny0asuvd9a67ner4nks58ff0q8g4";

		for address in [base58, bech32_upper, bech32_lower].iter() {
			for pj in [https, onion].iter() {
				let uri = format!("{}?amount=1&pj={}", address, pj);
				assert!(Uri::try_from(&*uri).is_ok());
			}
		}
	}

	#[test]
	fn test_unsupported() {
		assert!(!Uri::try_from("bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX")
			.unwrap()
			.extras
			.pj_is_supported());
	}
}
