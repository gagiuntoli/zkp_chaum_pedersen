mod secp256k1;

use num::traits::One;
use num_bigint::BigUint;
use rand::{thread_rng, distributions::Alphanumeric, Rng};
use secp256k1::Secp256k1Point;

/// The possible kind of errors returned by this library.
#[derive(Debug)]
pub enum Error {
    InvalidArguments,
}

/// An enum use to select from the beginning of the program execution which
/// cyclic group is going to be used.
#[derive(Debug, Default)]
pub enum Group {
    #[default]
    Scalar,
    EllipticCurve,
}

/// Structure to represent the cyclic group field.
#[derive(Debug, Clone, PartialEq)]
pub enum Point {
    Scalar(BigUint),
    ECPoint(BigUint, BigUint),
}

/// Detects if any argument is --scalar or --elliptic and returns the
/// corresponding cyclic group to use.
///
/// * `args` - Vector of command line arguments.
pub fn parse_group_from_command_line(args: Vec<String>) -> Group {
    match args.len() {
        2 => match args[1].trim() {
            "--elliptic" => Group::EllipticCurve,
            "--scalar" | "" => Group::Scalar,
            _ => panic!("Invalid argument [--scalar(default)|--elliptic] available."),
        },
        _ => Group::Scalar,
    }
}

/// Returns the default constants to use in both server and clients. Note that
/// they should be the same for both for the ZK algorithm to work.
///
/// * `group` - The cyclic group to use.
pub fn get_constants(group: &Group) -> (BigUint, BigUint, Point, Point) {
    match group {
        Group::Scalar => get_constants_scalar(),
        Group::EllipticCurve => get_constants_elliptic_curve(),
    }
}

pub fn get_constants_scalar() -> (BigUint, BigUint, Point, Point) {
    (
        BigUint::from(10009u32),
        BigUint::from(5004u32),
        Point::Scalar(BigUint::from(3u32)),
        Point::Scalar(BigUint::from(2892u32)),
    )
}

pub fn get_constants_elliptic_curve() -> (BigUint, BigUint, Point, Point) {
    let g = Secp256k1Point::generator();
    let h = g.clone().scale(BigUint::from(13u32));
    (
        Secp256k1Point::prime(),
        Secp256k1Point::n(),
        Point::from_secp256k1(&g),
        Point::from_secp256k1(&h),
    )
}

impl Point {
    /// Serializes the Point structure to an array of bytes to transferring it
    /// through the network.
    pub fn serialize(self: &Self) -> Vec<u8> {
        match self {
            Point::Scalar(x) => x.to_bytes_be(),
            Point::ECPoint(x, y) => {
                let mut x = x.to_bytes_be();
                let mut y = y.to_bytes_be();
                let diff = (x.len() as i32) - (y.len() as i32);
                if diff > 0 {
                    y.resize(y.len() + diff as usize, 0);
                    y.rotate_right(diff as usize);
                } else {
                    x.resize(x.len() + (-diff as usize), 0);
                    x.rotate_right((-diff) as usize);
                }
                x.append(&mut y);
                x
            }
        }
    }

    /// Deserializes the Point structure from an array of bytes and transforms
    /// it into an actual Point structure.
    pub fn deserialize(v: Vec<u8>, group: &Group) -> Point {
        match group {
            Group::Scalar => Point::deserialize_into_scalar(v),
            Group::EllipticCurve => Point::deserialize_into_ecpoint(v),
        }
    }

    pub fn deserialize_into_scalar(v: Vec<u8>) -> Point {
        Point::Scalar(BigUint::from_bytes_be(&v))
    }

    pub fn deserialize_into_ecpoint(v: Vec<u8>) -> Point {
        let len = v.len();

        assert!(
            len % 2 == 0,
            "The length of the serialized object should be even"
        );

        Point::ECPoint(
            BigUint::from_bytes_be(&v[..len / 2]),
            BigUint::from_bytes_be(&v[len / 2..]),
        )
    }

    /// Converts a point from the `secp256k1` library into a Point
    pub fn from_secp256k1(point: &Secp256k1Point) -> Point {
        match point {
            Secp256k1Point::Coor { x, y, .. } => Point::ECPoint(x.number.clone(), y.number.clone()),
            _ => panic!("elliptic_curves::Point::Zero not convertible into a point"),
        }
    }
}

