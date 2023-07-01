#define_import_path bevy_water::water_functions

#import bevy_water::noise::random
#import bevy_water::noise::vnoise

fn noise2(v: vec2<f32>) -> f32 {
  return vnoise2d(v);
}

#import bevy_water::noise::fbm

fn wave(p: vec2<f32>) -> f32 {
  let time = globals.time * .5 + 23.0;

  let time_x = time / 1.0;
  let time_y = time / 0.5;
  let wave_len_x = 5.0;
  let wave_len_y = 2.0;
  let wave_x = cos(p.x / wave_len_x + time_x);
  let wave_y = smoothstep(1.0, 0.0, abs(sin(p.y / wave_len_y + wave_x + time_y)));
  let n = fbm(p) / 2.0 - 1.0;
  return wave_y + n;
}

fn get_wave_height(p: vec2<f32>) -> f32 {
  let time = globals.time / 2.0;
  var d = wave((p + time) * 0.4) * 0.3;
  d = d + wave((p - time) * 0.3) * 0.3;
  d = d + wave((p + time) * 0.5) * 0.2;
  d = d + wave((p - time) * 0.6) * 0.2;
  return material.amplitude * d;
}

fn uv_to_coord(uv: vec2<f32>) -> vec2<f32> {
  // Invert the y UV coord.
  let w_uv = vec2<f32>(uv.x, 1.0 - uv.y);
  return material.coord_offset + (w_uv * material.coord_scale);
}
