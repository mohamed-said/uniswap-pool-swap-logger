use num_bigint::{BigUint, ToBigUint};
use std::fmt::Display;

use thiserror::Error;

pub mod swap_logger;

#[derive(Debug, Error)]
/// (Custom) Errors that might happen while the logger is running
pub enum LoggerError {
    #[error("Failed to create Event object with specified event name")]
    FailedToRetrieveEvent,
}


#[derive(Debug, Error)]
/// Errors that might occur with the amounts transferred
pub enum AmountError {
    #[error("Swap amounts cannot be both negative")]
    AllAmountsAreNegative,

    #[error("Amount data is corrupt or invalid")]
    AmountInvalid,

    #[error("Error while parsing amounts to decimal values")]
    ParsingFailed,
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
