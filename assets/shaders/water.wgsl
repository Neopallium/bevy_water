#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::prepass_utils

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions
#import bevy_pbr::mesh_functions

struct WaterMaterial {
  // StandardMaterial fields.
  base_color: vec4<f32>,
  emissive: vec4<f32>,
  perceptual_roughness: f32,
  metallic: f32,
  reflectance: f32,
  // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
  flags: u32,
  alpha_cutoff: f32,
  // WaterMaterial fields.
  amplitude: f32,
  coord_offset: vec2<f32>,
  coord_scale: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> material: WaterMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;
@group(1) @binding(3)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_sampler: sampler;
@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;
@group(1) @binding(7)
var occlusion_texture: texture_2d<f32>;
@group(1) @binding(8)
var occlusion_sampler: sampler;
@group(1) @binding(9)
var normal_map_texture: texture_2d<f32>;
@group(1) @binding(10)
var normal_map_sampler: sampler;

struct Vertex {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
  @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

struct VertexOutput {
  @builtin(position) frag_coord: vec4<f32>,
  #import bevy_pbr::mesh_vertex_output
};

struct FragmentInput {
  @builtin(front_facing) is_front: bool,
  @builtin(position) frag_coord: vec4<f32>,
#ifndef DEPTH_PREPASS
  @builtin(sample_index) sample_index: u32,
#endif
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
  return material.amplitude * d;
}

fn uv_to_coord(uv: vec2<f32>) -> vec2<f32> {
  // Invert the y UV coord.
  let w_uv = vec2<f32>(uv.x, 1.0 - uv.y);
  return material.coord_offset + (w_uv * material.coord_scale);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;

#ifdef SKINNED
  var model = skin_model(vertex.joint_indices, vertex.joint_weights);
  out.world_normal = skin_normals(model, vertex.normal);
#else
  var model = mesh.model;
  out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif

  let world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));

  // Add the wave height to the world position.
  let w_pos = uv_to_coord(vertex.uv);
  let height = get_wave_height(w_pos);

  out.world_position = world_position + vec4<f32>((out.world_normal * height), 0.);
  out.frag_coord = mesh_position_world_to_clip(out.world_position);

#ifdef VERTEX_UVS
  out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
  out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif

#ifdef VERTEX_COLORS
  out.color = vertex.color;
#endif

  return out;
}

fn ndc_depth_to_linear(ndc_depth: f32) -> f32 {
  return -view.projection[3][2] / ndc_depth;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
  var world_position: vec4<f32> = in.world_position;
  let w_pos = uv_to_coord(in.uv);
  // Calculate normal.
  let delta = 0.2;
  let height = get_wave_height(w_pos);
  let height_dx = get_wave_height(w_pos + vec2<f32>(delta, 0.0));
  let height_dz = get_wave_height(w_pos + vec2<f32>(0.0, delta));
  let world_normal = normalize(in.world_normal + (vec3<f32>(height - height_dx, delta, height - height_dz) * 8.0));

  var output_color: vec4<f32> = material.base_color;

#ifndef DEPTH_PREPASS
	let strength = 0.2;
  let z_depth_buffer_ndc = prepass_depth(in.frag_coord, in.sample_index);
  let z_depth_buffer_view = ndc_depth_to_linear(z_depth_buffer_ndc);
  let z_fragment_view = ndc_depth_to_linear(in.frag_coord.z);
  let depth_diff_view = z_fragment_view - z_depth_buffer_view;
	let mu = 0.1;
	let beers_law = exp(-depth_diff_view * mu);
	let deep_color = vec3<f32>(0.200, 0.412, 0.537);
	let shalow_color = vec3<f32>(0.443, 0.776, 0.812);
	let depth_color = mix(deep_color, shalow_color, beers_law);
  output_color = vec4<f32>(depth_color, 1.0 - beers_law);
#endif

#ifdef VERTEX_COLORS
  output_color = output_color * in.color;
#endif
#ifdef VERTEX_UVS
  if ((material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u) {
    output_color = output_color * textureSample(base_color_texture, base_color_sampler, in.uv);
  }
#endif
  // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
  // the material members
  var pbr_input: PbrInput;

  pbr_input.material.base_color = output_color;
  pbr_input.material.reflectance = material.reflectance;
  pbr_input.material.flags = material.flags;
  pbr_input.material.alpha_cutoff = material.alpha_cutoff;

  // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
  if ((material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u) {
    // TODO use .a for exposure compensation in HDR
    var emissive: vec4<f32> = material.emissive;
#ifdef VERTEX_UVS
    if ((material.flags & STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) != 0u) {
      emissive = vec4<f32>(emissive.rgb * textureSample(emissive_texture, emissive_sampler, in.uv).rgb, 1.0);
    }
#endif
    pbr_input.material.emissive = emissive;

    var metallic: f32 = material.metallic;
    var perceptual_roughness: f32 = material.perceptual_roughness;
#ifdef VERTEX_UVS
    if ((material.flags & STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0u) {
      let metallic_roughness = textureSample(metallic_roughness_texture, metallic_roughness_sampler, in.uv);
      // Sampling from GLTF standard channels for now
      metallic = metallic * metallic_roughness.b;
      perceptual_roughness = perceptual_roughness * metallic_roughness.g;
    }
#endif
    pbr_input.material.metallic = metallic;
    pbr_input.material.perceptual_roughness = perceptual_roughness;

    var occlusion: f32 = 1.0;
#ifdef VERTEX_UVS
    if ((material.flags & STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0u) {
      occlusion = textureSample(occlusion_texture, occlusion_sampler, in.uv).r;
    }
#endif
    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = world_position;
    pbr_input.world_normal = prepare_world_normal(
      world_normal,
      (material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
      in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
      material.flags,
      pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
      in.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
      in.uv,
#endif
    );
    pbr_input.V = calculate_view(world_position, pbr_input.is_orthographic);
    pbr_input.occlusion = occlusion;

    pbr_input.flags = mesh.flags;

    output_color = pbr(pbr_input);
  } else {
    output_color = alpha_discard(pbr_input.material, output_color);
  }

  // fog
  if (fog.mode != FOG_MODE_OFF && (material.flags & STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT) != 0u) {
    output_color = apply_fog(output_color, in.world_position.xyz, view.world_position.xyz);
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
  output_color = premultiply_alpha(material.flags, output_color);
#endif

  // show grid
  //let f_pos = step(fract((w_pos / 10.06274)), vec2<f32>(0.995));
  //let grid = step(f_pos.x + f_pos.y, 1.00);
  //output_color = output_color + vec4<f32>(grid, grid, grid, 0.00);

  return output_color;
}
