// Chunk system for Bevy Craft
// This module handles world chunking for efficient rendering and world generation

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use crate::block::BlockType;

/// Chunk priority for loading/unloading and processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkPriority {
    /// Highest priority - chunks that are visible to the player
    Visible = 3,
    /// Medium priority - chunks that are near the player but not visible
    Near = 2,
    /// Low priority - chunks that are within render distance but far
    Far = 1,
    /// Lowest priority - chunks that should be unloaded
    Unload = 0,
}

impl Default for ChunkPriority {
    fn default() -> Self {
        ChunkPriority::Far
    }
}

/// Constants for chunk system
pub const CHUNK_SIZE: usize = 16; // 16x16x16 chunks
pub const CHUNK_HEIGHT: usize = 128; // Maximum height for chunks
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;

/// Chunk position in world coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Convert block position to chunk position
    pub fn from_block_position(block_pos: IVec3) -> Self {
        let chunk_x = block_pos.x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = block_pos.z.div_euclid(CHUNK_SIZE as i32);
        Self::new(chunk_x, chunk_z)
    }

    /// Get the minimum block position for this chunk
    pub fn min_block_position(&self) -> IVec3 {
        IVec3::new(
            self.x * CHUNK_SIZE as i32,
            0,
            self.z * CHUNK_SIZE as i32,
        )
    }

    /// Get neighboring chunk positions (4 directions: N, S, E, W)
    pub fn neighbors(&self) -> [ChunkPosition; 4] {
        [
            ChunkPosition::new(self.x, self.z - 1), // North
            ChunkPosition::new(self.x, self.z + 1), // South
            ChunkPosition::new(self.x + 1, self.z), // East
            ChunkPosition::new(self.x - 1, self.z), // West
        ]
    }

    /// Get all neighboring chunk positions including diagonals (8 directions)
    pub fn all_neighbors(&self) -> [ChunkPosition; 8] {
        [
            ChunkPosition::new(self.x, self.z - 1), // North
            ChunkPosition::new(self.x, self.z + 1), // South
            ChunkPosition::new(self.x + 1, self.z), // East
            ChunkPosition::new(self.x - 1, self.z), // West
            ChunkPosition::new(self.x + 1, self.z - 1), // North-East
            ChunkPosition::new(self.x + 1, self.z + 1), // South-East
            ChunkPosition::new(self.x - 1, self.z - 1), // North-West
            ChunkPosition::new(self.x - 1, self.z + 1), // South-West
        ]
    }
}

/// Chunk data structure containing block information
#[derive(Debug, Default, Clone)]
pub struct ChunkData {
    pub blocks: Vec<Option<BlockType>>, // Using Option to represent air blocks
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            blocks: vec![None; CHUNK_VOLUME],
        }
    }

    /// Get block at local chunk coordinates
    pub fn get_block(&self, local_x: usize, y: usize, local_z: usize) -> Option<BlockType> {
        // Bounds checking to prevent overflow
        if local_x >= CHUNK_SIZE || local_z >= CHUNK_SIZE || y >= CHUNK_HEIGHT {
            return None;
        }
        let index = self.local_to_index(local_x, y, local_z);
        self.blocks[index]
    }

    /// Set block at local chunk coordinates
    pub fn set_block(&mut self, local_x: usize, y: usize, local_z: usize, block_type: BlockType) {
        // Bounds checking to prevent overflow
        if local_x >= CHUNK_SIZE || local_z >= CHUNK_SIZE || y >= CHUNK_HEIGHT {
            return;
        }
        let index = self.local_to_index(local_x, y, local_z);
        self.blocks[index] = Some(block_type);
    }

    /// Convert local coordinates to array index
    fn local_to_index(&self, local_x: usize, y: usize, local_z: usize) -> usize {
        y * CHUNK_AREA + local_z * CHUNK_SIZE + local_x
    }

    /// Convert array index to local coordinates
    fn index_to_local(&self, index: usize) -> (usize, usize, usize) {
        let y = index / CHUNK_AREA;
        let remainder = index % CHUNK_AREA;
        let local_z = remainder / CHUNK_SIZE;
        let local_x = remainder % CHUNK_SIZE;
        (local_x, y, local_z)
    }
}

