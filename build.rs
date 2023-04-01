/// The code for implementing the Rust types by reading the protocol description
/// was taken from:
/// https://betterprogramming.pub/building-a-grpc-server-with-rust-be2c52f0860e

fn main() {
    let proto_file = "./proto/zkp_auth.proto";

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src")
        .compile(&[proto_file], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    println!("cargo:rerun-if-changed={}", proto_file);
}
