use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::*};

#[derive(Clone, Default, ShaderType)]
pub struct WaterParams {
  pub time: f32,
}

#[derive(AsBindGroup, Clone, Default, TypeUuid)]
#[uuid = "ea9ad5fe-a9ce-4ff1-aea8-d756ed456c46"]
pub struct WaterMaterial {
  #[uniform(0)]
  pub params: WaterParams,
}

impl WaterMaterial {
  pub fn new() -> Self {
    Self {
      params: WaterParams { time: 1.0 },
    }
  }
}

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

/// Update Water time.
fn update_water_time(time: Res<Time>, mut materials: ResMut<Assets<WaterMaterial>>) {
  for (_, material) in materials.iter_mut() {
    material.params.time = time.seconds_since_startup() as f32;
  }
}

#[derive(Default, Clone, Debug)]
pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugin(MaterialPlugin::<WaterMaterial>::default())
      .add_system(update_water_time);
  }
}
