use bevy::prelude::*;

use crate::block::Block;
use crate::chunk::{Chunk, ChunkManager, ChunkPosition};

/// Constants for collision detection
const COLLISION_EPSILON: f32 = 0.001;
const GROUND_DETECTION_EPSILON: f32 = 0.01;
const GROUND_HYSTERESIS_THRESHOLD: f32 = 1.0; // Increased threshold for better stability
const GROUND_HYSTERESIS_INCREMENT: f32 = 0.2; // Larger increment for faster response
const GROUNDED_STABILITY_BUFFER: f32 = 0.01; // Small buffer to prevent micro-adjustments when grounded

/// Component to track collision information for entities
#[derive(Component, Debug)]
pub struct Collider {
    pub size: Vec3,
    pub offset: Vec3,
}

impl Collider {
    pub fn new(size: Vec3, offset: Vec3) -> Self {
        Self { size, offset }
    }
    
    /// Create a collider for the player
    pub fn player() -> Self {
        Self {
            size: Vec3::new(0.6, 1.8, 0.6), // Player size (width, height, depth)
            offset: Vec3::new(0.0, 0.9, 0.0), // Offset from center to bottom
        }
    }
}

/// System to check for collisions between entities and blocks
pub fn collision_detection_system(
    mut query: Query<(&mut Transform, &Collider, Option<&mut crate::player::Player>)>, 
    blocks: Query<&Block>,
    chunks: Query<&Chunk>,
    chunk_manager: Res<ChunkManager>,
) {
    for (mut transform, collider, mut player) in &mut query {
        let entity_position = transform.translation;
        let entity_aabb = get_entity_aabb(entity_position, collider);
        
        // Check collision with blocks
        let collision_result = check_block_collisions(entity_aabb, &blocks);
        
        if let Some(resolved_position) = collision_result {
            // If player is grounded, be more conservative about vertical collision resolution
            if let Some(ref player) = player.as_ref() {
                if player.is_grounded {
                    // Only allow upward collision resolution when grounded (prevent micro-adjustments downward)
                    // Also add a small stability buffer to prevent micro-adjustments
                    if resolved_position.y >= entity_position.y + GROUNDED_STABILITY_BUFFER {
                        println!("⚡ Block collision resolved: {:?} -> {:?}", entity_position, resolved_position);
                        transform.translation = resolved_position;
                    }
                } else {
                    println!("⚡ Block collision resolved: {:?} -> {:?}", entity_position, resolved_position);
                    transform.translation = resolved_position;
                }
            } else {
                println!("⚡ Block collision resolved: {:?} -> {:?}", entity_position, resolved_position);
                transform.translation = resolved_position;
            }
        }
        
        // Check collision with chunks (for chunk-based worlds)
        let chunk_collision_result = check_chunk_collisions(entity_aabb, &chunks, &chunk_manager);
        
        if let Some(resolved_position) = chunk_collision_result {
            // If player is grounded, be more conservative about vertical collision resolution
            if let Some(ref player) = player.as_ref() {
                if player.is_grounded {
                    // Only allow upward collision resolution when grounded (prevent micro-adjustments downward)
                    // Also add a small stability buffer to prevent micro-adjustments
                    if resolved_position.y >= entity_position.y + GROUNDED_STABILITY_BUFFER {
                        println!("⚡ Chunk collision resolved: {:?} -> {:?}", entity_position, resolved_position);
                        transform.translation = resolved_position;
                    }
                } else {
                    println!("⚡ Chunk collision resolved: {:?} -> {:?}", entity_position, resolved_position);
                    transform.translation = resolved_position;
                }
            } else {
                println!("⚡ Chunk collision resolved: {:?} -> {:?}", entity_position, resolved_position);
                transform.translation = resolved_position;
            }
        }
        
        // Ground detection for player
        if let Some(ref mut player) = player.as_mut() {
            let ground_check_position = entity_position - Vec3::new(0.0, 0.1, 0.0); // Check slightly below player
            let ground_check_aabb = (
                Vec3::new(ground_check_position.x - 0.2, ground_check_position.y - 0.1, ground_check_position.z - 0.2),
                Vec3::new(ground_check_position.x + 0.2, ground_check_position.y + 0.1, ground_check_position.z + 0.2)
            );
            
            // Check if there's ground below the player
            let ground_detected = check_ground_collision(ground_check_aabb, &blocks, &chunks, &chunk_manager);
            
            // Additional stability check: if player is very close to ground, consider them grounded
            // This prevents micro-movements from causing ground state toggling
            let distance_to_ground = if ground_detected { 0.0 } else { 1.0 }; // Simple approximation
            
            // Apply hysteresis to prevent rapid toggling of grounded state
            if ground_detected || distance_to_ground < 0.2 {
                // If we detect ground or are very close to it, reduce hysteresis more aggressively
                player.ground_detection_hysteresis = (player.ground_detection_hysteresis - GROUND_HYSTERESIS_INCREMENT).max(0.0);
                
                if !player.is_grounded && player.ground_detection_hysteresis == 0.0 {
                    println!("👣 Player is now grounded");
                    player.is_grounded = true;
                }
            } else {
                // If no ground detected and we're not close to ground, increase hysteresis
                player.ground_detection_hysteresis = (player.ground_detection_hysteresis + GROUND_HYSTERESIS_INCREMENT).min(GROUND_HYSTERESIS_THRESHOLD);
                
                if player.is_grounded && player.ground_detection_hysteresis >= GROUND_HYSTERESIS_THRESHOLD {
                    println!("👣 Player is no longer grounded");
                    player.is_grounded = false;
                }
            }
        }
        
        // Debug: Check if we're falling and should be hitting something
        if entity_position.y < 0.0 {
            println!("⚠️  Player falling through! Position: {:?}, AABB: {:?} - {:?}", 
                    entity_position, entity_aabb.0, entity_aabb.1);
            
            // Check what blocks are around
            let chunk_pos = ChunkPosition::from_block_position(entity_position.as_ivec3());
            println!("🔍 Checking chunk at {:?}", chunk_pos);
            
            if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
                if let Ok(chunk) = chunks.get(chunk_entity) {
                    println!("🔍 Chunk found, checking blocks around player...");
                    
                    // Check a few blocks below the player
                    for y in (entity_position.y as i32 - 2)..=(entity_position.y as i32 + 1) {
                        let test_pos = IVec3::new(entity_position.x as i32, y, entity_position.z as i32);
                        if let Some(block_type) = chunk.data.get_block(
                            (test_pos.x.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize,
                            test_pos.y as usize,
                            (test_pos.z.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize
                        ) {
                            if block_type.is_solid() {
                                println!("🔍 Found solid block at {:?}: {:?}", test_pos, block_type);
                            }
                        }
                    }
                }
            }
        }
        
        // Additional debug: Check if player is at very low Y position but should be on ground
        if entity_position.y < 1.0 && entity_position.y > -1.0 {
            println!("🔍 Player at low Y position: {:?}, checking for ground support", entity_position);
            
            // Check if there should be ground below
            let ground_check_position = entity_position - Vec3::new(0.0, 1.0, 0.0);
            let ground_check_aabb = (
                Vec3::new(ground_check_position.x - 0.5, ground_check_position.y - 0.1, ground_check_position.z - 0.5),
                Vec3::new(ground_check_position.x + 0.5, ground_check_position.y + 0.1, ground_check_position.z + 0.5)
            );
            
            let ground_detected = check_ground_collision(ground_check_aabb, &blocks, &chunks, &chunk_manager);
            if ground_detected {
                println!("🔍 Ground detected below player, but collision resolution failed!");
            } else {
                println!("🔍 No ground detected below player");
            }
        }
    }
}

/// Get the axis-aligned bounding box for an entity
fn get_entity_aabb(position: Vec3, collider: &Collider) -> (Vec3, Vec3) {
    let half_size = collider.size / 2.0;
    let min = position - half_size + collider.offset;
    let max = position + half_size + collider.offset;
    (min, max)
}

/// Check for collisions with individual blocks
fn check_block_collisions(entity_aabb: (Vec3, Vec3), blocks: &Query<&Block>) -> Option<Vec3> {
    let (entity_min, entity_max) = entity_aabb;
    
    for block in blocks.iter() {
        if block.block_type.is_solid() {
            let block_min = block.position.as_vec3();
            let block_max = block.position.as_vec3() + Vec3::ONE;
            
            if check_aabb_collision(entity_min, entity_max, block_min, block_max) {
                // Resolve collision by moving entity out of the block
                if let Some(resolved_pos) = resolve_aabb_collision(entity_min, entity_max, block_min, block_max) {
                    return Some(resolved_pos);
                }
            }
        }
    }
    
    None
}

/// Check for collisions with chunk-based world
fn check_chunk_collisions(
    entity_aabb: (Vec3, Vec3), 
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> Option<Vec3> {
    let (entity_min, entity_max) = entity_aabb;
    
    // Convert entity position to chunk coordinates
    let entity_center = (entity_min + entity_max) / 2.0;
    let chunk_pos = ChunkPosition::from_block_position(entity_center.as_ivec3());
    
    // Check the chunk where the entity is located
    if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
        if let Ok(chunk) = chunks.get(chunk_entity) {
            if let Some(resolved_pos) = check_chunk_block_collisions(entity_aabb, chunk) {
                return Some(resolved_pos);
            }
        }
    }
    
    // Also check neighboring chunks
    for neighbor_pos in chunk_pos.all_neighbors() {
        if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&neighbor_pos) {
            if let Ok(chunk) = chunks.get(chunk_entity) {
                if let Some(resolved_pos) = check_chunk_block_collisions(entity_aabb, chunk) {
                    return Some(resolved_pos);
                }
            }
        }
    }
    
    None
}

