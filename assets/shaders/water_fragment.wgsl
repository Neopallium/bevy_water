#import bevy_pbr::mesh_view_bindings  view
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput

#import bevy_pbr::pbr_functions as pbr_functions
#import bevy_pbr::pbr_bindings as pbr_bindings
#import bevy_pbr::pbr_types as pbr_types
#ifdef USE_DEPTH
#import bevy_pbr::prepass_utils
#endif

#import bevy_pbr::mesh_vertex_output       MeshVertexOutput
#import bevy_pbr::mesh_bindings            mesh
#import bevy_pbr::mesh_view_bindings       view, fog, screen_space_ambient_occlusion_texture
#import bevy_pbr::mesh_view_types          FOG_MODE_OFF
#import bevy_core_pipeline::tonemapping    screen_space_dither, powsafe, tone_mapping
#import bevy_pbr::parallax_mapping         parallaxed_uv

#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
#import bevy_pbr::gtao_utils gtao_multibounce
#endif

#import bevy_water::water_bindings as water_bindings
#import bevy_water::water_functions as water_fn

fn ndc_depth_to_linear(ndc_depth: f32) -> f32 {
  return -view.projection[3][2] / ndc_depth;
}

@fragment
fn fragment(
	in: MeshVertexOutput,
  @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
  var world_position: vec4<f32> = in.world_position;
  let w_pos = water_fn::uv_to_coord(in.uv);
  // Calculate normal.
  let delta = 0.2;
  let height = water_fn::get_wave_height(w_pos);
  let height_dx = water_fn::get_wave_height(w_pos + vec2<f32>(delta, 0.0));
  let height_dz = water_fn::get_wave_height(w_pos + vec2<f32>(0.0, delta));
  let world_normal = normalize(in.world_normal + (vec3<f32>(height - height_dx, delta, height - height_dz) * 8.0));

  var output_color: vec4<f32> = water_bindings::material.base_color;

  let deep_color = water_bindings::material.deep_color;
#ifdef USE_DEPTH
  let water_clarity = water_bindings::material.clarity;
  let shallow_color = water_bindings::material.shallow_color;
  let edge_scale = water_bindings::material.edge_scale;
  let edge_color = water_bindings::material.edge_color;

  let z_depth_buffer_ndc = bevy_pbr::prepass_utils::prepass_depth(in.position, 0u);
  let z_depth_buffer_view = ndc_depth_to_linear(z_depth_buffer_ndc);
  let z_fragment_view = ndc_depth_to_linear(in.position.z);
  let depth_diff_view = z_fragment_view - z_depth_buffer_view;
  let beers_law = exp(-depth_diff_view * water_clarity);
  let depth_color = vec4<f32>(mix(deep_color.xyz, shallow_color.xyz, beers_law), 1.0 - beers_law);
  let water_color = mix(edge_color, depth_color, smoothstep(0.0, edge_scale, depth_diff_view));
  output_color = output_color * water_color;
#else
  output_color = output_color * deep_color;
#endif
  //let foam_color = water_bindings::material.edge_color;
  //let foam = mix(foam_color, depth_color, smoothstep(0.0, edge_scale, depth_diff_view));

  let is_orthographic = view.projection[3].w == 1.0;
  let V = pbr_functions::calculate_view(world_position, is_orthographic);
#ifdef VERTEX_UVS
  var uv = in.uv;
#ifdef VERTEX_TANGENTS
  if ((water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DEPTH_MAP_BIT) != 0u) {
    let N = world_normal;
    let T = in.world_tangent.xyz;
    let B = in.world_tangent.w * cross(N, T);
    // Transform V from fragment to camera in world space to tangent space.
    let Vt = vec3(dot(V, T), dot(V, B), dot(V, N));
    uv = parallaxed_uv(
      water_bindings::material.parallax_depth_scale,
      water_bindings::material.max_parallax_layer_count,
      water_bindings::material.max_relief_mapping_search_steps,
      uv,
      // Flip the direction of Vt to go toward the surface to make the
      // parallax mapping algorithm easier to understand and reason
      // about.
      -Vt,
    );
  }
#endif
#endif

#ifdef VERTEX_COLORS
  output_color = output_color * in.color;
#endif
#ifdef VERTEX_UVS
  if ((water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u) {
    output_color = output_color * textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv, view.mip_bias);
  }
#endif

  // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
  // the material members
  var pbr_input: pbr_functions::PbrInput;

  pbr_input.material.base_color = output_color;
  pbr_input.material.reflectance = water_bindings::material.reflectance;
  pbr_input.material.flags = water_bindings::material.flags;
  pbr_input.material.alpha_cutoff = water_bindings::material.alpha_cutoff;

  // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
  if ((water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u) {
    // TODO use .a for exposure compensation in HDR
    var emissive: vec4<f32> = water_bindings::material.emissive;
#ifdef VERTEX_UVS
    if ((water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) != 0u) {
      emissive = vec4<f32>(emissive.rgb * textureSampleBias(pbr_bindings::emissive_texture, pbr_bindings::emissive_sampler, uv, view.mip_bias).rgb, 1.0);
    }
#endif
    pbr_input.material.emissive = emissive;

    var metallic: f32 = water_bindings::material.metallic;
    var perceptual_roughness: f32 = water_bindings::material.perceptual_roughness;
#ifdef VERTEX_UVS
    if ((water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0u) {
      let metallic_roughness = textureSampleBias(pbr_bindings::metallic_roughness_texture, pbr_bindings::metallic_roughness_sampler, uv, view.mip_bias);
      // Sampling from GLTF standard channels for now
      metallic = metallic * metallic_roughness.b;
      perceptual_roughness = perceptual_roughness * metallic_roughness.g;
    }
#endif
    pbr_input.material.metallic = metallic;
    pbr_input.material.perceptual_roughness = perceptual_roughness;

    // TODO: Split into diffuse/specular occlusion?
    var occlusion: vec3<f32> = vec3(1.0);
#ifdef VERTEX_UVS
    if ((water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0u) {
      occlusion = vec3(textureSampleBias(pbr_bindings::occlusion_texture, pbr_bindings::occlusion_sampler, uv, view.mip_bias).r);
    }
#endif
#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
    let ssao = textureLoad(screen_space_ambient_occlusion_texture, vec2<i32>(in.position.xy), 0i).r;
    let ssao_multibounce = gtao_multibounce(ssao, pbr_input.material.base_color.rgb);
    occlusion = min(occlusion, ssao_multibounce);
#endif
    pbr_input.occlusion = occlusion;

    pbr_input.frag_coord = in.position;
    pbr_input.world_position = world_position;

    pbr_input.world_normal = pbr_functions::prepare_world_normal(
      world_normal,
      (water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
      is_front,
    );

    pbr_input.is_orthographic = is_orthographic;

#ifdef LOAD_PREPASS_NORMALS
    pbr_input.N = bevy_pbr::prepass_utils::prepass_normal(in.position, 0u);
#else
    pbr_input.N = pbr_functions::apply_normal_mapping(
      water_bindings::material.flags,
      pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
      in.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
      uv,
#endif
      view.mip_bias,
    );
#endif

    pbr_input.V = V;
    pbr_input.occlusion = occlusion;

    pbr_input.flags = mesh.flags;

    output_color = pbr_functions::pbr(pbr_input);
  } else {
    output_color = pbr_functions::alpha_discard(pbr_input.material, output_color);
  }

  // fog
  if (fog.mode != FOG_MODE_OFF && (water_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT) != 0u) {
    output_color = pbr_functions::apply_fog(fog, output_color, world_position.xyz, view.world_position.xyz);
  }

#ifdef TONEMAP_IN_SHADER
  output_color = tone_mapping(output_color, view.color_grading);
#ifdef DEBAND_DITHER
  var output_rgb = output_color.rgb;
  output_rgb = powsafe(output_rgb, 1.0 / 2.2);
  output_rgb = output_rgb + screen_space_dither(in.position.xy);
  // This conversion back to linear space is required because our output texture format is
  // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
  output_rgb = powsafe(output_rgb, 2.2);
  output_color = vec4(output_rgb, output_color.a);
#endif
#endif
#ifdef PREMULTIPLY_ALPHA
  output_color = pbr_functions::premultiply_alpha(water_bindings::material.flags, output_color);
#endif

  // show grid
  //let f_pos = step(fract((w_pos / 10.06274)), vec2<f32>(0.995));
  //let grid = step(f_pos.x + f_pos.y, 1.00);
  //output_color = output_color + vec4<f32>(grid, grid, grid, 0.00);

  return output_color;
}
