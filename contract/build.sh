#!/bin/sh

echo ">> Building contract"

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release  && cp ./target/wasm32-unknown-unknown/release/axelar_auth_gateway.wasm ../out/axelar_auth_gateway.wasm
