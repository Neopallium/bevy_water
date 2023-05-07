use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid, render::render_resource::*};

#[derive(AsBindGroup, Clone, Default, TypeUuid)]
#[uuid = "ea9ad5fe-a9ce-4ff1-aea8-d756ed456c46"]
pub struct WaterMaterial {}

const NOISE_FBM_SHADER_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x47c86614dedb33fe);

const NOISE_RANDOM_SHADER_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x339ea286e4c7be3e);

const NOISE_VNOISE_SHADER_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x2cb48f03a340aedc);

const WATER_SHADER_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0xcea5177230c961ac);

impl Material for WaterMaterial {
  fn vertex_shader() -> ShaderRef {
    WATER_SHADER_HANDLE.typed().into()
  }

  fn fragment_shader() -> ShaderRef {
    WATER_SHADER_HANDLE.typed().into()
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
    load_internal_asset!(
      app,
      NOISE_FBM_SHADER_HANDLE,
      "../../assets/shaders/noise/fbm.wgsl",
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      NOISE_RANDOM_SHADER_HANDLE,
      "../../assets/shaders/noise/random.wgsl",
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      NOISE_VNOISE_SHADER_HANDLE,
      "../../assets/shaders/noise/vnoise.wgsl",
      Shader::from_wgsl
    );

    load_internal_asset!(
      app,
      WATER_SHADER_HANDLE,
      "../../assets/shaders/water.wgsl",
      Shader::from_wgsl
    );

    app.add_plugin(MaterialPlugin::<WaterMaterial>::default());
  }
}
