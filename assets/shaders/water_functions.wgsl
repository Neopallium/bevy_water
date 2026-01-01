#define_import_path bevy_water::water_functions

#ifdef PREPASS_PIPELINE
#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;
#else
#import bevy_pbr::mesh_view_bindings::globals
#endif

#import bevy_water::water_bindings::material
#import bevy_water::noise;

fn wave(p: vec2<f32>) -> f32 {
  // Internal time creates fluid oscillation within the pattern
  let time = globals.time * 0.5 + 23.0;
  let time_x = time / 1.0;
  let time_y = time / 0.5;
  // Pattern oriented so primary motion is along X (travel direction after rotation)
  let wave_len_x = 2.0;
  let wave_len_y = 5.0;
  let wave_y = cos(p.y / wave_len_y + time_y);
  let wave_x = smoothstep(1.0, 0.0, abs(sin(p.x / wave_len_x + wave_y + time_x)));
#if QUALITY < 3
  let n = noise::fbm::fbm_half(p) / 2.0 - 1.0;
#else
  let n = noise::fbm::fbm(p) / 2.0 - 1.0;
#endif
  return wave_x + n;
}

// Sample wave pattern for a single direction (dir must be pre-normalized on CPU)
fn sample_directional_wave(p: vec2<f32>, time: f32, dir: vec2<f32>) -> f32 {
  // Rotate coordinates so wave ridges are perpendicular to travel direction
  // Negate x-component so waves travel along dir, not against it
  let rotated_p = vec2<f32>(
    -(p.x * dir.x + p.y * dir.y),
    p.y * dir.x - p.x * dir.y
  );

  // Multiple layers with counter-directional scrolling for volume
  var result = wave((rotated_p - time) * 0.3) * 0.3;
#if QUALITY >= 2
  result = result + wave((rotated_p + time) * 0.4) * 0.3;
#endif
#if QUALITY >= 3
  result = result + wave((rotated_p + time) * 0.5) * 0.2;
#endif
#if QUALITY >= 4
  result = result + wave((rotated_p - time) * 0.6) * 0.2;
#endif

  return result;
}

fn get_wave_height(p: vec2<f32>) -> f32 {
  let time = globals.time / 2.0;

  // Sample both wave directions independently
  let wave_a = sample_directional_wave(p, time, material.wave_dir_a);
  let wave_b = sample_directional_wave(p, time, material.wave_dir_b);

  // Asymmetric smoothstep - old waves fade out faster than new waves fade in
  // This mimics how energy dissipates faster than it builds, looking more natural
  let blend = smoothstep(0.0, 0.85, material.wave_blend);

  // Blend wave patterns - NOT rotating, just fading between independent patterns
  let d = mix(wave_a, wave_b, blend);

  return material.amplitude * d;
}

fn uv_to_coord(uv: vec2<f32>) -> vec2<f32> {
  return material.coord_offset + (uv * material.coord_scale);
}
