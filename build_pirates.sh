#!/bin/sh

echo "Build Pirates WebGPU"
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --example pirates --target wasm32-unknown-unknown \
	--no-default-features --features webgpu,embed_shaders,depth_prepass,atmosphere

echo "wasm-bindgen"
wasm-bindgen --out-name pirates_webgpu --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/pirates.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o pirates_webgpu_bg.wasm ./pirates_webgpu_bg.wasm

echo "Build Pirates WebGL2"
cargo build --release --example pirates --target wasm32-unknown-unknown \
	--no-default-features --features webgl2,embed_shaders

echo "wasm-bindgen"
wasm-bindgen --out-name pirates_webgl2 --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/pirates.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o pirates_webgl2_bg.wasm ./pirates_webgl2_bg.wasm
