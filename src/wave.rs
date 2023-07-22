use bevy::prelude::*;

// wgsl compatible `fract`.
fn fract(x: f32) -> f32 {
  x - x.floor()
}
fn fract_vec2(v: Vec2) -> Vec2 {
  Vec2 {
    x: fract(v.x),
    y: fract(v.y),
  }
}

fn mix(x: f32, y: f32, a: f32) -> f32 {
  x * (1.0 - a) + y * a
}

fn random2d(v: Vec2) -> f32 {
  // Note: the large values here seem to cause some precision differences between the shader
  // and this Rust code.
	return fract(v.dot(Vec2::new(12.9898, 78.233)).sin() * 43758.5453123);
}

// Sometimes needed for noise functions that sample multiple corners.
fn random2di(v: Vec2) -> f32 {
	return random2d(v.floor());
}

fn cubic_hermite_curve_2d(p: Vec2) -> Vec2 {
  return Vec2 {
    x: smoothstep(0.0, 1.0, p.x),
    y: smoothstep(0.0, 1.0, p.y),
  }
}

fn vnoise2d(v: Vec2) -> f32 {
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
	return mix(a, b, u.x) +
		(c - a) * u.y * (1.0 - u.x) +
		(d - b) * u.x * u.y;
}

fn noise2(v: Vec2) -> f32 {
  return vnoise2d(v);
}

const M2: Mat2 = Mat2::from_cols(Vec2::new(0.8, 0.6), Vec2::new(-0.6, 0.8));
fn fbm(mut p: Vec2) -> f32 {
  let mut f = 0.;
  f = f + 0.5000 * noise2(p); p = M2 * p * 2.02;
  f = f + 0.2500 * noise2(p); p = M2 * p * 2.03;
  f = f + 0.1250 * noise2(p); p = M2 * p * 2.01;
  f = f + 0.0625 * noise2(p);
  return f / 0.9375;
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
  let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
  t * t * (3.0 - 2.0 * t)
}

fn wave(g_time: f32, p: Vec2) -> f32 {
  let time = g_time * 0.5 + 23.0;

  let time_x = time / 1.0;
  let time_y = time / 0.5;
  let wave_len_x = 5.0;
  let wave_len_y = 2.0;
  let wave_x = (p.x / wave_len_x + time_x).cos();
  let wave_y = smoothstep(1.0, 0.0, (p.y / wave_len_y + wave_x + time_y).sin().abs());
  let n = fbm(p) / 2.0 - 1.0;
  return wave_y + n;
}

pub(crate) fn get_wave_height_2d(g_time: f32, p: Vec2) -> f32 {
  let time = g_time / 2.0;
  let mut d = wave(g_time, (p + time) * 0.4) * 0.3;
  d = d + wave(g_time, (p - time) * 0.3) * 0.3;
  d = d + wave(g_time, (p + time) * 0.5) * 0.2;
  d = d + wave(g_time, (p - time) * 0.6) * 0.2;
  return d;
}

/// Calculate wave height at global position `pos`.
///
/// `time` - Bevy `time.elapsed_seconds_wrapped()`.
/// `base_height` - The base height from `WaterSettings`.
/// `amplitude` - The amplitude of the wave.
/// `pos` - Global world position.  Use your entity's `GlobalTransform` to get the world position.
pub fn get_wave_height(time: f32, base_height: f32, amplitude: f32, pos: Vec3) -> f32 {
  get_wave_height_2d(time, Vec2::new(pos.x, pos.z)) * amplitude + base_height
}

/// Calculate wave height at global position `pos` and return a point
/// on the surface of the water.
///
/// `time` - Bevy `time.elapsed_seconds_wrapped()`.
/// `base_height` - The base height from `WaterSettings`.
/// `amplitude` - The amplitude of the wave.
/// `pos` - Global world position.  Use your entity's `GlobalTransform` to get the world position.
pub fn get_wave_point(time: f32, base_height: f32, amplitude: f32, mut pos: Vec3) -> Vec3 {
  pos.y = get_wave_height(time, base_height, amplitude, pos);
  pos
}
