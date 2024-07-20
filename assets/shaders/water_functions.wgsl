#define_import_path bevy_water::water_functions

#ifdef PREPASS_PIPELINE
#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;
#else
#import bevy_pbr::mesh_view_bindings::globals
#endif

#import bevy_water::water_bindings::material
#import bevy_water::noise::fbm::fbm

fn wave(p: vec2<f32>) -> f32 {
  let time = globals.time * .5 + 23.0;

  let time_x = time / 1.0;
  let time_y = time / 0.5;
  let wave_len_x = 5.0;
  let wave_len_y = 2.0;
  let wave_x = cos(p.x / wave_len_x + time_x);
  let wave_y = smoothstep(1.0, 0.0, abs(sin(p.y / wave_len_y + wave_x + time_y)));
  let n = fbm(p, material.quality) / 2.0 - 1.0;
  return wave_y + n;
}

fn get_wave_height(p: vec2<f32>) -> f32 {
  let time = globals.time / 2.0;
  var d = wave((p - time) * 0.3) * 0.3;
  if material.quality > 1 {
    d = d + wave((p + time) * 0.4) * 0.3;
  }
  if material.quality > 2 {
    d = d + wave((p + time) * 0.5) * 0.2;
  }
  if material.quality > 3 {
    d = d + wave((p + time) * 0.6) * 0.2;
  }
  return material.amplitude * d;
}

fn uv_to_coord(uv: vec2<f32>) -> vec2<f32> {
  return material.coord_offset + (uv * material.coord_scale);
}
