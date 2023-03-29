use tonic::{transport::Server, Request, Response, Status};

pub mod zkp_auth {
    include!("../zkp_auth.rs");
}

use zkp_auth::auth_client::AuthClient;
use zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = "http://127.0.0.1:50051";

    println!("connecting to {}", server_addr);
    let mut client = AuthClient::connect(server_addr).await?;

    println!("sending register request to {}", server_addr);
    let response = client
        .register(RegisterRequest {
            user: String::from("guido"),
            y1: vec![0x01, 0x02, 0x03],
            y2: vec![0x01, 0x02, 0x03],
        })
        .await?;

    Ok(())
}
