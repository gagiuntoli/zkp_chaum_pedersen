use num_bigint::BigUint;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use tonic::{transport::Server, Code, Request, Response, Status};

use chaum_pedersen_zkp::{
    get_constants, get_random_number, get_random_string, parse_group_from_command_line, verify,
    Group, Point,
};

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
}

#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub auth_id: String,
    pub y1: Point,
    pub y2: Point,
    pub r1: Point,
    pub r2: Point,
    pub c: BigUint,
    pub session_id: String,
}

#[tonic::async_trait]
impl Auth for AuthImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let register_request = request.into_inner();
        let response = RegisterResponse {};

        let user_name = register_request.user.clone();
        println!("[SERVER] Registering user: {}", user_name);

        // we add a new UserInfo, replace old y1 & y2 if the user was already register.
        let user_info = UserInfo {
            user: user_name,
            y1: Point::deserialize(register_request.y1, &self.group),
            y2: Point::deserialize(register_request.y2, &self.group),
        };

        let user_registry = &mut *self.user_registry.lock().unwrap();
        user_registry.insert(register_request.user, user_info);

        Ok(Response::new(response))
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

        let auth_id = get_random_string(10);

        if let Some(user_info) = user_registry.get(&user) {
            let c = get_random_number();

            auth_registry.insert(
                auth_id.clone(),
                AuthInfo {
                    auth_id: auth_id.clone(),
                    y1: user_info.y1.clone(),
                    y2: user_info.y2.clone(),
                    r1,
                    r2,
                    c: c.clone(),
                    session_id: String::new(),
                },
            );

            let response = AuthenticationChallengeResponse {
                auth_id,
                c: c.to_bytes_be(),
            };

            Ok(Response::new(response))
        } else {
            println!("[SERVER] User {} not found\n", user);
            return Err(Status::new(Code::NotFound, "(Server) User not found"));
        }
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

        let (p, _, g, h) = get_constants(&self.group);

        if let Some(info) = auth_registry.get_mut(&auth_id) {
            match verify(
                &info.r1, &info.r2, &info.y1, &info.y2, &g, &h, &info.c, &s, &p,
            ) {
                Ok(verification) => {
                    if verification {
                        let session_id = get_random_string(10);
                        info.session_id = session_id.clone();

                        let response = AuthenticationAnswerResponse {
                            session_id: session_id.clone(),
                        };

                        println!("[SERVER] Successful login auth_id: {}\n", auth_id);
                        Ok(Response::new(response))
                    } else {
                        println!(
                            "[SERVER] Error: challenge not solved properly auth_id: {}\n",
                            auth_id
                        );

                        return Err(Status::new(
                            Code::NotFound,
                            "(Server): challenge not solved properly",
                        ));
                    }
                }
                Err(error) => {
                    println!(
                        "[SERVER] algorithm error during verification: {:?}\n",
                        error
                    );

                    return Err(Status::new(
                        Code::NotFound,
                        "(Server): algorithm error during verification",
                    ));
                }
            }
        } else {
            println!("[SERVER] auth_id {} not found", auth_id);
            return Err(Status::new(Code::NotFound, "auth_id doesn't exist"));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let mut auth = AuthImpl::default();

    let args: Vec<String> = env::args().collect();
    auth.group = parse_group_from_command_line(args);

    println!(
        "Bookstore server listening on {} ZKP: {:?}",
        addr, auth.group
    );

    Server::builder()
        .add_service(AuthServer::new(auth))
        .serve(addr)
        .await?;

    Ok(())
}
