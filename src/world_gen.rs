// World generation module for Bevy Craft
// This module handles procedural world generation using Perlin noise

use bevy::prelude::*;

use crate::chunk::{Chunk, CHUNK_SIZE, CHUNK_HEIGHT};
use crate::block::BlockType;

/// World generation settings
#[derive(Resource, Debug)]
pub struct WorldGenSettings {
    pub base_height: f32,
    pub height_scale: f32,
    pub frequency: f32,
    pub octaves: usize,
    pub persistence: f32,
    pub lacunarity: f32,
    pub perlin_seed: u32,
}

impl Default for WorldGenSettings {
    fn default() -> Self {
        Self {
            base_height: 10.0,   // Lower base height for extreme variation
            height_scale: 80.0,  // Very high height scale for dramatic terrain
            frequency: 0.015,    // Very low frequency for large-scale features
            octaves: 10,         // Many octaves for detailed terrain
            persistence: 0.3,    // Low persistence for extreme variation
            lacunarity: 2.3,     // Higher lacunarity for more detail
            perlin_seed: 42,
        }
    }
}

/// Generate heightmap for a chunk using Perlin noise
pub fn generate_chunk_heightmap(
    chunk: &mut Chunk,
    settings: &WorldGenSettings,
) {
    let chunk_x = chunk.position.x;
    let chunk_z = chunk.position.z;
    
    println!("🌱 Generating terrain for chunk ({}, {})", chunk_x, chunk_z);
    
    let mut min_height = i32::MAX;
    let mut max_height = i32::MIN;
    let mut total_height = 0;
    let mut height_count = 0;
    
    // Generate heightmap for this chunk
    for local_x in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            // Convert local coordinates to world coordinates
            let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;
            
            // Generate noise value for this position using CPU-based Perlin noise
            let noise_value = generate_fractal_noise(
                world_x as f32,
                world_z as f32,
                settings,
            );
            
            // Calculate height based on noise
            let height = calculate_height(noise_value, settings);
            
            // Track height statistics
            min_height = min_height.min(height);
            max_height = max_height.max(height);
            total_height += height;
            height_count += 1;
            
            // Generate terrain column
            generate_terrain_column(chunk, local_x, local_z, height);
        }
    }
    
    let average_height = total_height as f32 / height_count as f32;
    let height_range = max_height - min_height;
    
    println!("📊 Terrain stats for chunk ({}, {}): min={}, max={}, avg={:.1}, range={}", 
             chunk_x, chunk_z, min_height, max_height, average_height, height_range);
    
    chunk.is_generated = true;
    chunk.needs_mesh_update = true;
    println!("✓ Completed terrain generation for chunk ({}, {})", chunk_x, chunk_z);
}

/// Generate fractal noise (multiple octaves) for more natural terrain
fn generate_fractal_noise(
    x: f32,
    z: f32,
    settings: &WorldGenSettings,
) -> f32 {
    let mut total = 0.0;
    let mut frequency = settings.frequency;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..settings.octaves {
        // Generate noise for this octave using simple CPU Perlin noise
        let noise_value = cpu_perlin_noise(x * frequency, z * frequency, settings.perlin_seed);
        
        // Apply amplitude and add to total
        total += noise_value * amplitude;
        max_value += amplitude;
        
        // Prepare for next octave
        frequency *= settings.lacunarity;
        amplitude *= settings.persistence;
    }
    
    // Normalize the result
    total / max_value
}

/// Simple CPU-based Perlin noise implementation for world generation
fn cpu_perlin_noise(x: f32, z: f32, seed: u32) -> f32 {
    // Improved hash function for pseudo-random numbers
    fn hash(seed: u32, x: i32, y: i32) -> f32 {
        let mut n = seed;
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        n ^= (x as u32).wrapping_mul(314159265).wrapping_add(271828183); // Different multiplier for x
        n ^= (y as u32).wrapping_mul(271828183).wrapping_add(314159265); // Different multiplier for y
        // Mix it up more
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        n ^= n >> 16;
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        (n as f32) / (u32::MAX as f32)
    }
    
    // Get grid coordinates
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;
    
    // Get fractional parts
    let xf = x - xi as f32;
    let zf = z - zi as f32;
    
    // Fade curves for smooth interpolation
    let u = fade(xf);
    let v = fade(zf);
    
    // Hash coordinates to get pseudo-random gradient vectors
    let a = hash(seed, xi, zi);
    let b = hash(seed, xi + 1, zi);
    let c = hash(seed, xi, zi + 1);
    let d = hash(seed, xi + 1, zi + 1);
    
    // Interpolate
    let x1 = lerp(a, b, u);
    let x2 = lerp(c, d, u);
    let result = lerp(x1, x2, v);
    
    // Map from [0, 1] to [-1, 1]
    result * 2.0 - 1.0
}

