use num_bigint::{BigUint, ToBigUint};
use std::fmt::Display;

pub mod swap_logger;

#[derive(Debug)]
pub enum AmountError {
	AllAmountsAreNegative,
	AmountInvalid,
	ParsingFailed,
}

impl std::error::Error for AmountError {}

impl std::fmt::Display for AmountError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self {
			AmountError::AllAmountsAreNegative => write!(f, "Swap amounts cannot be both negative"),
			AmountError::AmountInvalid => write!(f, "Amount data is corrupt or invalid"),
			AmountError::ParsingFailed => {
				write!(f, "Error while parsing amounts to decimal values")
			},
		}
	}
}

pub enum AmountType {
	DAI,
	USDC,
}

impl Display for AmountType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self {
			AmountType::DAI => write!(f, "DAI"),
			AmountType::USDC => write!(f, "USDC"),
		}
	}
}

impl AmountType {
	pub fn to_biguint_factor(&self) -> BigUint {
		match self {
			Self::DAI => 10u64.pow(18).to_biguint().unwrap(),
			Self::USDC => 10u64.pow(6).to_biguint().unwrap(),
		}
	}
}
