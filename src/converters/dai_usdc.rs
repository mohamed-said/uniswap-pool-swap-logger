use crate::converters::AmountType;
use crate::converters::{BigUint, ToBigUint};
use crate::converters::{FromPrimitive,  Num};
use crate::logger::AmountError;

pub struct DaiUsdc;

impl DaiUsdc {
    fn convert_negative(number: &BigUint) -> BigUint {
        let complement_2s = format!("{:0256b}", number);
        let complement_1s = complement_2s.chars().map(|c| {
            match c {
                '0' => '1',
                '1' => '0',
                _ => panic!("invalid bit"),
            }
        }).collect::<String>();

        BigUint::parse_bytes(complement_1s.as_bytes(), 2).unwrap() + 1_u128
    }

    pub fn amount_to_decimal(value: &str, radix: u32, amount_type: &AmountType) -> Result<String, Box<dyn std::error::Error>> {
        let mut negative = false;
        let factor = amount_type.to_biguint_factor();

        let mut number = BigUint::from_str_radix(value, radix)
            .map_err(|_| Box::new(AmountError::AmountInvalid))?;

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

        let factor_applied = &number / &factor;
        number = match factor_applied < 0.to_biguint().unwrap() {
            true => number,
            false => number / factor,
        };

        let mut res = number.to_string();

        if negative {
            res.insert(0, '-');
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADIX: u32 = 16;

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
            assert_eq!(res.unwrap(), String::from("89278"));
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
            assert_eq!(res.unwrap(), String::from("-89269"));
        }

        #[test]
        fn valid_positive_number_is_parsed_successfully() {
            let amount_hex = "34d38ca30";
            let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), "14180");
        }

        #[test]
        fn invalid_number_returns_err() {
            let amount_hex = "34h38ca30";
            let res = DaiUsdc::amount_to_decimal(amount_hex, RADIX, &AMOUNT_TYPE);
            assert!(res.is_err());
        }
    }
}

