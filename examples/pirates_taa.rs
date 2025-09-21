//! Showcases dynamic ocean material + dynamic Sun/Atmosphere.
//! Most of the daylight cycle code taken from the examples of `bevy_atmosphere`.

use bevy::{
  anti_alias::taa::TemporalAntiAliasing,
  prelude::*,
};

mod pirates;
use pirates::*;

fn main() {
  let mut app = pirates_app("Pirates with TAA");

  // Setup
  app.add_systems(
    Startup,
    (setup_ocean, setup_orb, setup_camera_taa, setup_ships),
  );

  app.run();
}

/// set up a simple 3D scene
fn setup_camera_taa(mut commands: Commands, asset_server: Res<AssetServer>) {
  // camera
  let mut cam = make_camera(&mut commands, &asset_server);
  cam.insert((Msaa::Off, TemporalAntiAliasing::default()));
}
