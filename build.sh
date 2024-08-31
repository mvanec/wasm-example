#!/bin/zsh

cargo build --target=wasm32-unknown-unknown --release && \
rm -rfv ./pkg && wasm-bindgen --target nodejs --out-dir ./pkg ./target/wasm32-unknown-unknown/release/wasm_example.wasm && \
node server.js