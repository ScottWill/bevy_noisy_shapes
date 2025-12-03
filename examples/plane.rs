use bevy::prelude::*;
use bevy_noisy_shapes::plane::NoisyPlane3d;
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
        Transform::from_xyz(0.0, 12.0, 16.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
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
        Mesh3d(meshes.add(NoisyPlane3d::default()
            .mesh()
            .sampler(PerlinNoiseBuilder {
                amplitude: PI * 0.1,
                fractal_noise: Some(FractalNoiseBuilder {
                    fractal_type: FractalType::RigidMulti,
                    gain: 0.13,
                    lacunarity: 2.1,
                    octaves: 4,
                }),
                frequency: PI,
                interp: Interp::Hermite,
                seed: 8008135,
            })
            .subdivisions(64)
            .square(24.0)
            .vertex_colors(true)
        )),
    ))
    .observe(drag_shape);
}

fn drag_shape(
    drag: On<Pointer<Drag>>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    let Ok(mut transform) = transforms.get_mut(drag.entity) else { return };

    let delta = drag.delta.x * time.delta_secs() * 0.2;
    transform.rotate_y(delta);
}