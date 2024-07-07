#!/bin/sh
NAME=$1

echo "Build ${NAME} WebGPU"
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --example ${NAME} --target wasm32-unknown-unknown \
	--no-default-features --features webgpu,embed_shaders,depth_prepass,atmosphere,spectator

echo "wasm-bindgen"
wasm-bindgen --out-name ${NAME}_webgpu --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/${NAME}.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o ${NAME}_webgpu_bg.wasm ./${NAME}_webgpu_bg.wasm
