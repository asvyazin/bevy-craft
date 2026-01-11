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
            base_height: 32.0,
            height_scale: 16.0,
            frequency: 0.05,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
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
            
            // Generate terrain column
            generate_terrain_column(chunk, local_x, local_z, height);
        }
    }
    
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
    // Simple hash function for pseudo-random numbers
    fn hash(seed: u32, x: i32, y: i32) -> f32 {
        let mut n = seed;
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        n ^= (x as u32).wrapping_mul(1664525).wrapping_add(1013904223);
        n ^= (y as u32).wrapping_mul(1664525).wrapping_add(1013904223);
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
    
    // Apply height settings
    let height = settings.base_height + normalized * settings.height_scale;
    
    // Ensure height is within valid range
    height.clamp(1.0, (CHUNK_HEIGHT - 1) as f32) as i32
}

/// Generate a terrain column (vertical stack of blocks)
fn generate_terrain_column(chunk: &mut Chunk, local_x: usize, local_z: usize, height: i32) {
    // Define minimum terrain height to prevent voids
    const MIN_TERRAIN_HEIGHT: i32 = 5;
    
    // Use the maximum of calculated height and minimum height to ensure solid foundation
    let effective_height = height.max(MIN_TERRAIN_HEIGHT);
    
    // Generate bedrock layer at the bottom
    chunk.data.set_block(local_x, 0, local_z, BlockType::Bedrock);
    
    // Fill with stone up to 80% of the terrain height for more natural variation
    let stone_height = (effective_height as f32 * 0.8) as i32;
    for y in 1..stone_height.min(effective_height) {
        chunk.data.set_block(local_x, y as usize, local_z, BlockType::Stone);
    }
    
    // Fill with dirt for the remaining 20% up to the surface
    for y in stone_height.min(effective_height)..effective_height {
        chunk.data.set_block(local_x, y as usize, local_z, BlockType::Dirt);
    }
    
    // Add grass on top if there's terrain
    if effective_height > 0 {
        chunk.data.set_block(local_x, effective_height as usize, local_z, BlockType::Grass);
    }
    
    // Add some sand near water level (around y=5-10) for beach-like areas
    if effective_height > 8 && effective_height < 15 {
        for y in 5..=8 {
            if y < effective_height {
                chunk.data.set_block(local_x, y as usize, local_z, BlockType::Sand);
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