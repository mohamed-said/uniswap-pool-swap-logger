use crate::{
	converters::{AmountType, BigUint, FromPrimitive, Num, ToPrimitive},
	logger::AmountError,
};

pub struct DaiUsdc;

impl DaiUsdc {
	/// convert negative hex number to decimal representation
	fn convert_negative(number: &BigUint) -> BigUint {
		let complement_2s = format!("{:0256b}", number);
		let complement_1s = complement_2s
			.chars()
			.map(|c| match c {
				'0' => '1',
				'1' => '0',
				_ => panic!("invalid bit"),
			})
			.collect::<String>();

		BigUint::parse_bytes(complement_1s.as_bytes(), 2).unwrap() + 1_u128
	}

	/// Converts the transferred (or swapped) amount from hex format to decimal format
	pub fn amount_to_decimal(
		value: &str,
		radix: u32,
		amount_type: &AmountType,
	) -> Result<String, Box<dyn std::error::Error>> {
		let mut negative = false;

		// precision factor depending on whether the type is DAI or USDC
		let factor = amount_type.to_biguint_factor();

		let mut number = BigUint::from_str_radix(value, radix)
			.map_err(|_| Box::new(AmountError::AmountInvalid))?;

		// check if the numver is negative by checking the most significant bit
		if let Some(mask) = BigUint::from_i8(1) {
			number = if (&number >> 255) & &mask == mask {
				negative = true;
				Self::convert_negative(&number)
			} else {
				number
			};
		} else {
			return Err(Box::new(AmountError::ParsingFailed));
		}

		let factor_applied = number.to_f64().unwrap() / factor.to_f64().unwrap();

		let mut res = match factor_applied < 0_f64 {
			true => number.to_f64().unwrap(),
			false => factor_applied,
		};

		if negative {
			res = -res;
		}

		Ok(res.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const RADIX: u32 = 16;

	/// Hex to decimal calculations verified with: https://www.exploringbinary.com/twos-complement-converter/
	mod dai {
		use super::*;
		const AMOUNT_TYPE: AmountType = AmountType::DAI;

		#[test]
		fn valid_negative_number_is_parsed_successfully() {
			let amount_hex = "ffffffffffffffffffffffffffffffffffffffffffffffe22f377a065f280000";
			let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
			assert!(res.is_ok());
			assert_eq!(res.unwrap(), String::from("-550"));
		}

		#[test]
		fn valid_positive_number_is_parsed_successfully() {
			let amount_hex = "12e7c5758742fa0d8000";
			let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
			assert!(res.is_ok());
			// to save time, but testing floating points should be done more carefully
			assert_eq!(res.unwrap(), String::from("89278.023"));
		}

		#[test]
		fn invalid_number_returns_err() {
			let amount_hex = "12g7c5758742fa0d8000";
			let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
			assert!(res.is_err());
		}
	}

	mod usdc {
		use super::*;
		const AMOUNT_TYPE: AmountType = AmountType::USDC;

		#[test]
		fn valid_negative_number_is_parsed_successfully() {
			let amount_hex = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffeb372399e8";
			let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
			assert!(res.is_ok());
			assert_eq!(res.unwrap(), String::from("-89269.233176"));
		}

		#[test]
		fn valid_positive_number_is_parsed_successfully() {
			let amount_hex = "34d38ca30";
			let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
			assert!(res.is_ok());
			assert_eq!(res.unwrap(), "14180.469296");
		}

		#[test]
		fn invalid_number_returns_err() {
			let amount_hex = "34h38ca30";
			let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
			assert!(res.is_err());
		}
	}
}
