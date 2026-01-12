// Test system to verify procedural texture integration
use bevy::prelude::*;
use crate::texture_atlas::TextureAtlas;
use crate::texture_gen::BlockTextures;

/// System to test procedural texture integration
pub fn test_procedural_texture_integration(
    texture_atlas: Res<TextureAtlas>,
    block_textures: Res<BlockTextures>,
) {
    println!("🧪 Testing procedural texture integration...");
    
    // Check if BlockTextures has any textures
    println!("  BlockTextures count: {}", block_textures.textures.len());
    
    // List all textures in BlockTextures
    for (name, handle) in &block_textures.textures {
        println!("  Found texture: {} - {:?}", name, handle);
    }
    
    // Check if TextureAtlas has procedural textures loaded
    println!("  TextureAtlas has_procedural_textures: {}", texture_atlas.has_procedural_textures());
    println!("  TextureAtlas procedural_textures count: {}", texture_atlas.procedural_textures.len());
    
    // List all procedural textures in TextureAtlas
    for (block_type, handle) in &texture_atlas.procedural_textures {
        println!("  TextureAtlas has procedural texture for {:?} - {:?}", block_type, handle);
    }
    
    // Verify the integration is working
    if texture_atlas.has_procedural_textures() && !texture_atlas.procedural_textures.is_empty() {
        println!("✅ Procedural texture integration is working!");
    } else {
        println!("❌ Procedural texture integration is NOT working!");
        
        // Debug: Check if BlockTextures is empty
        if block_textures.textures.is_empty() {
            println!("  Issue: BlockTextures resource is empty");
        }
        
        // Debug: Check if the loading failed
        if !block_textures.textures.is_empty() && texture_atlas.procedural_textures.is_empty() {
            println!("  Issue: TextureAtlas failed to load textures from BlockTextures");
        }
    }
}