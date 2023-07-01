#!/bin/sh

RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --example ocean --target wasm32-unknown-unknown \
	--no-default-features --features webgpu,embed_shaders,depth_prepass

echo "wasm-bindgen"
wasm-bindgen --out-name ocean_webgpu --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/ocean.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o ocean_webgpu_bg.wasm ./ocean_webgpu_bg.wasm
