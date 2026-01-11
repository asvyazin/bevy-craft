// Chunk mesh system for Bevy Craft
// This module handles efficient mesh generation and rendering for chunks

use bevy::prelude::*;
use bevy::render::mesh::{Mesh, Indices};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;

use crate::block::BlockType;

/// Component that stores the mesh data for a chunk
#[derive(Component, Debug)]
pub struct ChunkMesh {
    /// Handle to the mesh asset
    pub mesh_handle: Handle<Mesh>,
    /// Material handles for different block types in this chunk
    pub material_handles: HashMap<BlockType, Handle<StandardMaterial>>,
    /// Flag indicating if the mesh needs to be regenerated
    pub needs_rebuild: bool,
}

impl ChunkMesh {
    pub fn new() -> Self {
        Self {
            mesh_handle: Handle::default(),
            material_handles: HashMap::new(),
            needs_rebuild: true,
        }
    }
}

/// Resource for managing chunk mesh materials
#[derive(Resource, Default, Debug)]
pub struct ChunkMeshMaterials {
    /// Map of block types to their material handles
    pub materials: HashMap<BlockType, Handle<StandardMaterial>>,
}

impl ChunkMeshMaterials {
    /// Initialize default materials for all block types
    pub fn initialize(
        &mut self,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        // Create materials for each block type
        for block_type in [
            BlockType::Dirt,
            BlockType::Stone,
            BlockType::Grass,
            BlockType::Wood,
            BlockType::Leaves,
            BlockType::Sand,
            BlockType::Water,
            BlockType::Bedrock,
        ] {
            let material = materials.add(StandardMaterial {
                base_color: block_type.color(),
                ..default()
            });
            self.materials.insert(block_type, material);
        }
    }
    
    /// Get material handle for a block type
    pub fn get_material(&self, block_type: BlockType) -> Option<Handle<StandardMaterial>> {
        self.materials.get(&block_type).cloned()
    }
}

/// Simple mesh generation for a chunk (basic implementation)
pub fn generate_simple_chunk_mesh(_chunk_data: &crate::chunk::ChunkData) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );
    
    // For now, create a simple mesh that represents the chunk
    // This will be replaced with a proper greedy meshing algorithm later
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Simple cube mesh for testing
    // Front face
    positions.extend_from_slice(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]);
    
    normals.extend_from_slice(&[
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ]);
    
    uvs.extend_from_slice(&[
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ]);
    
    indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    
    // Insert mesh data
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

/// Generate mesh for a chunk with neighbor awareness
/// This function checks neighboring chunks to avoid rendering faces that are adjacent to solid blocks
pub fn generate_chunk_mesh_with_neighbors(
    chunk_data: &crate::chunk::ChunkData,
    chunk_pos: &crate::chunk::ChunkPosition,
    chunk_manager: &crate::chunk::ChunkManager,
    chunks: &Query<&crate::chunk::Chunk>,
) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Iterate through all blocks in the chunk
    for local_x in 0..crate::chunk::CHUNK_SIZE {
        for local_z in 0..crate::chunk::CHUNK_SIZE {
            for y in 0..crate::chunk::CHUNK_HEIGHT {
                if let Some(block_type) = chunk_data.get_block(local_x, y, local_z) {
                    if block_type != crate::block::BlockType::Air {
                        // Check each face to see if it should be rendered
                        let should_render = check_face_visibility(
                            chunk_data,
                            chunk_pos,
                            chunk_manager,
                            chunks,
                            local_x,
                            y,
                            local_z,
                        );
                        
                        // If any face should be rendered, add the block mesh
                        if should_render.any() {
                            add_block_mesh(
                                &mut positions,
                                &mut normals,
                                &mut uvs,
                                &mut indices,
                                local_x,
                                y,
                                local_z,
                                &should_render,
                                block_type,
                            );
                        }
                    }
                }
            }
        }
    }
    
    // Insert mesh data
    if !positions.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
    }
    
    mesh
}

/// Check which faces of a block should be visible
/// Returns a struct indicating which faces should be rendered
#[derive(Default)]
struct FaceVisibility {
    front: bool,
    back: bool,
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}

impl FaceVisibility {
    fn any(&self) -> bool {
        self.front || self.back || self.left || self.right || self.top || self.bottom
    }
}

