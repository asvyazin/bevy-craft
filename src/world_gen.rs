// World generation module for Bevy Craft
// This module handles procedural world generation using noise algorithms

use bevy::prelude::*;

use crate::block::BlockType;
use crate::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk};
use crate::noise::{NoiseSettings, generate_biome_info, generate_heightmap};

/// World generation settings
#[derive(Resource, Debug)]
pub struct WorldGenSettings {
    pub base_height: f32,
    pub height_scale: f32,
    pub frequency: f32,
    pub octaves: usize,
    pub persistence: f32,
    pub lacunarity: f32,
    pub biome_scale: f32,
}

impl Default for WorldGenSettings {
    fn default() -> Self {
        Self {
            base_height: 10.0,  // Lower base height for extreme variation
            height_scale: 80.0, // Very high height scale for dramatic terrain
            frequency: 0.015,   // Very low frequency for large-scale features
            octaves: 10,        // Many octaves for detailed terrain
            persistence: 0.3,   // Low persistence for extreme variation
            lacunarity: 2.3,    // Higher lacunarity for more detail
            biome_scale: 0.005, // Scale for biome generation
        }
    }
}

/// Convert WorldGenSettings to NoiseSettings for noise generation
fn create_noise_settings(settings: &WorldGenSettings) -> NoiseSettings {
    NoiseSettings {
        base_height: settings.base_height,
        height_scale: settings.height_scale,
        scale: settings.frequency,
        octaves: settings.octaves,
        persistence: settings.persistence,
        lacunarity: settings.lacunarity,
        biome_scale: settings.biome_scale,
    }
}

/// Generate heightmap for a chunk using noise algorithms
pub fn generate_chunk_heightmap(chunk: &mut Chunk, settings: &WorldGenSettings) {
    let chunk_x = chunk.position.x;
    let chunk_z = chunk.position.z;

    println!(
        "🌱 Generating terrain for chunk ({}, {}) with noise algorithms",
        chunk_x, chunk_z
    );

    let mut min_height = i32::MAX;
    let mut max_height = i32::MIN;
    let mut total_height = 0;
    let mut height_count = 0;

    // Create noise settings from world generation settings
    let noise_settings = create_noise_settings(settings);

    // Generate heights using simple, deterministic approach with world coordinates
    let mut raw_heights = [[0.0; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

    for local_x in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            // Use world coordinates directly - this is the most reliable approach
            let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;

            // Generate noise using world coordinates - should be deterministic
            raw_heights[local_x as usize][local_z as usize] =
                generate_heightmap(world_x as f32, world_z as f32, &noise_settings);
        }
    }

    // Generate terrain with the extracted chunk heights
    for local_x in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            let height = raw_heights[local_x as usize][local_z as usize] as i32;

            // Check for big differences with neighbors (for debugging)
            if local_x > 0 {
                let left_diff =
                    (height - raw_heights[local_x as usize - 1][local_z as usize] as i32).abs();
                if left_diff > 5 {
                    let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
                    let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;
                    println!(
                        "BIG X difference: {} at ({}, {}) vs ({}, {})",
                        left_diff,
                        world_x,
                        world_z,
                        world_x - 1,
                        world_z
                    );
                }
            }
            if local_z > 0 {
                let back_diff =
                    (height - raw_heights[local_x as usize][local_z as usize - 1] as i32).abs();
                if back_diff > 5 {
                    let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
                    let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;
                    println!(
                        "BIG Z difference: {} at ({}, {}) vs ({}, {})",
                        back_diff,
                        world_x,
                        world_z,
                        world_x,
                        world_z - 1
                    );
                }
            }

            // Track height statistics
            min_height = min_height.min(height);
            max_height = max_height.max(height);
            total_height += height;
            height_count += 1;

            // Generate terrain column with biome information
            let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;
            let (temperature, moisture) =
                generate_biome_info(world_x as f32, world_z as f32, &noise_settings);

            // Determine biome type for this position
            let biome_type = determine_biome_type(temperature, moisture, height);

            // Store biome data in chunk
            chunk
                .biome_data
                .set_biome_data(local_x, local_z, temperature, moisture, biome_type);

            generate_terrain_column_with_biome(
                chunk,
                local_x,
                local_z,
                height,
                temperature,
                moisture,
            );
        }
    }

    let average_height = total_height as f32 / height_count as f32;
    let height_range = max_height - min_height;

    println!(
        "📊 Terrain stats for chunk ({}, {}): min={}, max={}, avg={:.1}, range={}",
        chunk_x, chunk_z, min_height, max_height, average_height, height_range
    );
    println!(
        "🚀 Using noise generation with {} octaves",
        noise_settings.octaves
    );

    chunk.is_generated = true;
    chunk.needs_mesh_update = true;
    println!(
        "✓ Completed Alkyd terrain generation for chunk ({}, {})",
        chunk_x, chunk_z
    );
}