/// Biome data for a single position in the chunk
#[derive(Debug, Clone, Default)]
pub struct BiomeData {
    pub temperature: f32,
    pub moisture: f32,
    pub biome_type: String,
}

/// Chunk biome data storage
#[derive(Debug, Default, Clone)]
pub struct ChunkBiomeData {
    pub data: Vec<BiomeData>,
}

impl ChunkBiomeData {
    pub fn new() -> Self {
        Self {
            data: vec![BiomeData::default(); CHUNK_AREA],
        }
    }
    
    /// Set biome data for a specific (x, z) position in the chunk
    pub fn set_biome_data(&mut self, local_x: usize, local_z: usize, temperature: f32, moisture: f32, biome_type: &str) {
        let index = local_z * CHUNK_SIZE + local_x;
        if index < self.data.len() {
            self.data[index] = BiomeData {
                temperature,
                moisture,
                biome_type: biome_type.to_string(),
            };
        }
    }
    
    /// Get biome data for a specific (x, z) position in the chunk
    pub fn get_biome_data(&self, local_x: usize, local_z: usize) -> Option<BiomeData> {
        let index = local_z * CHUNK_SIZE + local_x;
        if index < self.data.len() {
            Some(self.data[index].clone())
        } else {
            None
        }
    }
}

/// Chunk component that will be attached to chunk entities
#[derive(Component, Debug)]
pub struct Chunk {
    pub position: ChunkPosition,
    pub data: ChunkData,
    pub biome_data: ChunkBiomeData,
    pub is_generated: bool,
    pub needs_mesh_update: bool,
    pub priority: ChunkPriority,
    pub is_visible: bool,
}

impl Chunk {
    pub fn new(position: ChunkPosition) -> Self {
        Self {
            position,
            data: ChunkData::new(),
            biome_data: ChunkBiomeData::new(),
            is_generated: false,
            needs_mesh_update: true,
            priority: ChunkPriority::Far,
            is_visible: false,
        }
    }

    /// Get block at world position relative to this chunk
    pub fn get_block_world(&self, world_pos: IVec3) -> Option<BlockType> {
        let local_pos = self.world_to_local(world_pos);
        self.data.get_block(local_pos.x as usize, local_pos.y as usize, local_pos.z as usize)
    }

    /// Set block at world position relative to this chunk
    pub fn set_block_world(&mut self, world_pos: IVec3, block_type: BlockType) {
        let local_pos = self.world_to_local(world_pos);
        self.data.set_block(local_pos.x as usize, local_pos.y as usize, local_pos.z as usize, block_type);
        self.needs_mesh_update = true;
    }

    /// Convert world position to local chunk coordinates
    fn world_to_local(&self, world_pos: IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.rem_euclid(CHUNK_SIZE as i32),
            world_pos.y.clamp(0, CHUNK_HEIGHT as i32 - 1),
            world_pos.z.rem_euclid(CHUNK_SIZE as i32),
        )
    }

    /// Check if a world position is within this chunk
    pub fn contains(&self, world_pos: IVec3) -> bool {
        let min_pos = self.position.min_block_position();
        let max_pos = min_pos + IVec3::new(CHUNK_SIZE as i32, CHUNK_HEIGHT as i32, CHUNK_SIZE as i32);
        
        world_pos.x >= min_pos.x && world_pos.x < max_pos.x &&
        world_pos.y >= min_pos.y && world_pos.y < max_pos.y &&
        world_pos.z >= min_pos.z && world_pos.z < max_pos.z
    }
}

/// Resource to track loaded chunks
#[derive(Resource, Default, Debug)]
pub struct ChunkManager {
    pub loaded_chunks: HashMap<ChunkPosition, Entity>,
    pub render_distance: i32,
    /// Spatial grid for faster chunk lookup
    /// This grid divides the world into regions to optimize chunk management
    pub spatial_grid: HashMap<(i32, i32), Vec<ChunkPosition>>, // Region coordinates -> chunk positions
    pub grid_region_size: i32, // Size of each grid region in chunks
    
