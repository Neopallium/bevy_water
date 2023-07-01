#define_import_path bevy_water::water_bindings

struct WaterMaterial {
  // StandardMaterial fields.
  base_color: vec4<f32>,
  emissive: vec4<f32>,
  perceptual_roughness: f32,
  metallic: f32,
  reflectance: f32,
  // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
  flags: u32,
  alpha_cutoff: f32,
  // WaterMaterial fields.
  amplitude: f32,
  clarity: f32,
  edge_scale: f32,
  deep_color: vec4<f32>,
  shallow_color: vec4<f32>,
  edge_color: vec4<f32>,
  coord_offset: vec2<f32>,
  coord_scale: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> material: WaterMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;
@group(1) @binding(3)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_sampler: sampler;
@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;
@group(1) @binding(7)
var occlusion_texture: texture_2d<f32>;
@group(1) @binding(8)
var occlusion_sampler: sampler;
@group(1) @binding(9)
var normal_map_texture: texture_2d<f32>;
@group(1) @binding(10)
var normal_map_sampler: sampler;
