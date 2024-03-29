#import bevy_pbr::{
	pbr_fragment::pbr_input_from_standard_material,
	pbr_functions::alpha_discard,
	mesh_view_bindings::view,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
	prepass_io::{VertexOutput, FragmentOutput},
	pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
	forward_io::{VertexOutput, FragmentOutput},
	pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
	pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

#import bevy_water::water_bindings
#import bevy_water::water_functions as water_fn

fn ndc_depth_to_linear(ndc_depth: f32) -> f32 {
  return -view.projection[3][2] / ndc_depth;
}

@fragment
fn fragment(
	p_in: VertexOutput,
  @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
	var in = p_in;
  var world_position: vec4<f32> = in.world_position;
  let w_pos = water_fn::uv_to_coord(in.uv);
  // Calculate normal.
  let delta = 0.2;
  let height = water_fn::get_wave_height(w_pos);
  let height_dx = water_fn::get_wave_height(w_pos + vec2<f32>(delta, 0.0));
  let height_dz = water_fn::get_wave_height(w_pos + vec2<f32>(0.0, delta));
  let world_normal = normalize(in.world_normal + (vec3<f32>(height - height_dx, delta, height - height_dz) * 8.0));
  in.world_normal = world_normal;

	// get PbrInput from StandardMaterial bindings.
	var pbr_input = pbr_input_from_standard_material(in, is_front);

  let deep_color = water_bindings::material.deep_color;
	var water_color = deep_color;
#ifdef DEPTH_PREPASS
#ifndef WEBGL2
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
  water_color = mix(edge_color, depth_color, smoothstep(0.0, edge_scale, depth_diff_view));
#endif
#endif
  pbr_input.material.base_color *= water_color;

  //let foam_color = water_bindings::material.edge_color;
  //let foam = mix(foam_color, depth_color, smoothstep(0.0, edge_scale, depth_diff_view));

	// alpha discard
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
	// No lighting in deferred mode.
	let out = deferred_output(in, pbr_input);
#else
	var out: FragmentOutput;
  if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
		out.color = apply_pbr_lighting(pbr_input);
	} else {
		out.color = pbr_input.material.base_color;
	}

	// Apply PBR post processing.
	out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

  // show grid
  //let f_pos = step(fract((w_pos / 10.06274)), vec2<f32>(0.995));
  //let grid = step(f_pos.x + f_pos.y, 1.00);
  //out.color += vec4<f32>(grid, grid, grid, 0.00);

  return out;
}
