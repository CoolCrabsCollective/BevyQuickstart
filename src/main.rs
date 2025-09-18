mod debug_camera_controller;
mod game;
mod mesh_loader;

use crate::game::GamePlugin;
use crate::mesh_loader::MeshLoaderPlugin;
use bevy::app::{App, PluginGroup};
use bevy::asset::AssetMetaCheck;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, FilterMode};
use bevy::render::RenderPlugin;
use bevy::DefaultPlugins;

fn main() {
    let mut app = App::new();

    let default_sampler = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::from(AddressMode::Repeat),
        address_mode_v: ImageAddressMode::from(AddressMode::Repeat),
        address_mode_w: ImageAddressMode::from(AddressMode::Repeat),
        mag_filter: ImageFilterMode::from(FilterMode::Linear),
        min_filter: ImageFilterMode::from(FilterMode::Linear),
        mipmap_filter: ImageFilterMode::from(FilterMode::Linear),
        ..default()
    };
    if cfg!(target_arch = "wasm32") {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        title: "Bevy Quickstart Game".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin { default_sampler }),
        );
    } else {
        app.add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: Default::default(),
                    synchronous_pipeline_compilation: false,
                    debug_flags: Default::default(),
                })
                .set(ImagePlugin { default_sampler }),
        );
    }
    app.add_plugins(MeshLoaderPlugin);
    app.add_plugins(GamePlugin);

    app.run();
}