fn check_face_visibility(
    chunk_data: &crate::chunk::ChunkData,
    chunk_pos: &crate::chunk::ChunkPosition,
    chunk_manager: &crate::chunk::ChunkManager,
    chunks: &Query<&crate::chunk::Chunk>,
    local_x: usize,
    y: usize,
    local_z: usize,
) -> FaceVisibility {
    let mut visibility = FaceVisibility::default();
    
    // Check front face (positive Z direction)
    if local_z == crate::chunk::CHUNK_SIZE - 1 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x, chunk_pos.z + 1);
        if let Some(neighbor_block) = chunk_manager.get_neighbor_block(
            chunks,
            chunk_pos,
            &neighbor_pos,
            local_x,
            y,
            0, // Front face of neighbor is at local_z = 0
        ) {
            visibility.front = neighbor_block == crate::block::BlockType::Air;
        } else {
            // No neighbor chunk, render the face
            visibility.front = true;
        }
    } else {
        // Within chunk, check adjacent block
        if let Some(adjacent_block) = chunk_data.get_block(local_x, y, local_z + 1) {
            visibility.front = adjacent_block == crate::block::BlockType::Air;
        } else {
            visibility.front = true;
        }
    }
    
    // Check back face (negative Z direction)
    if local_z == 0 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x, chunk_pos.z - 1);
        if let Some(neighbor_block) = chunk_manager.get_neighbor_block(
            chunks,
            chunk_pos,
            &neighbor_pos,
            local_x,
            y,
            crate::chunk::CHUNK_SIZE - 1, // Back face of neighbor is at local_z = CHUNK_SIZE - 1
        ) {
            visibility.back = neighbor_block == crate::block::BlockType::Air;
        } else {
            // No neighbor chunk, render the face
            visibility.back = true;
        }
    } else {
        // Within chunk, check adjacent block
        if let Some(adjacent_block) = chunk_data.get_block(local_x, y, local_z - 1) {
            visibility.back = adjacent_block == crate::block::BlockType::Air;
        } else {
            visibility.back = true;
        }
    }
    
    // Check right face (positive X direction)
    if local_x == crate::chunk::CHUNK_SIZE - 1 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x + 1, chunk_pos.z);
        if let Some(neighbor_block) = chunk_manager.get_neighbor_block(
            chunks,
            chunk_pos,
            &neighbor_pos,
            0, // Right face of neighbor is at local_x = 0
            y,
            local_z,
        ) {
            visibility.right = neighbor_block == crate::block::BlockType::Air;
        } else {
            // No neighbor chunk, render the face
            visibility.right = true;
        }
    } else {
        // Within chunk, check adjacent block
        if let Some(adjacent_block) = chunk_data.get_block(local_x + 1, y, local_z) {
            visibility.right = adjacent_block == crate::block::BlockType::Air;
        } else {
            visibility.right = true;
        }
    }
    
    // Check left face (negative X direction)
    if local_x == 0 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x - 1, chunk_pos.z);
        if let Some(neighbor_block) = chunk_manager.get_neighbor_block(
            chunks,
            chunk_pos,
            &neighbor_pos,
            crate::chunk::CHUNK_SIZE - 1, // Left face of neighbor is at local_x = CHUNK_SIZE - 1
            y,
            local_z,
        ) {
            visibility.left = neighbor_block == crate::block::BlockType::Air;
        } else {
            // No neighbor chunk, render the face
            visibility.left = true;
        }
    } else {
        // Within chunk, check adjacent block
        if let Some(adjacent_block) = chunk_data.get_block(local_x - 1, y, local_z) {
            visibility.left = adjacent_block == crate::block::BlockType::Air;
        } else {
            visibility.left = true;
        }
    }
    
    // Check top face (positive Y direction)
    if y < crate::chunk::CHUNK_HEIGHT - 1 {
        // Within chunk, check adjacent block
        if let Some(adjacent_block) = chunk_data.get_block(local_x, y + 1, local_z) {
            visibility.top = adjacent_block == crate::block::BlockType::Air;
        } else {
            visibility.top = true;
        }
    } else {
        // At top of chunk, always render (no chunks above)
        visibility.top = true;
    }
    
    // Check bottom face (negative Y direction)
    if y > 0 {
        // Within chunk, check adjacent block
        if let Some(adjacent_block) = chunk_data.get_block(local_x, y - 1, local_z) {
            visibility.bottom = adjacent_block == crate::block::BlockType::Air;
        } else {
            visibility.bottom = true;
        }
    } else {
        // At bottom of chunk, always render (no chunks below)
        visibility.bottom = true;
    }
    
    visibility
}