/// Exponenciates two points g & h:
///  - For the integer or scalar group the new ones are: g^exp & h^exp
///  - For the elliptic curve group the new ones are: exp * g & exp * h
pub fn exponentiates_points(
    exp: &BigUint,
    g: &Point,
    h: &Point,
    p: &BigUint,
) -> Result<(Point, Point), Error> {
    match (g, h) {
        (Point::Scalar(g), Point::Scalar(h)) => Ok(exponentiates_points_scalar(exp, g, h, p)),
        (Point::ECPoint(gx, gy), Point::ECPoint(hx, hy)) => {
            Ok(exponentiates_points_elliptic_curve(exp, gx, gy, hx, hy))
        }
        _ => Err(Error::InvalidArguments),
    }
}

pub fn exponentiates_points_scalar(
    exp: &BigUint,
    g: &BigUint,
    h: &BigUint,
    p: &BigUint,
) -> (Point, Point) {
    (
        Point::Scalar(g.modpow(exp, p)),
        Point::Scalar(h.modpow(exp, p)),
    )
}

pub fn exponentiates_points_elliptic_curve(
    exp: &BigUint,
    gx: &BigUint,
    gy: &BigUint,
    hx: &BigUint,
    hy: &BigUint,
) -> (Point, Point) {
    let g = Secp256k1Point::from_bigint(&gx, &gy);
    let h = Secp256k1Point::from_bigint(&hx, &hy);

    let g = g.scale(exp.clone());
    let h = h.scale(exp.clone());

    let g_new = match g {
        secp256k1::Point::Coor { x, y, .. } => Point::ECPoint(x.number, y.number),
        _ => panic!("You reach the Zero in elliptic curve multiplication"),
    };

    let h_new = match h {
        secp256k1::Point::Coor { x, y, .. } => Point::ECPoint(x.number, y.number),
        _ => panic!("You reach the Zero in elliptic curve multiplication"),
    };

    (g_new, h_new)
}

/// This function solves the ZK challenge `s` proposed by the verifier.
///
/// s = (k - c * x) mod q
///
/// * `x_secret` - secret password.
/// * `k` - random number selected by the prover.
/// * `c` - random number selected by the verifier.
/// * `q` - the order of the cyclic group
pub fn solve_zk_challenge_s(x_secret: &BigUint, k: &BigUint, c: &BigUint, q: &BigUint) -> BigUint {
    let cx = c * x_secret;
    if *k > cx {
        (k - cx).modpow(&BigUint::one(), q)
    } else {
        q - (cx - k).modpow(&BigUint::one(), q)
    }
}

pub fn verify(
    r1: &Point,
    r2: &Point,
    y1: &Point,
    y2: &Point,
    g: &Point,
    h: &Point,
    c: &BigUint,
    s: &BigUint,
    p: &BigUint,
) -> Result<bool, Error> {
    match (r1, r2, y1, y2, g, h) {
        (
            Point::Scalar(r1),
            Point::Scalar(r2),
            Point::Scalar(y1),
            Point::Scalar(y2),
            Point::Scalar(g),
            Point::Scalar(h),
        ) => Ok(verify_scalar(r1, r2, y1, y2, g, h, c, s, p)),
        (
            Point::ECPoint(r1x, r1y),
            Point::ECPoint(r2x, r2y),
            Point::ECPoint(y1x, y1y),
            Point::ECPoint(y2x, y2y),
            Point::ECPoint(gx, gy),
            Point::ECPoint(hx, hy),
        ) => Ok(verify_ecpoint(
            r1x, r1y, r2x, r2y, y1x, y1y, y2x, y2y, gx, gy, hx, hy, c, s,
        )),
        _ => Err(Error::InvalidArguments),
    }
}

