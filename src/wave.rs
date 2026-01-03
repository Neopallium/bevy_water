use bevy::prelude::*;

// wgsl compatible `fract`.
pub(crate) fn fract(x: f32) -> f32 {
  x - x.floor()
}

pub(crate) fn fract_vec2(v: Vec2) -> Vec2 {
  Vec2 {
    x: fract(v.x),
    y: fract(v.y),
  }
}

pub(crate) fn mix(x: f32, y: f32, a: f32) -> f32 {
  x * (1.0 - a) + y * a
}

pub(crate) fn mix2d(x: Vec2, y: Vec2, a: f32) -> Vec2 {
  Vec2 {
    x: mix(x.x, y.x, a),
    y: mix(x.y, y.y, a),
  }
}

pub(crate) fn random2d(v: Vec2) -> f32 {
  // Note: the large values here seem to cause some precision differences between the shader
  // and this Rust code.
  return fract(v.dot(Vec2::new(12.9898, 78.233)).sin() * 43758.5453123);
}

// Sometimes needed for noise functions that sample multiple corners.
pub(crate) fn random2di(v: Vec2) -> f32 {
  return random2d(v.floor());
}

pub(crate) fn cubic_hermite_curve_2d(p: Vec2) -> Vec2 {
  return Vec2 {
    x: smoothstep(0.0, 1.0, p.x),
    y: smoothstep(0.0, 1.0, p.y),
  };
}

pub(crate) fn vnoise2d(v: Vec2) -> f32 {
  let i = v.floor();
  let f = fract_vec2(v);

  // corners.
  let a = random2di(i);
  let b = random2di(i + Vec2::new(1.0, 0.0));
  let c = random2di(i + Vec2::new(0.0, 1.0));
  let d = random2di(i + Vec2::new(1.0, 1.0));

  // Smooth
  let u = cubic_hermite_curve_2d(f);

  // Mix
  return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

pub(crate) fn noise2(v: Vec2) -> f32 {
  return vnoise2d(v);
}

const M2: Mat2 = Mat2::from_cols(Vec2::new(0.8, 0.6), Vec2::new(-0.6, 0.8));
pub(crate) fn fbm(mut p: Vec2) -> f32 {
  let mut f = 0.5000 * noise2(p);
  p = M2 * p * 2.02;
  f = f + 0.2500 * noise2(p);
  p = M2 * p * 2.03;
  f = f + 0.1250 * noise2(p);
  p = M2 * p * 2.01;
  f = f + 0.0625 * noise2(p);
  return f / 0.9375;
}

pub(crate) fn fbm_half(mut p: Vec2) -> f32 {
  let mut f = 0.5000 * noise2(p);
  p = M2 * p * 2.02;
  f = f + 0.2500 * noise2(p);
  return f / 0.9375;
}

pub(crate) fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
  let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
  t * t * (3.0 - 2.0 * t)
}

pub(crate) fn wave(p: Vec2, g_time: f32, quality: u32) -> f32 {
  // Internal time creates fluid oscillation within the pattern
  let time = g_time * 0.5 + 23.0;
  let time_x = time / 1.0;
  let time_y = time / 0.5;
  // Pattern oriented so primary motion is along X (travel direction after rotation)
  let wave_len_x = 2.0;
  let wave_len_y = 5.0;
  let wave_y = (p.y / wave_len_y + time_y).cos();
  let wave_x = smoothstep(1.0, 0.0, (p.x / wave_len_x + wave_y + time_x).sin().abs());
  let n = if quality < 3 {
    fbm_half(p) / 2.0 - 1.0
  } else {
    fbm(p) / 2.0 - 1.0
  };
  return wave_x + n;
}

/// Sample wave pattern for a single direction.
pub(crate) fn sample_directional_wave(
  p: Vec2,
  time: f32,
  g_time: f32,
  wave_direction: Vec2,
  quality: u32,
) -> f32 {
  let dir = wave_direction.normalize_or_zero();
  // Rotate coordinates so wave ridges are perpendicular to travel direction
  // Negate x-component so waves travel along dir, not against it
  let rotated_p = Vec2::new(-(p.x * dir.x + p.y * dir.y), p.y * dir.x - p.x * dir.y);

  // Multiple layers with counter-directional scrolling for volume
  let time_vec = Vec2::splat(time);
  let mut d = wave((rotated_p - time_vec) * 0.3, g_time, quality) * 0.3;
  if quality >= 2 {
    d = d + wave((rotated_p + time_vec) * 0.4, g_time, quality) * 0.3;
  }
  if quality >= 3 {
    d = d + wave((rotated_p + time_vec) * 0.5, g_time, quality) * 0.2;
  }
  if quality >= 4 {
    d = d + wave((rotated_p - time_vec) * 0.6, g_time, quality) * 0.2;
  }
  d
}

pub(crate) fn get_wave_height_2d(g_time: f32, p: Vec2, wave_direction: Vec2, quality: u32) -> f32 {
  let time = g_time / 2.0;
  sample_directional_wave(p, time, g_time, wave_direction, quality)
}

/// Sample wave with dual-direction crossfade blending (matches High/Ultra shader quality).
pub fn sample_directional_wave_blended(
  g_time: f32,
  p: Vec2,
  dir_a: Vec2,
  dir_b: Vec2,
  blend: f32,
  quality: u32,
) -> f32 {
  let time = g_time / 2.0;
  let wave_a = sample_directional_wave(p, time, g_time, dir_a, quality);
  let wave_b = sample_directional_wave(p, time, g_time, dir_b, quality);

  // Asymmetric smoothstep - matches shader behavior
  let blend_smooth = smoothstep(0.0, 0.85, blend);

  mix(wave_a, wave_b, blend_smooth)
}

/// Calculate wave height at global position `pos`.
///
/// `time` - Bevy `time.elapsed_seconds_wrapped()`.
/// `base_height` - The base height from `WaterSettings`.
/// `amplitude` - The amplitude of the wave.
/// `wave_direction` - The wave movement direction.
/// `pos` - Global world position.  Use your entity's `GlobalTransform` to get the world position.
pub fn get_wave_height(
  time: f32,
  base_height: f32,
  amplitude: f32,
  wave_direction: Vec2,
  pos: Vec3,
) -> f32 {
  get_wave_height_2d(time, Vec2::new(pos.x, pos.z), wave_direction, u32::MAX) * amplitude
    + base_height
}

/// Calculate wave height at global position `pos` and return a point
/// on the surface of the water.
///
/// `time` - Bevy `time.elapsed_seconds_wrapped()`.
/// `base_height` - The base height from `WaterSettings`.
/// `amplitude` - The amplitude of the wave.
/// `wave_direction` - The wave movement direction.
/// `pos` - Global world position.  Use your entity's `GlobalTransform` to get the world position.
pub fn get_wave_point(
  time: f32,
  base_height: f32,
  amplitude: f32,
  wave_direction: Vec2,
  mut pos: Vec3,
) -> Vec3 {
  pos.y = get_wave_height(time, base_height, amplitude, wave_direction, pos);
  pos
}
