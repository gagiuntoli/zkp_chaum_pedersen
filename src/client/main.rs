use num::BigUint;
use num::traits::One;
use std::io::{stdin, stdout, Write};
use std::env;

pub mod zkp_auth {
    include!("../zkp_auth.rs");
}

use zkp_auth::auth_client::AuthClient;
use zkp_auth::{AuthenticationAnswerRequest, AuthenticationChallengeRequest, RegisterRequest};

use chaum_pedersen_zkp::{
    parse_group_from_command_line, get_constants, get_random_number, exponentiates_points,
    solve_zk_challenge_s,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let group = parse_group_from_command_line(args);

    let server_addr = "http://127.0.0.1:50051";

    println!(
        "Running client connecting to {} ZKP: {:?}",
        server_addr, group
    );

    let mut client = AuthClient::connect(server_addr).await?;

    let (p, q, g, h) = get_constants(&group);

    'main_loop: loop {
        let x = get_random_number();
        println!("Your new password is: {:?}", x);

        let (y1, y2) = exponentiates_points(&x, &g, &h, &p).unwrap();

        println!("Enter your name to register");

        let mut stdin_string = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut stdin_string)
            .expect("Did not enter a correct string");
        let user_name = stdin_string.trim().to_string();

        // (y1, y2) = (g^x, h^x) secret x
        println!("sending register request");

        let server_response = client
            .register(RegisterRequest {
                user: user_name.clone(),
                y1: y1.serialize(),
                y2: y2.serialize(),
            })
            .await;

        if let Err(registration_response) = &server_response {
            println!(
                "[CLIENT] Error occurred during registration: {:?}",
                registration_response.message()
            );
            continue 'main_loop;
        }

        // (r1, r2) = (g^k, h^k) random k
        println!("Sending authentication challenge request");

        let k = get_random_number();

        let (r1, r2) = exponentiates_points(&k, &g, &h, &p).unwrap();

        let server_response = client
            .create_authentication_challenge(AuthenticationChallengeRequest {
                user: user_name,
                r1: r1.serialize(),
                r2: r2.serialize(),
            })
            .await;

        if let Err(registration_response) = &server_response {
            println!(
                "[CLIENT] Error occurred during challenge request: {:?}",
                registration_response.message()
            );
            continue 'main_loop;
        }

        println!("Solving challenge, would you like to solve it right?\nIf `no` we add 1 to the solution which is wrong and see what happens [Y/n]");

        let solve_challenge_right;

        'option_loop: loop {
            let mut stdin_string = String::new();
            let _ = stdout().flush();
            stdin()
                .read_line(&mut stdin_string)
                .expect("Did not enter a correct string");

            match stdin_string.trim() {
                "y" | "Y" | "yes" | "Yes" | "" => {
                    solve_challenge_right = true;
                    break 'option_loop;
                }
                "n" | "N" | "no" | "No" => {
                    solve_challenge_right = false;
                    break 'option_loop;
                }
                _ => {
                    println!("Entered option should be yes or no: (y, Y, yes, Yes, n, N, no, No or simply `Enter`)");
                    continue 'option_loop;
                }
            }
        }

        let response = server_response?.into_inner();
        let auth_id = response.auth_id;
        println!("[CLIENT] Auth ID received: {}", auth_id);

        let c = response.c;
        let c = BigUint::from_bytes_be(&c);
        let mut s = solve_zk_challenge_s(&x, &k, &c, &q);

        if !solve_challenge_right {
            s += BigUint::one();
        }

        println!("[CLIENT] Solve and send challenge solution");

        let server_response = client
            .verify_authentication(AuthenticationAnswerRequest {
                auth_id,
                s: s.to_bytes_be(),
            })
            .await;

        match server_response {
            Ok(auth_response) => {
                println!(
                    "[CLIENT] Session ID: {:?}\n",
                    auth_response.into_inner().session_id
                )
            }
            Err(auth_response) => {
                println!(
                    "[CLIENT] Error occurred (server response): {:?}\n",
                    auth_response.message()
                )
            }
        }
    }
}
