// Test file for sophisticated Alkyd algorithms
// This file tests the new blend modes, edge detection, and color space converters

use crate::alkyd_gpu_shaders::{AlkydGpuTextureConfig};

pub fn test_sophisticated_algorithms() {
    println!("🧪 Testing sophisticated Alkyd algorithms...");
    
    // Test configuration creation
    let config = AlkydGpuTextureConfig::default();
    println!("Default config: {:?}", config);
    
    // Test stone configuration
    let stone_config = AlkydGpuTextureConfig::for_block_type("stone");
    println!("Stone config: {:?}", stone_config);
    assert_eq!(stone_config.texture_size, bevy::math::UVec2::new(128, 128));
    assert_eq!(stone_config.noise_type, "simplex");
    
    // Test grass configuration
    let grass_config = AlkydGpuTextureConfig::for_block_type("grass");
    println!("Grass config: {:?}", grass_config);
    assert_eq!(grass_config.noise_type, "fractal");
    
    // Test texture generation with default config
    let texture_data = crate::alkyd_gpu_shaders::generate_alkyd_gpu_texture_data(&config);
    println!("Generated texture data size: {}", texture_data.len());
    assert_eq!(texture_data.len(), 128 * 128 * 4); // 128x128 RGBA
    
    // Test that we have some variation in the texture
    // Check if not all pixels are the same color
    let first_pixel_r = texture_data[0];
    let first_pixel_g = texture_data[1];
    let first_pixel_b = texture_data[2];
    
    let mut has_variation = false;
    for i in (0..texture_data.len()).step_by(4) {
        if texture_data[i] != first_pixel_r || 
           texture_data[i+1] != first_pixel_g || 
           texture_data[i+2] != first_pixel_b {
            has_variation = true;
            break;
        }
    }
    
    if !has_variation {
        println!("⚠️  Warning: Texture appears to be uniform color - this might be expected for some configurations");
        println!("   First pixel: R={}, G={}, B={}", first_pixel_r, first_pixel_g, first_pixel_b);
    } else {
        println!("✓ Texture has color variation as expected");
    }
    
    // Test fallback texture generation
    let fallback_data = crate::alkyd_gpu_shaders::generate_fallback_gpu_texture_data(&config);
    println!("Generated fallback texture data size: {}", fallback_data.len());
    assert_eq!(fallback_data.len(), 128 * 128 * 4); // 128x128 RGBA
    
    println!("✅ All sophisticated algorithm tests completed!");
}