    /// Chunk cache for intelligent memory management
    /// This implements a simple LRU (Least Recently Used) cache for chunks
    chunk_cache: HashMap<ChunkPosition, CachedChunkData>,
    cache_access_order: VecDeque<ChunkPosition>, // Tracks access order for LRU eviction
    max_cache_size: usize, // Maximum number of chunks to keep in cache
}

/// Data stored in the chunk cache
#[derive(Debug, Clone)]
pub struct CachedChunkData {
    pub data: ChunkData,
    pub biome_data: ChunkBiomeData,
    pub is_generated: bool,
    pub last_accessed: f64, // Timestamp of last access
}

impl ChunkManager {
    pub fn new(render_distance: i32) -> Self {
        // Set grid region size based on render distance for optimal performance
        let grid_region_size = (render_distance / 2).max(4).min(8);
        // Set cache size based on render distance - cache more chunks for larger worlds
        let max_cache_size = ((render_distance * 2).pow(2) * 2) as usize; // Cache ~2x the visible area
        
        Self {
            loaded_chunks: HashMap::new(),
            render_distance,
            spatial_grid: HashMap::new(),
            grid_region_size,
            chunk_cache: HashMap::new(),
            cache_access_order: VecDeque::new(),
            max_cache_size,
        }
    }

    /// Check if a chunk should be loaded based on player position
    pub fn should_load_chunk(&self, chunk_pos: ChunkPosition, player_chunk_pos: ChunkPosition) -> bool {
        let dx = (chunk_pos.x - player_chunk_pos.x).abs();
        let dz = (chunk_pos.z - player_chunk_pos.z).abs();
        dx <= self.render_distance && dz <= self.render_distance
    }

    /// Check if a chunk should be unloaded based on player position
    pub fn should_unload_chunk(&self, chunk_pos: ChunkPosition, player_chunk_pos: ChunkPosition) -> bool {
        let dx = (chunk_pos.x - player_chunk_pos.x).abs();
        let dz = (chunk_pos.z - player_chunk_pos.z).abs();
        dx > self.render_distance || dz > self.render_distance
    }

    /// Get a neighboring chunk entity if it exists
    pub fn get_neighbor_chunk(&self, _chunk_pos: &ChunkPosition, neighbor_pos: &ChunkPosition) -> Option<Entity> {
        self.loaded_chunks.get(neighbor_pos).copied()
    }

    /// Get all neighboring chunks for a given chunk position
    pub fn get_neighboring_chunks(&self, chunk_pos: &ChunkPosition) -> Vec<(ChunkPosition, Entity)> {
        chunk_pos.neighbors()
            .iter()
            .filter_map(|neighbor_pos| {
                self.loaded_chunks.get(neighbor_pos)
                    .map(|&entity| (neighbor_pos.clone(), entity))
            })
            .collect()
    }

    /// Get all neighboring chunks including diagonals
    pub fn get_all_neighboring_chunks(&self, chunk_pos: &ChunkPosition) -> Vec<(ChunkPosition, Entity)> {
        chunk_pos.all_neighbors()
            .iter()
            .filter_map(|neighbor_pos| {
                self.loaded_chunks.get(neighbor_pos)
                    .map(|&entity| (neighbor_pos.clone(), entity))
            })
            .collect()
    }

    /// Check if a chunk has a specific neighbor loaded
    pub fn has_neighbor(&self, chunk_pos: &ChunkPosition, direction: &str) -> bool {
        let neighbor_pos = match direction {
            "north" => ChunkPosition::new(chunk_pos.x, chunk_pos.z - 1),
            "south" => ChunkPosition::new(chunk_pos.x, chunk_pos.z + 1),
            "east" => ChunkPosition::new(chunk_pos.x + 1, chunk_pos.z),
            "west" => ChunkPosition::new(chunk_pos.x - 1, chunk_pos.z),
            _ => return false,
        };
        self.loaded_chunks.contains_key(&neighbor_pos)
    }

