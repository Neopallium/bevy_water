# bevy_water

Dynamic water material (with waves) for [Bevy](https://bevyengine.org/).

# Example

A fleet of pirate ships moving with the waves.

```
cargo run --release --example pirates
```
![](showcase.webp)

[Dutch ship model from polyhaven (CC0)](https://polyhaven.com/a/dutch_ship_medium)

## WASM examples

[See the WebGPU and WebGL versions online here](https://neopallium.github.io/bevy_water/pirates.html)

### Setup

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

### Build & Run

Following is an example for `pirates`. For other examples, change the `pirates` in the
following commands.

WebGPU:
```sh
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --example pirates \
	--target wasm32-unknown-unknown \
	--no-default-features --features webgpu,embed_shaders,depth_prepass

echo "wasm-bindgen"
wasm-bindgen --out-name pirates_webgpu \
  --out-dir examples/wasm/target \
  --target web target/wasm32-unknown-unknown/release/examples/pirates.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz \
	-o ./examples/wasm/target/pirates_webgpu_bg.wasm ./examples/wasm/target/pirates_webgpu_bg.wasm
```

WebGL2:
```sh
cargo build --release --example pirates --target wasm32-unknown-unknown \
	--no-default-features --features webgl2,embed_shaders

echo "wasm-bindgen"
wasm-bindgen --out-name pirates_webgl2 \
  --out-dir examples/wasm/target \
  --target web target/wasm32-unknown-unknown/release/examples/pirates.wasm

echo "Optimize wasm"
wasm-opt --strip-debug --vacuum -Oz \
	-o ./examples/wasm/target/pirates_webgl2_bg.wasm ./examples/wasm/target/pirates_webgl2_bg.wasm
```

Then serve `examples/wasm` directory to browser. i.e.

```sh
# cargo install basic-http-server
basic-http-server examples/wasm
```

# Features

- Moving 3d waves (vertex height offset).
- Get the wave height using `get_wave_point` to dynamically move objects based on the water height.
- Tileable - allows for adding/removing tiles of water for endless ocean.
- Normals calculated based on wave height for lighting.
- Imports `bevy_pbr::*` shader for lighting/shadow support.

# Ideas/Improvements

- [ ] Improve water color/texture.
- [ ] Heightmap support to adjust waves based on water depth.
- [ ] Mask texture to remove water from areas that shouldn't have water.
- [ ] Volumetic water below the surface.
- [ ] Dynamic depth buffer for objects partially below the surface (boats, peers).  Render pass?

# Versions

- Bevy 0.13.1: `bevy_water = "0.13"`
- Bevy 0.12: `bevy_water = "0.12"`
- Bevy 0.11: `bevy_water = "0.11"`
- Bevy 0.10: `bevy_water = "0.10"`
- Bevy 0.9: `bevy_water = "0.9"`
- Bevy 0.8: `bevy_water = "0.8"`
