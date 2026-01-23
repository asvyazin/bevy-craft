use crate::chunk::Chunk;
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
    pub hunger: f32,
    pub max_hunger: f32,
    pub thirst: f32,
    pub max_thirst: f32,
    pub max_fall_velocity: f32,
    pub was_grounded: bool,
    pub oxygen: f32,
    pub max_oxygen: f32,
    pub is_underwater: bool,
    pub is_in_lava: bool,
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
                hunger: 100.0,
                max_hunger: 100.0,
                thirst: 100.0,
                max_thirst: 100.0,
                max_fall_velocity: 0.0,
                was_grounded: false,
                oxygen: 100.0,
                max_oxygen: 100.0,
                is_underwater: false,
                is_in_lava: false,
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

    pub fn reduce_hunger(&mut self, amount: f32) {
        self.hunger = (self.hunger - amount).max(0.0);
    }

    pub fn restore_hunger(&mut self, amount: f32) {
        self.hunger = (self.hunger + amount).min(self.max_hunger);
    }

    pub fn reduce_thirst(&mut self, amount: f32) {
        self.thirst = (self.thirst - amount).max(0.0);
    }

    pub fn restore_thirst(&mut self, amount: f32) {
        self.thirst = (self.thirst + amount).min(self.max_thirst);
    }

    pub fn is_starving(&self) -> bool {
        self.hunger <= 0.0
    }

    pub fn is_dehydrated(&self) -> bool {
        self.thirst <= 0.0
    }

    pub fn reduce_oxygen(&mut self, amount: f32) {
        self.oxygen = (self.oxygen - amount).max(0.0);
    }

    pub fn restore_oxygen(&mut self, amount: f32) {
        self.oxygen = (self.oxygen + amount).min(self.max_oxygen);
    }

    pub fn is_drowning(&self) -> bool {
        self.oxygen <= 0.0
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

/// System to detect when player is underwater
pub fn drowning_detection_system(
    mut player_query: Query<&mut Player, (With<crate::camera::GameCamera>)>,
    camera_query: Query<&Transform, With<crate::camera::GameCamera>>,
    chunks: Query<&Chunk>,
    chunk_manager: Res<crate::chunk::ChunkManager>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let player_position = camera_transform.translation;

        let block_pos = crate::chunk::ChunkPosition::from_block_position(IVec3::new(
            player_position.x as i32,
            player_position.y as i32,
            player_position.z as i32,
        ));

        if let Some(chunk_entity) = chunk_manager.loaded_chunks.get(&block_pos) {
            if let Ok(chunk) = chunks.get(*chunk_entity) {
                if let Some(block_type) = chunk.get_block_world(IVec3::new(
                    player_position.x as i32,
                    (player_position.y + 1.6) as i32,
                    player_position.z as i32,
                )) {
                    for mut player in &mut player_query {
                        player.is_underwater = block_type == crate::block::BlockType::Water;
                    }
                }
            }
        }
    }
}

/// System to handle oxygen consumption and drowning damage
pub fn drowning_damage_system(
    mut query: Query<&mut Player>,
    time: Res<Time>,
    mut damage_events: EventWriter<PlayerDamageEvent>,
    mut death_events: EventWriter<PlayerDeathEvent>,
) {
    for mut player in &mut query {
        if player.is_underwater {
            // Consume oxygen over time (2.0 oxygen per second)
            let oxygen_decay = 2.0 * time.delta_secs();
            player.reduce_oxygen(oxygen_decay);
        } else if player.oxygen < player.max_oxygen {
            // Restore oxygen gradually when above water
            let oxygen_restore = 10.0 * time.delta_secs();
            player.restore_oxygen(oxygen_restore);
        }

        // Take damage when drowning (oxygen <= 0)
        if player.is_drowning() {
            let drowning_damage = 3.0 * time.delta_secs();
            let actual_damage = player.take_damage(drowning_damage);

            if actual_damage > 0.0 {
                damage_events.send(PlayerDamageEvent {
                    amount: actual_damage,
                    new_health: player.health,
                });
            }

            if !player.is_alive() {
                death_events.send(PlayerDeathEvent);
            }
        }
    }
}

