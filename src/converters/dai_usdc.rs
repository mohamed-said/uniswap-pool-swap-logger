use crate::converters::AmountType;
use crate::converters::{BigUint, ToBigUint};
use crate::converters::{FromPrimitive,  Num};

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

        let mut number = BigUint::from_str_radix(value, radix)?;

        number = if (&number >> 255) & BigUint::from_i8(1).unwrap() == BigUint::from_i8(1).unwrap() {
            negative = true;
            Self::convert_negative(&number)
        } else {
            number
        };

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