/// Add a block mesh with only the visible faces
fn add_block_mesh(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    local_x: usize,
    y: usize,
    local_z: usize,
    visibility: &FaceVisibility,
    block_type: crate::block::BlockType,
) {
    let base_index = positions.len() as u32;
    
    // Front face (positive Z)
    if visibility.front {
        let z = local_z as f32 + 1.0;
        positions.extend_from_slice(&[
            [local_x as f32, y as f32, z],
            [local_x as f32 + 1.0, y as f32, z],
            [local_x as f32 + 1.0, y as f32 + 1.0, z],
            [local_x as f32, y as f32 + 1.0, z],
        ]);
        
        normals.extend_from_slice(&[
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ]);
        
        // UV coordinates based on block type
        let uv = get_block_uv(block_type);
        uvs.extend_from_slice(&[
            [uv.0, uv.1],
            [uv.2, uv.1],
            [uv.2, uv.3],
            [uv.0, uv.3],
        ]);
        
        indices.extend_from_slice(&[base_index, base_index + 1, base_index + 2, base_index, base_index + 2, base_index + 3]);
    }
    
    // Back face (negative Z)
    if visibility.back {
        let z = local_z as f32;
        positions.extend_from_slice(&[
            [local_x as f32 + 1.0, y as f32, z],
            [local_x as f32, y as f32, z],
            [local_x as f32, y as f32 + 1.0, z],
            [local_x as f32 + 1.0, y as f32 + 1.0, z],
        ]);
        
        normals.extend_from_slice(&[
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ]);
        
        let uv = get_block_uv(block_type);
        uvs.extend_from_slice(&[
            [uv.0, uv.1],
            [uv.2, uv.1],
            [uv.2, uv.3],
            [uv.0, uv.3],
        ]);
        
        indices.extend_from_slice(&[base_index + 4, base_index + 5, base_index + 6, base_index + 4, base_index + 6, base_index + 7]);
    }
    
    // Right face (positive X)
    if visibility.right {
        let x = local_x as f32 + 1.0;
        positions.extend_from_slice(&[
            [x, y as f32, local_z as f32 + 1.0],
            [x, y as f32, local_z as f32],
            [x, y as f32 + 1.0, local_z as f32],
            [x, y as f32 + 1.0, local_z as f32 + 1.0],
        ]);
        
        normals.extend_from_slice(&[
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ]);
        
        let uv = get_block_uv(block_type);
        uvs.extend_from_slice(&[
            [uv.0, uv.1],
            [uv.2, uv.1],
            [uv.2, uv.3],
            [uv.0, uv.3],
        ]);
        
        indices.extend_from_slice(&[base_index + 8, base_index + 9, base_index + 10, base_index + 8, base_index + 10, base_index + 11]);
    }
    
    // Left face (negative X)
    if visibility.left {
        let x = local_x as f32;
        positions.extend_from_slice(&[
            [x, y as f32, local_z as f32],
            [x, y as f32, local_z as f32 + 1.0],
            [x, y as f32 + 1.0, local_z as f32 + 1.0],
            [x, y as f32 + 1.0, local_z as f32],
        ]);
        
        normals.extend_from_slice(&[
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ]);
        
        let uv = get_block_uv(block_type);
        uvs.extend_from_slice(&[
            [uv.0, uv.1],
            [uv.2, uv.1],
            [uv.2, uv.3],
            [uv.0, uv.3],
        ]);
        
        indices.extend_from_slice(&[base_index + 12, base_index + 13, base_index + 14, base_index + 12, base_index + 14, base_index + 15]);
    }
    
    // Top face (positive Y)
    if visibility.top {
        let y_top = y as f32 + 1.0;
        positions.extend_from_slice(&[
            [local_x as f32, y_top, local_z as f32 + 1.0],
            [local_x as f32 + 1.0, y_top, local_z as f32 + 1.0],
            [local_x as f32 + 1.0, y_top, local_z as f32],
            [local_x as f32, y_top, local_z as f32],
        ]);
        
        normals.extend_from_slice(&[
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        
        let uv = get_block_uv(block_type);
        uvs.extend_from_slice(&[
            [uv.0, uv.1],
            [uv.2, uv.1],
            [uv.2, uv.3],
            [uv.0, uv.3],
        ]);
        
        indices.extend_from_slice(&[base_index + 16, base_index + 17, base_index + 18, base_index + 16, base_index + 18, base_index + 19]);
    }
    
    // Bottom face (negative Y)
    if visibility.bottom {
        let y_bottom = y as f32;
        positions.extend_from_slice(&[
            [local_x as f32, y_bottom, local_z as f32],
            [local_x as f32 + 1.0, y_bottom, local_z as f32],
            [local_x as f32 + 1.0, y_bottom, local_z as f32 + 1.0],
            [local_x as f32, y_bottom, local_z as f32 + 1.0],
        ]);
        
        normals.extend_from_slice(&[
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ]);
        
        let uv = get_block_uv(block_type);
        uvs.extend_from_slice(&[
            [uv.0, uv.1],
            [uv.2, uv.1],
            [uv.2, uv.3],
            [uv.0, uv.3],
        ]);
        
        indices.extend_from_slice(&[base_index + 20, base_index + 21, base_index + 22, base_index + 20, base_index + 22, base_index + 23]);
    }
}

/// Get UV coordinates for a block type (simple implementation for now)
fn get_block_uv(block_type: crate::block::BlockType) -> (f32, f32, f32, f32) {
    // For now, use simple UV mapping based on block type
    // This will be replaced with a proper texture atlas later
    match block_type {
        crate::block::BlockType::Grass => (0.0, 0.0, 1.0, 1.0),
        crate::block::BlockType::Dirt => (0.0, 0.0, 1.0, 1.0),
        crate::block::BlockType::Stone => (0.0, 0.0, 1.0, 1.0),
        crate::block::BlockType::Wood => (0.0, 0.0, 1.0, 1.0),
        crate::block::BlockType::Leaves => (0.0, 0.0, 1.0, 1.0),
        crate::block::BlockType::Sand => (0.0, 0.0, 1.0, 1.0),
        crate::block::BlockType::Water => (0.0, 0.0, 1.0, 1.0),
        crate::block::BlockType::Bedrock => (0.0, 0.0, 1.0, 1.0),
        _ => (0.0, 0.0, 1.0, 1.0),
    }
}

/// Greedy meshing algorithm implementation
/// This algorithm merges adjacent blocks of the same type into larger quads for better performance
pub fn generate_chunk_mesh_greedy(
    chunk_data: &crate::chunk::ChunkData,
    chunk_pos: &crate::chunk::ChunkPosition,
    chunk_manager: &crate::chunk::ChunkManager,
    chunks: &Query<&crate::chunk::Chunk>,
) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // We'll process the chunk in 3 axes for greedy meshing
    // For now, let's implement a simplified version that processes each direction separately

    // Process each direction (X, Y, Z axes)
    for direction in [Direction::X, Direction::Y, Direction::Z] {
        greedy_mesh_direction(
            chunk_data,
            chunk_pos,
            chunk_manager,
            chunks,
            direction,
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
        );
    }

    // Insert mesh data
    if !positions.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
    }

    mesh
}

