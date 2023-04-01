use crate::finite_field::FiniteField;
use hex;
use num::{One, Zero};
use num_bigint::{BigInt, BigUint};
use std::fmt::{self, Debug};
use std::ops::Add;

#[derive(PartialEq, Clone)]
pub enum Point {
    Coor {
        a: FiniteField,
        b: FiniteField,
        x: FiniteField,
        y: FiniteField,
    },
    Zero,
}

impl Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Point::Coor { x, y, .. } = self {
            write!(
                f,
                "Point [x = {} y = {}]",
                hex::encode(&x.number.to_bytes_be()),
                hex::encode(&y.number.to_bytes_be())
            )
        } else {
            write!(f, "Point = Zero")
        }
    }
}

impl Point {
    pub fn new(a: &FiniteField, b: &FiniteField, x: &FiniteField, y: &FiniteField) -> Point {
        let point = Point::Coor {
            a: a.clone(),
            b: b.clone(),
            x: x.clone(),
            y: y.clone(),
        };
        if !Self::is_on_curve(&point) {
            panic!("({:?},{:?}) point is not in the curve", x, y);
        }
        point
    }

    #[allow(dead_code)]
    fn zero() -> Self {
        Point::Zero
    }

    #[allow(dead_code)]
    fn is_zero(self) -> bool {
        self == Point::Zero
    }

    pub fn is_on_curve(p: &Point) -> bool {
        match p {
            Point::Coor { a, b, x, y } => {
                return y.clone().pow(&BigInt::from(2u32))
                    == x.clone().pow(&BigInt::from(3u32)) + a.clone() * x.clone() + b.clone()
            }
            Point::Zero => true,
        }
    }

