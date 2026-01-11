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

/// Check which faces of a block should be visible
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

/// Check face visibility for a block at given position
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
    
    // Helper function to check if a block should be rendered (is air or transparent)
    let should_render_face = |block_type: Option<BlockType>| {
        block_type.map_or(true, |bt| bt == BlockType::Air || bt.is_transparent())
    };
    
    // Check front face (positive Z direction)
    if local_z == crate::chunk::CHUNK_SIZE - 1 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x, chunk_pos.z + 1);
        visibility.front = chunk_manager.get_neighbor_block(
            chunks, chunk_pos, &neighbor_pos, local_x, y, 0
        ).map_or(true, |bt| bt == BlockType::Air || bt.is_transparent());
    } else {
        // Within chunk, check adjacent block
        visibility.front = should_render_face(chunk_data.get_block(local_x, y, local_z + 1));
    }
    
    // Check back face (negative Z direction)
    if local_z == 0 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x, chunk_pos.z - 1);
        visibility.back = chunk_manager.get_neighbor_block(
            chunks, chunk_pos, &neighbor_pos, local_x, y, crate::chunk::CHUNK_SIZE - 1
        ).map_or(true, |bt| bt == BlockType::Air || bt.is_transparent());
    } else {
        // Within chunk, check adjacent block
        visibility.back = should_render_face(chunk_data.get_block(local_x, y, local_z - 1));
    }
    
    // Check right face (positive X direction)
    if local_x == crate::chunk::CHUNK_SIZE - 1 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x + 1, chunk_pos.z);
        visibility.right = chunk_manager.get_neighbor_block(
            chunks, chunk_pos, &neighbor_pos, 0, y, local_z
        ).map_or(true, |bt| bt == BlockType::Air || bt.is_transparent());
    } else {
        // Within chunk, check adjacent block
        visibility.right = should_render_face(chunk_data.get_block(local_x + 1, y, local_z));
    }
    
    // Check left face (negative X direction)
    if local_x == 0 {
        // At chunk boundary, check neighbor chunk
        let neighbor_pos = crate::chunk::ChunkPosition::new(chunk_pos.x - 1, chunk_pos.z);
        visibility.left = chunk_manager.get_neighbor_block(
            chunks, chunk_pos, &neighbor_pos, crate::chunk::CHUNK_SIZE - 1, y, local_z
        ).map_or(true, |bt| bt == BlockType::Air || bt.is_transparent());
    } else {
        // Within chunk, check adjacent block
        visibility.left = should_render_face(chunk_data.get_block(local_x - 1, y, local_z));
    }
    
    // Check top face (positive Y direction)
    if y < crate::chunk::CHUNK_HEIGHT - 1 {
        // Within chunk, check adjacent block
        visibility.top = should_render_face(chunk_data.get_block(local_x, y + 1, local_z));
    } else {
        // At top of chunk, always render (no chunks above)
        visibility.top = true;
    }
    
    // Check bottom face (negative Y direction)
    if y > 0 {
        // Within chunk, check adjacent block
        visibility.bottom = should_render_face(chunk_data.get_block(local_x, y - 1, local_z));
    } else {
        // At bottom of chunk, always render (no chunks below)
        visibility.bottom = true;
    }
    
    visibility
}

/// Add a single face to the mesh
fn add_face(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    base_index: u32,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    uv: (f32, f32, f32, f32),
) {
    positions.extend_from_slice(&vertices);
    normals.extend_from_slice(&[normal; 4]);
    
    uvs.extend_from_slice(&[
        [uv.0, uv.1],
        [uv.2, uv.1],
        [uv.2, uv.3],
        [uv.0, uv.3],
    ]);
    
    indices.extend_from_slice(&[base_index, base_index + 1, base_index + 2, base_index, base_index + 2, base_index + 3]);
}

