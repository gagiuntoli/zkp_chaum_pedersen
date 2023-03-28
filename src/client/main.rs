use tonic::{transport::Server, Request, Response, Status};

pub mod zkp_auth {
    include!("../zkp_auth.rs");
}

use zkp_auth::auth_client::AuthClient;
use zkp_auth::auth_server::{Auth, AuthServer};
use zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

// mod zkp_auth {
//     include!("zkp_auth.rs");
// }

#[derive(Default)]
pub struct AuthImpl {}

#[tonic::async_trait]
impl Auth for AuthImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        println!("Request from {:?}", request.remote_addr());

        let response = RegisterResponse {};

        Ok(Response::new(response))
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        todo!()
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://127.0.0.1:50051";
    let auth = AuthImpl::default();

    let mut client = AuthClient::connect(addr).await?;
    println!("connecting to {}", addr);

    Ok(())
}