/// Direction enum for greedy meshing
#[derive(Debug, Clone, Copy)]
enum Direction {
    X, // Right/Left faces
    Y, // Top/Bottom faces  
    Z, // Front/Back faces
}

/// Greedy mesh generation for a specific direction
fn greedy_mesh_direction(
    chunk_data: &crate::chunk::ChunkData,
    chunk_pos: &crate::chunk::ChunkPosition,
    chunk_manager: &crate::chunk::ChunkManager,
    chunks: &Query<&crate::chunk::Chunk>,
    direction: Direction,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    // Determine the dimensions based on direction
    let (width, height, depth) = match direction {
        Direction::X => (crate::chunk::CHUNK_HEIGHT, crate::chunk::CHUNK_SIZE, crate::chunk::CHUNK_SIZE),
        Direction::Y => (crate::chunk::CHUNK_SIZE, crate::chunk::CHUNK_HEIGHT, crate::chunk::CHUNK_SIZE),
        Direction::Z => (crate::chunk::CHUNK_SIZE, crate::chunk::CHUNK_SIZE, crate::chunk::CHUNK_HEIGHT),
    };

    // We'll implement a simplified greedy meshing approach
    // For each slice perpendicular to the direction, find contiguous blocks

    // Iterate through each slice
    for slice in 0..width {
        // Iterate through each row in the slice
        for y in 0..height {
            // Find contiguous blocks in each row
            let mut x = 0;
            while x < depth {
                // Find the start of a contiguous block
                let block_type = match direction {
                    Direction::X => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, x, y, slice, direction),
                    Direction::Y => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, slice, x, y, direction),
                    Direction::Z => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, slice, y, x, direction),
                };

                if let Some(block_type) = block_type {
                    if block_type != crate::block::BlockType::Air {
                        // Find the width of this contiguous block
                        let mut block_width = 1;
                        while x + block_width < depth {
                            let next_block = match direction {
                                Direction::X => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, x + block_width, y, slice, direction),
                                Direction::Y => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, slice, x + block_width, y, direction),
                                Direction::Z => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, slice, y, x + block_width, direction),
                            };

                            if next_block == Some(block_type) {
                                block_width += 1;
                            } else {
                                break;
                            }
                        }

                        // Find the height of this contiguous block
                        let mut block_height = 1;
                        while y + block_height < height {
                            let mut all_same = true;
                            for wx in 0..block_width {
                                let test_block = match direction {
                                    Direction::X => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, x + wx, y + block_height, slice, direction),
                                    Direction::Y => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, slice, x + wx, y + block_height, direction),
                                    Direction::Z => get_block_in_direction(chunk_data, chunk_pos, chunk_manager, chunks, slice, y + wx, x + block_height, direction),
                                };

                                if test_block != Some(block_type) {
                                    all_same = false;
                                    break;
                                }
                            }

                            if all_same {
                                block_height += 1;
                            } else {
                                break;
                            }
                        }

                        // Add the quad for this contiguous block
                        add_greedy_quad(
                            positions,
                            normals,
                            uvs,
                            indices,
                            x,
                            y,
                            slice,
                            block_width,
                            block_height,
                            direction,
                            block_type,
                        );

                        // Skip the processed blocks
                        x += block_width;
                    } else {
                        x += 1;
                    }
                } else {
                    x += 1;
                }
            }
        }
    }
}

