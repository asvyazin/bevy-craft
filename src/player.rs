use bevy::prelude::*;
use bevy::math::primitives::Cuboid;

/// Player component representing the player character
#[derive(Component, Debug)]
pub struct Player {
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
    pub velocity: Vec3,
    pub gravity: f32,
}

impl Player {
    pub fn new(spawn_position: Vec3) -> impl Bundle {
        (
            Player {
                speed: 5.0,
                jump_force: 7.0,
                is_grounded: false,
                velocity: Vec3::ZERO,
                gravity: 20.0,
            },
            TransformBundle::from_transform(Transform::from_translation(spawn_position)),
        )
    }
}

/// Settings for player movement
#[derive(Resource, Debug)]
pub struct PlayerMovementSettings {
    pub move_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub mouse_sensitivity: f32,
}

impl Default for PlayerMovementSettings {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            jump_force: 7.0,
            gravity: 20.0,
            mouse_sensitivity: 0.001,
        }
    }
}

/// System to spawn the player with proper mesh and materials
pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create player mesh
    let player_mesh = meshes.add(Mesh::from(Cuboid { half_size: Vec3::new(0.3, 0.8, 0.3) }));
    let player_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.1, 0.5, 0.9),
        ..default()
    });

    // Spawn the player
    commands.spawn(Player::new(Vec3::new(0.0, 3.0, 0.0)))
        .insert(PbrBundle {
            mesh: player_mesh,
            material: player_material,
            ..default()
        });
}

/// System for handling player movement with keyboard controls
pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player)>,
    camera_query: Query<&crate::camera::GameCamera>,
    time: Res<Time>,
) {
    // Get camera rotation for movement direction
    let camera_rotation = if let Ok(camera) = camera_query.get_single() {
        camera
    } else {
        return; // No camera, can't determine movement direction
    };
    
    for (mut transform, mut player) in &mut query {
        // Reset horizontal velocity
        player.velocity.x = 0.0;
        player.velocity.z = 0.0;

        // Handle movement input
        let mut move_direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            move_direction.z += 1.0;  // Fixed: W now moves forward (positive Z)
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            move_direction.z -= 1.0;  // Fixed: S now moves backward (negative Z)
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            move_direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            move_direction.x += 1.0;
        }

        // Handle jump
        if keyboard_input.just_pressed(KeyCode::Space) && player.is_grounded {
            player.velocity.y = player.jump_force;
            player.is_grounded = false;
        }

        // Normalize movement direction and apply speed
        if move_direction.length() > 0.0 {
            move_direction = move_direction.normalize();
            
            // Store speed in local variable to avoid borrowing issues
            let speed = player.speed;
            
            // Rotate movement direction based on camera yaw
            let yaw_rad = camera_rotation.yaw;
            let forward = Vec3::new(-yaw_rad.sin(), 0.0, -yaw_rad.cos());
            let right = Vec3::new(yaw_rad.cos(), 0.0, -yaw_rad.sin());
            
            // Apply movement relative to camera direction
            player.velocity += forward * move_direction.z * speed;
            player.velocity += right * move_direction.x * speed;
        }

        // Apply gravity
        if !player.is_grounded {
            player.velocity.y -= player.gravity * time.delta_seconds();
        }

        // Apply velocity to position
        transform.translation += player.velocity * time.delta_seconds();

        // Simple ground detection (for now, just check if we're below a certain height)
        if transform.translation.y <= 0.8 { // Player height is 1.6, so 0.8 is close to ground
            transform.translation.y = 0.8;
            player.velocity.y = 0.0;
            player.is_grounded = true;
        }
    }
}