    /// Get the block type from a neighboring chunk at a specific position
    pub fn get_neighbor_block(&self, chunks: &Query<&Chunk>, chunk_pos: &ChunkPosition, neighbor_pos: &ChunkPosition, local_x: usize, y: usize, local_z: usize) -> Option<BlockType> {
        // Check if the neighbor chunk exists
        if let Some(&neighbor_entity) = self.loaded_chunks.get(neighbor_pos) {
            // Get the neighbor chunk
            if let Ok(neighbor_chunk) = chunks.get(neighbor_entity) {
                // Calculate the local position in the neighbor chunk
                let neighbor_local_x = match neighbor_pos.x.cmp(&chunk_pos.x) {
                    std::cmp::Ordering::Greater => 0, // East neighbor
                    std::cmp::Ordering::Less => CHUNK_SIZE - 1, // West neighbor
                    std::cmp::Ordering::Equal => local_x, // Same X, must be North/South
                };

                let neighbor_local_z = match neighbor_pos.z.cmp(&chunk_pos.z) {
                    std::cmp::Ordering::Greater => 0, // South neighbor
                    std::cmp::Ordering::Less => CHUNK_SIZE - 1, // North neighbor
                    std::cmp::Ordering::Equal => local_z, // Same Z, must be East/West
                };

                // Get the block from the neighbor chunk
                return neighbor_chunk.data.get_block(neighbor_local_x, y, neighbor_local_z);
            }
        }
        None
    }

    /// Convert chunk position to spatial grid region coordinates
    pub fn chunk_pos_to_grid_region(&self, chunk_pos: &ChunkPosition) -> (i32, i32) {
        let region_x = chunk_pos.x.div_euclid(self.grid_region_size);
        let region_z = chunk_pos.z.div_euclid(self.grid_region_size);
        (region_x, region_z)
    }

    /// Add a chunk to the spatial grid
    fn add_chunk_to_spatial_grid(&mut self, chunk_pos: ChunkPosition) {
        let region_coords = self.chunk_pos_to_grid_region(&chunk_pos);
        self.spatial_grid.entry(region_coords)
            .or_insert_with(Vec::new)
            .push(chunk_pos);
    }

    /// Remove a chunk from the spatial grid
    fn remove_chunk_from_spatial_grid(&mut self, chunk_pos: &ChunkPosition) {
        let region_coords = self.chunk_pos_to_grid_region(chunk_pos);
        if let Some(chunks_in_region) = self.spatial_grid.get_mut(&region_coords) {
            chunks_in_region.retain(|&pos| pos != *chunk_pos);
            // Clean up empty regions
            if chunks_in_region.is_empty() {
                self.spatial_grid.remove(&region_coords);
            }
        }
    }

    /// Get chunks in the same and neighboring grid regions (spatial partitioning)
    pub fn get_chunks_in_spatial_region(&self, chunk_pos: &ChunkPosition) -> Vec<ChunkPosition> {
        let center_region = self.chunk_pos_to_grid_region(chunk_pos);
        let mut result = Vec::new();
        
        // Check the center region and all 8 neighboring regions
        for dx in -1..=1 {
            for dz in -1..=1 {
                let region_x = center_region.0 + dx;
                let region_z = center_region.1 + dz;
                let region_coords = (region_x, region_z);
                
                if let Some(chunks) = self.spatial_grid.get(&region_coords) {
                    result.extend(chunks.iter().cloned());
                }
            }
        }
        
        result
    }

    /// Get chunks within a certain distance using spatial partitioning for better performance
    pub fn get_chunks_within_distance_spatial(&self, center_chunk_pos: &ChunkPosition, max_distance: i32) -> Vec<ChunkPosition> {
        let center_region = self.chunk_pos_to_grid_region(center_chunk_pos);
        let mut result = Vec::new();
        
        // Calculate how many regions we need to check based on max_distance
        let region_radius = (max_distance as f32 / self.grid_region_size as f32).ceil() as i32 + 1;
        
        for dx in -region_radius..=region_radius {
            for dz in -region_radius..=region_radius {
                let region_x = center_region.0 + dx;
                let region_z = center_region.1 + dz;
                let region_coords = (region_x, region_z);
                
                if let Some(chunks) = self.spatial_grid.get(&region_coords) {
                    // Filter chunks within the actual distance limit
                    for &chunk_pos in chunks {
                        let distance_x = (chunk_pos.x - center_chunk_pos.x).abs();
                        let distance_z = (chunk_pos.z - center_chunk_pos.z).abs();
                        if distance_x <= max_distance && distance_z <= max_distance {
                            result.push(chunk_pos);
                        }
                    }
                }
            }
        }
        
        result
    }

