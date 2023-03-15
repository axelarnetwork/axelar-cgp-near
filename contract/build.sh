#!/bin/sh

echo ">> Building contract"

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release  && rm -rf ../dist && mkdir ../dist && cp ./target/wasm32-unknown-unknown/release/axelar_cgp_near.wasm ../dist/
