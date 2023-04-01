#![allow(dead_code)]

use crate::finite_field::FiniteField;
use crate::elliptic_curves::Point;

use num_bigint::BigUint;

pub type Secp256k1Point = Point;

impl Secp256k1Point {
    pub fn prime() -> BigUint {
        let prime = hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")
            .unwrap();
        BigUint::from_bytes_be(&prime)
    }

    pub fn n() -> BigUint {
        let n = hex::decode("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141")
            .unwrap();
        BigUint::from_bytes_be(&n)
    }

    pub fn a() -> FiniteField {
        FiniteField::from_bytes_be(&[0u8], &Self::prime().to_bytes_be())
    }

    pub fn b() -> FiniteField {
        FiniteField::from_bytes_be(&[7u8], &Self::prime().to_bytes_be())
    }

    pub fn generator() -> Point {
        let gx = hex::decode("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
            .unwrap();
        let gy = hex::decode("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8")
            .unwrap();

        Secp256k1Point::from_bytes_be(&gx, &gy)
    }

    pub fn compute_public_key(e: &BigUint) -> Point {
        Secp256k1Point::generator().scale(e.clone())
    }

    pub fn n_minus_2() -> BigUint {
        Self::n() - BigUint::from(2u32)
    }

    pub fn from_bigint(x: &BigUint, y: &BigUint) -> Point {
        Self::from_bytes_be(&x.to_bytes_be(), &y.to_bytes_be())
    }

    pub fn from_bytes_be(x: &[u8], y: &[u8]) -> Point {
        let prime = hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")
            .unwrap();

        let x = FiniteField::from_bytes_be(&x, &prime);
        let y = FiniteField::from_bytes_be(&y, &prime);

        let point = Point::Coor {
            a: Self::a(),
            b: Self::b(),
            x: x.clone(),
            y: y.clone(),
        };

        if !Point::is_on_curve(&point) {
            panic!("({:?},{:?}) point is not in the curve", x, y);
        }

        point
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_bitcoin_generator_point() {
        let point = Secp256k1Point::generator();

        assert!(Point::is_on_curve(&point));

        let n = hex::decode("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141")
            .unwrap();

        assert_eq!(point.scale(BigUint::from_bytes_be(&n)), Point::Zero);
    }
}