    /// Insert a chunk into both the main hashmap and spatial grid
    pub fn insert_chunk(&mut self, chunk_pos: ChunkPosition, entity: Entity) {
        self.loaded_chunks.insert(chunk_pos, entity);
        self.add_chunk_to_spatial_grid(chunk_pos);
    }

    /// Remove a chunk from both the main hashmap and spatial grid
    pub fn remove_chunk(&mut self, chunk_pos: &ChunkPosition) -> Option<Entity> {
        self.remove_chunk_from_spatial_grid(chunk_pos);
        self.loaded_chunks.remove(chunk_pos)
    }

    /// Calculate chunk priority based on distance from player and visibility
    pub fn calculate_chunk_priority(&self, chunk_pos: &ChunkPosition, player_chunk_pos: &ChunkPosition, is_visible: bool) -> ChunkPriority {
        let dx = (chunk_pos.x - player_chunk_pos.x).abs();
        let dz = (chunk_pos.z - player_chunk_pos.z).abs();
        let distance = (dx.max(dz)) as i32;

        if is_visible {
            ChunkPriority::Visible
        } else if distance <= self.render_distance / 2 {
            ChunkPriority::Near
        } else if distance <= self.render_distance {
            ChunkPriority::Far
        } else {
            ChunkPriority::Unload
        }
    }

    /// Get chunks sorted by priority (highest first)
    pub fn get_chunks_sorted_by_priority(&self, player_chunk_pos: &ChunkPosition) -> Vec<(ChunkPosition, ChunkPriority)> {
        let mut chunks_with_priority: Vec<(ChunkPosition, ChunkPriority)> = self.loaded_chunks.keys()
            .map(|&chunk_pos| {
                // For now, we'll use a simple distance-based priority
                // In a real implementation, we'd use actual visibility data
                let dx = (chunk_pos.x - player_chunk_pos.x).abs();
                let dz = (chunk_pos.z - player_chunk_pos.z).abs();
                let distance = (dx.max(dz)) as i32;
                
                let priority = if distance <= self.render_distance / 2 {
                    ChunkPriority::Near
                } else if distance <= self.render_distance {
                    ChunkPriority::Far
                } else {
                    ChunkPriority::Unload
                };
                
                (chunk_pos, priority)
            })
            .collect();

        // Sort by priority (highest first)
        chunks_with_priority.sort_by(|a, b| b.1.cmp(&a.1));
        chunks_with_priority
    }

    /// Update priorities for all loaded chunks
    pub fn update_chunk_priorities(&mut self, player_chunk_pos: &ChunkPosition) {
        for (&chunk_pos, _) in &self.loaded_chunks {
            // This would be enhanced with actual visibility checking in a real implementation
            let dx = (chunk_pos.x - player_chunk_pos.x).abs();
            let dz = (chunk_pos.z - player_chunk_pos.z).abs();
            let distance = (dx.max(dz)) as i32;

            // Simple priority calculation - visible chunks would get higher priority
            let priority = if distance <= self.render_distance / 2 {
                ChunkPriority::Near
            } else if distance <= self.render_distance {
                ChunkPriority::Far
            } else {
                ChunkPriority::Unload
            };

            // In a real implementation, we would update the chunk component here
            // For now, we'll just track it in the manager
        }
    }

    /// Get the number of loaded chunks for performance monitoring
    pub fn get_loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// Get memory usage estimate for loaded chunks
    pub fn estimate_memory_usage(&self) -> usize {
        // Estimate memory usage: each chunk has CHUNK_VOLUME blocks
        // Each block is an Option<BlockType> which is roughly 1 byte (enum discriminant) + some overhead
        // This is a rough estimate for monitoring purposes
        self.loaded_chunks.len() * CHUNK_VOLUME
    }