/// Get block type in a specific direction
fn get_block_in_direction(
    chunk_data: &crate::chunk::ChunkData,
    chunk_pos: &crate::chunk::ChunkPosition,
    chunk_manager: &crate::chunk::ChunkManager,
    chunks: &Query<&crate::chunk::Chunk>,
    x: usize,
    y: usize,
    z: usize,
    direction: Direction,
) -> Option<crate::block::BlockType> {
    match direction {
        Direction::X => {
            // X direction: slice = Y, y = Z, x = X
            if x < crate::chunk::CHUNK_SIZE && y < crate::chunk::CHUNK_SIZE && z < crate::chunk::CHUNK_HEIGHT {
                chunk_data.get_block(x, z, y)
            } else {
                // Handle chunk boundaries
                get_neighbor_block_for_greedy(chunk_data, chunk_pos, chunk_manager, chunks, x, y, z, direction)
            }
        },
        Direction::Y => {
            // Y direction: slice = X, y = Y, x = Z
            if x < crate::chunk::CHUNK_SIZE && y < crate::chunk::CHUNK_HEIGHT && z < crate::chunk::CHUNK_SIZE {
                chunk_data.get_block(z, y, x)
            } else {
                // Handle chunk boundaries
                get_neighbor_block_for_greedy(chunk_data, chunk_pos, chunk_manager, chunks, x, y, z, direction)
            }
        },
        Direction::Z => {
            // Z direction: slice = X, y = Y, x = Z
            if x < crate::chunk::CHUNK_SIZE && y < crate::chunk::CHUNK_SIZE && z < crate::chunk::CHUNK_HEIGHT {
                chunk_data.get_block(x, y, z)
            } else {
                // Handle chunk boundaries
                get_neighbor_block_for_greedy(chunk_data, chunk_pos, chunk_manager, chunks, x, y, z, direction)
            }
        },
    }
}

