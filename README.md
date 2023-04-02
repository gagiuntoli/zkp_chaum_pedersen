# Chaum-Pedersen Zero Knowledge Proof

This program is an implementation of the Chaum-Pedersen ZKP protocol that allows
the registration of a user client in a server without the need of transferring
his password. The mathematics of the algorithm are explained in the bibliography attached: [1], [2] and [3].

# Features

The code currently supports:

-  Integer cyclic group activated by default or with the `--scalar` command line option.
-  Elliptic curve secp256k1 cyclic group activated with the `--elliptic` curve command line option.
-  Support for very large integers by using the `num-bigint` Rust crate.
-  Docker containerization.

# Default parameters

For the integer and elliptic curve cyclic groups we have hardcoded the known parameters of the algorithm.

1. Scalar or integer cyclic groups:

```
p = 10009
q = 5004
g = 3
h = 2892 (g^13 mod p)
```

Note that these numbers are very small. They shouldn't be use in production.

2. An elliptic curve cyclic group based on the secp256k1 curve

```
p = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f
q = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
g = (
    x:0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798,
    y:0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8
    )
h = 13 * g
```

The `src/secp256k1.rs` library is included in the code. This is a copy from one
of my projects ([4]) based on the Programming Bitcoin book from Jimmy Song. I
find it easier to use than other secp256k1 libraries out there.

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

If you have all the dependencies installed, then run from the main directory the
following command:

```bash
$ cargo build --release
```

Note that this should generate on the `./src` folder a file called
`zpk_auth.rs`. This file is the interface generated with `tonic` from the
`./proto/zkp_auth.proto` Protobuf file. This file specifies the communication
protocol between server and client.

Then, test that everything works fine by executing:

```bash
$ cargo test
```

# Run locally

I suggest opening 2 separate terminals, one for running the server and the other
for the client since both produce useful outputs for debugging and understanding
what happens.

Execute the server:

```bash
$ cargo run --bin server -- [--scalar(default)|--elliptic]
```

The server listens all the time for any message of any client and communicates
using the gRPC protocol.

Execute the client:

```bash
$ cargo run --bin client -- [--scalar(default)|--elliptic]
```

Note that both, the server and the client, should use the same cyclic group,
i.e, both using the integer (`--scalar`) fields, or both using the elliptic
curves field (`--elliptic`).

# Run with Docker

You will need to have `docker` and `docker-compose`. Open two terminals and in
one build the docker image and run the server:

```bash
$ docker-compose run --rm zpkserver
...
[+] Building 193.1s (11/11) FINISHED
...

root@<...>:/zpk-app# cargo run --bin server --release -- --elliptic
Bookstore server listening on 127.0.0.1:50051 ZKP: EllipticCurve
```

On the other terminal, connect to the running docker container and run the
client:

```bash
$ docker exec -it zpkserver /bin/bash
root@<...>:/zpk-app# cargo run --bin client --release -- --elliptic
Running client connecting to http://127.0.0.1:50051 ZKP: EllipticCurve
Your new password is: 19263258492685931671967943988117500779858528929052995681138386950379191891393
Enter your name to register
```

The client connects to the server and then runs a for-loop that:

1. Ask for a username.
2. Ask if you want to solve the challenge correctly.
3. Logs and shows if the login was successful or not.

# Sample Outputs

From the client side we have the option to correctly solve the ZK challenge or
not. Solving it wrong means to compute the correct value of `s` but then the sum
`1` to it. We can experiment with server and client passing correct or
incorrect ZK solutions and use the elliptic curve or integer cyclic groups.

Output from the client side:

```bash
root@68204f9d2567:/zpk-app# cargo run --bin client --release
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/client`
Running client connecting to http://127.0.0.1:50051 ZKP: EllipticCurve
Your new password is: 56996818256788956454763166456654500327071775372141719170469740704131163907775
Enter your name to register
Guido
sending register request
Sending authentication challenge request
Solving challenge, would you like to solve it right?
If `no` we add 1 to the solution which is wrong and see what happens [Y/n]
Yes
[CLIENT] Auth ID received: Hb53NTWGOi
[CLIENT] Solve and send challenge solution
[CLIENT] Session ID: "cFcaI5Gz1D"

Your new password is: 27597106728298662838402878713294363712617563516513680626432016059032263303014
Enter your name to register
Jorge
sending register request
Sending authentication challenge request
Solving challenge, would you like to solve it right?
If `no` we add 1 to the solution which is wrong and see what happens [Y/n]
No
[CLIENT] Auth ID received: Nk0a88RJg9
[CLIENT] Solve and send challenge solution
[CLIENT] Error occurred (server response): "(Server): challenge not solved properly"
```

Output from the server side (for the same previous execution):

```bash
root@68204f9d2567:/zpk-app# cargo run --bin server --release
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/server`
Bookstore server listening on 127.0.0.1:50051 ZKP: EllipticCurve
[SERVER] Registering user: Guido
[SERVER] Successful login auth_id: Hb53NTWGOi

[SERVER] Registering user: Jorge
[SERVER] Error: challenge not solved properly auth_id: Nk0a88RJg9

```

# References

1. [Cryptography: An Introduction](https://www.cs.umd.edu/~waa/414-F11/IntroToCrypto.pdf)
2. [Questions about parameter selection](https://crypto.stackexchange.com/questions/99262/chaum-pedersen-protocol)
3. [Questions about using elliptic curves](https://crypto.stackexchange.com/questions/105889/chaum-pedersen-protocol-adapted-to-elliptic-curves?noredirect=1#comment226693_105889)
4. [Bitcoin Rust](https://github.com/gagiuntoli/bitcoin_rust)