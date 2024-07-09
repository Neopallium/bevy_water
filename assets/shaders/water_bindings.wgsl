#define_import_path bevy_water::water_bindings

struct WaterMaterial {
  deep_color: vec4<f32>,
  shallow_color: vec4<f32>,
  edge_color: vec4<f32>,
  coord_offset: vec2<f32>,
  coord_scale: vec2<f32>,
  amplitude: f32,
  clarity: f32,
  edge_scale: f32,
  quality: u32
};

@group(2) @binding(100)
var<uniform> material: WaterMaterial;