/// Generate mesh for a chunk with neighbor awareness
pub fn generate_chunk_mesh(
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
                    if block_type != BlockType::Air {
                        // Check which faces should be rendered
                        let visibility = check_face_visibility(
                            chunk_data, chunk_pos, chunk_manager, chunks, local_x, y, local_z
                        );
                        
                        // If any face should be rendered, add the block mesh
                        if visibility.any() {
                            add_block_mesh(
                                &mut positions,
                                &mut normals,
                                &mut uvs,
                                &mut indices,
                                local_x,
                                y,
                                local_z,
                                &visibility,
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
    block_type: BlockType,
) {
    let base_index = positions.len() as u32;
    let uv = get_block_uv(block_type);
    let mut current_index = base_index;
    
    // Front face (positive Z)
    if visibility.front {
        let z = local_z as f32 + 1.0;
        let vertices = [
            [local_x as f32, y as f32, z],
            [local_x as f32 + 1.0, y as f32, z],
            [local_x as f32 + 1.0, y as f32 + 1.0, z],
            [local_x as f32, y as f32 + 1.0, z],
        ];
        add_face(positions, normals, uvs, indices, current_index, vertices, [0.0, 0.0, 1.0], uv);
        current_index += 4;
    }
    
    // Back face (negative Z)
    if visibility.back {
        let z = local_z as f32;
        let vertices = [
            [local_x as f32 + 1.0, y as f32, z],
            [local_x as f32, y as f32, z],
            [local_x as f32, y as f32 + 1.0, z],
            [local_x as f32 + 1.0, y as f32 + 1.0, z],
        ];
        add_face(positions, normals, uvs, indices, current_index, vertices, [0.0, 0.0, -1.0], uv);
        current_index += 4;
    }
    
    // Right face (positive X)
    if visibility.right {
        let x = local_x as f32 + 1.0;
        let vertices = [
            [x, y as f32, local_z as f32 + 1.0],
            [x, y as f32, local_z as f32],
            [x, y as f32 + 1.0, local_z as f32],
            [x, y as f32 + 1.0, local_z as f32 + 1.0],
        ];
        add_face(positions, normals, uvs, indices, current_index, vertices, [1.0, 0.0, 0.0], uv);
        current_index += 4;
    }
    
    // Left face (negative X)
    if visibility.left {
        let x = local_x as f32;
        let vertices = [
            [x, y as f32, local_z as f32],
            [x, y as f32, local_z as f32 + 1.0],
            [x, y as f32 + 1.0, local_z as f32 + 1.0],
            [x, y as f32 + 1.0, local_z as f32],
        ];
        add_face(positions, normals, uvs, indices, current_index, vertices, [-1.0, 0.0, 0.0], uv);
        current_index += 4;
    }
    
    // Top face (positive Y)
    if visibility.top {
        let y_top = y as f32 + 1.0;
        let vertices = [
            [local_x as f32, y_top, local_z as f32 + 1.0],
            [local_x as f32 + 1.0, y_top, local_z as f32 + 1.0],
            [local_x as f32 + 1.0, y_top, local_z as f32],
            [local_x as f32, y_top, local_z as f32],
        ];
        add_face(positions, normals, uvs, indices, current_index, vertices, [0.0, 1.0, 0.0], uv);
        current_index += 4;
    }
    
    // Bottom face (negative Y)
    if visibility.bottom {
        let y_bottom = y as f32;
        let vertices = [
            [local_x as f32, y_bottom, local_z as f32],
            [local_x as f32 + 1.0, y_bottom, local_z as f32],
            [local_x as f32 + 1.0, y_bottom, local_z as f32 + 1.0],
            [local_x as f32, y_bottom, local_z as f32 + 1.0],
        ];
        add_face(positions, normals, uvs, indices, current_index, vertices, [0.0, -1.0, 0.0], uv);
    }
}

/// Get UV coordinates for a block type
fn get_block_uv(block_type: BlockType) -> (f32, f32, f32, f32) {
    // Simple UV mapping - all blocks use the same texture for now
    // This will be replaced with a proper texture atlas later
    match block_type {
        BlockType::Grass => (0.0, 0.0, 1.0, 1.0),
        BlockType::Dirt => (0.0, 0.0, 1.0, 1.0),
        BlockType::Stone => (0.0, 0.0, 1.0, 1.0),
        BlockType::Wood => (0.0, 0.0, 1.0, 1.0),
        BlockType::Leaves => (0.0, 0.0, 1.0, 1.0),
        BlockType::Sand => (0.0, 0.0, 1.0, 1.0),
        BlockType::Water => (0.0, 0.0, 1.0, 1.0),
        BlockType::Bedrock => (0.0, 0.0, 1.0, 1.0),
        _ => (0.0, 0.0, 1.0, 1.0),
    }
}
