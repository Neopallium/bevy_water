#define_import_path bevy_water::noise::random

fn random(v: f32) -> f32 {
	return fract(sin(dot(vec2<f32>(v), vec2<f32>(12.9898,78.233))) * 43758.5453123);
}

fn random2d(v: vec2<f32>) -> f32 {
	return fract(sin(dot(v, vec2<f32>(12.9898,78.233))) * 43758.5453123);
}

// Sometimes needed for noise functions that sample multiple corners.
fn random2di(v: vec2<f32>) -> f32 {
	return random2d(floor(v));
}

fn cubic_hermite_curve(x: f32) -> f32 {
	//y = x*x*(3.0-2.0*x);
  return smoothstep(0., 1., x);
}

fn cubic_hermite_curve_2d(x: vec2<f32>) -> vec2<f32> {
	//y = x*x*(3.0-2.0*x);
  return smoothstep(vec2<f32>(0.), vec2<f32>(1.), x);
}
