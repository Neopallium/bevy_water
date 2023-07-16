use bevy::{
  prelude::*,
  render::{
    texture::ImageSampler,
    render_resource::{
      AddressMode,
      FilterMode,
      SamplerDescriptor, TextureFormat, TextureViewDescriptor, TextureViewDimension
    },
  }
};

pub enum ImageAction {
  Reformat(TextureFormat),
  Sampler(SamplerDescriptor<'static>),
  Cubemap,
}

#[derive(Component)]
pub struct ImageReformat {
  name: String,
  image: Handle<Image>,
  action: ImageAction,
}

impl ImageReformat {
  pub fn new(commands: &mut Commands, asset_server: &AssetServer, name: &str, action: ImageAction) -> Handle<Image> {
    let image = asset_server.load(name);
    commands.spawn(Self {
      name: name.to_string(),
      image: image.clone(),
      action,
    });
    image
  }

  pub fn reformat(commands: &mut Commands, asset_server: &AssetServer, name: &str, format: TextureFormat) -> Handle<Image> {
    Self::new(commands, asset_server, name, ImageAction::Reformat(format))
  }

  pub fn cubemap(commands: &mut Commands, asset_server: &AssetServer, name: &str) -> Handle<Image> {
    Self::new(commands, asset_server, name, ImageAction::Cubemap)
  }

  /// Change Sampler UV address mode to repeat.
  pub fn uv_repeat(commands: &mut Commands, asset_server: &AssetServer, name: &str) -> Handle<Image> {
    let sampler = SamplerDescriptor {
      address_mode_u: AddressMode::Repeat,
      address_mode_v: AddressMode::Repeat,
      address_mode_w: AddressMode::Repeat,
      mipmap_filter: FilterMode::Linear,
      ..default()
    };
    Self::new(commands, asset_server, name, ImageAction::Sampler(sampler))
  }
}

/// Reformat image.
fn reformat_image(
  query: Query<(Entity, &ImageReformat)>,
  mut commands: Commands,
  mut images: ResMut<Assets<Image>>,
) {
  for (entity, reformat) in &query {
    if let Some(image) = images.get_mut(&reformat.image) {
      match &reformat.action {
        ImageAction::Reformat(format) => {
          info!("Reformat {}", reformat.name);
          image.texture_descriptor.format = *format;
        }
        ImageAction::Cubemap => {
          if image.texture_descriptor.array_layer_count() == 1 {
            info!("Reinterpret 2D image {}", reformat.name);
            image.reinterpret_stacked_2d_as_array(
              image.texture_descriptor.size.height / image.texture_descriptor.size.width,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
              dimension: Some(TextureViewDimension::Cube),
              ..default()
            });
          }
        }
        ImageAction::Sampler(sampler) => {
          info!("Change image sampler {}", reformat.name);
          image.sampler_descriptor = ImageSampler::Descriptor(sampler.clone());
        }
      }
      commands.entity(entity).despawn();
    }
  }
}

#[derive(Default, Clone, Debug)]
pub struct ImageUtilsPlugin;

impl Plugin for ImageUtilsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, reformat_image);
  }
}