/// Get neighbor block for greedy meshing (handles chunk boundaries)
fn get_neighbor_block_for_greedy(
    chunk_data: &crate::chunk::ChunkData,
    chunk_pos: &crate::chunk::ChunkPosition,
    chunk_manager: &crate::chunk::ChunkManager,
    chunks: &Query<&crate::chunk::Chunk>,
    x: usize,
    y: usize,
    z: usize,
    direction: Direction,
) -> Option<crate::block::BlockType> {
    // Debug log for neighbor block access
    // println!("🔍 Checking neighbor block at (x:{}, y:{}, z:{}) for direction {:?}", x, y, z, direction);
    // Determine if we're at a chunk boundary and need to check neighbor chunks
    match direction {
        Direction::X => {
            // X direction: slice = Y, y = Z, x = X
            if x >= crate::chunk::CHUNK_SIZE {
                // Need to check east neighbor chunk
                let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x + 1, chunk_pos.z);
                if let Some(&neighbor_entity) = chunk_manager.loaded_chunks.get(&neighbor_pos) {
                    if let Ok(neighbor_chunk) = chunks.get(neighbor_entity) {
                        // In east neighbor, we're at local_x = 0
                        return neighbor_chunk.data.get_block(0, z, y);
                    }
                }
                // No neighbor chunk or neighbor chunk not loaded, treat as air
                Some(crate::block::BlockType::Air)
            } else {
                // Within chunk bounds
                chunk_data.get_block(x, z, y)
            }
        },
        Direction::Y => {
            // Y direction: slice = X, y = Y, x = Z
            if y >= crate::chunk::CHUNK_HEIGHT {
                // Above chunk, treat as air (no chunks above)
                Some(crate::block::BlockType::Air)
            } else if y > 0 && y < crate::chunk::CHUNK_HEIGHT {
                // Within chunk bounds
                chunk_data.get_block(z, y, x)
            } else {
                // Below chunk (y == 0), treat as air (no chunks below)
                Some(crate::block::BlockType::Air)
            }
        },
        Direction::Z => {
            // Z direction: slice = X, y = Y, x = Z
            if z >= crate::chunk::CHUNK_SIZE {
                // Need to check south neighbor chunk
                let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x, chunk_pos.z + 1);
                if let Some(&neighbor_entity) = chunk_manager.loaded_chunks.get(&neighbor_pos) {
                    if let Ok(neighbor_chunk) = chunks.get(neighbor_entity) {
                        // In south neighbor, we're at local_z = 0
                        return neighbor_chunk.data.get_block(x, y, 0);
                    }
                }
                // No neighbor chunk or neighbor chunk not loaded, treat as air
                Some(crate::block::BlockType::Air)
            } else {
                // Within chunk bounds
                chunk_data.get_block(x, y, z)
            }
        },
    }
}

/// Add a quad for greedy meshing
fn add_greedy_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    x: usize,
    y: usize,
    slice: usize,
    width: usize,
    height: usize,
    direction: Direction,
    block_type: crate::block::BlockType,
) {
    let base_index = positions.len() as u32;
    
    // Calculate the normal based on direction
    let normal = match direction {
        Direction::X => [1.0, 0.0, 0.0], // Right face
        Direction::Y => [0.0, 1.0, 0.0], // Top face
        Direction::Z => [0.0, 0.0, 1.0], // Front face
    };

    // Calculate the quad vertices based on direction
    let (v0, v1, v2, v3) = match direction {
        Direction::X => {
            // Right face quad
            let x_pos = slice as f32 + 1.0;
            (
                [x_pos, y as f32, x as f32],
                [x_pos, y as f32, x as f32 + height as f32],
                [x_pos, y as f32 + width as f32, x as f32 + height as f32],
                [x_pos, y as f32 + width as f32, x as f32],
            )
        },
        Direction::Y => {
            // Top face quad
            let y_pos = slice as f32 + 1.0;
            let z_val = x; // Use the x parameter as z coordinate for Y direction
            (
                [x as f32, y_pos, z_val as f32],
                [x as f32 + width as f32, y_pos, z_val as f32],
                [x as f32 + width as f32, y_pos, z_val as f32 + height as f32],
                [x as f32, y_pos, z_val as f32 + height as f32],
            )
        },
        Direction::Z => {
            // Front face quad
            let z_pos = slice as f32 + 1.0;
            (
                [x as f32, y as f32, z_pos],
                [x as f32 + width as f32, y as f32, z_pos],
                [x as f32 + width as f32, y as f32 + height as f32, z_pos],
                [x as f32, y as f32 + height as f32, z_pos],
            )
        },
    };

    positions.extend_from_slice(&[v0, v1, v2, v3]);
    normals.extend_from_slice(&[normal, normal, normal, normal]);

    // UV coordinates
    let uv = get_block_uv(block_type);
    uvs.extend_from_slice(&[
        [uv.0, uv.1],
        [uv.2, uv.1],
        [uv.2, uv.3],
        [uv.0, uv.3],
    ]);

    // Indices for two triangles
    indices.extend_from_slice(&[base_index, base_index + 1, base_index + 2, base_index, base_index + 2, base_index + 3]);
}