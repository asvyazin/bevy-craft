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