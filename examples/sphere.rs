use bevy::prelude::*;
use bevy_noisy_shapes::sphere::{NoisySphere, NoisySphereKind};
use fastnoise::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
        ))
        .add_systems(Startup, (setup_scene, setup_shape))
        .run();
}

fn setup_scene(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 32.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10000000.0,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(0.0, 16.0, 8.0),
    ));
}

fn setup_shape(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Mesh3d(meshes.add(NoisySphere::new(7.0)
            .mesh()
            .kind(NoisySphereKind::Cubed { subdivisions: 64 })
            .sampler(PerlinNoiseBuilder {
                amplitude: -0.02,
                fractal_noise: Some(FractalNoiseBuilder {
                    fractal_type: FractalType::RigidMulti,
                    gain: 1.3,
                    lacunarity: 2.1,
                    octaves: 6,
                }),
                frequency: PI * 0.1,
                interp: Interp::Hermite,
                seed: 8008135,
                ..Default::default()
            })
            .vertex_colors(true)
        ))
    ))
    .observe(drag_shape);
}

fn drag_shape(
    drag: On<Pointer<Drag>>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    let Ok(mut transform) = transforms.get_mut(drag.entity) else { return };

    let delta = drag.delta * time.delta_secs() * 0.2;
    transform.rotate_y(delta.x);
    transform.rotate_x(delta.y);
}