/// Fade function for Perlin noise
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Calculate height from noise value
fn calculate_height(noise_value: f32, settings: &WorldGenSettings) -> i32 {
    // Map noise range [-1, 1] to height range
    let normalized = (noise_value + 1.0) / 2.0; // Map to [0, 1]
    
    // Apply height settings with extreme variation
    let height = settings.base_height + normalized * settings.height_scale;
    
    // Add significant additional variation to make terrain dramatically different
    let variation_factor = 1.0 + (noise_value.abs() * 1.2); // Add up to 120% more variation
    let final_height = height * variation_factor;
    
    // Ensure height is within valid range, but allow very low terrain
    final_height.clamp(2.0, (CHUNK_HEIGHT - 1) as f32) as i32
}

/// Generate a terrain column (vertical stack of blocks)
fn generate_terrain_column(chunk: &mut Chunk, local_x: usize, local_z: usize, height: i32) {
    // Define minimum terrain height to prevent voids
    const MIN_TERRAIN_HEIGHT: i32 = 3;
    
    // Use the maximum of calculated height and minimum height to ensure solid foundation
    let effective_height = height.max(MIN_TERRAIN_HEIGHT);
    
    // Generate bedrock layer at the bottom
    chunk.data.set_block(local_x, 0, local_z, BlockType::Bedrock);
    
    // Create more dynamic layering based on height
    let stone_height = if effective_height < 10 {
        // For lower terrain, have more stone relative to height
        (effective_height as f32 * 0.9) as i32
    } else if effective_height < 30 {
        // For medium terrain, standard 80% stone
        (effective_height as f32 * 0.8) as i32
    } else {
        // For high terrain, less stone relative to height for more dramatic mountains
        (effective_height as f32 * 0.6) as i32
    };
    
    // Fill with stone
    for y in 1..stone_height.min(effective_height) {
        chunk.data.set_block(local_x, y as usize, local_z, BlockType::Stone);
    }
    
    // Fill with dirt for the remaining part up to the surface
    for y in stone_height.min(effective_height)..effective_height {
        chunk.data.set_block(local_x, y as usize, local_z, BlockType::Dirt);
    }
    
    // Add grass on top if there's terrain
    if effective_height > 0 {
        chunk.data.set_block(local_x, effective_height as usize, local_z, BlockType::Grass);
    }
    
    // Add environmental features based on height and position
    add_environmental_features(chunk, local_x, local_z, effective_height);
}

/// Add environmental features like sand, water, etc. based on terrain characteristics
fn add_environmental_features(chunk: &mut Chunk, local_x: usize, local_z: usize, height: i32) {
    // Add sand for beach-like areas (low terrain near "water level")
    if height > 5 && height < 12 {
        for y in 3..=6 {
            if y < height {
                chunk.data.set_block(local_x, y as usize, local_z, BlockType::Sand);
            }
        }
    }
    
    // Add some stone variation at higher elevations for more interesting mountains
    if height > 25 {
        // Add some exposed stone at the top of mountains
        if height > 30 {
            for y in (height - 3)..height {
                if y > 0 && y < height {
                    chunk.data.set_block(local_x, y as usize, local_z, BlockType::Stone);
                }
            }
        }
    }
}

/// System to generate chunks that need generation
pub fn generate_chunks_system(
    mut chunks: Query<&mut Chunk>,
    settings: Res<WorldGenSettings>,
) {
    // Limit the number of chunks generated per frame to prevent performance issues
    let mut chunks_generated = 0;
    const MAX_CHUNKS_PER_FRAME: usize = 2;
    
    for mut chunk in &mut chunks {
        if !chunk.is_generated {
            generate_chunk_heightmap(&mut chunk, &settings);
            chunks_generated += 1;
            
            // Stop if we've generated enough chunks for this frame
            if chunks_generated >= MAX_CHUNKS_PER_FRAME {
                break;
            }
        }
    }
}