    // TODO: take a reference for the scalar
    #[allow(dead_code)]
    pub fn scale(self, _scalar: BigUint) -> Self {
        let mut current = self.clone();
        let mut scalar = _scalar;
        let mut result = Point::Zero;

        while scalar != BigUint::zero() {
            if &scalar & BigUint::one() != BigUint::zero() {
                result = current.clone() + result;
            }
            current = current.clone() + current;
            scalar = scalar >> 1;
        }
        return result;
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        match (self.clone(), rhs.clone()) {
            (Point::Zero, _) => return rhs,
            (_, Point::Zero) => return self,
            (
                Point::Coor { a, b, x, y },
                Point::Coor {
                    a: a_rhs,
                    b: b_rhs,
                    x: x_rhs,
                    y: y_rhs,
                    ..
                },
            ) => {
                if a != a_rhs || b != b_rhs {
                    panic!(
                        "The points (x:{:?},y:{:?},a:{:?},b:{:?}) and (x:{:?},y:{:?},a:{:?},b:{:?}) belong to different curves",
                        x, y, a, b, x_rhs, y_rhs, a_rhs, b_rhs
                    );
                }
                if x == x_rhs && y != y_rhs {
                    Point::Zero
                } else if self == rhs && y == x_rhs.clone().scale(BigUint::zero()) {
                    Point::Zero
                } else if x != x_rhs {
                    let s = (y_rhs.clone() - y.clone()) / (x_rhs.clone() - x.clone());
                    let x_res = s.clone().pow(&BigInt::from(2u32)) - x.clone() - x_rhs.clone();
                    let y_res = s.clone() * (x.clone() - x_res.clone()) - y;

                    Point::Coor {
                        a,
                        b,
                        x: x_res,
                        y: y_res,
                    }
                } else {
                    let s = (x
                        .clone()
                        .pow(&BigInt::from(2u32))
                        .scale(BigUint::from(3u32))
                        + a.clone())
                        / (y.clone().scale(BigUint::from(2u32)));
                    let x_res =
                        s.clone().pow(&BigInt::from(2u32)) - x.clone().scale(BigUint::from(2u32));
                    let y_res = s * (x - x_res.clone()) - y;
                    return Point::Coor {
                        a,
                        b,
                        x: x_res,
                        y: y_res,
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_on_curve() {
        let prime = 223;
        let a = FiniteField::from((0, prime));
        let b = FiniteField::from((7, prime));

        // on the curve
        let x = FiniteField::from((192, prime));
        let y = FiniteField::from((105, prime));

        assert!(Point::is_on_curve(&Point::Coor {
            a: a.clone(),
            b: b.clone(),
            x,
            y
        }));

        let x = FiniteField::from((17, prime));
        let y = FiniteField::from((56, prime));

        assert!(Point::is_on_curve(&Point::Coor {
            a: a.clone(),
            b: b.clone(),
            x,
            y
        }));

        let x = FiniteField::from((1, prime));
        let y = FiniteField::from((193, prime));

        assert!(Point::is_on_curve(&Point::Coor {
            a: a.clone(),
            b: b.clone(),
            x,
            y
        }));

        // not on the curve
        let x = FiniteField::from((200, prime));
        let y = FiniteField::from((119, prime));

        assert!(!Point::is_on_curve(&Point::Coor {
            a: a.clone(),
            b: b.clone(),
            x,
            y
        }));

        let x = FiniteField::from((42, prime));
        let y = FiniteField::from((99, prime));

        assert!(!Point::is_on_curve(&Point::Coor { a, b, x, y }));
    }

    #[test]
    fn test_point_addition() {
        let prime = 223;
        let a = FiniteField::from((0, prime));
        let b = FiniteField::from((7, prime));

        let x = FiniteField::from((192, prime));
        let y = FiniteField::from((105, prime));

        let p1 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((17, prime));
        let y = FiniteField::from((56, prime));

        let p2 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((170, prime));
        let y = FiniteField::from((142, prime));

        let p3 = Point::new(&a, &b, &x, &y);

        assert_eq!(p1 + p2, p3);

        // (170,142) + (60, 139)
        let x = FiniteField::from((170, prime));
        let y = FiniteField::from((142, prime));

        let p1 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((60, prime));
        let y = FiniteField::from((139, prime));

        let p2 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((220, prime));
        let y = FiniteField::from((181, prime));

        let p3 = Point::new(&a, &b, &x, &y);

        assert_eq!(p1 + p2, p3);

        // (47,71) + (17,56)
        let x = FiniteField::from((47, prime));
        let y = FiniteField::from((71, prime));

        let p1 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((17, prime));
        let y = FiniteField::from((56, prime));

        let p2 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((215, prime));
        let y = FiniteField::from((68, prime));

        let p3 = Point::new(&a, &b, &x, &y);

        assert_eq!(p1 + p2, p3);

        // (143,98) + (76,66)
        let x = FiniteField::from((143, prime));
        let y = FiniteField::from((98, prime));

        let p1 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((76, prime));
        let y = FiniteField::from((66, prime));

        let p2 = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((47, prime));
        let y = FiniteField::from((71, prime));

        let p3 = Point::new(&a, &b, &x, &y);

        assert_eq!(p1 + p2, p3);
    }

    #[test]
    fn test_scale() {
        let prime = 223;
        let a = FiniteField::from((0, prime));
        let b = FiniteField::from((7, prime));

        let x = FiniteField::from((47, prime));
        let y = FiniteField::from((71, prime));

        let p = Point::new(&a, &b, &x, &y);

        let x = FiniteField::from((47, prime));
        let y = FiniteField::from((71, prime));
        let pr = Point::new(&a, &b, &x, &y);
        assert_eq!(p.clone().scale(BigUint::from(1u32)), pr);

        let x = FiniteField::from((36, prime));
        let y = FiniteField::from((111, prime));
        let pr = Point::new(&a, &b, &x, &y);
        assert_eq!(p.clone().scale(BigUint::from(2u32)), pr);

        let x = FiniteField::from((15, prime));
        let y = FiniteField::from((137, prime));
        let pr = Point::new(&a, &b, &x, &y);
        assert_eq!(p.clone().scale(BigUint::from(3u32)), pr);

        let x = FiniteField::from((194, prime));
        let y = FiniteField::from((51, prime));
        let pr = Point::new(&a, &b, &x, &y);
        assert_eq!(p.clone().scale(BigUint::from(4u32)), pr);

        let x = FiniteField::from((47, prime));
        let y = FiniteField::from((152, prime));
        let pr = Point::new(&a, &b, &x, &y);
        assert_eq!(p.clone().scale(BigUint::from(20u32)), pr);

        assert_eq!(p.scale(BigUint::from(21u32)), Point::Zero);
    }

    #[test]
    fn test_bitcoin_generator_point() {
        let prime = hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")
            .unwrap();

        let a = hex::decode("00").unwrap();
        let b = hex::decode("07").unwrap();

        let gx = hex::decode("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
            .unwrap();
        let gy = hex::decode("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8")
            .unwrap();

        let a = FiniteField::from_bytes_be(&a, &prime);
        let b = FiniteField::from_bytes_be(&b, &prime);
        let gx = FiniteField::from_bytes_be(&gx, &prime);
        let gy = FiniteField::from_bytes_be(&gy, &prime);

        assert!(Point::is_on_curve(&Point::Coor {
            a: a.clone(),
            b: b.clone(),
            x: gx.clone(),
            y: gy.clone()
        }));

        let n = hex::decode("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141")
            .unwrap();
        let p = Point::Coor {
            a,
            b,
            x: gx.clone(),
            y: gy.clone(),
        };

        assert_eq!(p.scale(BigUint::from_bytes_be(&n)), Point::Zero);
    }
}