    /// Optimize cache based on current memory constraints
    pub fn optimize_cache(&mut self, available_memory_mb: usize) {
        // Convert available memory to approximate chunk count
        // Each chunk is roughly CHUNK_VOLUME bytes
        let max_chunks_by_memory = (available_memory_mb * 1024 * 1024) / CHUNK_VOLUME;
        
        // Adjust cache size based on available memory
        let target_cache_size = max_chunks_by_memory.min(self.max_cache_size);
        
        // If we need to reduce cache size, evict least recently used chunks
        while self.chunk_cache.len() > target_cache_size {
            if let Some(oldest_chunk_pos) = self.cache_access_order.pop_back() {
                self.chunk_cache.remove(&oldest_chunk_pos);
            }
        }
        
        // Update max cache size for future caching
        self.max_cache_size = target_cache_size;
    }

    /// Get chunks that need immediate processing (high priority)
    pub fn get_high_priority_chunks(&self, player_chunk_pos: &ChunkPosition) -> Vec<ChunkPosition> {
        self.loaded_chunks.keys()
            .filter(|&&chunk_pos| {
                let dx = (chunk_pos.x - player_chunk_pos.x).abs();
                let dz = (chunk_pos.z - player_chunk_pos.z).abs();
                let distance = (dx.max(dz)) as i32;
                distance <= self.render_distance / 2 // Near chunks get priority
            })
            .cloned()
            .collect()
    }

    /// Cache a chunk's data when it's unloaded
    pub fn cache_chunk(&mut self, chunk_pos: ChunkPosition, chunk_data: ChunkData, biome_data: ChunkBiomeData, is_generated: bool, current_time: f64) {
        // Remove from cache if it already exists to update access time
        self.cache_access_order.retain(|&pos| pos != chunk_pos);
        
        // Add to cache
        self.chunk_cache.insert(chunk_pos, CachedChunkData {
            data: chunk_data,
            biome_data,
            is_generated,
            last_accessed: current_time,
        });
        
        // Add to access order (front = most recently used)
        self.cache_access_order.push_front(chunk_pos);
        
        // Evict least recently used chunks if cache is full
        self.evict_lru_chunks_if_needed();
    }

    /// Evict least recently used chunks if cache exceeds maximum size
    fn evict_lru_chunks_if_needed(&mut self) {
        while self.chunk_cache.len() > self.max_cache_size {
            // Remove from the end (least recently used)
            if let Some(oldest_chunk_pos) = self.cache_access_order.pop_back() {
                self.chunk_cache.remove(&oldest_chunk_pos);
            }
        }
    }

    /// Get cached chunk data if available
    pub fn get_cached_chunk(&mut self, chunk_pos: &ChunkPosition, current_time: f64) -> Option<CachedChunkData> {
        if let Some(cached_data) = self.chunk_cache.get(chunk_pos) {
            // Update access time
            self.cache_access_order.retain(|&pos| pos != *chunk_pos);
            self.cache_access_order.push_front(*chunk_pos);
            
            // Return a clone of the cached data
            Some(cached_data.clone())
        } else {
            None
        }
    }

    /// Clear the chunk cache (useful when memory is needed for other purposes)
    pub fn clear_cache(&mut self) {
        self.chunk_cache.clear();
        self.cache_access_order.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_chunks: self.chunk_cache.len(),
            max_cache_size: self.max_cache_size,
            cache_hit_rate: 0.0, // Would need tracking for accurate calculation
        }
    }

    /// Remove a chunk from cache (when it's no longer needed)
    pub fn remove_from_cache(&mut self, chunk_pos: &ChunkPosition) {
        self.chunk_cache.remove(chunk_pos);
        self.cache_access_order.retain(|&pos| pos != *chunk_pos);
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cached_chunks: usize,
    pub max_cache_size: usize,
    pub cache_hit_rate: f32, // Percentage of cache hits
}