#![allow(dead_code)]

use num::{Integer, One};
use num_bigint::{BigInt, BigUint, ToBigInt};
use std::ops::{Add, Div, Mul, Sub};

#[derive(PartialEq, Debug, Clone)]
pub struct FiniteField {
    pub number: BigUint,
    pub prime: BigUint,
}

impl FiniteField {
    pub fn from_bytes_be(number: &[u8], prime: &[u8]) -> Self {
        let number = BigUint::from_bytes_be(number);
        let prime = BigUint::from_bytes_be(prime);

        FiniteField { number, prime }
    }

    fn check_equal_order_and_panic(self: &Self, rhs: &FiniteField) {
        if self.prime != rhs.prime {
            panic!(
                "Finite fields elements have different order lhs: {}, rhs: {}",
                self.prime, rhs.prime
            )
        }
    }

    pub fn pow(self, exp: &BigInt) -> FiniteField {
        let exp = exp.mod_floor(&(self.prime.clone() - BigUint::one()).to_bigint().unwrap());
        let exp = exp.to_biguint().unwrap();

        let exp = exp.modpow(&BigUint::one(), &(self.prime.clone() - BigUint::one()));

        FiniteField {
            number: self.number.modpow(&exp, &self.prime),
            prime: self.prime,
        }
    }

    pub fn scale(self, scalar: BigUint) -> FiniteField {
        FiniteField {
            number: (self.number * scalar) % self.prime.clone(),
            prime: self.prime,
        }
    }
}

impl From<(u32, u32)> for FiniteField {
    fn from(tuple: (u32, u32)) -> Self {
        FiniteField {
            number: BigUint::from(tuple.0),
            prime: BigUint::from(tuple.1),
        }
    }
}

impl From<(BigUint, BigUint)> for FiniteField {
    fn from(tuple: (BigUint, BigUint)) -> Self {
        FiniteField {
            number: tuple.0,
            prime: tuple.1,
        }
    }
}

impl Add for FiniteField {
    type Output = FiniteField;

    fn add(self, _rhs: FiniteField) -> FiniteField {
        self.check_equal_order_and_panic(&_rhs);

        FiniteField {
            number: (self.number + _rhs.number) % self.prime.clone(),
            prime: self.prime,
        }
    }
}

impl Sub for FiniteField {
    type Output = FiniteField;

    fn sub(self, rhs: FiniteField) -> FiniteField {
        self.check_equal_order_and_panic(&rhs);

        if self.number >= rhs.number {
            FiniteField {
                number: (self.number - rhs.number) % self.prime.clone(),
                prime: self.prime,
            }
        } else {
            FiniteField {
                number: (self.number + self.prime.clone() - rhs.number) % self.prime.clone(),
                prime: self.prime,
            }
        }
    }
}

impl Mul for FiniteField {
    type Output = FiniteField;

    fn mul(self, rhs: FiniteField) -> FiniteField {
        self.check_equal_order_and_panic(&rhs);

        FiniteField {
            number: (self.number * rhs.number) % self.prime.clone(),
            prime: self.prime,
        }
    }
}

impl Div for FiniteField {
    type Output = FiniteField;

