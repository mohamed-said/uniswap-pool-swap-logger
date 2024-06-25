use crate::loggers::AmountType;
use num_bigint::BigUint;
use num_traits::{FromPrimitive, Num, ToPrimitive};

pub mod dai_usdc;

/// Radix values to be used in conversion methods
// The purpose of this enum is to eliminate the usage of magic numbers
// or possible typos in values or in types (u16, u32, ...).
//
// Could be easily extended by adding more radixes if/when needed
pub enum Radix {
	Base16,
}

impl Radix {
	pub fn to_uint(&self) -> u32 {
		match self {
			Self::Base16 => 16_u32,
		}
	}
}
