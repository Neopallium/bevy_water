#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::pbr_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

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
  @location(0) world_position: vec4<f32>,
  @location(1) world_normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
  @location(3) world_tangent: vec4<f32>,
#endif
  @location(4) base_height: f32,
};

struct FragmentInput {
  @builtin(front_facing) is_front: bool,
  @builtin(position) frag_coord: vec4<f32>,
  @location(0) world_position: vec4<f32>,
  @location(1) world_normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
  @location(3) world_tangent: vec4<f32>,
#endif
  @location(4) base_height: f32,
};

#import "shaders/noise/random.wgsl"
#import "shaders/noise/vnoise.wgsl"

fn noise2(v: vec2<f32>) -> f32 {
  return vnoise2d(v);
}

#import "shaders/noise/fbm.wgsl"

fn wave(p: vec2<f32>) -> f32 {
  let time = globals.time * .5 + 23.0;

  let time_x = time / 1.0;
  let time_y = time / 0.5;
  let time_uv = vec2<f32>(time_x, time_y) / 16.0;
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
  return d;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  // Need the world position when calculating wave height.
  var world_position = mesh.model * vec4<f32>(vertex.pos, 1.0);
  let base_height = world_position.y;

  // Add the wave height to the world position.
  let height = get_wave_height(world_position.xz);
  //let height = get_wave_height(vertex.pos.xz);
  world_position = world_position + vec4<f32>(0., height, 0., 0.);

  var out: VertexOutput;
  out.world_position = world_position;
#ifdef VERTEX_TANGENTS
  out.world_tangent = vec4<f32>(
    mat3x3<f32>(
      mesh.model[0].xyz,
      mesh.model[1].xyz,
      mesh.model[2].xyz
    ) * vertex.tangent.xyz,
    vertex.tangent.w
  );
#endif
  out.frag_coord = view.view_proj * out.world_position;
  out.uv = vertex.uv;
  out.base_height = base_height;
  return out;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
  let w_pos = in.world_position.xz;
  let height = in.world_position.y - in.base_height;
  // Calculate normal.
  let b = get_wave_height(w_pos + vec2<f32>(0.0, 1.0));
  let c = get_wave_height(w_pos + vec2<f32>(1.0, 1.0));
  let normal = normalize(vec3<f32>(height - b, 1.0, height - c));

  let color = vec3<f32>(0.01, 0.03, 0.05);
  
  // show grid
  //let f_pos = step(fract((w_pos / 17.06274)), vec2<f32>(0.995));
  //let grid = step(f_pos.x + f_pos.y, 1.00);
  //let color = color + vec3<f32>(grid);

  // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
  // the material members
  var pbr_input = pbr_input_new();

  pbr_input.material.base_color = vec4<f32>(color.xyz, 0.97);
  pbr_input.material.reflectance = 0.5;
  pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;

  // TODO use .a for exposure compensation in HDR
  pbr_input.material.emissive = vec4<f32>(0.,0.,0.,1.);

  pbr_input.material.metallic = 0.0;
  pbr_input.material.perceptual_roughness = 0.22;

  pbr_input.occlusion = 1.0;

  pbr_input.frag_coord = in.frag_coord;
  pbr_input.world_position = in.world_position;

  pbr_input.is_orthographic = view.projection[3].w == 1.0;

	pbr_input.world_normal = prepare_world_normal(
      normal,
      (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
      in.is_front,
  );

  pbr_input.N = apply_normal_mapping(
    pbr_input.material.flags,
    normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
    in.world_tangent,
#endif
#endif
    in.uv,
  );
  pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

  return tone_mapping(pbr(pbr_input));
}
