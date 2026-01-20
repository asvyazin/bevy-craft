// Chunk system for Bevy Craft
// This module handles world chunking for efficient rendering and world generation

use bevy::prelude::*;
use std::collections::HashMap;

use crate::block::BlockType;

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
#[derive(Debug, Default)]
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
#[derive(Debug, Default)]
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
}

impl Chunk {
    pub fn new(position: ChunkPosition) -> Self {
        Self {
            position,
            data: ChunkData::new(),
            biome_data: ChunkBiomeData::new(),
            is_generated: false,
            needs_mesh_update: true,
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
}

impl ChunkManager {
    pub fn new(render_distance: i32) -> Self {
        Self {
            loaded_chunks: HashMap::new(),
            render_distance,
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
}