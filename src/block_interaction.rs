// Block interaction system for Bevy Craft
// This module handles block breaking and placement

use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

use crate::block::BlockType;
use crate::chunk::{Chunk, ChunkManager, ChunkPosition};
use crate::inventory::{Inventory, ItemType};

/// Resource to track block breaking progress
#[derive(Resource, Default)]
pub struct BlockBreakingProgress {
    pub target_block_pos: Option<IVec3>,
    pub accumulated_damage: f32,
    pub is_breaking: bool,
}

/// System to handle block breaking with left mouse button
pub fn block_breaking_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Transform, &crate::camera::GameCamera)>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    chunk_manager: Res<ChunkManager>,
    mut inventory: ResMut<Inventory>,
    mut chunks: Query<&mut Chunk>,
    mut breaking_progress: ResMut<BlockBreakingProgress>,
    time: Res<Time>,
) {
    let (camera_transform, _camera) = if let Ok(result) = camera_query.get_single() {
        result
    } else {
        return;
    };

    let _player_transform = if let Ok(result) = player_query.get_single() {
        result
    } else {
        return;
    };

    let ray_origin = camera_transform.translation;
    let ray_direction: Vec3 = camera_transform.forward().into();

    // Offset ray origin slightly to avoid detecting the block the player/camera is inside
    let ray_origin = ray_origin + ray_direction * 0.5;

    // Handle block breaking with left mouse button
    if mouse_button_input.pressed(MouseButton::Left) {
        debug!("Left mouse button pressed for block breaking");
        // Perform raycast to find the block the player is looking at
        if let Some((target_block_pos, distance)) =
            raycast_for_block_mutable(ray_origin, ray_direction, &mut chunks, &chunk_manager, 5.0)
        {
            debug!(
                "Raycast hit block at {:?}, distance: {}",
                target_block_pos, distance
            );
            // Find which chunk contains this block
            let chunk_pos = ChunkPosition::from_block_position(target_block_pos);

            // Find the chunk entity and modify it
            if let Some(chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
                if let Ok(mut chunk) = chunks.get_mut(*chunk_entity) {
                    if let Some(current_block_type) = chunk.get_block_world(target_block_pos) {
                        if current_block_type != BlockType::Air {
                            if let Some(hardness) = current_block_type.hardness() {
                                // Check if this is the same block we were previously breaking
                                if breaking_progress.target_block_pos != Some(target_block_pos) {
                                    breaking_progress.target_block_pos = Some(target_block_pos);
                                    breaking_progress.accumulated_damage = 0.0;
                                }

                                breaking_progress.is_breaking = true;

                                // Accumulate damage based on delta time and hardness
                                let damage_per_second = 10.0 / hardness;
                                breaking_progress.accumulated_damage +=
                                    damage_per_second * time.delta_secs();

                                // Check if block should break
                                if breaking_progress.accumulated_damage >= 1.0 {
                                    // Remove the block (set to Air)
                                    chunk.set_block_world(target_block_pos, BlockType::Air);

                                    // Add the broken block to inventory
                                    inventory.add_item(ItemType::Block(current_block_type), 1);

                                    // Reset breaking progress
                                    breaking_progress.target_block_pos = None;
                                    breaking_progress.accumulated_damage = 0.0;
                                    breaking_progress.is_breaking = false;

                                    info!("Block broken at {:?}", target_block_pos);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Not looking at a breakable block
            breaking_progress.target_block_pos = None;
            breaking_progress.accumulated_damage = 0.0;
            breaking_progress.is_breaking = false;
        }
    } else {
        // Left mouse button not pressed - reset breaking progress
        breaking_progress.target_block_pos = None;
        breaking_progress.accumulated_damage = 0.0;
        breaking_progress.is_breaking = false;
    }
}

/// System to handle block placement with right mouse button
pub fn block_placement_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Transform, &crate::camera::GameCamera)>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    chunk_manager: Res<ChunkManager>,
    mut inventory: ResMut<Inventory>,
    mut chunks: Query<&mut Chunk>,
) {
    let (camera_transform, _camera) = if let Ok(result) = camera_query.get_single() {
        result
    } else {
        return;
    };

    let _player_transform = if let Ok(result) = player_query.get_single() {
        result
    } else {
        return;
    };

    let ray_origin = camera_transform.translation;
    let ray_direction: Vec3 = camera_transform.forward().into();

    // Offset ray origin slightly to avoid detecting the block the player/camera is inside
    let ray_origin = ray_origin + ray_direction * 0.5;

    // Handle block placement with right mouse button (on just pressed, not held)
    if mouse_button_input.just_pressed(MouseButton::Right) {
        debug!("Right mouse button pressed for block placement");

        // Get the currently selected item from hotbar
        if let Some(selected_item) = inventory.get_selected_item() {
            debug!(
                "Selected item: {:?}, quantity: {}",
                selected_item.item_type, selected_item.quantity
            );
            if !selected_item.is_empty() {
                // Check if the selected item is a block type
                if let ItemType::Block(block_type) = selected_item.item_type {
                    if block_type != BlockType::Air {
                        // Perform raycast to find the block the player is looking at
                        if let Some((target_block_pos, _)) = raycast_for_block_mutable(
                            ray_origin,
                            ray_direction,
                            &mut chunks,
                            &chunk_manager,
                            5.0,
                        ) {
                            // Calculate the adjacent block position for placement
                            let placement_pos = find_adjacent_block_position(
                                target_block_pos,
                                ray_origin,
                                ray_direction,
                            );

                            // Find which chunk contains this block
                            let chunk_pos = ChunkPosition::from_block_position(placement_pos);

                            // Find the chunk entity and modify it
                            if let Some(chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos)
                            {
                                if let Ok(mut chunk) = chunks.get_mut(*chunk_entity) {
                                    // Check if the placement position is empty (Air)
                                    if let Some(current_block) =
                                        chunk.get_block_world(placement_pos)
                                    {
                                        if current_block == BlockType::Air {
                                            // Place the block
                                            chunk.set_block_world(placement_pos, block_type);

                                            // Remove one block from inventory
                                            inventory.remove_item(ItemType::Block(block_type), 1);

                                            info!(
                                                "Block {:?} placed at {:?}",
                                                block_type, placement_pos
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Block placement is now part of the block_interaction_system above

/// Perform raycasting to find the first block intersected by a ray (immutable version)
/// Returns the position of the intersected block and the distance
fn raycast_for_block_immutable(
    ray_origin: Vec3,
    ray_direction: Vec3,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
    max_distance: f32,
) -> Option<(IVec3, f32)> {
    const STEP_SIZE: f32 = 0.1;
    #[allow(dead_code)]
    const BLOCK_SIZE: f32 = 1.0;

    let mut current_pos = ray_origin;
    let mut distance_traveled = 0.0;

    while distance_traveled < max_distance {
        // Convert current position to block coordinates
        let block_pos = IVec3::new(
            current_pos.x.floor() as i32,
            current_pos
                .y
                .floor()
                .clamp(0.0, crate::chunk::CHUNK_HEIGHT as f32 - 1.0) as i32,
            current_pos.z.floor() as i32,
        );

        // Check if this block position is within any loaded chunk
        let chunk_pos = ChunkPosition::from_block_position(block_pos);

        if let Some(chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
            if let Ok(chunk) = chunks.get(*chunk_entity) {
                // Check if this block is solid (not air)
                if let Some(block_type) = chunk.get_block_world(block_pos) {
                    if block_type != BlockType::Air {
                        // Found a solid block!
                        return Some((block_pos, distance_traveled));
                    }
                }
            }
        }

        // Step forward along the ray
        current_pos += ray_direction * STEP_SIZE;
        distance_traveled += STEP_SIZE;
    }

    None // No block found within max distance
}

/// Perform raycasting to find the first block intersected by a ray (mutable version)
/// Returns the position of the intersected block and the distance
fn raycast_for_block_mutable(
    ray_origin: Vec3,
    ray_direction: Vec3,
    chunks: &Query<&mut Chunk>,
    chunk_manager: &ChunkManager,
    max_distance: f32,
) -> Option<(IVec3, f32)> {
    const STEP_SIZE: f32 = 0.1;
    #[allow(dead_code)]
    const BLOCK_SIZE: f32 = 1.0;

    let mut current_pos = ray_origin;
    let mut distance_traveled = 0.0;

    while distance_traveled < max_distance {
        // Convert current position to block coordinates
        let block_pos = IVec3::new(
            current_pos.x.floor() as i32,
            current_pos
                .y
                .floor()
                .clamp(0.0, crate::chunk::CHUNK_HEIGHT as f32 - 1.0) as i32,
            current_pos.z.floor() as i32,
        );

        // Check if this block position is within any loaded chunk
        let chunk_pos = ChunkPosition::from_block_position(block_pos);

        if let Some(chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
            if let Ok(chunk) = chunks.get(*chunk_entity) {
                // Check if this block is solid (not air)
                if let Some(block_type) = chunk.get_block_world(block_pos) {
                    if block_type != BlockType::Air {
                        // Found a solid block!
                        return Some((block_pos, distance_traveled));
                    }
                }
            }
        }

        // Step forward along the ray
        current_pos += ray_direction * STEP_SIZE;
        distance_traveled += STEP_SIZE;
    }

    None // No block found within max distance
}

/// Find the adjacent block position for placement
/// Given a target block position and ray information, determine where to place a new block
fn find_adjacent_block_position(
    target_block_pos: IVec3,
    _ray_origin: Vec3,
    ray_direction: Vec3,
) -> IVec3 {
    // Convert target block position to Vec3 for calculations
    let _target_block_center = target_block_pos.as_vec3() + Vec3::splat(0.5);

    // Determine which face of the target block is being looked at
    // by finding the component with the largest absolute value
    let abs_direction = ray_direction.abs();

    // Find which axis has the largest component (this is the face we're looking at)
    if abs_direction.x >= abs_direction.y && abs_direction.x >= abs_direction.z {
        // Looking at X face (either positive or negative)
        if ray_direction.x > 0.0 {
            // Looking at negative X face, place on positive X side
            target_block_pos + IVec3::new(1, 0, 0)
        } else {
            // Looking at positive X face, place on negative X side
            target_block_pos + IVec3::new(-1, 0, 0)
        }
    } else if abs_direction.y >= abs_direction.x && abs_direction.y >= abs_direction.z {
        // Looking at Y face
        if ray_direction.y > 0.0 {
            // Looking at negative Y face, place on positive Y side
            target_block_pos + IVec3::new(0, 1, 0)
        } else {
            // Looking at positive Y face, place on negative Y side
            target_block_pos + IVec3::new(0, -1, 0)
        }
    } else {
        // Looking at Z face
        if ray_direction.z > 0.0 {
            // Looking at negative Z face, place on positive Z side
            target_block_pos + IVec3::new(0, 0, 1)
        } else {
            // Looking at positive Z face, place on negative Z side
            target_block_pos + IVec3::new(0, 0, -1)
        }
    }
}

/// System to provide visual feedback for targeted blocks
pub fn block_targeting_feedback_system(
    camera_query: Query<(&Transform, &crate::camera::GameCamera)>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    chunks: Query<&Chunk>,
    chunk_manager: Res<ChunkManager>,
) {
    // Get camera and player transforms
    let (camera_transform, _) = if let Ok(result) = camera_query.get_single() {
        result
    } else {
        return;
    };

    let _player_transform = if let Ok(result) = player_query.get_single() {
        result
    } else {
        return;
    };

    // Calculate ray origin (camera position) and direction
    let ray_origin = camera_transform.translation;
    let ray_direction: Vec3 = camera_transform.forward().into();

    // Perform raycast to find the block the player is looking at
    if let Some((_target_block_pos, _distance)) =
        raycast_for_block_immutable(ray_origin, ray_direction, &chunks, &chunk_manager, 5.0)
    {
        // TODO: Add visual highlight for targeted block
    }
}
