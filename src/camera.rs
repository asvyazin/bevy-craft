use bevy::prelude::*;
use crate::player::Player;

/// Camera component that marks the main game camera
#[derive(Component)]
pub struct GameCamera;

/// System to make the camera follow the player
pub fn camera_follow_system(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    // Get player transform
    if let Ok(player_transform) = player_query.get_single() {
        // Get camera transform
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            // Calculate desired camera position (behind and above the player)
            let offset = Vec3::new(0.0, 2.0, 5.0);
            let desired_position = player_transform.translation + offset;
            
            // Smoothly interpolate camera position
            camera_transform.translation = camera_transform.translation.lerp(desired_position, 0.1);
            
            // Make camera look at player
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}

/// System to spawn the game camera
pub fn spawn_game_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GameCamera,
    ));
}