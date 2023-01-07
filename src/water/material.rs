use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::*};

#[derive(AsBindGroup, Clone, Default, TypeUuid)]
#[uuid = "ea9ad5fe-a9ce-4ff1-aea8-d756ed456c46"]
pub struct WaterMaterial {}

impl Material for WaterMaterial {
  fn vertex_shader() -> ShaderRef {
    "shaders/water.wgsl".into()
  }

  fn fragment_shader() -> ShaderRef {
    "shaders/water.wgsl".into()
  }

  #[inline]
  fn alpha_mode(&self) -> AlphaMode {
    AlphaMode::Blend
  }
}

#[derive(Default, Clone, Debug)]
pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugin(MaterialPlugin::<WaterMaterial>::default());
  }
}
