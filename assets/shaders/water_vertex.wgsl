#import bevy_pbr::{
	mesh_functions,
	skinning,
	view_transformations::position_world_to_clip,
}

#import bevy_water::water_functions as water_fn

#ifdef PREPASS_PIPELINE
#import bevy_pbr::prepass_io::{Vertex, VertexOutput}
#else
#import bevy_pbr::forward_io::{Vertex, VertexOutput}
#endif

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;

#ifdef SKINNED
  var model = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
  var model = mesh_functions::get_model_matrix(vertex.instance_index);
#endif

#ifdef VERTEX_UVS
#ifdef SKINNED
  out.world_normal = skinning::skin_normals(model, vertex.normal);
#else
  out.world_normal = mesh_functions::mesh_normal_local_to_world(
		vertex.normal,
		vertex.instance_index
	);
#endif
#endif

  let world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));

  // Add the wave height to the world position.
  let w_pos = water_fn::uv_to_coord(vertex.uv);
  let height = water_fn::get_wave_height(w_pos);

  out.world_position = world_position + vec4<f32>((out.world_normal * height), 0.);
  out.position = position_world_to_clip(out.world_position.xyz);

#ifdef VERTEX_UVS
  out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
  out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
		model,
		vertex.tangent,
		vertex.instance_index
	);
#endif

#ifdef VERTEX_COLORS
  out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
	out.instance_index = vertex.instance_index;
#endif

  return out;
}