/// Check if there's ground below the player
fn check_ground_collision(
    ground_aabb: (Vec3, Vec3),
    blocks: &Query<&Block>,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> bool {
    let (ground_min, ground_max) = ground_aabb;
    
    // Check individual blocks first
    for block in blocks.iter() {
        if block.block_type.is_solid() {
            let block_min = block.position.as_vec3();
            let block_max = block.position.as_vec3() + Vec3::ONE;
            
            if check_aabb_collision(ground_min, ground_max, block_min, block_max) {
                // Check if the collision is significant enough (not just a micro-collision)
                let penetration_y = block_max.y - ground_min.y;
                if penetration_y > GROUND_DETECTION_EPSILON {
                    return true;
                }
            }
        }
    }
    
    // Check chunks
    let ground_center = (ground_min + ground_max) / 2.0;
    let chunk_pos = ChunkPosition::from_block_position(ground_center.as_ivec3());
    
    if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
        if let Ok(chunk) = chunks.get(chunk_entity) {
            if check_chunk_ground_collision(ground_aabb, chunk) {
                return true;
            }
        }
    }
    
    // Check neighboring chunks
    for neighbor_pos in chunk_pos.all_neighbors() {
        if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&neighbor_pos) {
            if let Ok(chunk) = chunks.get(chunk_entity) {
                if check_chunk_ground_collision(ground_aabb, chunk) {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Check for ground collision within a specific chunk
fn check_chunk_ground_collision(ground_aabb: (Vec3, Vec3), chunk: &Chunk) -> bool {
    let (ground_min, ground_max) = ground_aabb;
    
    // Convert AABB to chunk-local coordinates
    let chunk_min_pos = chunk.position.min_block_position();
    let local_min = (ground_min - chunk_min_pos.as_vec3()).as_ivec3();
    let local_max = (ground_max - chunk_min_pos.as_vec3()).as_ivec3();
    
    // Clamp to chunk boundaries
    let start_x = local_min.x.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    let end_x = local_max.x.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    let start_y = local_min.y.max(0).min(crate::chunk::CHUNK_HEIGHT as i32 - 1);
    let end_y = local_max.y.max(0).min(crate::chunk::CHUNK_HEIGHT as i32 - 1);
    let start_z = local_min.z.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    let end_z = local_max.z.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    
    // Check all blocks in the overlapping region
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            for z in start_z..=end_z {
                if let Some(block_type) = chunk.data.get_block(x as usize, y as usize, z as usize) {
                    if block_type.is_solid() {
                        let block_world_pos = chunk_min_pos + IVec3::new(x, y, z);
                        let block_min = block_world_pos.as_vec3();
                        let block_max = block_world_pos.as_vec3() + Vec3::ONE;
                        
                        if check_aabb_collision(ground_min, ground_max, block_min, block_max) {
                            // Check if the collision is significant enough
                            let penetration_y = block_max.y - ground_min.y;
                            if penetration_y > GROUND_DETECTION_EPSILON {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    
    false
}

/// Check collisions within a specific chunk
fn check_chunk_block_collisions(entity_aabb: (Vec3, Vec3), chunk: &Chunk) -> Option<Vec3> {
    let (entity_min, entity_max) = entity_aabb;
    
    // Convert AABB to chunk-local coordinates
    let chunk_min_pos = chunk.position.min_block_position();
    let local_min = (entity_min - chunk_min_pos.as_vec3()).as_ivec3();
    let local_max = (entity_max - chunk_min_pos.as_vec3()).as_ivec3();
    
    // Clamp to chunk boundaries
    let start_x = local_min.x.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    let end_x = local_max.x.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    let start_y = local_min.y.max(0).min(crate::chunk::CHUNK_HEIGHT as i32 - 1);
    let end_y = local_max.y.max(0).min(crate::chunk::CHUNK_HEIGHT as i32 - 1);
    let start_z = local_min.z.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    let end_z = local_max.z.max(0).min(crate::chunk::CHUNK_SIZE as i32 - 1);
    
    // Check all blocks in the overlapping region
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            for z in start_z..=end_z {
                if let Some(block_type) = chunk.data.get_block(x as usize, y as usize, z as usize) {
                    if block_type.is_solid() {
                        let block_world_pos = chunk_min_pos + IVec3::new(x, y, z);
                        let block_min = block_world_pos.as_vec3();
                        let block_max = block_world_pos.as_vec3() + Vec3::ONE;
                        
                        if check_aabb_collision(entity_min, entity_max, block_min, block_max) {
                            if let Some(resolved_pos) = resolve_aabb_collision(entity_min, entity_max, block_min, block_max) {
                                return Some(resolved_pos);
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Check if two AABBs are colliding
fn check_aabb_collision(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
    a_min.x < b_max.x && a_max.x > b_min.x &&
    a_min.y < b_max.y && a_max.y > b_min.y &&
    a_min.z < b_max.z && a_max.z > b_min.z
}

/// Resolve AABB collision by moving the entity out of the block
fn resolve_aabb_collision(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> Option<Vec3> {
    let entity_center = (a_min + a_max) / 2.0;
    let block_center = (b_min + b_max) / 2.0;
    
    // Calculate penetration depths for each axis
    let penetrate_x = if entity_center.x < block_center.x {
        b_min.x - a_max.x // Left penetration
    } else {
        b_max.x - a_min.x // Right penetration
    };
    
    let penetrate_y = if entity_center.y < block_center.y {
        b_min.y - a_max.y // Bottom penetration
    } else {
        b_max.y - a_min.y // Top penetration
    };
    
    let penetrate_z = if entity_center.z < block_center.z {
        b_min.z - a_max.z // Front penetration
    } else {
        b_max.z - a_min.z // Back penetration
    };
    
    // Find the axis with the smallest penetration (most shallow collision)
    let abs_penetrate_x = penetrate_x.abs();
    let abs_penetrate_y = penetrate_y.abs();
    let abs_penetrate_z = penetrate_z.abs();
    
    // Ignore micro-collisions that are smaller than our epsilon threshold
    if abs_penetrate_x < COLLISION_EPSILON && abs_penetrate_y < COLLISION_EPSILON && abs_penetrate_z < COLLISION_EPSILON {
        return None; // No significant collision to resolve
    }
    
    // Resolve collision by moving along the axis with smallest penetration
    // Prioritize vertical collisions when they are significant to prevent jittering
    if abs_penetrate_y > COLLISION_EPSILON && penetrate_y < 0.0 {
        // Always prioritize resolving downward collisions (standing on ground)
        // This prevents the player from falling through the ground
        Some(Vec3::new(entity_center.x, entity_center.y + penetrate_y, entity_center.z))
    } else if abs_penetrate_x < abs_penetrate_y && abs_penetrate_x < abs_penetrate_z {
        Some(Vec3::new(entity_center.x + penetrate_x, entity_center.y, entity_center.z))
    } else if abs_penetrate_y < abs_penetrate_x && abs_penetrate_y < abs_penetrate_z {
        Some(Vec3::new(entity_center.x, entity_center.y + penetrate_y, entity_center.z))
    } else if abs_penetrate_z < abs_penetrate_x && abs_penetrate_z < abs_penetrate_y {
        Some(Vec3::new(entity_center.x, entity_center.y, entity_center.z + penetrate_z))
    } else {
        // If all penetrations are equal or no clear smallest, prioritize vertical resolution
        // This helps prevent the player from getting stuck in corners
        if penetrate_y < 0.0 { // If we're penetrating from below (standing on ground)
            Some(Vec3::new(entity_center.x, entity_center.y + penetrate_y, entity_center.z))
        } else if penetrate_y > 0.0 { // If we're penetrating from above (hitting ceiling)
            Some(Vec3::new(entity_center.x, entity_center.y + penetrate_y, entity_center.z))
        } else if abs_penetrate_x < abs_penetrate_z {
            Some(Vec3::new(entity_center.x + penetrate_x, entity_center.y, entity_center.z))
        } else {
            Some(Vec3::new(entity_center.x, entity_center.y, entity_center.z + penetrate_z))
        }
    }
}

/// System to find a safe spawn position for the player
pub fn find_safe_spawn_position(
    blocks: &Query<&Block>,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
    desired_position: Vec3,
) -> Vec3 {
    // First, find the highest solid ground at the desired X,Z position
    let ground_y = find_highest_solid_ground(desired_position.x, desired_position.z, blocks, chunks, chunk_manager);
    
    if ground_y > -1000.0 { // Valid ground found
        let spawn_position = Vec3::new(desired_position.x, ground_y + 1.0, desired_position.z);
        
        println!("✓ Found solid ground at y={}, spawning player at {:?}", ground_y, spawn_position);
        return spawn_position;
    }
    
    // If no ground found with chunks, try individual blocks
    let mut highest_block_y = -1000.0;
    for block in blocks.iter() {
        if block.position.x as f32 == desired_position.x && block.position.z as f32 == desired_position.z {
            if block.block_type.is_solid() && block.position.y as f32 > highest_block_y {
                highest_block_y = block.position.y as f32;
            }
        }
    }
    
    if highest_block_y > -1000.0 {
        let spawn_position = Vec3::new(desired_position.x, highest_block_y + 1.0, desired_position.z);
        println!("✓ Found solid block at y={}, spawning player at {:?}", highest_block_y, spawn_position);
        return spawn_position;
    }
    
    // If still no ground found, use a safe fallback position above potential terrain
    let fallback_position = Vec3::new(desired_position.x, 10.0, desired_position.z);
    println!("⚠ No solid ground found at ({}, {}), using safe fallback position {:?}", 
            desired_position.x, desired_position.z, fallback_position);
    
    fallback_position
}

/// Find the highest solid ground at a specific X,Z position
fn find_highest_solid_ground(
    x: f32,
    z: f32, 
    blocks: &Query<&Block>,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> f32 {
    let block_x = x as i32;
    let block_z = z as i32;
    let chunk_pos = ChunkPosition::from_block_position(IVec3::new(block_x, 0, block_z));
    
    println!("🔍 Looking for ground at world position ({}, {}) - block ({}, {})", x, z, block_x, block_z);
    
    // First check individual blocks
    let mut highest_solid_y = -1000.0;
    let mut found_blocks = Vec::new();
    
    for block in blocks.iter() {
        if block.position.x == block_x && block.position.z == block_z {
            found_blocks.push((block.position.y, block.block_type));
            if block.block_type.is_solid() {
                if block.position.y as f32 > highest_solid_y {
                    highest_solid_y = block.position.y as f32;
                }
            }
        }
    }
    
    println!("🔍 Found individual blocks at ({}, {}): {:?}", block_x, block_z, found_blocks);
    
    // Then check chunks
    if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
        println!("🔍 Found chunk at {:?}", chunk_pos);
        if let Ok(chunk) = chunks.get(chunk_entity) {
            let local_x = (block_x.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize;
            let local_z = (block_z.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize;
            
            println!("🔍 Checking chunk blocks at local position ({}, {})", local_x, local_z);
            
            // Check from top down to find the highest solid block
            for y in (0..crate::chunk::CHUNK_HEIGHT).rev() {
                if let Some(block_type) = chunk.data.get_block(local_x, y, local_z) {
                    if block_type.is_solid() {
                        println!("🔍 Found solid chunk block at y={}: {:?}", y, block_type);
                        return y as f32; // Return immediately when we find the highest solid block
                    }
                }
            }
            println!("🔍 No solid blocks found in chunk at ({}, {})", local_x, local_z);
        }
    } else {
        println!("🔍 No chunk found at {:?}", chunk_pos);
    }
    
    // If we found blocks but no chunks, return the highest block found
    if highest_solid_y > -1000.0 {
        println!("🔍 Returning highest individual block at y={}", highest_solid_y);
        return highest_solid_y;
    }
    
    println!("🔍 No solid ground found at all");
    -1000.0 // No ground found
}

/// Check if a position is safe for spawning (no collisions with blocks)
fn is_position_safe(
    position: Vec3,
    blocks: &Query<&Block>,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> bool {
    // Create a temporary collider for the player
    let collider = Collider::player();
    let entity_aabb = get_entity_aabb(position, &collider);
    
    // Check if there are any collisions at this position
    let has_block_collision = check_block_collisions(entity_aabb, blocks).is_some();
    let has_chunk_collision = check_chunk_collisions(entity_aabb, chunks, chunk_manager).is_some();
    
    !has_block_collision && !has_chunk_collision
}

/// Find the ground level at a specific X,Z position
fn find_ground_level(
    x: f32,
    z: f32, 
    blocks: &Query<&Block>,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> f32 {
    // Start checking from a reasonable height and go down
    let mut y = 10.0;
    let mut ground_y = -1000.0; // Invalid value
    
    // First try checking individual blocks
    while y >= 0.0 {
        let position = Vec3::new(x, y, z);
        let block_position = position.as_ivec3();
        
        // Check if there's a solid block at this position
        let has_solid_block = check_block_at_position(block_position, blocks, chunks, chunk_manager);
        
        if has_solid_block {
            ground_y = y;
            break;
        }
        
        y -= 1.0;
    }
    
    // If no ground found with blocks, try chunk-based checking
    if ground_y < 0.0 {
        ground_y = find_ground_level_in_chunks(x, z, chunks, chunk_manager);
    }
    
    ground_y
}

/// Check if there's a solid block at a specific position
fn check_block_at_position(
    position: IVec3,
    blocks: &Query<&Block>,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> bool {
    // Check individual blocks first
    for block in blocks.iter() {
        if block.position == position && block.block_type.is_solid() {
            return true;
        }
    }
    
    // Check chunks
    let chunk_pos = ChunkPosition::from_block_position(position);
    if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
        if let Ok(chunk) = chunks.get(chunk_entity) {
            if let Some(block_type) = chunk.data.get_block(
                (position.x.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize,
                position.y as usize,
                (position.z.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize
            ) {
                if block_type.is_solid() {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Find ground level using chunk-based approach
fn find_ground_level_in_chunks(
    x: f32,
    z: f32,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> f32 {
    let block_x = x as i32;
    let block_z = z as i32;
    let chunk_pos = ChunkPosition::from_block_position(IVec3::new(block_x, 0, block_z));
    
    if let Some(&chunk_entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
        if let Ok(chunk) = chunks.get(chunk_entity) {
            let local_x = (block_x.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize;
            let local_z = (block_z.rem_euclid(crate::chunk::CHUNK_SIZE as i32)) as usize;
            
            // Check from top down to find the highest solid block
            for y in (0..crate::chunk::CHUNK_HEIGHT).rev() {
                if let Some(block_type) = chunk.data.get_block(local_x, y, local_z) {
                    if block_type.is_solid() {
                        return y as f32;
                    }
                }
            }
        }
    }
    
    -1000.0 // No ground found
}

/// Find an empty space near a desired position
fn find_empty_space_near_position(
    desired_position: Vec3,
    blocks: &Query<&Block>,
    chunks: &Query<&Chunk>,
    chunk_manager: &ChunkManager,
) -> Vec3 {
    let mut best_position = desired_position;
    let mut best_distance = f32::MAX;
    
    // Try positions in a 5x5x5 area around the desired position
    for x_offset in -2..=2 {
        for y_offset in -2..=2 {
            for z_offset in -2..=2 {
                let test_position = Vec3::new(
                    desired_position.x + x_offset as f32,
                    desired_position.y + y_offset as f32,
                    desired_position.z + z_offset as f32,
                );
                
                let distance = (test_position - desired_position).length();
                
                if distance < best_distance {
                    let test_block_pos = test_position.as_ivec3();
                    let is_empty = !check_block_at_position(test_block_pos, blocks, chunks, chunk_manager);
                    
                    if is_empty {
                        // Also check if the space above is empty (for player height)
                        let above_pos = test_block_pos + IVec3::new(0, 1, 0);
                        let above_empty = !check_block_at_position(above_pos, blocks, chunks, chunk_manager);
                        
                        if above_empty {
                            best_position = test_position;
                            best_distance = distance;
                        }
                    }
                }
            }
        }
    }
    
    best_position
}