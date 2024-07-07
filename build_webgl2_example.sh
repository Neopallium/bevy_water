#!/bin/sh
NAME=$1

echo "Build ${NAME} WebGL2"
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --example ${NAME} --target wasm32-unknown-unknown \
	--no-default-features --features webgl2,embed_shaders,spectator

echo "wasm-bindgen"
wasm-bindgen --out-name ${NAME}_webgl2 --out-dir ./ --target web ./target/wasm32-unknown-unknown/release/examples/${NAME}.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz -o ${NAME}_webgl2_bg.wasm ./${NAME}_webgl2_bg.wasm
