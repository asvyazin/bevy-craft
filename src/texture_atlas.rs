// Texture atlas system for Bevy Craft
// This module handles loading and managing the texture atlas for block textures

use bevy::prelude::*;
use std::collections::HashMap;

use crate::block::BlockType;

/// Resource that stores the loaded texture atlas
#[derive(Resource, Debug)]
pub struct TextureAtlas {
    /// Handle to the texture atlas image
    pub texture_handle: Handle<Image>,
    /// Map of block types to their UV coordinates in the atlas
    pub block_uvs: HashMap<BlockType, (f32, f32, f32, f32)>,  // (u_min, v_min, u_max, v_max)
    /// Flag indicating if the atlas is loaded
    pub is_loaded: bool,
}

impl Default for TextureAtlas {
    fn default() -> Self {
        Self {
            texture_handle: Handle::default(),
            block_uvs: HashMap::new(),
            is_loaded: false,
        }
    }
}

impl TextureAtlas {
    /// Initialize the texture atlas by loading the image and setting up UV coordinates
    pub fn initialize(
        &mut self,
        asset_server: &Res<AssetServer>,
        _images: &mut ResMut<Assets<Image>>,
    ) {
        println!("🎨 Initializing texture atlas...");
        
        // Load the texture atlas image
        let texture_handle = asset_server.load("textures/block_atlas.png");
        
        // Set up UV coordinates for each block type
        // The texture atlas is organized in a 4x2 grid:
        // Row 0 (Top): Grass, Dirt, Stone, Wood
        // Row 1 (Bottom): Leaves, Sand, Water, Bedrock
        
        const CELL_WIDTH: f32 = 0.25;  // 1/4 of texture width
        const CELL_HEIGHT: f32 = 0.5;   // 1/2 of texture height
        
        // Initialize UV coordinates for each block type
        self.block_uvs.insert(BlockType::Grass, (0.0, 0.0, CELL_WIDTH, CELL_HEIGHT));
        self.block_uvs.insert(BlockType::Dirt, (CELL_WIDTH, 0.0, CELL_WIDTH * 2.0, CELL_HEIGHT));
        self.block_uvs.insert(BlockType::Stone, (CELL_WIDTH * 2.0, 0.0, CELL_WIDTH * 3.0, CELL_HEIGHT));
        self.block_uvs.insert(BlockType::Wood, (CELL_WIDTH * 3.0, 0.0, CELL_WIDTH * 4.0, CELL_HEIGHT));
        self.block_uvs.insert(BlockType::Leaves, (0.0, CELL_HEIGHT, CELL_WIDTH, CELL_HEIGHT * 2.0));
        self.block_uvs.insert(BlockType::Sand, (CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH * 2.0, CELL_HEIGHT * 2.0));
        self.block_uvs.insert(BlockType::Water, (CELL_WIDTH * 2.0, CELL_HEIGHT, CELL_WIDTH * 3.0, CELL_HEIGHT * 2.0));
        self.block_uvs.insert(BlockType::Bedrock, (CELL_WIDTH * 3.0, CELL_HEIGHT, CELL_WIDTH * 4.0, CELL_HEIGHT * 2.0));
        
        self.texture_handle = texture_handle;
        self.is_loaded = true;
        
        println!("✓ Texture atlas initialized successfully");
    }
    
    /// Get UV coordinates for a specific block type
    pub fn get_uv(&self, block_type: BlockType) -> (f32, f32, f32, f32) {
        self.block_uvs.get(&block_type)
            .copied()
            .unwrap_or((0.0, 0.0, 1.0, 1.0))  // Default fallback
    }
    
    /// Check if the texture atlas is loaded
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }
    
    /// Get the texture handle
    pub fn texture_handle(&self) -> &Handle<Image> {
        &self.texture_handle
    }
}

/// System to initialize the texture atlas
pub fn initialize_texture_atlas(
    mut texture_atlas: ResMut<TextureAtlas>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    texture_atlas.initialize(&asset_server, &mut images);
}