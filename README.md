# Substrate Proof of Existence Module

This is a simple Substrate runtime module to store online distributed [proof of existence](https://www.proofofexistence.com/) for any file.

## Purpose

This module enables users submit a proof of existence for a file. This proof of existence may also be used as a soft measure of ownership.

Files are not directly uploaded to the blockchain. Instead, a [file digest](https://en.wikipedia.org/wiki/File_verification) is generated, and the resulting digest is stored on chain with the time of upload and the user who made the claim.

Anyone who has the source file can also generate the same digest and check the proof of existence on-chain.

## Tutorial

This repository is a 3-part tutorial to equip you with the basic skills to build your own custom **Substrate runtime modules**. You can follow it along by checking out the following branches, that act as starting point for each *level* of the tutorial.

- The **level-0** branch starts from a blank [substrate node template](https://github.com/substrate-developer-hub/substrate-node-template). Follow the instructions on the [level-0](level-0.md) page.
- The **level-1** branch adds the time dimension to the basic proof-of-existence system built at **level-0**, and demonstrate how to leverage **built-in Substrate runtime module** (e.g. *Timestamp* module). See the [level-1](level-1.md) tutorial page.
- The **level-2** branch expands on the code from **level-1** to add the economic dimension with account balances and fees, and wraps up with a fully-functional proof-of-existence Substrate runtime module. Follow the step at [level-2](level-2.md) tutorial page.
- The **master** branch contains the fully implemented **proof-of-existence Substrate runtime module**.

___

Below, the instructions to setup your dev environment, and at any time build & run your Substrate node, that includes your shiny Proof-of-existence runtime module.

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build
```

## Run

### Single node development chain

You can start a development chain with:

```bash
cargo run -- --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-node local testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.
