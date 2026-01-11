// Block interaction system for Bevy Craft
// This module handles block breaking and placement

use bevy::prelude::*;
use bevy::input::mouse::MouseButton;

use crate::chunk::{Chunk, ChunkManager, ChunkPosition};
use crate::block::BlockType;

/// System to handle block breaking with left mouse button
pub fn block_breaking_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Transform, &crate::camera::GameCamera)>, 
    player_query: Query<&Transform, With<crate::player::Player>>,
    mut chunks: Query<&mut Chunk>,
    chunk_manager: Res<ChunkManager>,
) {
    // Only process when left mouse button is pressed
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    // Get camera and player transforms
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
    
    // Calculate ray origin (camera position) and direction
    let ray_origin = camera_transform.translation;
    let ray_direction: Vec3 = camera_transform.forward().into();
    
    // Perform raycast to find the block the player is looking at
    if let Some((target_block_pos, _)) = raycast_for_block_mutable(ray_origin, ray_direction, &mut chunks, &chunk_manager, 5.0) {
        println!("🎯 Targeting block at: {:?}", target_block_pos);
        
        // Find which chunk contains this block
        let chunk_pos = ChunkPosition::from_block_position(target_block_pos);
        
        // Find the chunk entity and modify it
        if let Some(chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
            if let Ok(mut chunk) = chunks.get_mut(*chunk_entity) {
                // Remove the block (set to Air)
                chunk.set_block_world(target_block_pos, BlockType::Air);
                println!("✅ Block broken at: {:?}", target_block_pos);
            }
        }
    }
}

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
    const BLOCK_SIZE: f32 = 1.0;
    
    let mut current_pos = ray_origin;
    let mut distance_traveled = 0.0;
    
    while distance_traveled < max_distance {
        // Convert current position to block coordinates
        let block_pos = IVec3::new(
            current_pos.x.floor() as i32,
            current_pos.y.floor() as i32,
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
    const BLOCK_SIZE: f32 = 1.0;
    
    let mut current_pos = ray_origin;
    let mut distance_traveled = 0.0;
    
    while distance_traveled < max_distance {
        // Convert current position to block coordinates
        let block_pos = IVec3::new(
            current_pos.x.floor() as i32,
            current_pos.y.floor() as i32,
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
    if let Some((target_block_pos, distance)) = raycast_for_block_immutable(ray_origin, ray_direction, &chunks, &chunk_manager, 5.0) {
        println!("👀 Looking at block at: {:?} (distance: {:.2})", target_block_pos, distance);
        // TODO: Add visual highlight for targeted block
    }
}