use tonic::{transport::Server, Request, Response, Status, Code};
use std::collections::HashMap;
use std::sync::Mutex;
use num_bigint::BigUint;
use num::traits::Zero;

use chaum_pedersen_zkp::{Group, Point, get_random_number, verify, get_scalar_constants};

pub mod zkp_auth {
    include!("../zkp_auth.rs");
}

use zkp_auth::auth_server::{Auth, AuthServer};
use zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

#[derive(Default)]
pub struct AuthImpl {
    user_registry: Mutex<HashMap<String, UserInfo>>,
    auth_registry: Mutex<HashMap<String, AuthInfo>>,
    group: Group,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user: String,
    pub y1: Point,
    pub y2: Point,
    pub r1: Point,
    pub r2: Point,
}

#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub auth_id: String,
    pub y1: Point,
    pub y2: Point,
    pub r1: Point,
    pub r2: Point,
    pub c: BigUint,
}

#[tonic::async_trait]
impl Auth for AuthImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        println!("Request: {:?}", request);

        let register_request = request.into_inner();
        let response = RegisterResponse {};

        // we add a new UserInfo if the id is new, we update y1 & y2 if `id` already exists.
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

        let user_registry = &mut *self.user_registry.lock().unwrap();
        user_registry.insert(register_request.user, user_info);

        println!("User registry: {:?}", self.user_registry);

        Ok(Response::new(response))
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        let register_request = request.into_inner();

        let auth_id = register_request.auth_id;
        let s = register_request.s;
        let s = BigUint::from_bytes_be(&s);

        let auth_registry = &mut *self.auth_registry.lock().unwrap();

        let (p, q, g, h) = get_scalar_constants();

        if let Some(info) = auth_registry.get(&auth_id) {
            if verify(
                &info.r1, &info.r2, &info.y1, &info.y2, &g, &h, &info.c, &s, &p,
            ) {
                let response = AuthenticationAnswerResponse {
                    session_id: String::from("you_solved_the_zkp_challenge_congratulations"),
                };

                Ok(Response::new(response))
            } else {
                return Err(Status::new(
                    Code::NotFound,
                    "the challenge was not solved properly",
                ));
            }
        } else {
            return Err(Status::new(Code::NotFound, "auth_id doesn't exist"));
        }
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {
        let register_request = request.into_inner();

        let user = register_request.user;

        let r1 = Point::deserialize(register_request.r1, &self.group);
        let r2 = Point::deserialize(register_request.r2, &self.group);

        let user_registry = &mut *self.user_registry.lock().unwrap();
        let auth_registry = &mut *self.auth_registry.lock().unwrap();

        let auth_id = "aoskasokd".to_string();

        if let Some(user_info) = user_registry.get(&user) {
            let c = get_random_number::<2>();

            auth_registry.insert(
                auth_id.clone(),
                AuthInfo {
                    auth_id: auth_id.clone(),
                    y1: user_info.y1.clone(),
                    y2: user_info.y2.clone(),
                    r1,
                    r2,
                    c: c.clone(),
                },
            );

            let response = AuthenticationChallengeResponse {
                auth_id,
                c: c.to_bytes_be(),
            };

            Ok(Response::new(response))
        } else {
            return Err(Status::new(Code::NotFound, "user doesn't exist"));
        }
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
