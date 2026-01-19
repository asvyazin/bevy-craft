use bevy::prelude::*;

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

    // Create a proper sphere mesh for the skybox
    // Using a high-resolution sphere for smooth appearance
    let sky_mesh = meshes.add(Mesh::try_from(bevy::prelude::Sphere {
        radius: 1000.0,
    }).unwrap());

    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(sky_material),
        Transform::from_translation(Vec3::ZERO),
        Skybox,
    ));
}