/// This function verifies that the challenge `s` was properly solved by the
/// prover for integer cyclic groups.
/// r1 = g^s * y1^c && r2 = h^s * y2^c
///
/// * `r1` - g^k generated by the prover.
/// * `r2` - h^k generated by the prover.
/// * `y1` - g^x generated by the prover.
/// * `y2` - h^x generated by the prover.
/// * `g` - predefined element of the cyclic group.
/// * `h` - predefined element of the cyclic group (g^n mod q ?).
/// * `c` - random number generated by the verifier.
/// * `s` - solution to the challenge computed by the prover.
/// * `p` - the prime number used to defined the cyclic group.
pub fn verify_scalar(
    r1: &BigUint,
    r2: &BigUint,
    y1: &BigUint,
    y2: &BigUint,
    g: &BigUint,
    h: &BigUint,
    c: &BigUint,
    s: &BigUint,
    p: &BigUint,
) -> bool {
    let condition_1 = *r1 == (g.modpow(s, p) * y1.modpow(c, p)).modpow(&BigUint::one(), p);
    let condition_2 = *r2 == (h.modpow(s, p) * y2.modpow(c, p)).modpow(&BigUint::one(), p);
    condition_1 && condition_2
}

/// This function verifies that the challenge `s` was properly solved by the
/// prover for elliptic curves cyclic group (secp256k1).
/// r1 = s * g + c * y1 && r2 = s * h + c * y2
///
/// * `r1x` - x coordinate of k * g generated by the prover.
/// * `r1y` - y coordinate of k * g generated by the prover.
/// * `r2x` - x coordinate of k * h generated by the prover.
/// * `r2y` - y coordinate of k * h generated by the prover.
/// * `y1x` - x coordinate of x * g generated by the prover.
/// * `y1y` - y coordinate of x * g generated by the prover.
/// * `y2x` - x coordinate of x * h generated by the prover.
/// * `y2y` - x coordinate of x * h generated by the prover.
/// * `gx` - x coordinate of predefined point of the cyclic group.
/// * `gy` - y coordinate of predefined point of the cyclic group.
/// * `hx` - x coordinate of predefined point of the cyclic group.
/// * `hy` - y coordinate of predefined point of the cyclic group.
/// * `c` - random number generated by the verifier.
/// * `s` - solution to the challenge computed by the prover.
/// * `p` - the prime number used to defined the cyclic group.
pub fn verify_ecpoint(
    r1x: &BigUint,
    r1y: &BigUint,
    r2x: &BigUint,
    r2y: &BigUint,
    y1x: &BigUint,
    y1y: &BigUint,
    y2x: &BigUint,
    y2y: &BigUint,
    gx: &BigUint,
    gy: &BigUint,
    hx: &BigUint,
    hy: &BigUint,
    c: &BigUint,
    s: &BigUint,
) -> bool {
    let g = Secp256k1Point::from_bigint(&gx, &gy);
    let h = Secp256k1Point::from_bigint(&hx, &hy);
    let y1 = Secp256k1Point::from_bigint(&y1x, &y1y);
    let y2 = Secp256k1Point::from_bigint(&y2x, &y2y);
    let r1 = Secp256k1Point::from_bigint(&r1x, &r1y);
    let r2 = Secp256k1Point::from_bigint(&r2x, &r2y);

    let sg = g.scale(s.clone());
    let sh = h.scale(s.clone());
    let cy1 = y1.scale(c.clone());
    let cy2 = y2.scale(c.clone());

    (r1 == sg + cy1) && (r2 == sh + cy2)
}

/// Generates a random array of bytes which can be use as a secret.
///
/// Warning: Don't use it for production purposes. Better pseudo random
/// generators should be used.
pub fn get_random_array<const BYTES: usize>() -> [u8; BYTES] {
    let mut arr = [0u8; BYTES];
    thread_rng()
        .try_fill(&mut arr[..])
        .expect("Fail to generate array of random number.");
    return arr;
}

/// Generates a 32-bytes random number
///
/// Warning: Don't use it for production purposes.
pub fn get_random_number() -> BigUint {
    BigUint::from_bytes_be(&get_random_array::<32>())
}

