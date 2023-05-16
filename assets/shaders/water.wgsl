#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::pbr_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions

struct WaterMaterial {
  amplitude: f32,
};

@group(1) @binding(0)
var<uniform> water_material: WaterMaterial;

struct Vertex {
  @location(0) pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
  @location(3) tangent: vec4<f32>,
#endif
};

struct VertexOutput {
  @builtin(position) frag_coord: vec4<f32>,
  #import bevy_pbr::mesh_vertex_output
};

struct FragmentInput {
  @builtin(front_facing) is_front: bool,
  @builtin(position) frag_coord: vec4<f32>,
  #import bevy_pbr::mesh_vertex_output
};

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
  return water_material.amplitude * d;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  // Need the world position when calculating wave height.
  var world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.pos, 1.0));

  // Add the wave height to the world position.
  let height = get_wave_height(world_position.xz);

  var out: VertexOutput;
  out.world_position = world_position + vec4<f32>(0., height, 0., 0.);
#ifdef VERTEX_TANGENTS
  out.world_tangent = mesh_tangent_local_to_world(mesh.model, vertex.tangent);
#endif
  out.frag_coord = mesh_position_world_to_clip(out.world_position);
  out.uv = vertex.uv;
  return out;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
  var world_position: vec4<f32> = in.world_position;
  let w_pos = world_position.xz;
  // Calculate normal.
  let height = get_wave_height(w_pos);
  let height_dx = get_wave_height(w_pos + vec2<f32>(1.0, 0.0));
  let height_dz = get_wave_height(w_pos + vec2<f32>(0.0, 1.0));
  let normal = normalize(vec3<f32>(height - height_dx, 1.0, height - height_dz));
  world_position.y = height;

  let color = vec3<f32>(0.01, 0.03, 0.05);

  var output_color: vec4<f32> = vec4<f32>(color.xyz, 0.97);
  
  // show grid
  //let f_pos = step(fract((w_pos / 1.06274)), vec2<f32>(0.995));
  //let grid = step(f_pos.x + f_pos.y, 1.00);
  //output_color = output_color + vec4<f32>(grid, grid, grid, 0.00);

  // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
  // the material members
  var pbr_input = pbr_input_new();

  pbr_input.material.base_color = output_color;
  pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;

  pbr_input.material.perceptual_roughness = 0.22;

  pbr_input.frag_coord = in.frag_coord;
  pbr_input.world_position = world_position;
  pbr_input.world_normal = prepare_world_normal(
      normal,
      (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
      in.is_front,
  );

  pbr_input.is_orthographic = view.projection[3].w == 1.0;

  pbr_input.N = apply_normal_mapping(
    pbr_input.material.flags,
    pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
    in.world_tangent,
#endif
#endif
    in.uv,
  );
  pbr_input.V = calculate_view(world_position, pbr_input.is_orthographic);

  pbr_input.flags = mesh.flags;

  output_color = pbr(pbr_input);

  // fog
  if (fog.mode != FOG_MODE_OFF && (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT) != 0u) {
    output_color = apply_fog(output_color, world_position.xyz, view.world_position.xyz);
  }

#ifdef TONEMAP_IN_SHADER
  output_color = tone_mapping(output_color);
#endif
#ifdef DEBAND_DITHER
  var output_rgb = output_color.rgb;
  output_rgb = powsafe(output_rgb, 1.0 / 2.2);
  output_rgb = output_rgb + screen_space_dither(in.frag_coord.xy);
  // This conversion back to linear space is required because our output texture format is
  // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
  output_rgb = powsafe(output_rgb, 2.2);
  output_color = vec4(output_rgb, output_color.a);
#endif
#ifdef PREMULTIPLY_ALPHA
  output_color = premultiply_alpha(pbr_input.material.flags, output_color);
#endif

  return output_color;
}
