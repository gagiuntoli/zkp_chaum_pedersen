# Chaum-Pedersen Zero Knowledge Proof

This program is an implementation of the Chaum-Pedersen ZKP protocol that allows
the registration of a user client in a server without the need of transferring
his password. The mathematics of the algorithm are explained in the bibliography attached: [1], [2] and [3].

We have implemented the algorithm using 2 cyclic groups:

1. With simple integers numbers and very small and hardcoded values which
shouldn't be use for production:

```
p = 10009
q = 5004
g = 3
h = 2892
```

Even though the number are small this program supports very large precision
since the fundamental structures use `num-bigint`.

2. An elliptic curve cyclic group based on the secp256k1 curve. An additional
library, `src/secp256k1.rs` is included in the code and is a copy and paste from
one of my projects [4] based on the Programming Bitcoin book from Jimmy Song.

```
p = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f
q = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
g = (
    x:0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798,
    y:0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8
    )
h = 13 * g
```

Note that the constant `13` for computing `h` was arbitrary selected. From what
[1] states `g` and `h` should be of prime order `q`:

```
g ^ q mod p = 1
h ^ q mod p = 1
```

# Dependencies

- `rustc` (compiler) and `rustup` (package manager)
- `cmake` for the gRCP library
- `docker` and `docker-compose` if you are going to run in a docker container

If you have all the dependencies installed run in the main directory:

```bash
cargo build --release
```

Note that this should generate on the `./src` folder a file called
`zpk_auth.rs`. This file is the interface generated with `tonic` from the
`./proto/zkp_auth.proto` file.

Test that everything works fine:

```bash
cargo test
```

# Run the code locally

I suggest opening 2 terminals, one for running the server and the other for the
client since both produce some output which can be useful for debugging.

Execute the server:

```bash
cargo run --bin server -- [--scalar(default)|--elliptic]
```

Execute the client:

```bash
cargo run --bin client -- [--scalar(default)|--elliptic]
```

Note that both, the server and the client, should run the same algorithm, i.e,
both using scalar fields, or both using the elliptic curves field.

# Run the code from the Docker instance

You will need `docker` and `docker-compose`. In one terminal inside the main folder execute:

Build the docker image and run the server in one terminal:

```bash
docker-compose run --rm zpkserver
root@<...>:/zpk-app# cargo run --bin server
```

On another terminal search for the docker image and connect with an interactive terminal:

```bash
docker compose images
docker exec -it chaum-pedersen-zkp_zpkserver_run_<...> /bin/bash
root@<...>:/zpk-app# cargo run --bin client
```

# Sample Outputs

From the client side we have the option to correctly solve the ZK challenge or
not. Solving it bad means to compute the right value of `s` and the sum `1` to
it. We can experiment with both: server and client passing correct or wrong ZK
solutions and use the elliptic curve or integer cyclic groups.

Output from the client side:

```bash
root@68204f9d2567:/zpk-app# cargo run --bin client
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/client`
Running client connecting to http://127.0.0.1:50051 ZKP: Scalar
Your new password is: 60522
Enter your name to register
Guido
sending register request
Sending authentication challenge request
Solving challenge, would you like to solve it right?
If `no` we add 1 to the solution which is wrong and see what happens [Y/n]
Yes
[CLIENT] Auth ID received: EGPsXFW5zg
[CLIENT] Solve and send challenge solution
[CLIENT] Session ID: "3jpXqnufXs"

Your new password is: 514
Enter your name to register
Jorge
sending register request
Sending authentication challenge request
Solving challenge, would you like to solve it right?
If `no` we add 1 to the solution which is wrong and see what happens [Y/n]
No
[CLIENT] Auth ID received: ZEICkzo3jk
[CLIENT] Solve and send challenge solution
[CLIENT] Error occurred (server response): "(Server): the challenge not solved properly"
```

Output from the server side (for the same previous execution):

```bash
root@68204f9d2567:/zpk-app# cargo run --bin server
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/server`
Bookstore server listening on 127.0.0.1:50051 ZKP: Scalar
[SERVER] Registering user: Guido
[SERVER] Successful login auth_id: aCMZiazfGi

[SERVER] Registering user: Jorge
[SERVER] Error challenge not solved properly auth_id: K6UOkCHdv3
```

# References

1. [Cryptography: An Introduction](https://www.cs.umd.edu/~waa/414-F11/IntroToCrypto.pdf)
2. [Questions about parameter selection](https://crypto.stackexchange.com/questions/99262/chaum-pedersen-protocol)
3. [Questions about using elliptic curves](https://crypto.stackexchange.com/questions/105889/chaum-pedersen-protocol-adapted-to-elliptic-curves?noredirect=1#comment226693_105889)
4. [Bitcoin Rust](https://github.com/gagiuntoli/bitcoin_rust)