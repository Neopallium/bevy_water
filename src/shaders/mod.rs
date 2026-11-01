use bevy::{prelude::*, shader::load_shader_library};

/// Embed the water shaders.
///
/// They are importable from WESL as `bevy_water::shaders::*`.
pub(crate) fn load_shaders(app: &mut App) {
  load_shader_library!(app, "noise/random.wesl");
  load_shader_library!(app, "noise/vnoise.wesl");
  load_shader_library!(app, "noise/fbm.wesl");
  load_shader_library!(app, "water_bindings.wesl");
  load_shader_library!(app, "water_functions.wesl");
  load_shader_library!(app, "water_vertex.wesl");
  load_shader_library!(app, "water_fragment.wesl");
}
