use crate::collision::CollisionState;
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

/// Player component representing the player character
#[derive(Component, Debug)]
pub struct Player {
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
    pub velocity: Vec3,
    pub gravity: f32,
    pub health: f32,
    pub max_health: f32,
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
                health: 100.0,
                max_health: 100.0,
            },
            CollisionState::default(),
            Transform::from_translation(spawn_position),
        )
    }

    pub fn take_damage(&mut self, amount: f32) -> f32 {
        let old_health = self.health;
        self.health = (self.health - amount).max(0.0);
        old_health - self.health
    }

    pub fn heal(&mut self, amount: f32) -> f32 {
        let old_health = self.health;
        self.health = (self.health + amount).min(self.max_health);
        self.health - old_health
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }
}

/// Settings for player movement
#[derive(Resource, Debug)]
pub struct PlayerMovementSettings {
    #[allow(dead_code)]
    pub move_speed: f32,
    #[allow(dead_code)]
    pub jump_force: f32,
    #[allow(dead_code)]
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
#[allow(dead_code)]
pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create player mesh
    let player_mesh = meshes.add(Mesh::from(Cuboid {
        half_size: Vec3::new(0.3, 0.8, 0.3),
    }));
    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.5, 0.9),
        ..default()
    });

    // Spawn the player
    commands
        .spawn(Player::new(Vec3::new(0.0, 3.0, 0.0)))
        .insert((MeshMaterial3d(player_material), Mesh3d(player_mesh)));
}

/// Event sent when player dies
#[derive(Event, Debug)]
pub struct PlayerDeathEvent;

/// Event sent when player takes damage
#[derive(Event, Debug)]
pub struct PlayerDamageEvent {
    pub amount: f32,
    pub new_health: f32,
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
            move_direction.z += 1.0; // Fixed: W now moves forward (positive Z)
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            move_direction.z -= 1.0; // Fixed: S now moves backward (negative Z)
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
            player.velocity.y -= player.gravity * time.delta_secs();
        } else {
            // When grounded, ensure we don't have any downward velocity
            // that could cause penetration into the ground
            if player.velocity.y < 0.0 {
                player.velocity.y = 0.0;
            }
        }

        // Clamp vertical velocity to prevent micro-oscillations when grounded
        if player.is_grounded && player.velocity.y.abs() < 0.1 {
            player.velocity.y = 0.0;
        }

        // Apply velocity to position (collision system will handle actual positioning)
        transform.translation += player.velocity * time.delta_secs();

        // Simple ground detection is now handled by collision system
        // The collision system will prevent the player from falling through the ground
    }
}

/// System to handle player death
pub fn player_death_system(
    mut query: Query<&mut Transform, With<Player>>,
    mut death_events: EventReader<PlayerDeathEvent>,
) {
    for _ in death_events.read() {
        for mut transform in &mut query {
            // Move player to spawn position (temporary solution)
            transform.translation = Vec3::new(0.0, 20.0, 0.0);
            info!("💀 Player died! Respawning...");
        }
    }
}

/// System to take damage from events
pub fn player_take_damage_system(
    mut query: Query<&mut Player>,
    mut damage_events: EventWriter<PlayerDamageEvent>,
    mut death_events: EventWriter<PlayerDeathEvent>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for mut player in &mut query {
        // Check for fall damage (simple implementation)
        if player.velocity.y < -15.0 && player.is_grounded {
            let fall_damage = (player.velocity.y.abs() - 15.0) * 2.0;
            let actual_damage = player.take_damage(fall_damage);

            if actual_damage > 0.0 {
                damage_events.send(PlayerDamageEvent {
                    amount: actual_damage,
                    new_health: player.health,
                });
                info!(
                    "🩸 Player took {:.1} fall damage! Health: {:.1}",
                    actual_damage, player.health
                );
            }

            if !player.is_alive() {
                death_events.send(PlayerDeathEvent);
            }
        }

        // Test damage with H key
        if keyboard.just_pressed(KeyCode::KeyH) {
            let test_damage = player.take_damage(10.0);
            damage_events.send(PlayerDamageEvent {
                amount: test_damage,
                new_health: player.health,
            });
            info!(
                "🩸 Test: Player took {:.1} damage! Health: {:.1}",
                test_damage, player.health
            );

            if !player.is_alive() {
                death_events.send(PlayerDeathEvent);
            }
        }

        // Test heal with R key
        if keyboard.just_pressed(KeyCode::KeyR) {
            let test_heal = player.heal(20.0);
            damage_events.send(PlayerDamageEvent {
                amount: -test_heal,
                new_health: player.health,
            });
            info!(
                "💚 Test: Player healed for {:.1}! Health: {:.1}",
                test_heal, player.health
            );
        }
    }
}

/// System to handle damage events
pub fn handle_damage_events(mut damage_events: EventReader<PlayerDamageEvent>) {
    for event in damage_events.read() {
        info!(
            "💔 Player took {:.1} damage! Health: {:.1}",
            event.amount, event.new_health
        );
    }
}
