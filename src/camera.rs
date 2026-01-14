use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::CursorGrabMode;
use crate::player::{Player, PlayerMovementSettings};

/// Camera component that marks the main game camera
#[derive(Component)]
pub struct GameCamera {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

/// System to handle mouse input for camera rotation
pub fn camera_mouse_control_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut GameCamera>,
    settings: Res<PlayerMovementSettings>,
) {
    let mut camera = query.single_mut();
    
    // Process mouse motion events
    for event in mouse_motion_events.read() {
        // Apply mouse sensitivity
        camera.yaw -= event.delta.x * settings.mouse_sensitivity;
        camera.pitch -= event.delta.y * settings.mouse_sensitivity;
        
        // Limit pitch to avoid over-rotation
        camera.pitch = camera.pitch.clamp(-1.5, 1.5);
    }
}

/// System to update camera transform based on rotation
pub fn camera_rotation_system(
    player_query: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut camera_query: Query<(&mut Transform, &GameCamera), Without<Player>>, 
) {
    // Get player transform
    if let Ok(player_transform) = player_query.get_single() {
        // Get camera transform and rotation
        if let Ok((mut camera_transform, camera_rotation)) = camera_query.get_single_mut() {
            // Calculate camera rotation from yaw and pitch
            let yaw_rad = camera_rotation.yaw;
            let pitch_rad = camera_rotation.pitch;
            
            // Calculate camera position (first-person view, slightly above player)
            let camera_offset = Vec3::new(0.0, 1.7, 0.0); // Eye level offset
            let camera_position = player_transform.translation + camera_offset;
            
            // Apply rotation to camera
            camera_transform.translation = camera_position;
            camera_transform.rotation = Quat::from_euler(
                EulerRot::ZYX, 
                0.0, 
                yaw_rad, 
                pitch_rad
            );
        }
    }
}

/// System to spawn the game camera
pub fn spawn_game_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 0.0),
        GameCamera::default(),
    ));
}

/// System to handle cursor visibility and grabbing for automatic camera control
pub fn cursor_control_system(
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();
    
    // Always grab and hide cursor for automatic camera control
    // This ensures the cursor is always locked and hidden, providing automatic camera control
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
}
