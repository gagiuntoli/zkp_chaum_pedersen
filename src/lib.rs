use num::traits::One;
use num_bigint::BigUint;
use rand::thread_rng;
use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum Point {
    Scalar(BigUint),
    ECPoint(BigUint, BigUint),
}

impl Point {
    /// The serialization of the ECPoint consist of joining both coordinates and
    /// padding 0s at the begginning of the shortest to make it equal to the
    /// longest.
    pub fn serialize(self: &Self) -> Vec<u8> {
        match self {
            Point::Scalar(x) => x.to_bytes_be(),
            Point::ECPoint(x, y) => {
                let mut x = x.to_bytes_be();
                let mut y = y.to_bytes_be();
                let diff = x.len() as i32 - y.len() as i32;
                if diff > 0 {
                    y.resize(y.len() + diff as usize, 0);
                    y.rotate_right(diff as usize);
                } else {
                    x.resize(x.len() - (-diff as usize), 0);
                    x.rotate_right((-diff) as usize);
                }
                x.append(&mut y);
                x
            }
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
}

pub fn compute_y(secret: BigUint, g: Point, h: Point, q: &BigUint) -> (Point, Point) {
    match (g, h) {
        (Point::Scalar(g), Point::Scalar(h)) => compute_y_scalar(&secret, &g, &h, &q),
        (Point::ECPoint(gx, gy), Point::ECPoint(hx, hy)) => {
            compute_y_ecpoint(&secret, &gx, &gy, &hx, &hy, &q)
        }
        _ => panic!("g & h should be the same type"),
    }
}

pub fn compute_y_scalar(
    x_secret: &BigUint,
    g: &BigUint,
    h: &BigUint,
    q: &BigUint,
) -> (Point, Point) {
    (
        Point::Scalar(g.modpow(x_secret, q)),
        Point::Scalar(h.modpow(x_secret, q)),
    )
}

pub fn compute_y_ecpoint(
    x_secret: &BigUint,
    gx: &BigUint,
    gy: &BigUint,
    hx: &BigUint,
    hy: &BigUint,
    q: &BigUint,
) -> (Point, Point) {
    todo!()
}

/// This function computes `s` which is the challenge proposed by the verifier.
/// s = k - cx mod q
///
/// * `x_secret` - secret password.
/// * `k` - random number selected by the prover.
/// * `c` - random number selected by the verifier.
/// * `q` - a prime number that divides p - 1 evenly.
pub fn compute_challenge_s(x_secret: &BigUint, k: &BigUint, c: &BigUint, q: &BigUint) -> BigUint {
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
) -> bool {
    match (r1, r2, y1, y2, g, h) {
        (
            Point::Scalar(r1),
            Point::Scalar(r2),
            Point::Scalar(y1),
            Point::Scalar(y2),
            Point::Scalar(g),
            Point::Scalar(h),
        ) => verify_scalar(r1, r2, y1, y2, g, h, c, s, p),
        (
            Point::ECPoint(r1x, r1y),
            Point::ECPoint(r2x, r2y),
            Point::ECPoint(y1x, y1y),
            Point::ECPoint(y2x, y2y),
            Point::ECPoint(gx, gy),
            Point::ECPoint(hx, hy),
        ) => verify_ecpoint(
            r1x, r1y, r2x, r2y, y1x, y1y, y2x, y2y, gx, gy, hx, hy, c, s, p,
        ),
        _ => panic!("g & h should be the same type"),
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
    p: &BigUint,
) -> bool {
    todo!()
}

/// This functions generates a random 256 bits number than can be use as a
/// secret.
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

pub fn get_random_number<const BYTES: usize>() -> BigUint {
    BigUint::from_bytes_be(&get_random_array::<BYTES>())
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
        let a = get_random_number::<32>();
        let b = get_random_number::<32>();
        let c = get_random_number::<32>();
        let d = get_random_number::<32>();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(c, d);
    }

    #[test]
    fn test_compute_y_scalar() {
        let q = BigUint::from(10009u32);
        let g = BigUint::from(3u32);
        let h = BigUint::from(2892u32);

        let secret = BigUint::from(300u32);

        let (y1, y2) = compute_y_scalar(&secret, &g, &h, &q);

        assert_eq!(y1, Point::Scalar(BigUint::from(6419u32)));
        assert_eq!(y2, Point::Scalar(BigUint::from(4984u32)));
    }

    #[test]
    fn test_compute_challenge_s() {
        // test positive k - cx
        let x = BigUint::from(3u32);
        let c = BigUint::from(3u32);
        let k = BigUint::from(10u32);
        let q = BigUint::from(10u32);

        // s = 10 - 3 * 3 mod 10 = 1
        assert_eq!(compute_challenge_s(&x, &k, &c, &q), BigUint::one());

        // test negative k - cx
        let x = BigUint::from(4u32);
        let c = BigUint::from(3u32);
        let k = BigUint::from(10u32);
        let q = BigUint::from(10u32);

        // s = 10 - 3 * 4 mod 10 = 8
        assert_eq!(compute_challenge_s(&x, &k, &c, &q), BigUint::from(8u32));
    }

    #[test]
    fn test_verify_scalar_success_example_1() {
        let p = BigUint::from(10009u32);
        let q = (&p - BigUint::one()) / BigUint::from(2u32);

        let x = BigUint::from(300u32);
        let g = BigUint::from(3u32);
        let h = BigUint::from(2892u32);

        let y1 = g.modpow(&x, &p);
        let y2 = h.modpow(&x, &p);

        let k = BigUint::from(10u32);
        let r1 = g.modpow(&k, &p);
        let r2 = h.modpow(&k, &p);

        let c = BigUint::from(894u32);

        let s = compute_challenge_s(&x, &k, &c, &q);

        let verification = verify_scalar(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p);
        assert!(verification)
    }

    #[test]
    fn test_verify_scalar_success_example_2() {
        let p = BigUint::from(23u32);
        let q = BigUint::from(11u32);

        let x = BigUint::from(6u32);
        let g = BigUint::from(4u32);
        let h = BigUint::from(9u32);

        let y1 = g.modpow(&x, &p);
        let y2 = h.modpow(&x, &p);
        assert_eq!(y1, BigUint::from(2u32));
        assert_eq!(y2, BigUint::from(3u32));

        let k = BigUint::from(7u32);
        let r1 = g.modpow(&k, &p);
        let r2 = h.modpow(&k, &p);
        assert_eq!(r1, BigUint::from(8u32));
        assert_eq!(r2, BigUint::from(4u32));

        let c = BigUint::from(4u32);

        let s = compute_challenge_s(&x, &k, &c, &q);
        assert_eq!(s, BigUint::from(5u32));

        let verification = verify_scalar(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p);
        assert!(verification)
    }

    #[test]
    fn test_verify_scalar_failure_example_1() {
        let p = BigUint::from(23u32);
        let q = BigUint::from(11u32);

        let x = BigUint::from(6u32);
        let g = BigUint::from(4u32);
        let h = BigUint::from(9u32);

        let y1 = g.modpow(&x, &p);
        let y2 = h.modpow(&x, &p);
        assert_eq!(y1, BigUint::from(2u32));
        assert_eq!(y2, BigUint::from(3u32));

        let k = BigUint::from(7u32);
        let r1 = g.modpow(&k, &p);
        let r2 = h.modpow(&k, &p);
        assert_eq!(r1, BigUint::from(8u32));
        assert_eq!(r2, BigUint::from(4u32));

        let c = BigUint::from(4u32);

        let mut s = compute_challenge_s(&x, &k, &c, &q);

        // we compute `s` slightly bad
        s = s - BigUint::one();

        let verification = verify_scalar(&r1, &r2, &y1, &y2, &g, &h, &c, &s, &p);
        assert!(!verification)
    }
}
