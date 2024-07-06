#!/bin/sh

echo "Build ocean WebGPU"
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --example ocean --target wasm32-unknown-unknown \
	--no-default-features --features webgpu,embed_shaders,depth_prepass,atmosphere,spectator

echo "wasm-bindgen"
wasm-bindgen --out-name ocean_webgpu --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/ocean.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o ocean_webgpu_bg.wasm ./ocean_webgpu_bg.wasm

echo "Build ocean WebGL2"
cargo build --release --example ocean --target wasm32-unknown-unknown \
	--no-default-features --features webgl2,embed_shaders,spectator

echo "wasm-bindgen"
wasm-bindgen --out-name ocean_webgl2 --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/ocean.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o ocean_webgl2_bg.wasm ./ocean_webgl2_bg.wasm
