// Test module for verifying alkyd-enhanced texture generation

use bevy::prelude::*;
use crate::alkyd_integration::{AlkydTextureConfig, generate_alkyd_texture_data, generate_fallback_texture_data, EnhancedBlockTextures};

pub fn test_alkyd_enhanced_textures(
    enhanced_textures: Res<EnhancedBlockTextures>,
) {
    println!("🧪 Testing alkyd-enhanced texture generation...");
    
    // Verify that enhanced textures were generated
    let expected_block_types = ["stone", "dirt", "grass", "wood", "sand", "water", "bedrock", "leaves"];
    
    for block_type in expected_block_types {
        if let Some(texture_handle) = enhanced_textures.textures.get(block_type) {
            println!("✓ {} texture generated: {:?}", block_type, texture_handle);
        } else {
            println!("✗ {} texture missing!", block_type);
        }
    }
    
    // Test configuration settings
    for block_type in expected_block_types {
        if let Some(config) = enhanced_textures.texture_configs.get(block_type) {
            println!("✓ {} config: GPU={}, Noise={}, Edge={}, Blend={}", 
                     block_type, config.use_gpu_acceleration, config.noise_type, 
                     config.enable_edge_detection, config.enable_color_blending);
        }
    }
    
    println!("✓ Alkyd-enhanced texture generation test completed");
}

pub fn test_texture_data_generation() {
    println!("🧪 Testing texture data generation functions...");
    
    // Test different block type configurations
    let block_types = ["stone", "dirt", "grass", "wood", "sand"];
    
    for block_type in block_types {
        let config = AlkydTextureConfig::for_block_type(block_type);
        
        // Test CPU fallback generation
        let cpu_data = generate_fallback_texture_data(&config);
        let expected_size = (config.texture_size.x * config.texture_size.y * 4) as usize;
        
        if cpu_data.len() == expected_size {
            println!("✓ {} CPU texture data: {} bytes (correct)", block_type, cpu_data.len());
        } else {
            println!("✗ {} CPU texture data: {} bytes (expected {})", block_type, cpu_data.len(), expected_size);
        }
        
        // Test enhanced generation
        let enhanced_data = generate_alkyd_texture_data(&config);
        
        if enhanced_data.len() == expected_size {
            println!("✓ {} Enhanced texture data: {} bytes (correct)", block_type, enhanced_data.len());
        } else {
            println!("✗ {} Enhanced texture data: {} bytes (expected {})", block_type, enhanced_data.len(), expected_size);
        }
    }
    
    println!("✓ Texture data generation test completed");
}

pub fn setup_alkyd_test_systems(app: &mut App) {
    app
        .add_systems(Startup, test_texture_data_generation)
        .add_systems(Update, test_alkyd_enhanced_textures);
}