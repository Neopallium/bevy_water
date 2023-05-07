#define_import_path bevy_water::noise::vnoise

fn vnoise2d(v: vec2<f32>) -> f32 {
	let i = floor(v);
	let f = fract(v);

	// corners.
	let a = random2di(i);
	let b = random2di(i + vec2<f32>(1.0, 0.0));
	let c = random2di(i + vec2<f32>(0.0, 1.0));
	let d = random2di(i + vec2<f32>(1.0, 1.0));

	// Smooth
  let u = cubic_hermite_curve_2d(f);

	// Mix
	return mix(a, b, u.x) +
		(c - a) * u.y * (1.0 - u.x) +
		(d - b) * u.x * u.y;
}
