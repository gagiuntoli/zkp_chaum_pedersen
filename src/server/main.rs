use tonic::{transport::Server, Request, Response, Status};
use std::collections::HashMap;
use std::sync::Mutex;
use num_bigint::BigUint;
use std::cell::RefCell;
use num::traits::Zero;

use chaum_pedersen_zkp::{Point, get_random_number};

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
pub struct InfoRegister {
    pub user: String,
    pub y1: Point,
    pub y2: Point,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user: String,
    pub y1: Point,
    pub y2: Point,
    pub r1: Point,
    pub r2: Point,
}

#[tonic::async_trait]
impl Auth for AuthImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        //println!("Request from {:?}", request.remote_addr());
        println!("Request: {:?}", request);

        let register_request = request.into_inner();
        let response = RegisterResponse {};

        // we add a new UserInfo if the id is new, we update y1 & y2 if `id` already exists.
        println!("Registry: {:?}", self.registry);

        let user_info = match &self.group {
            Group::Scalar => UserInfo {
                user: register_request.user.clone(),
                y1: Point::deserialize_into_scalar(register_request.y1),
                y2: Point::deserialize_into_scalar(register_request.y2),
                r1: Point::Scalar(BigUint::zero()),
                r2: Point::Scalar(BigUint::zero()),
            },
            Group::EllipticCurve => UserInfo {
                user: register_request.user.clone(),
                y1: Point::deserialize_into_ecpoint(register_request.y1),
                y2: Point::deserialize_into_ecpoint(register_request.y2),
                r1: Point::Scalar(BigUint::zero()),
                r2: Point::Scalar(BigUint::zero()),
            },
        };

        let registry = &mut *self.registry.lock().unwrap();
        registry.insert(register_request.user, user_info);

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
        let response = AuthenticationChallengeResponse {
            auth_id: String::from("a"),
            c: get_random_number::<32>().to_bytes_be(),
        };

        Ok(Response::new(response))
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
