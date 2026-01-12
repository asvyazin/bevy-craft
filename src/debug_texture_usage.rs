// Debug system to verify which textures are actually being used in rendering
use bevy::prelude::*;
use crate::texture_atlas::TextureAtlas;
use crate::chunk_mesh::ChunkMeshMaterials;

use crate::initialize_chunk_mesh_materials;

/// System to debug texture usage and verify procedural textures are being used
pub fn debug_texture_usage(
    texture_atlas: Res<TextureAtlas>,
    chunk_materials: Res<ChunkMeshMaterials>,
) {
    println!("🔍 Debugging texture usage...");
    
    // Check if procedural textures are available
    println!("  Procedural textures available: {}", texture_atlas.has_procedural_textures());
    println!("  Procedural textures count: {}", texture_atlas.procedural_textures.len());
    
    // Check which materials are being used
    println!("  Chunk materials count: {}", chunk_materials.materials.len());
    
    // List all materials and their types
    for (block_type, material_handle) in &chunk_materials.materials {
        println!("  Material for {:?}: {:?}", block_type, material_handle);
    }
    
    // Check if procedural materials are being used
    let using_procedural = chunk_materials.materials.len() > 1; // If more than default material, likely using procedural
    
    if using_procedural {
        println!("✅ Game is using procedural textures!");
    } else {
        println!("❌ Game is using atlas textures (not procedural)");
        
        // Debug: Check why procedural textures might not be used
        if !texture_atlas.has_procedural_textures() {
            println!("  Reason: No procedural textures available in atlas");
        } else if chunk_materials.materials.is_empty() {
            println!("  Reason: No materials created");
        } else {
            println!("  Reason: Unknown - materials exist but not using procedural textures");
        }
    }
}

/// System to add debug texture usage check to the app
pub fn setup_debug_texture_usage(app: &mut App) {
    app.add_systems(Startup, debug_texture_usage.after(initialize_chunk_mesh_materials));
}