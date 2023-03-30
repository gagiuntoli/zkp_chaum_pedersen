use tonic::{transport::Server, Request, Response, Status};
use std::collections::HashMap;
use std::sync::Mutex;
use num_bigint::BigUint;
use std::cell::RefCell;

use chaum_pedersen_zkp::{Point};

pub mod zkp_auth {
    include!("../zkp_auth.rs");
}

use zkp_auth::auth_server::{Auth, AuthServer};
use zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

#[derive(Default)]
enum Group {
    #[default]
    Scalar,
    EllipticCurve,
}

#[derive(Default)]
pub struct AuthImpl {
    registry: Mutex<HashMap<String, UserInfo>>,
    group: Group,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user: String,
    pub y1: Point,
    pub y2: Point,
}

#[tonic::async_trait]
impl Auth for AuthImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        println!("Request from {:?}", request.remote_addr());
        println!("Request: {:?}", request);

        let incoming_user = match (request.into_inner(), &self.group) {
            (RegisterRequest { user, y1, y2 }, Group::Scalar) => UserInfo {
                user,
                y1: Point::deserialize_into_scalar(y1),
                y2: Point::deserialize_into_scalar(y2),
            },
            _ => panic!("Register request is incompatible"),
        };

        let response = RegisterResponse {};

        // we add a new UserInfo if the id is new, we update y1 & y2 if `id` already exists.
        println!("Registry: {:?}", self.registry);

        let registry = &mut *self.registry.lock().unwrap();
        registry.insert(incoming_user.user.clone(), incoming_user);
        println!("Registry: {:?}", self.registry);

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
    let addr = "127.0.0.1:50051".parse().unwrap();
    let auth = AuthImpl::default();
    let registry = HashMap::<String, UserInfo>::new();

    println!("Bookstore server listening on {}", addr);

    Server::builder()
        .add_service(AuthServer::new(auth))
        .serve(addr)
        .await?;

    Ok(())
}