/// System to detect when player is in lava
pub fn fire_damage_detection_system(
    mut player_query: Query<&mut Player, With<crate::camera::GameCamera>>,
    camera_query: Query<&Transform, With<crate::camera::GameCamera>>,
    chunks: Query<&Chunk>,
    chunk_manager: Res<crate::chunk::ChunkManager>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let player_position = camera_transform.translation;

        let block_pos = crate::chunk::ChunkPosition::from_block_position(IVec3::new(
            player_position.x as i32,
            player_position.y as i32,
            player_position.z as i32,
        ));

        if let Some(chunk_entity) = chunk_manager.loaded_chunks.get(&block_pos) {
            if let Ok(chunk) = chunks.get(*chunk_entity) {
                if let Some(block_type) = chunk.get_block_world(IVec3::new(
                    player_position.x as i32,
                    (player_position.y + 1.6) as i32,
                    player_position.z as i32,
                )) {
                    for mut player in &mut player_query {
                        player.is_in_lava = block_type == crate::block::BlockType::Lava;
                    }
                }
            }
        }
    }
}

/// System to handle fire damage
pub fn fire_damage_system(
    mut query: Query<&mut Player>,
    time: Res<Time>,
    mut damage_events: EventWriter<PlayerDamageEvent>,
    mut death_events: EventWriter<PlayerDeathEvent>,
) {
    for mut player in &mut query {
        // Take fire damage when in lava
        if player.is_in_lava {
            let fire_damage = 5.0 * time.delta_secs();
            let actual_damage = player.take_damage(fire_damage);

            if actual_damage > 0.0 {
                damage_events.send(PlayerDamageEvent {
                    amount: actual_damage,
                    new_health: player.health,
                });
            }

            if !player.is_alive() {
                death_events.send(PlayerDeathEvent);
            }
        }
    }
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

/// Settings for player health regeneration
#[derive(Resource, Debug)]
pub struct HealthRegenerationSettings {
    pub hunger_threshold: f32,
    pub regeneration_rate: f32,
    pub regeneration_interval: f32,
}

impl Default for HealthRegenerationSettings {
    fn default() -> Self {
        Self {
            hunger_threshold: 90.0,
            regeneration_rate: 1.0,
            regeneration_interval: 4.0,
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
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for mut player in &mut query {
        // Check for fall damage when player lands
        if player.was_grounded == false && player.is_grounded == true {
            // Player just landed
            if player.max_fall_velocity < -10.0 {
                let fall_damage = (player.max_fall_velocity.abs() - 10.0) * 2.0;
                let actual_damage = player.take_damage(fall_damage);

                if actual_damage > 0.0 {
                    damage_events.send(PlayerDamageEvent {
                        amount: actual_damage,
                        new_health: player.health,
                    });
                    info!(
                        "🩸 Player fell at {:.1} m/s and took {:.1} damage! Health: {:.1}",
                        player.max_fall_velocity.abs(),
                        actual_damage,
                        player.health
                    );
                }

                if !player.is_alive() {
                    death_events.send(PlayerDeathEvent);
                }
            }
            // Reset max fall velocity after landing
            player.max_fall_velocity = 0.0;
        }

        // Track max fall velocity while in air
        if !player.is_grounded && player.velocity.y < player.max_fall_velocity {
            player.max_fall_velocity = player.velocity.y;
        }

        // Update was_grounded for next frame
        player.was_grounded = player.is_grounded;

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

/// System to handle hunger and thirst decay over time
pub fn hunger_thirst_decay_system(
    mut query: Query<&mut Player>,
    time: Res<Time>,
    mut damage_events: EventWriter<PlayerDamageEvent>,
    mut death_events: EventWriter<PlayerDeathEvent>,
) {
    for mut player in &mut query {
        // Decay hunger over time (0.5 hunger per second, or 30 per minute)
        let hunger_decay = 0.5 * time.delta_secs();
        player.reduce_hunger(hunger_decay);

        // Decay thirst over time (0.7 thirst per second, or 42 per minute)
        let thirst_decay = 0.7 * time.delta_secs();
        player.reduce_thirst(thirst_decay);

        // Take damage when starving (hunger <= 0)
        if player.is_starving() {
            let starvation_damage = 1.0 * time.delta_secs();
            let actual_damage = player.take_damage(starvation_damage);

            if actual_damage > 0.0 {
                damage_events.send(PlayerDamageEvent {
                    amount: actual_damage,
                    new_health: player.health,
                });
            }

            if !player.is_alive() {
                death_events.send(PlayerDeathEvent);
            }
        }

        // Take damage when dehydrated (thirst <= 0)
        if player.is_dehydrated() {
            let dehydration_damage = 2.0 * time.delta_secs();
            let actual_damage = player.take_damage(dehydration_damage);

            if actual_damage > 0.0 {
                damage_events.send(PlayerDamageEvent {
                    amount: actual_damage,
                    new_health: player.health,
                });
            }

            if !player.is_alive() {
                death_events.send(PlayerDeathEvent);
            }
        }
    }
}

/// Event sent when player consumes food
#[derive(Event, Debug)]
pub struct FoodConsumedEvent {
    pub food_type: crate::inventory::FoodType,
    pub hunger_restored: f32,
    pub thirst_restored: f32,
}

/// System to handle food consumption
pub fn food_consumption_system(
    mut query: Query<&mut Player>,
    mut inventory: ResMut<crate::inventory::Inventory>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut food_consumed_events: EventWriter<FoodConsumedEvent>,
) {
    // Check for right mouse button click (consume food)
    if mouse_input.just_pressed(MouseButton::Right) {
        // Get the currently selected item from the hotbar
        if let Some(selected_item) = inventory.get_selected_item() {
            if !selected_item.is_empty() {
                // Check if the selected item is food
                if let crate::inventory::ItemType::Food(food_type) = selected_item.item_type {
                    // Apply food effects to player
                    for mut player in &mut query {
                        let hunger_restore = food_type.hunger_restore();
                        let thirst_restore = food_type.thirst_restore();

                        player.restore_hunger(hunger_restore);
                        player.restore_thirst(thirst_restore);

                        info!(
                            "🍎 Consumed {}! Hunger: {:.1}, Thirst: {:.1}",
                            food_type.name(),
                            player.hunger,
                            player.thirst
                        );

                        // Send event
                        food_consumed_events.send(FoodConsumedEvent {
                            food_type,
                            hunger_restored: hunger_restore,
                            thirst_restored: thirst_restore,
                        });
                    }

                    // Remove the consumed item from inventory
                    inventory.remove_item(crate::inventory::ItemType::Food(food_type), 1);
                }
            }
        }
    }
}

/// System to display hunger and thirst status
pub fn display_hunger_thirst_status(player_query: Query<&Player>, time: Res<Time>) {
    // Only display status every 5 seconds to reduce spam
    if time.elapsed_secs_f64() % 5.0 < 0.1 {
        if let Ok(player) = player_query.get_single() {
            info!(
                "📊 Status - Health: {:.1}/{:.1} | Hunger: {:.1}/{:.1} | Thirst: {:.1}/{:.1} | Oxygen: {:.1}/{:.1}",
                player.health,
                player.max_health,
                player.hunger,
                player.max_hunger,
                player.thirst,
                player.max_thirst,
                player.oxygen,
                player.max_oxygen
            );
        }
    }
}

/// System to regenerate health based on hunger level
pub fn health_regeneration_system(
    mut query: Query<&mut Player>,
    time: Res<Time>,
    settings: Res<HealthRegenerationSettings>,
    mut damage_events: EventWriter<PlayerDamageEvent>,
) {
    for mut player in &mut query {
        // Only regenerate if player is alive
        if !player.is_alive() {
            continue;
        }

        // Only regenerate if hunger is above threshold
        let hunger_percent = (player.hunger / player.max_hunger) * 100.0;
        if hunger_percent >= settings.hunger_threshold {
            // Check if it's time to regenerate (based on interval)
            let elapsed = time.elapsed_secs();
            if elapsed % settings.regeneration_interval < time.delta_secs() {
                // Regenerate health
                let health_restored = player.heal(settings.regeneration_rate);

                if health_restored > 0.0 {
                    // Send damage event with negative amount to indicate healing
                    damage_events.send(PlayerDamageEvent {
                        amount: -health_restored,
                        new_health: player.health,
                    });
                    info!(
                        "💚 Health regenerated: {:.1} (Hunger: {:.1}%)",
                        health_restored, hunger_percent
                    );
                }
            }
        }
    }
}
