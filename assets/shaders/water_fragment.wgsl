#import bevy_pbr::{
  pbr_functions::alpha_discard,
  pbr_fragment::pbr_input_from_standard_material,
  view_transformations::depth_ndc_to_view_z,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
  prepass_io::{VertexOutput, FragmentOutput},
  pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
  forward_io::{VertexOutput, FragmentOutput},
  pbr_functions,
  pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
  pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

#import bevy_water::water_bindings
#ifdef VERTEX_UVS
#import bevy_water::water_functions as water_fn
#endif

@fragment
fn fragment(
#ifdef MESHLET_MESH_MATERIAL_PASS
    @builtin(position) frag_coord: vec4<f32>,
#else
  p_in: VertexOutput,
  @builtin(front_facing) is_front: bool,
#endif
) -> FragmentOutput {
#ifdef MESHLET_MESH_MATERIAL_PASS
  let p_in = resolve_vertex_output(frag_coord);
  let is_front = true;
#endif

  var in = p_in;
#ifdef VERTEX_UVS
  // Flat-water tiles. Planet shell has no UVs and keeps the mesh normal:
  // screen-space derivatives of the wave were a gray speckle on the sphere.
#if QUALITY > 2
  let w_pos = water_fn::uv_to_coord(in.uv);
  let height = water_fn::get_wave_height(w_pos);
  let delta = 0.5;
  let height_dx = water_fn::get_wave_height(w_pos + vec2<f32>(delta, 0.0));
  let height_dz = water_fn::get_wave_height(w_pos + vec2<f32>(0.0, delta));
  in.world_normal = normalize(vec3<f32>(height - height_dx, delta, height - height_dz));
#endif
#endif
 
  // If we're in the crossfade section of a visibility range, conditionally
  // discard the fragment according to the visibility pattern.
#ifdef VISIBILITY_RANGE_DITHER
  pbr_functions::visibility_range_dither(in.position, in.visibility_range_dither);
#endif

  // generate a PbrInput struct from the StandardMaterial bindings
  var pbr_input = pbr_input_from_standard_material(in, is_front);

  let deep_color = water_bindings::material.deep_color;
  let near_alpha = pbr_input.material.base_color.a;
  pbr_input.material.base_color *= deep_color;
  // Cap blue so a stale saturated deep_color does not stay electric.
  var rgb = max(pbr_input.material.base_color.rgb, vec3<f32>(0.012, 0.05, 0.08));
  rgb.b = min(rgb.b, 0.22);
  // Night ocean was lifted by the sky showing through. Darker albedo.
  rgb *= 0.65;
  // Sky brightness is 1000, so any alpha below 1 leaves stars after tonemap.
  // Open the shell only when an opaque surface sits within `edge_scale` meters
  // behind it. The far plane (empty sky under the shell) stays exactly opaque.
  var alpha = 1.0;
#ifdef DEPTH_PREPASS
#ifndef PREPASS_PIPELINE
#ifndef WEBGL2
  let z_depth_buffer_ndc = bevy_pbr::prepass_utils::prepass_depth(in.position, 0u);
  let z_depth_buffer_view = depth_ndc_to_view_z(z_depth_buffer_ndc);
  let z_fragment_view = depth_ndc_to_view_z(in.position.z);
  let behind = z_fragment_view - z_depth_buffer_view;
  let reach = max(water_bindings::material.edge_scale, 0.001);
  if (behind > 0.05 && behind < reach) {
    alpha = mix(near_alpha, 1.0, behind / reach);
  }
#endif
#endif
#endif
  pbr_input.material.base_color = vec4<f32>(rgb, alpha);

  // alpha discard
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
  let out = deferred_output(in, pbr_input);
#else
  var out: FragmentOutput;
  if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
    out.color = apply_pbr_lighting(pbr_input);
  } else {
    out.color = pbr_input.material.base_color;
  }

  out.color = main_pass_post_lighting_processing(pbr_input, out.color);
  out.color.a = pbr_input.material.base_color.a;
#endif

  return out;
}
