use num::BigUint;
use tonic::{transport::Server, Request, Response, Status};

pub mod zkp_auth {
    include!("../zkp_auth.rs");
}

use zkp_auth::auth_client::AuthClient;
use zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

use chaum_pedersen_zkp::{
    Point, get_scalar_constants, get_random_number, compute_new_points, compute_challenge_s,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = "http://127.0.0.1:50051";

    println!("connecting to {}", server_addr);
    let mut client = AuthClient::connect(server_addr).await?;

    let (p, q, g, h) = get_scalar_constants();

    let x = get_random_number::<2>();
    println!("My secret is: {:?}", x);

    let (y1, y2) = compute_new_points(&x, &g, &h, &p);

    // (y1, y2) = (g^x, h^x) secret x
    println!("sending register request to {}", server_addr);
    let response = client
        .register(RegisterRequest {
            user: String::from("guido"),
            y1: y1.serialize(),
            y2: y2.serialize(),
        })
        .await?;
    println!("Response from Server {:?}", response);

    // (r1, r2) = (g^k, h^k) random k
    println!("Sending authentication challenge request");

    let k = get_random_number::<2>();
    println!("The random K is: {:?}", k);

    let (r1, r2) = compute_new_points(&k, &g, &h, &q);

    let response = client
        .create_authentication_challenge(AuthenticationChallengeRequest {
            user: String::from("guido"),
            r1: r1.serialize(),
            r2: r2.serialize(),
        })
        .await?;
    println!("Response from Server {:?}", response);

    let c = response.into_inner().c;
    let c = BigUint::from_bytes_be(&c);
    let s = compute_challenge_s(&x, &k, &c, &q);

    Ok(())
}