    fn div(self, rhs: FiniteField) -> FiniteField {
        self.check_equal_order_and_panic(&rhs);

        self.clone() * rhs.pow(&(self.prime - BigUint::from(2u32)).to_bigint().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_and_non_equal() {
        let prime = 11;
        let a = FiniteField::from((6, prime));
        let b = FiniteField::from((5, prime));
        let c = FiniteField::from((5, prime));

        assert_ne!(a, b);
        assert_eq!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn test_add() {
        let a = FiniteField::from((6, 11));
        let b = FiniteField::from((5, 11));
        let c = FiniteField::from((0, 11));

        assert_eq!(a + b, c);

        let a = FiniteField::from((44, 57));
        let b = FiniteField::from((33, 57));
        let c = FiniteField::from((20, 57));

        assert_eq!(a + b, c);

        let a = FiniteField::from((17, 57));
        let b = FiniteField::from((42, 57));
        let c = FiniteField::from((49, 57));
        let d = FiniteField::from((51, 57));

        assert_eq!(a + b + c, d);
    }

    #[test]
    #[should_panic(expected = "Finite fields elements have different order lhs: 12, rhs: 11")]
    fn test_fail_adding_different_orders() {
        let a = FiniteField::from((6, 12));
        let b = FiniteField::from((5, 11));

        let _c = a + b;
    }

    #[test]
    fn test_substract() {
        let a = FiniteField::from((6, 11));
        let b = FiniteField::from((5, 11));
        let c = FiniteField::from((1, 11));

        assert_eq!(a - b, c);

        let a = FiniteField::from((52, 57));
        let b = FiniteField::from((30, 57));
        let c = FiniteField::from((38, 57));
        let d = FiniteField::from((41, 57));

        assert_eq!(a - b - c, d);
    }

    #[test]
    #[should_panic(expected = "Finite fields elements have different order lhs: 12, rhs: 11")]
    fn test_fail_substracting_different_orders() {
        let a = FiniteField::from((6, 12));
        let b = FiniteField::from((5, 11));

        let _c = a - b;
    }

    #[test]
    fn test_multiply() {
        let a = FiniteField::from((6, 11));
        let b = FiniteField::from((5, 11));
        let c = FiniteField::from((8, 11));

        assert_eq!(a * b, c);
    }

    #[test]
    #[should_panic(expected = "Finite fields elements have different order lhs: 12, rhs: 11")]
    fn test_fail_multiplying_different_orders() {
        let a = FiniteField::from((6, 12));
        let b = FiniteField::from((5, 11));

        let _c = a * b;
    }

    #[test]
    fn test_exponentiation() {
        let a = FiniteField::from((7, 19));
        let b = FiniteField::from((1, 19));

        assert_eq!(a.pow(&BigInt::from(3u32)), b);

        let a = FiniteField::from((9, 19));
        let b = FiniteField::from((7, 19));

        assert_eq!(a.pow(&BigInt::from(12u32)), b);

        let a = FiniteField::from((12, 97));
        let b = FiniteField::from((77, 97));
        let c = FiniteField::from((63, 97));

        assert_eq!(a.pow(&BigInt::from(7u32)) * b.pow(&BigInt::from(49u32)), c);
    }

    #[test]
    fn test_exercise_5() {
        let all_elements = (0..19)
            .map(|i| FiniteField::from((i, 19)))
            .collect::<Vec<FiniteField>>();

        assert!((0..19)
            .map(|i| FiniteField::from(((i * 1) % 19, 19)))
            .all(|elem| all_elements.contains(&elem)));

        assert!((0..19)
            .map(|i| FiniteField::from(((i * 3) % 19, 19)))
            .all(|elem| all_elements.contains(&elem)));

        assert!((0..19)
            .map(|i| FiniteField::from(((i * 7) % 19, 19)))
            .all(|elem| all_elements.contains(&elem)));

        assert!((0..19)
            .map(|i| FiniteField::from(((i * 13) % 19, 19)))
            .all(|elem| all_elements.contains(&elem)));

        assert!((0..19)
            .map(|i| FiniteField::from(((i * 18) % 19, 19)))
            .all(|elem| all_elements.contains(&elem)));
    }

    #[test]
    fn test_exercise_7() {
        assert!((1..7)
            .map(|i| FiniteField::from((i, 7)).pow(&BigInt::from(7u32 - 1)))
            .all(|elem| elem == FiniteField::from((1, 7))));

        assert!((1..11)
            .map(|i| FiniteField::from((i, 11)).pow(&BigInt::from(11u32 - 1)))
            .all(|elem| elem == FiniteField::from((1, 11))));

        assert!((1..17)
            .map(|i| FiniteField::from((i, 17)).pow(&BigInt::from(17u32 - 1)))
            .all(|elem| elem == FiniteField::from((1, 17))));

        assert!((1..31)
            .map(|i| FiniteField::from((i, 31)).pow(&BigInt::from(31u32 - 1)))
            .all(|elem| elem == FiniteField::from((1, 31))));
    }

    #[test]
    fn test_operations() {
        let a = FiniteField::from((3, 31));
        let b = FiniteField::from((24, 31));
        let c = FiniteField::from((4, 31));

        assert_eq!(a / b, c);

        let a = FiniteField::from((17, 31));
        let b = FiniteField::from((29, 31));

        assert_eq!(a.pow(&BigInt::from(-3)), b);

        let a = FiniteField::from((4, 31));
        let b = FiniteField::from((11, 31));
        let c = FiniteField::from((13, 31));

        assert_eq!(a.pow(&BigInt::from(-4)) * b, c);
    }

    #[test]
    fn test_from_bytes_be() {
        let a = FiniteField::from_bytes_be(&[0x01, 0x02], &[0x01, 0x12]);
        let b = FiniteField::from((258, 274));

        assert_eq!(a, b);
    }
}
