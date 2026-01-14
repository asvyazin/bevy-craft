use bevy::prelude::*;
use bevy::math::primitives::Cuboid;

/// Component for the skybox entity
#[derive(Component)]
pub struct Skybox;

/// System to spawn the skybox
pub fn spawn_skybox(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Create skybox material with realistic sky colors
    let sky_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.7, 0.9), // Base sky color
        unlit: true, // Sky should not be affected by lighting
        ..default()
    });

    // Create a large cube mesh for the skybox (temporary solution)
    // This will be replaced with a proper sphere once we find the right Bevy primitive
    let sky_mesh = meshes.add(Mesh::from(Cuboid {
        half_size: Vec3::new(1000.0, 1000.0, 1000.0),  // Very large size to ensure it's always visible
    }));

    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(sky_material),
        Transform::from_translation(Vec3::ZERO),
        Skybox,
    ));
}