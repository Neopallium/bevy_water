#!/bin/sh

cargo build --release --example ocean --target wasm32-unknown-unknown \
	--no-default-features --features webgl2,embed_shaders

echo "wasm-bindgen"
wasm-bindgen --out-name ocean_webgl2 --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/ocean.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o ocean_webgl2_bg.wasm ./ocean_webgl2_bg.wasm
