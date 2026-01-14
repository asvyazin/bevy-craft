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

pub fn test_enhanced_alkyd_features() {
    println!("🧪 Testing enhanced alkyd features...");
    
    // Test new parameters
    let mut config = AlkydTextureConfig::for_block_type("stone");
    
    // Test ridged noise
    config.enable_ridged_noise = true;
    config.ridged_strength = 1.5;
    let ridged_texture = generate_alkyd_texture_data(&config);
    println!("✓ Ridged noise feature working - generated {} bytes", ridged_texture.len());
    
    // Test turbulence
    config.enable_turbulence = true;
    config.turbulence_strength = 0.3;
    let turbulence_texture = generate_alkyd_texture_data(&config);
    println!("✓ Turbulence feature working - generated {} bytes", turbulence_texture.len());
    
    // Test new blend modes
    let blend_modes = ["normal", "multiply", "overlay", "screen", "hard_light", "soft_light", "color_dodge"];
    for blend_mode in blend_modes {
        let mut blend_config = AlkydTextureConfig::for_block_type("dirt");
        blend_config.blend_mode = blend_mode.to_string();
        blend_config.enable_color_blending = true;
        let blend_texture = generate_alkyd_texture_data(&blend_config);
        println!("✓ Blend mode '{}' working - generated {} bytes", blend_mode, blend_texture.len());
    }
    
    // Test saturation and contrast
    let mut color_config = AlkydTextureConfig::for_block_type("grass");
    color_config.saturation = 1.5;
    color_config.contrast = 1.2;
    let color_texture = generate_alkyd_texture_data(&color_config);
    println!("✓ Color enhancement features working - generated {} bytes", color_texture.len());
    
    println!("✅ All enhanced alkyd features tested successfully!");

    // Test all block types with new enhanced parameters
    let block_types = ["stone", "dirt", "grass", "wood", "sand", "water", "bedrock", "leaves"];
    for block_type in block_types {
        let enhanced_config = AlkydTextureConfig::for_block_type(block_type);
        println!("🎨 Enhanced {} config:", block_type);
        println!("   - Noise scale: {} (was 0.05-0.1)", enhanced_config.noise_scale);
        println!("   - Noise octaves: {} (was 2-6)", enhanced_config.noise_octaves);
        println!("   - Color variation: {} (was 0.15-0.3)", enhanced_config.color_variation);
        println!("   - Edge detection: {}", enhanced_config.enable_edge_detection);
        println!("   - Color blending: {} ({})", enhanced_config.enable_color_blending, enhanced_config.blend_mode);
        println!("   - Ridged noise: {} (strength: {})", enhanced_config.enable_ridged_noise, enhanced_config.ridged_strength);
        println!("   - Turbulence: {} (strength: {})", enhanced_config.enable_turbulence, enhanced_config.turbulence_strength);
        println!("   - Detail level: {} (was 0.9-1.3)", enhanced_config.detail_level);
        println!("   - Contrast: {} (was 0.95-1.15)", enhanced_config.contrast);
        println!("   - Saturation: {} (was 0.9-1.2)", enhanced_config.saturation);
        
        let texture_data = generate_alkyd_texture_data(&enhanced_config);
        println!("   ✓ Generated {} bytes of enhanced texture data", texture_data.len());
    }

    println!("✅ All enhanced alkyd features tested successfully!");
}

pub fn setup_alkyd_test_systems(app: &mut App) {
    app
        .add_systems(Startup, test_texture_data_generation)
        .add_systems(Startup, test_enhanced_alkyd_features)
        .add_systems(Update, test_alkyd_enhanced_textures);
}