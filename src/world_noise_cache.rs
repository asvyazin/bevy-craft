// World Noise Cache System
// This module provides caching for generated noise to ensure continuity between chunks

use bevy::prelude::*;
use std::collections::HashMap;
use crate::chunk::{CHUNK_SIZE, Chunk};

/// Resource to store cached noise data for world generation
#[derive(Resource, Debug, Default)]
pub struct WorldNoiseCache {
    // Cache stores noise data for 3x3 chunk regions
    // Key: (center_chunk_x, center_chunk_z), Value: 3x3 noise region
    cache: HashMap<(i32, i32), [[f32; CHUNK_SIZE * 3]; CHUNK_SIZE * 3]>,
    
    // Track which chunks are currently using each cached region
    cache_users: HashMap<(i32, i32), Vec<(i32, i32)>>,
}

impl WorldNoiseCache {
    /// Get or generate noise data for a chunk with proper caching
    pub fn get_chunk_noise(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        noise_generator: &impl Fn(f32, f32) -> f32,
    ) -> [[f32; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] {
        // Calculate the center chunk for the 3x3 region that contains this chunk
        let center_x = Self::chunk_to_center_chunk(chunk_x);
        let center_z = Self::chunk_to_center_chunk(chunk_z);
        
        // Check if we have this region cached
        if let Some(cached_region) = self.cache.get(&(center_x, center_z)) {
            // Extract the requested chunk from the cached region
            Self::extract_chunk_from_region(cached_region, chunk_x, chunk_z)
        } else {
            // Generate the 3x3 region and cache it
            let region = self.generate_and_cache_region(center_x, center_z, chunk_x, chunk_z, noise_generator);
            
            // Extract the requested chunk from the newly generated region
            Self::extract_chunk_from_region(&region, chunk_x, chunk_z)
        }
    }
    
    /// Convert chunk coordinate to center chunk coordinate for 3x3 regions
    fn chunk_to_center_chunk(coord: i32) -> i32 {
        // Chunks are grouped in 3x3 regions, so we find which region this chunk belongs to
        let region_size = CHUNK_SIZE as i32 * 3;
        let region_x = (coord as f32 / region_size as f32).floor() as i32;
        region_x * 3 + 1 // Center chunk of the region
    }
    
    /// Generate a 3x3 chunk region and cache it
    fn generate_and_cache_region(
        &mut self,
        center_x: i32,
        center_z: i32,
        chunk_x: i32,
        chunk_z: i32,
        noise_generator: &impl Fn(f32, f32) -> f32,
    ) -> [[f32; CHUNK_SIZE * 3]; CHUNK_SIZE * 3] {
        // Calculate world coordinates for the region
        let region_size = CHUNK_SIZE as i32 * 3;
        let start_x = (center_x - 1) * CHUNK_SIZE as i32;
        let start_z = (center_z - 1) * CHUNK_SIZE as i32;
        
        // Generate noise for the entire 3x3 region
        let mut region = [[0.0; CHUNK_SIZE * 3]; CHUNK_SIZE * 3];
        
        for region_x in 0..region_size as usize {
            for region_z in 0..region_size as usize {
                let world_x = start_x + region_x as i32;
                let world_z = start_z + region_z as i32;
                region[region_x][region_z] = noise_generator(world_x as f32, world_z as f32);
            }
        }
        
        // Cache the generated region
        self.cache.insert((center_x, center_z), region);
        
        // Track that this chunk is using this region
        self.cache_users.entry((center_x, center_z))
            .or_insert_with(Vec::new)
            .push((chunk_x, chunk_z));
        
        region
    }
    
    /// Extract a single chunk from a 3x3 region
    fn extract_chunk_from_region(
        region: &[[f32; CHUNK_SIZE * 3]; CHUNK_SIZE * 3],
        chunk_x: i32,
        chunk_z: i32,
    ) -> [[f32; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] {
        let mut chunk_data = [[0.0; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        
        // Calculate position within the 3x3 region
        let region_x = ((chunk_x - Self::chunk_to_center_chunk(chunk_x)) + 1) as usize;
        let region_z = ((chunk_z - Self::chunk_to_center_chunk(chunk_z)) + 1) as usize;
        
        // Extract the chunk data
        for x in 0..CHUNK_SIZE as usize {
            for z in 0..CHUNK_SIZE as usize {
                chunk_data[x][z] = region[region_x * CHUNK_SIZE as usize + x][region_z * CHUNK_SIZE as usize + z];
            }
        }
        
        chunk_data
    }
    
    /// Clear unused cache entries to save memory
    pub fn clear_unused_cache(&mut self, active_chunks: &[(i32, i32)]) {
        let active_centers: Vec<(i32, i32)> = active_chunks.iter()
            .map(|(x, z)| (Self::chunk_to_center_chunk(*x), Self::chunk_to_center_chunk(*z)))
            .collect();
        
        // Find cache entries that are no longer needed
        let centers_to_keep: std::collections::HashSet<_> = active_centers.iter().collect();
        
        self.cache.retain(|center, _| centers_to_keep.contains(center));
        self.cache_users.retain(|center, _| centers_to_keep.contains(center));
    }
}

/// System to initialize the world noise cache
pub fn initialize_world_noise_cache(mut commands: Commands) {
    commands.insert_resource(WorldNoiseCache::default());
}

/// System to clean up unused noise cache entries
pub fn cleanup_world_noise_cache(
    mut cache: ResMut<WorldNoiseCache>,
    chunks: Query<&Chunk>,
) {
    // Collect active chunk positions
    let active_chunks: Vec<(i32, i32)> = chunks.iter()
        .map(|chunk| (chunk.position.x, chunk.position.z))
        .collect();
    
    // Clear unused cache entries
    cache.clear_unused_cache(&active_chunks);
}