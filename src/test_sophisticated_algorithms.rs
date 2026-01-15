// Test file for sophisticated Alkyd algorithms
// This file tests the new blend modes, edge detection, and color space converters

use crate::alkyd_gpu_shaders::{apply_alkyd_blend_mode, apply_sobel_edge_detection, convert_color_space, AlkydGpuTextureConfig};

#[test]
fn test_alkyd_blend_modes() {
    // Test color for blending
    let color = [128u8, 128u8, 128u8, 255u8];
    let noise_value = 0.5f32;
    let config = AlkydGpuTextureConfig::default();
    let texture_data = vec![];
    
    // Test multiply blend mode
    let result = apply_alkyd_blend_mode(&color, noise_value, "multiply", &texture_data, 0, 0, &config);
    println!("Multiply blend mode result: {:?}", result);
    
    // Test screen blend mode
    let result = apply_alkyd_blend_mode(&color, noise_value, "screen", &texture_data, 0, 0, &config);
    println!("Screen blend mode result: {:?}", result);
    
    // Test overlay blend mode
    let result = apply_alkyd_blend_mode(&color, noise_value, "overlay", &texture_data, 0, 0, &config);
    println!("Overlay blend mode result: {:?}", result);
    
    // Test soft_light blend mode
    let result = apply_alkyd_blend_mode(&color, noise_value, "soft_light", &texture_data, 0, 0, &config);
    println!("Soft light blend mode result: {:?}", result);
}

#[test]
fn test_color_space_conversion() {
    // Test color: bright red
    let color = [255u8, 0u8, 0u8, 255u8];
    
    // Convert from sRGB to linear
    let linear_color = convert_color_space(&color, "srgb", "linear");
    println!("sRGB to Linear: {:?}", linear_color);
    
    // Convert back to sRGB
    let back_to_srgb = convert_color_space(&linear_color, "linear", "srgb");
    println!("Linear to sRGB: {:?}", back_to_srgb);
    
    // Test HSV conversion
    let hsv_color = convert_color_space(&color, "srgb", "hsv");
    println!("sRGB to HSV: {:?}", hsv_color);
    
    // Test HSL conversion
    let hsl_color = convert_color_space(&color, "srgb", "hsl");
    println!("sRGB to HSL: {:?}", hsl_color);
}

#[test]
fn test_sobel_edge_detection() {
    // Create a simple test texture with a gradient
    let width = 4;
    let height = 4;
    let mut texture_data = Vec::new();
    
    // Create a simple gradient texture
    for y in 0..height {
        for x in 0..width {
            let intensity = (x + y) as u8 * 32;
            texture_data.extend_from_slice(&[intensity, intensity, intensity, 255]);
        }
    }
    
    // Test color in the middle
    let color = [128u8, 128u8, 128u8, 255u8];
    let config = AlkydGpuTextureConfig::default();
    
    // Apply Sobel edge detection
    let result = apply_sobel_edge_detection(&color, 1, 1, &config, &texture_data, width, height);
    println!("Sobel edge detection result: {:?}", result);
}

pub fn test_sophisticated_algorithms() {
    println!("🧪 Testing sophisticated Alkyd algorithms...");
    
    test_alkyd_blend_modes();
    test_color_space_conversion();
    test_sobel_edge_detection();
    
    println!("✅ All sophisticated algorithm tests completed!");
}