use crate::mesh_loader::{self, load_level, GLTFLoadConfig, MeshLoader};
use bevy::core_pipeline::experimental::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing};
use bevy::core_pipeline::Skybox;
use bevy::image::CompressedImageFormats;
use bevy::pbr::{
    CascadeShadowConfigBuilder, DirectionalLightShadowMap, ScreenSpaceAmbientOcclusion,
    ScreenSpaceAmbientOcclusionQualityLevel,
};
use bevy::prelude::*;
use bevy::render::camera::TemporalJitter;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};
use bevy_rapier3d::prelude::*;
use bevy_water::{WaterPlugin, WaterSettings};

pub struct SceneLoaderPlugin;

pub const CUBEMAPS: &[(&str, CompressedImageFormats)] =
    &[("test_skybox.png", CompressedImageFormats::NONE)];

#[derive(Resource)]
pub struct Cubemap {
    pub(crate) is_loaded: bool,
    pub(crate) image_handle: Handle<Image>,
}

impl Plugin for SceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(mesh_loader::setup));

        app.add_systems(Update, asset_loaded);
        app.add_plugins((
            WaterPlugin,
            TemporalAntiAliasPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default().disabled(),
        ))
        .add_systems(Update, debug_render_toggle)
        .insert_resource(WaterSettings {
            height: -10.0,
            edge_scale: 0.5,
            ..default()
        });
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut mesh_loader: ResMut<MeshLoader>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("test_song.ogg")),
        PlaybackSettings::LOOP,
    ));

    commands.insert_resource(ClearColor(Color::srgb(0.3, 0.6, 0.9)));
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: true,
    });
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 5000.0,
            shadows_enabled: true,
            affects_lightmapped_mesh_diffuse: true,
            shadow_depth_bias: 1.0,
            shadow_normal_bias: 1.0,
        },
        CascadeShadowConfigBuilder {
            maximum_distance: 500.0,
            ..default()
        }
        .build(),
    ));
    let skybox_handle = asset_server.load(CUBEMAPS[0].0);

    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: skybox_handle.clone(),
    });

    commands.spawn((
        Camera3d::default(),
        Camera {
            // renders after / on top of the main camera
            order: 1,
            hdr: true,
            // don't clear the color while rendering this camera
            clear_color: ClearColorConfig::Default,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 55.0f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(-0.5, 0.3, 4.5).with_rotation(Quat::from_axis_angle(Vec3::Y, 0.0)),
        Skybox {
            image: skybox_handle.clone(),
            brightness: 1000.0,
            rotation: Default::default(),
        },
        DistanceFog {
            color: Color::srgb(0.25, 0.25, 0.25),
            falloff: FogFalloff::Linear {
                start: 500.0,
                end: 600.0,
            },
            ..default()
        },
        Msaa::Off,
        ScreenSpaceAmbientOcclusion {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
            ..default()
        },
        TemporalAntiAliasing::default(),
        TemporalJitter::default(),
    ));

    load_level(
        String::from("test_scene2.glb"),
        GLTFLoadConfig {
            spawn: true,
            generate_collider: true,
            collision_groups: CollisionGroups {
                memberships: Default::default(),
                filters: Default::default(),
            },
        },
        &mut asset_server,
        &mut mesh_loader,
    );
}

fn debug_render_toggle(mut context: ResMut<DebugRenderContext>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_released(KeyCode::F12) {
        context.enabled = !context.enabled;
    }
}

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle).is_loaded() {
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}