/// Generates a random string of any length. It is useful to generates user or
/// session IDs.
pub fn get_random_string(n: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_random_array() {
        let a = get_random_array::<32>();
        let b = get_random_array::<32>();
        let c = get_random_array::<32>();
        let d = get_random_array::<32>();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(c, d);
    }

    #[test]
    fn test_get_random_number() {
        let a = get_random_number();
        let b = get_random_number();
        let c = get_random_number();
        let d = get_random_number();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(c, d);
    }

    #[test]
    fn test_exponentiates_points_scalar() {
        let p = BigUint::from(10009u32);
        let g = BigUint::from(3u32);
        let h = BigUint::from(2892u32);

        let secret = BigUint::from(300u32);

        let (y1, y2) = exponentiates_points_scalar(&secret, &g, &h, &p);

        assert_eq!(y1, Point::Scalar(BigUint::from(6419u32)));
        assert_eq!(y2, Point::Scalar(BigUint::from(4984u32)));
    }

    #[test]
    fn test_solve_zk_challenge_s() {
        // test positive k - cx
        let x = BigUint::from(3u32);
        let c = BigUint::from(3u32);
        let k = BigUint::from(10u32);
        let q = BigUint::from(10u32);

        // s = 10 - 3 * 3 mod 10 = 1
        assert_eq!(solve_zk_challenge_s(&x, &k, &c, &q), BigUint::one());

        // test negative k - cx
        let x = BigUint::from(4u32);
        let c = BigUint::from(3u32);
        let k = BigUint::from(10u32);
        let q = BigUint::from(10u32);

        // s = 10 - 3 * 4 mod 10 = 8
        assert_eq!(solve_zk_challenge_s(&x, &k, &c, &q), BigUint::from(8u32));
    }

    #[test]
    fn test_verify_scalar_success_toy_example_1() {
        let p = BigUint::from(10009u32);
        let q = (&p - BigUint::one()) / BigUint::from(2u32);

        let x = BigUint::from(300u32);
        let g = Point::Scalar(BigUint::from(3u32));
        let h = Point::Scalar(BigUint::from(2892u32));

        let (y1, y2) = exponentiates_points(&x, &g, &h, &p).unwrap();

        let k = BigUint::from(10u32);
        let (r1, r2) = exponentiates_points(&k, &g, &h, &p).unwrap();

        let c = BigUint::from(894u32);

        let s = solve_zk_challenge_s(&x, &k, &c, &q);

        let verification = verify(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p).unwrap();
        assert!(verification)
    }

    #[test]
    fn test_verify_scalar_success_toy_example_2() {
        let p = BigUint::from(23u32);
        let q = BigUint::from(11u32);

        let x = BigUint::from(6u32);
        let g = Point::Scalar(BigUint::from(4u32));
        let h = Point::Scalar(BigUint::from(9u32));

        let (y1, y2) = exponentiates_points(&x, &g, &h, &p).unwrap();

        assert_eq!(y1, Point::Scalar(BigUint::from(2u32)));
        assert_eq!(y2, Point::Scalar(BigUint::from(3u32)));

        let k = BigUint::from(7u32);
        let (r1, r2) = exponentiates_points(&k, &g, &h, &p).unwrap();

        assert_eq!(r1, Point::Scalar(BigUint::from(8u32)));
        assert_eq!(r2, Point::Scalar(BigUint::from(4u32)));

        let c = BigUint::from(4u32);

        let s = solve_zk_challenge_s(&x, &k, &c, &q);
        assert_eq!(s, BigUint::from(5u32));

        let verification = verify(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p).unwrap();
        assert!(verification)
    }

    #[test]
    fn test_verify_scalar_failure_toy_example_1() {
        let p = BigUint::from(23u32);
        let q = BigUint::from(11u32);

        let x = BigUint::from(6u32);

        let g = Point::Scalar(BigUint::from(4u32));
        let h = Point::Scalar(BigUint::from(9u32));

        let (y1, y2) = exponentiates_points(&x, &g, &h, &p).unwrap();
        assert_eq!(y1, Point::Scalar(BigUint::from(2u32)));
        assert_eq!(y2, Point::Scalar(BigUint::from(3u32)));

        let k = BigUint::from(7u32);
        let (r1, r2) = exponentiates_points(&k, &g, &h, &p).unwrap();

        assert_eq!(r1, Point::Scalar(BigUint::from(8u32)));
        assert_eq!(r2, Point::Scalar(BigUint::from(4u32)));

        let c = BigUint::from(4u32);

        let mut s = solve_zk_challenge_s(&x, &k, &c, &q);

        // we compute `s` slightly bad
        s = s - BigUint::one();

        let verification = verify(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p).unwrap();
        assert!(!verification)
    }

    #[test]
    fn test_verify_elliptic_curve_success_example_1() {
        let p = Secp256k1Point::prime();
        let q = Secp256k1Point::n();

        let x = BigUint::from(300u32);
        let g = Secp256k1Point::generator();
        let h = g.clone().scale(BigUint::from(13u32));

        let g = Point::from_secp256k1(&g);
        let h = Point::from_secp256k1(&h);
        let (y1, y2) = exponentiates_points(&x, &g, &h, &p).unwrap();

        let k = BigUint::from(10u32);
        let (r1, r2) = exponentiates_points(&k, &g, &h, &p).unwrap();

        let c = BigUint::from(894u32);

        let s = solve_zk_challenge_s(&x, &k, &c, &q);

        let verification = verify(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p).unwrap();
        assert!(verification)
    }

    #[test]
    fn test_verify_elliptic_curve_failure_example_1() {
        let p = Secp256k1Point::prime();
        let q = Secp256k1Point::n();

        let x = BigUint::from(300u32);
        let g = Secp256k1Point::generator();
        let h = g.clone().scale(BigUint::from(13u32));

        let g = Point::from_secp256k1(&g);
        let h = Point::from_secp256k1(&h);
        let (y1, y2) = exponentiates_points(&x, &g, &h, &p).unwrap();

        let k = BigUint::from(10u32);
        let (r1, r2) = exponentiates_points(&k, &g, &h, &p).unwrap();

        let c = BigUint::from(894u32);

        let s = solve_zk_challenge_s(&x, &k, &c, &q) + BigUint::one();

        let verification = verify(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p).unwrap();
        assert!(!verification)
    }

    #[test]
    fn test_serialize() {
        let p = Point::Scalar(BigUint::from(65256u32));

        assert_eq!(p.serialize(), vec![0xfe, 0xe8]);

        let p = Point::ECPoint(BigUint::from(65256u32), BigUint::from(8475u32));

        assert_eq!(p.serialize(), vec![0xfe, 0xe8, 0x21, 0x1b]);

        // one array is longer than the other
        let p = Point::ECPoint(BigUint::from(65256u32), BigUint::from(83957234u32));

        assert_eq!(
            p.serialize(),
            vec![0x00, 0x00, 0xfe, 0xe8, 0x05, 0x01, 0x15, 0xf2]
        );

        // the other way around
        let p = Point::ECPoint(BigUint::from(83957234u32), BigUint::from(65256u32));

        assert_eq!(
            p.serialize(),
            vec![0x05, 0x01, 0x15, 0xf2, 0x00, 0x00, 0xfe, 0xe8]
        );
    }

    #[test]
    fn test_deserialize() {
        let p = Point::deserialize(vec![0xfe, 0xe8], &Group::Scalar);

        assert_eq!(p, Point::Scalar(BigUint::from(65256u32)));

        let p = Point::deserialize(vec![0xfe, 0xe8, 0x21, 0x1b], &Group::EllipticCurve);

        assert_eq!(
            p,
            Point::ECPoint(BigUint::from(65256u32), BigUint::from(8475u32))
        );

        // one array is longer than the other
        let p = Point::deserialize(
            vec![0x00, 0x00, 0xfe, 0xe8, 0x05, 0x01, 0x15, 0xf2],
            &Group::EllipticCurve,
        );

        assert_eq!(
            p,
            Point::ECPoint(BigUint::from(65256u32), BigUint::from(83957234u32))
        );

        // the other way around
        let p = Point::deserialize(
            vec![0x05, 0x01, 0x15, 0xf2, 0x00, 0x00, 0xfe, 0xe8],
            &Group::EllipticCurve,
        );

        assert_eq!(
            p,
            Point::ECPoint(BigUint::from(83957234u32), BigUint::from(65256u32))
        );
    }
}
