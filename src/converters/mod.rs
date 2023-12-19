use crate::logger::AmountType;
use num_bigint::BigUint;
use num_traits::{FromPrimitive, Num, ToPrimitive};

pub mod dai_usdc;

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
