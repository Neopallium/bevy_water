//  MIT License. © Inigo Quilez, Munrocket
//  noise2() is any noise here: Value, Perlin, Simplex, Worley
//
#define_import_path bevy_water::noise::fbm

#import bevy_water::noise::vnoise::vnoise2d

fn fbm(v2: vec2<f32>, quality: u32) -> f32 {
    let m2 = mat2x2<f32>(vec2<f32>(0.8, 0.6), vec2<f32>(-0.6, 0.8));
    var p = v2;
    var f = 0.;
    f = f + 0.5000 * vnoise2d(p);
    if quality > 1 {
        p = m2 * p * 2.02;
        f = f + 0.2500 * vnoise2d(p);
    }
    if quality > 3 {
        p = m2 * p * 2.03;
        f = f + 0.1250 * vnoise2d(p);
        p = m2 * p * 2.01;
        f = f + 0.0625 * vnoise2d(p);
    }
    return f / 0.9375;

}
