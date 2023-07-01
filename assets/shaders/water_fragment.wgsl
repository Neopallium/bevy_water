#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#ifdef USE_DEPTH
#import bevy_pbr::prepass_utils
#endif

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions

#import bevy_water::water_bindings
#import bevy_water::water_functions

struct FragmentInput {
  @builtin(front_facing) is_front: bool,
  @builtin(position) frag_coord: vec4<f32>,
#ifdef USE_DEPTH
#ifdef MULTISAMPLED
  @builtin(sample_index) sample_index: u32,
#endif
#endif
  #import bevy_pbr::mesh_vertex_output
};

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

#ifdef USE_DEPTH
  let water_clarity = material.clarity;
  let deep_color = material.deep_color;
  let shallow_color = material.shallow_color;
  let edge_scale = material.edge_scale;
  let edge_color = material.edge_color;

#ifdef MULTISAMPLED
  let z_depth_buffer_ndc = prepass_depth(in.frag_coord, in.sample_index);
#else
  let z_depth_buffer_ndc = prepass_depth(in.frag_coord, 0);
#endif
  let z_depth_buffer_view = ndc_depth_to_linear(z_depth_buffer_ndc);
  let z_fragment_view = ndc_depth_to_linear(in.frag_coord.z);
  let depth_diff_view = z_fragment_view - z_depth_buffer_view;
  let beers_law = exp(-depth_diff_view * water_clarity);
  let depth_color = vec4<f32>(mix(deep_color.xyz, shallow_color.xyz, beers_law), 1.0 - beers_law);
  let water_color = mix(edge_color, depth_color, smoothstep(0.0, edge_scale, depth_diff_view));
  output_color = output_color * water_color;
#endif
  //let foam_color = material.edge_color;
  //let foam = mix(foam_color, depth_color, smoothstep(0.0, edge_scale, depth_diff_view));

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