/// Generate fractal noise (multiple octaves) for more natural terrain
#[allow(dead_code)]
fn generate_fractal_noise(x: f32, z: f32, settings: &WorldGenSettings) -> f32 {
    let mut total = 0.0;
    let mut frequency = settings.frequency;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..settings.octaves {
        // Generate noise for this octave using simple CPU Perlin noise
        let noise_value = cpu_perlin_noise(x * frequency, z * frequency, 42); // Use fixed seed for now

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
#[allow(dead_code)]
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
#[allow(dead_code)]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation
#[allow(dead_code)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Calculate height from noise value
#[allow(dead_code)]
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

/// Generate a terrain column (vertical stack of blocks) with biome information
fn generate_terrain_column_with_biome(
    chunk: &mut Chunk,
    local_x: usize,
    local_z: usize,
    height: i32,
    temperature: f32,
    moisture: f32,
) {
    // Define minimum terrain height to prevent voids
    const MIN_TERRAIN_HEIGHT: i32 = 3;

    // Use the maximum of calculated height and minimum height to ensure solid foundation
    let effective_height = height.max(MIN_TERRAIN_HEIGHT);

    // Generate bedrock layer at the bottom
    chunk
        .data
        .set_block(local_x, 0, local_z, BlockType::Bedrock);

    // Determine biome-based terrain composition
    let (stone_height, surface_block, sub_surface_block) =
        determine_biome_terrain(effective_height, temperature, moisture);

    // Fill with stone or biome-specific sub-surface material
    for y in 1..stone_height.min(effective_height) {
        chunk
            .data
            .set_block(local_x, y as usize, local_z, sub_surface_block);
    }

    // Fill with dirt or biome-specific material for the remaining part up to the surface
    for y in stone_height.min(effective_height)..effective_height {
        chunk
            .data
            .set_block(local_x, y as usize, local_z, sub_surface_block);
    }

    // Add biome-specific surface block
    if effective_height > 0 {
        chunk
            .data
            .set_block(local_x, effective_height as usize, local_z, surface_block);
    }

    // Add environmental features based on height, position, and biome
    add_environmental_features_with_biome(
        chunk,
        local_x,
        local_z,
        effective_height,
        temperature,
        moisture,
    );
}

/// Determine terrain composition based on biome information and height
fn determine_biome_terrain(
    effective_height: i32,
    temperature: f32,
    moisture: f32,
) -> (i32, BlockType, BlockType) {
    // Determine stone height based on terrain height (same logic as before)
    let stone_height = if effective_height < 10 {
        (effective_height as f32 * 0.9) as i32
    } else if effective_height < 30 {
        (effective_height as f32 * 0.8) as i32
    } else {
        (effective_height as f32 * 0.6) as i32
    };

    // Determine biome based on temperature, moisture, and height
    let biome_type = determine_biome_type(temperature, moisture, effective_height);

    // Return appropriate surface and sub-surface blocks based on biome and height
    match biome_type {
        "desert" => determine_desert_terrain(effective_height, stone_height),
        "forest" => determine_forest_terrain(effective_height, stone_height),
        "mountain" => determine_mountain_terrain(effective_height, stone_height),
        "snowy_mountain" => determine_snowy_mountain_terrain(effective_height, stone_height),
        "hills" => determine_hills_terrain(effective_height, stone_height),
        "plains" => determine_plains_terrain(effective_height, stone_height),
        "swamp" => determine_swamp_terrain(effective_height, stone_height),
        "tundra" => determine_tundra_terrain(effective_height, stone_height),
        "beach" => determine_beach_terrain(effective_height, stone_height),
        _ => (stone_height, BlockType::Grass, BlockType::Dirt), // Default
    }
}

/// Determine desert terrain with height variation
fn determine_desert_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 5 {
        // Low desert areas might have some grass near water sources
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else if effective_height < 15 {
        // Typical desert - all sand
        (stone_height, BlockType::Sand, BlockType::Sand)
    } else {
        // High desert areas might have more stone exposure
        (stone_height, BlockType::Sand, BlockType::Stone)
    }
}

/// Determine forest terrain with height variation
fn determine_forest_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 10 {
        // Low forest areas - more dirt exposed
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else if effective_height < 25 {
        // Typical forest - grass with dirt
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else {
        // High forest/mountain areas - more stone exposure
        (stone_height, BlockType::Grass, BlockType::Stone)
    }
}

/// Determine mountain terrain with height variation
fn determine_mountain_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 20 {
        // Lower mountain areas - some grass
        (stone_height, BlockType::Grass, BlockType::Stone)
    } else if effective_height < 40 {
        // Mid mountain areas - mostly stone
        (stone_height, BlockType::Stone, BlockType::Stone)
    } else {
        // High mountain areas - all stone
        (stone_height, BlockType::Stone, BlockType::Stone)
    }
}

/// Determine snowy mountain terrain with height variation
fn determine_snowy_mountain_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 25 {
        // Lower snowy areas - some grass
        (stone_height, BlockType::Grass, BlockType::Stone)
    } else if effective_height < 45 {
        // Mid snowy areas - stone with snow
        (stone_height, BlockType::Stone, BlockType::Stone)
    } else {
        // High snowy areas - all stone (snow would be a separate layer)
        (stone_height, BlockType::Stone, BlockType::Stone)
    }
}

/// Determine plains terrain with height variation
fn determine_plains_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 8 {
        // Low plains - might have some sand near water
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else if effective_height < 18 {
        // Typical plains - grass with dirt
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else {
        // High plains - more stone exposure
        (stone_height, BlockType::Grass, BlockType::Stone)
    }
}

/// Determine swamp terrain with height variation
fn determine_swamp_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 6 {
        // Low swamp areas - might be waterlogged
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else if effective_height < 15 {
        // Typical swamp - grass with more moisture
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else {
        // High swamp areas - transition to forest
        (stone_height, BlockType::Grass, BlockType::Dirt)
    }
}

/// Determine tundra terrain with height variation
fn determine_tundra_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 10 {
        // Low tundra - some grass
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else if effective_height < 25 {
        // Typical tundra - grass with stone
        (stone_height, BlockType::Grass, BlockType::Stone)
    } else {
        // High tundra - mostly stone
        (stone_height, BlockType::Stone, BlockType::Stone)
    }
}

/// Determine beach terrain with height variation
fn determine_beach_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 4 {
        // Very low beach - might be underwater
        (stone_height, BlockType::Sand, BlockType::Sand)
    } else if effective_height < 10 {
        // Typical beach - all sand
        (stone_height, BlockType::Sand, BlockType::Sand)
    } else {
        // High beach - transition to grass
        (stone_height, BlockType::Grass, BlockType::Dirt)
    }
}

/// Determine hills terrain with height variation
fn determine_hills_terrain(
    effective_height: i32,
    stone_height: i32,
) -> (i32, BlockType, BlockType) {
    if effective_height < 15 {
        // Low hills - mostly grass with some stone
        (stone_height, BlockType::Grass, BlockType::Dirt)
    } else if effective_height < 30 {
        // Typical hills - grass with more stone exposure
        (stone_height, BlockType::Grass, BlockType::Stone)
    } else {
        // High hills - transition to mountains
        (stone_height, BlockType::Stone, BlockType::Stone)
    }
}

/// Determine biome type based on temperature, moisture, and height
fn determine_biome_type(temperature: f32, moisture: f32, height: i32) -> &'static str {
    // Enhanced biome classification based on temperature, moisture, and height
    if height < 5 {
        // Low areas near water level - beaches or swamps
        if moisture > 0.5 { "swamp" } else { "beach" }
    } else if height > 50 {
        // Very high areas - snowy mountains (increased threshold)
        "snowy_mountain"
    } else if height > 40 {
        // High areas - mountains (increased threshold)
        "mountain"
    } else if height > 25 {
        // Medium-high areas - hills (new biome type)
        "hills"
    } else {
        // Normal terrain classification based on temperature and moisture
        if temperature > 0.7 {
            if moisture < 0.3 { "desert" } else { "plains" }
        } else if temperature > 0.5 {
            if moisture > 0.6 { "forest" } else { "plains" }
        } else if temperature > 0.3 {
            if moisture > 0.5 { "swamp" } else { "mountain" }
        } else {
            "tundra"
        }
    }
}

/// Add environmental features with biome information
fn add_environmental_features_with_biome(
    chunk: &mut Chunk,
    local_x: usize,
    local_z: usize,
    height: i32,
    temperature: f32,
    moisture: f32,
) {
    let biome_type = determine_biome_type(temperature, moisture, height);

    // Add sand for beach-like areas (low terrain near "water level")
    if height > 5 && height < 12 {
        for y in 3..=6 {
            if y < height {
                chunk
                    .data
                    .set_block(local_x, y as usize, local_z, BlockType::Sand);
            }
        }
    }

    // Add some stone variation at higher elevations for more interesting mountains
    if height > 25 {
        // Add some exposed stone at the top of mountains
        if height > 30 {
            for y in (height - 3)..height {
                if y > 0 && y < height {
                    chunk
                        .data
                        .set_block(local_x, y as usize, local_z, BlockType::Stone);
                }
            }
        }
    }

    // Add biome-specific features
    match biome_type {
        "desert" => {
            // Deserts have more sand at higher elevations
            if height > 8 && height < 15 {
                for y in 5..=8 {
                    if y < height {
                        chunk
                            .data
                            .set_block(local_x, y as usize, local_z, BlockType::Sand);
                    }
                }
            }
        }
        "hills" => {
            // Hills have some stone outcrops
            if height > 20 && height < 28 {
                for y in (height - 3)..height {
                    if y > 0 && y < height && y % 3 == 0 {
                        chunk
                            .data
                            .set_block(local_x, y as usize, local_z, BlockType::Stone);
                    }
                }
            }
        }
        "forest" => {
            // Forests might have more dirt variation
            if height > 10 {
                for y in (height - 2)..height {
                    if y > 0 && y < height {
                        chunk
                            .data
                            .set_block(local_x, y as usize, local_z, BlockType::Dirt);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Original terrain column function (kept for compatibility)
#[allow(dead_code)]
fn generate_terrain_column(chunk: &mut Chunk, local_x: usize, local_z: usize, height: i32) {
    generate_terrain_column_with_biome(chunk, local_x, local_z, height, 0.5, 0.5);
}

/// Add environmental features like sand, water, etc. based on terrain characteristics
#[allow(dead_code)]
fn add_environmental_features(chunk: &mut Chunk, local_x: usize, local_z: usize, height: i32) {
    // Add sand for beach-like areas (low terrain near "water level")
    if height > 5 && height < 12 {
        for y in 3..=6 {
            if y < height {
                chunk
                    .data
                    .set_block(local_x, y as usize, local_z, BlockType::Sand);
            }
        }
    }

    // Add some stone variation at higher elevations for more interesting mountains
    if height > 25 {
        // Add some exposed stone at the top of mountains
        if height > 30 {
            for y in (height - 3)..height {
                if y > 0 && y < height {
                    chunk
                        .data
                        .set_block(local_x, y as usize, local_z, BlockType::Stone);
                }
            }
        }
    }
}

/// System to generate chunks that need generation using noise algorithms
pub fn generate_chunks_system(mut chunks: Query<&mut Chunk>, settings: Res<WorldGenSettings>) {
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
