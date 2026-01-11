// Texture atlas system for Bevy Craft
// This module handles loading and managing the texture atlas for block textures

use bevy::prelude::*;
use std::collections::HashMap;

use crate::block::BlockType;
use crate::texture_gen::BlockTextures;

/// Enum representing different faces of a block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    Side,
}

/// Resource that stores the loaded texture atlas
#[derive(Resource, Debug)]
pub struct TextureAtlas {
    /// Handle to the texture atlas image
    pub texture_handle: Handle<Image>,
    /// Map of block types to their UV coordinates in the atlas for each face
    pub block_face_uvs: HashMap<BlockType, HashMap<BlockFace, (f32, f32, f32, f32)>>,  // (u_min, v_min, u_max, v_max)
    /// Map of block types to their procedural texture handles
    pub procedural_textures: HashMap<BlockType, Handle<Image>>,
    /// Flag indicating if the atlas is loaded
    pub is_loaded: bool,
    /// Flag indicating if procedural textures are available
    pub has_procedural_textures: bool,
}

impl Default for TextureAtlas {
    fn default() -> Self {
        Self {
            texture_handle: Handle::default(),
            block_face_uvs: HashMap::new(),
            procedural_textures: HashMap::new(),
            is_loaded: false,
            has_procedural_textures: false,
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
        println!("🎨 Initializing texture atlas with face-specific textures...");
        
        // Load the texture atlas image
        let texture_handle = asset_server.load("textures/block_atlas.png");
        
        // The existing texture is 512x256 (2:1 aspect ratio)
        // We'll adapt the face-specific system to work with this texture
        // by using a logical 4x2 grid where we can simulate face-specific textures
        
        const CELL_WIDTH: f32 = 0.25;  // 1/4 of texture width
        const CELL_HEIGHT: f32 = 0.5;   // 1/2 of texture height
        
        // Initialize face-specific UV coordinates for each block type
        // For now, we'll use the existing texture layout but provide face-specific mappings
        
        // Grass block: use top row for top face, bottom row for side/bottom faces
        let mut grass_uvs = HashMap::new();
        grass_uvs.insert(BlockFace::Top, (0.0, 0.0, CELL_WIDTH, CELL_HEIGHT));
        grass_uvs.insert(BlockFace::Side, (0.0, CELL_HEIGHT, CELL_WIDTH, CELL_HEIGHT * 2.0));
        grass_uvs.insert(BlockFace::Bottom, (0.0, CELL_HEIGHT, CELL_WIDTH, CELL_HEIGHT * 2.0));
        self.block_face_uvs.insert(BlockType::Grass, grass_uvs);
        
        // Dirt block: use top row for top face, bottom row for side/bottom faces
        let mut dirt_uvs = HashMap::new();
        dirt_uvs.insert(BlockFace::Top, (CELL_WIDTH, 0.0, CELL_WIDTH * 2.0, CELL_HEIGHT));
        dirt_uvs.insert(BlockFace::Side, (CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH * 2.0, CELL_HEIGHT * 2.0));
        dirt_uvs.insert(BlockFace::Bottom, (CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH * 2.0, CELL_HEIGHT * 2.0));
        self.block_face_uvs.insert(BlockType::Dirt, dirt_uvs);
        
        // Stone block: same texture for all faces
        let mut stone_uvs = HashMap::new();
        let stone_uv = (CELL_WIDTH * 2.0, 0.0, CELL_WIDTH * 3.0, CELL_HEIGHT);
        stone_uvs.insert(BlockFace::Top, stone_uv);
        stone_uvs.insert(BlockFace::Side, stone_uv);
        stone_uvs.insert(BlockFace::Bottom, stone_uv);
        self.block_face_uvs.insert(BlockType::Stone, stone_uvs);
        
        // Wood block: use top row for top/bottom, bottom row for sides
        let mut wood_uvs = HashMap::new();
        wood_uvs.insert(BlockFace::Top, (CELL_WIDTH * 3.0, 0.0, CELL_WIDTH * 4.0, CELL_HEIGHT));
        wood_uvs.insert(BlockFace::Bottom, (CELL_WIDTH * 3.0, 0.0, CELL_WIDTH * 4.0, CELL_HEIGHT));
        wood_uvs.insert(BlockFace::Side, (CELL_WIDTH * 3.0, CELL_HEIGHT, CELL_WIDTH * 4.0, CELL_HEIGHT * 2.0));
        self.block_face_uvs.insert(BlockType::Wood, wood_uvs);
        
        // Leaves block: same texture for all faces
        let mut leaves_uvs = HashMap::new();
        let leaves_uv = (0.0, CELL_HEIGHT, CELL_WIDTH, CELL_HEIGHT * 2.0);
        leaves_uvs.insert(BlockFace::Top, leaves_uv);
        leaves_uvs.insert(BlockFace::Side, leaves_uv);
        leaves_uvs.insert(BlockFace::Bottom, leaves_uv);
        self.block_face_uvs.insert(BlockType::Leaves, leaves_uvs);
        
        // Sand block: same texture for all faces
        let mut sand_uvs = HashMap::new();
        let sand_uv = (CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH * 2.0, CELL_HEIGHT * 2.0);
        sand_uvs.insert(BlockFace::Top, sand_uv);
        sand_uvs.insert(BlockFace::Side, sand_uv);
        sand_uvs.insert(BlockFace::Bottom, sand_uv);
        self.block_face_uvs.insert(BlockType::Sand, sand_uvs);
        
        // Water block: same texture for all faces
        let mut water_uvs = HashMap::new();
        let water_uv = (CELL_WIDTH * 2.0, CELL_HEIGHT, CELL_WIDTH * 3.0, CELL_HEIGHT * 2.0);
        water_uvs.insert(BlockFace::Top, water_uv);
        water_uvs.insert(BlockFace::Side, water_uv);
        water_uvs.insert(BlockFace::Bottom, water_uv);
        self.block_face_uvs.insert(BlockType::Water, water_uvs);
        
        // Bedrock block: same texture for all faces
        let mut bedrock_uvs = HashMap::new();
        let bedrock_uv = (CELL_WIDTH * 3.0, CELL_HEIGHT, CELL_WIDTH * 4.0, CELL_HEIGHT * 2.0);
        bedrock_uvs.insert(BlockFace::Top, bedrock_uv);
        bedrock_uvs.insert(BlockFace::Side, bedrock_uv);
        bedrock_uvs.insert(BlockFace::Bottom, bedrock_uv);
        self.block_face_uvs.insert(BlockType::Bedrock, bedrock_uvs);
        
        self.texture_handle = texture_handle;
        self.is_loaded = true;
        
        println!("✓ Texture atlas with face-specific textures initialized successfully");
        println!("  - Grass: Top face uses grass texture, sides use dirt texture");
        println!("  - Dirt: Top face uses dirt texture, sides use dirt texture");
        println!("  - Wood: Top/bottom use wood top, sides use wood side");
        println!("  - Other blocks: Same texture for all faces");
    }
    
    /// Get UV coordinates for a specific block type and face
    pub fn get_uv(&self, block_type: BlockType, face: BlockFace) -> (f32, f32, f32, f32) {
        self.block_face_uvs.get(&block_type)
            .and_then(|face_uvs| face_uvs.get(&face))
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
    
    /// Load procedural textures from BlockTextures resource
    pub fn load_procedural_textures(
        &mut self,
        block_textures: &Res<BlockTextures>,
    ) {
        println!("🎨 Loading procedural textures into texture atlas...");
        
        // Map block type names to BlockType enum
        let block_type_mapping = [
            ("stone", BlockType::Stone),
            ("dirt", BlockType::Dirt),
            ("grass", BlockType::Grass),
            ("wood", BlockType::Wood),
            ("sand", BlockType::Sand),
            ("water", BlockType::Water),
            ("bedrock", BlockType::Bedrock),
            ("leaves", BlockType::Leaves),
        ];
        
        for (name, block_type) in block_type_mapping {
            if let Some(texture_handle) = block_textures.textures.get(name) {
                self.procedural_textures.insert(block_type, texture_handle.clone());
                println!("  ✓ Loaded procedural texture for {:?}", block_type);
            }
        }
        
        self.has_procedural_textures = !self.procedural_textures.is_empty();
        
        if self.has_procedural_textures {
            println!("✓ Procedural textures loaded successfully");
        } else {
            println!("⚠️  No procedural textures found");
        }
    }
    
    /// Get procedural texture handle for a block type, if available
    pub fn get_procedural_texture(&self, block_type: BlockType) -> Option<&Handle<Image>> {
        self.procedural_textures.get(&block_type)
    }
    
    /// Check if procedural textures are available
    pub fn has_procedural_textures(&self) -> bool {
        self.has_procedural_textures
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

/// System to load procedural textures into the texture atlas
pub fn load_procedural_textures_into_atlas(
    mut texture_atlas: ResMut<TextureAtlas>,
    block_textures: Res<BlockTextures>,
) {
    texture_atlas.load_procedural_textures(&block_textures);
}