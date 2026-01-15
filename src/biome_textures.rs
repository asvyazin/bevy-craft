// Biome-based texture parameterization system
// This module handles texture variations based on biome and height parameters

use bevy::prelude::*;
use std::collections::HashMap;

use crate::block::BlockType;
use crate::alkyd_integration::AlkydTextureConfig;

/// Biome texture parameters that influence texture generation
#[derive(Debug, Clone)]
pub struct BiomeTextureParams {
    pub temperature: f32,      // 0.0 (cold) to 1.0 (hot)
    pub moisture: f32,         // 0.0 (dry) to 1.0 (wet)
    pub height: i32,           // Absolute height in world
    pub relative_height: f32,  // 0.0 (low) to 1.0 (high) within biome
    pub biome_type: String,    // Biome identifier
}

impl BiomeTextureParams {
    pub fn new(temperature: f32, moisture: f32, height: i32, biome_type: &str) -> Self {
        // Calculate relative height based on biome (simplified for now)
        let relative_height = Self::calculate_relative_height(height, biome_type);
        
        Self {
            temperature,
            moisture,
            height,
            relative_height,
            biome_type: biome_type.to_string(),
        }
    }
    
    fn calculate_relative_height(height: i32, biome_type: &str) -> f32 {
        // Different biomes have different height ranges
        let (min_height, max_height) = match biome_type {
            "desert" => (5, 25),
            "forest" => (10, 40),
            "mountain" | "snowy_mountain" => (20, 80),
            "hills" => (15, 35),
            "plains" => (5, 20),
            "swamp" => (2, 15),
            "tundra" => (10, 30),
            "beach" => (1, 10),
            _ => (5, 30),
        };
        
        // Clamp height and normalize
        let clamped_height = height.max(min_height).min(max_height);
        (clamped_height - min_height) as f32 / (max_height - min_height) as f32
    }
}

/// Biome texture configuration that defines how textures vary
#[derive(Debug, Clone)]
pub struct BiomeTextureConfig {
    pub base_config: AlkydTextureConfig,
    pub temperature_effect: f32,    // How much temperature affects texture
    pub moisture_effect: f32,       // How much moisture affects texture
    pub height_effect: f32,         // How much height affects texture
    pub color_variation_range: (f32, f32), // Min/max color variation
}

impl BiomeTextureConfig {
    pub fn for_block_type(block_type: BlockType, biome_params: &BiomeTextureParams) -> Self {
        let base_config = match block_type {
            BlockType::Grass => AlkydTextureConfig {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.07,
                noise_octaves: 4,
                use_simplex_noise: true,
                base_color: [0.2, 0.8, 0.2], // Base grass green
                color_variation: 0.3,
                use_gpu_acceleration: true,
                enable_edge_detection: true,
                enable_color_blending: true,
                blend_mode: "soft_light".to_string(),
                ..Default::default()
            },
            BlockType::Dirt => AlkydTextureConfig {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.08,
                noise_octaves: 5,
                use_simplex_noise: true,
                base_color: [0.5, 0.3, 0.2], // Base dirt brown
                color_variation: 0.2,
                use_gpu_acceleration: true,
                enable_edge_detection: true,
                enable_color_blending: true,
                blend_mode: "multiply".to_string(),
                ..Default::default()
            },
            BlockType::Stone => AlkydTextureConfig {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.1,
                noise_octaves: 6,
                use_simplex_noise: true,
                base_color: [0.7, 0.7, 0.7], // Base stone gray
                color_variation: 0.15,
                use_gpu_acceleration: true,
                enable_ridged_noise: true,
                ridged_strength: 0.6,
                enable_edge_detection: true,
                ..Default::default()
            },
            BlockType::Sand => AlkydTextureConfig {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.09,
                noise_octaves: 3,
                use_simplex_noise: true,
                base_color: [0.9, 0.8, 0.5], // Base sand color
                color_variation: 0.1,
                use_gpu_acceleration: true,
                enable_turbulence: true,
                turbulence_strength: 0.08,
                ..Default::default()
            },
            BlockType::Wood => AlkydTextureConfig {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.06,
                noise_octaves: 4,
                use_simplex_noise: true,
                base_color: [0.6, 0.45, 0.3], // Base wood brown
                color_variation: 0.3,
                use_gpu_acceleration: true,
                enable_edge_detection: true,
                enable_color_blending: true,
                blend_mode: "hard_light".to_string(),
                ..Default::default()
            },
            _ => AlkydTextureConfig::default(),
        };
        
        // Adjust effects based on biome
        let (temperature_effect, moisture_effect, height_effect) = 
            Self::get_biome_effects(&biome_params.biome_type);
        
        Self {
            base_config,
            temperature_effect,
            moisture_effect,
            height_effect,
            color_variation_range: (0.1, 0.4),
        }
    }
    
    fn get_biome_effects(biome_type: &str) -> (f32, f32, f32) {
        match biome_type {
            "desert" => (0.8, 0.3, 0.5),    // High temp effect, low moisture
            "forest" => (0.4, 0.7, 0.6),    // Balanced with high moisture
            "mountain" | "snowy_mountain" => (0.3, 0.4, 0.9), // High height effect
            "hills" => (0.5, 0.5, 0.7),     // Moderate height effect
            "plains" => (0.6, 0.6, 0.4),    // Balanced
            "swamp" => (0.3, 0.8, 0.4),     // High moisture effect
            "tundra" => (0.2, 0.5, 0.6),    // Low temp, moderate height
            "beach" => (0.5, 0.7, 0.3),     // Moderate moisture
            "savanna" => (0.7, 0.4, 0.5),   // High temp, moderate moisture
            "taiga" => (0.3, 0.6, 0.5),     // Low temp, moderate moisture
            "jungle" => (0.6, 0.9, 0.4),    // High temp and moisture
            "badlands" => (0.9, 0.2, 0.6),  // Very high temp, low moisture
            _ => (0.5, 0.5, 0.5),            // Default balanced
        }
    }
}

/// Apply biome parameters to texture configuration
pub fn apply_biome_parameters_to_config(
    config: &AlkydTextureConfig,
    biome_params: &BiomeTextureParams,
    biome_config: &BiomeTextureConfig,
) -> AlkydTextureConfig {
    let mut modified_config = config.clone();
    
    // Apply temperature effects
    if biome_config.temperature_effect > 0.0 {
        let temp_factor = biome_params.temperature * biome_config.temperature_effect;
        
        // Adjust base color based on temperature
        if biome_params.biome_type.contains("desert") || biome_params.temperature > 0.7 {
            // Hot biomes: more yellow/brown tones
            modified_config.base_color[0] = (config.base_color[0] + temp_factor * 0.3).min(1.0); // More red
            modified_config.base_color[1] = (config.base_color[1] - temp_factor * 0.2).max(0.0); // Less green
            modified_config.base_color[2] = (config.base_color[2] - temp_factor * 0.1).max(0.0); // Less blue
        } else if biome_params.temperature < 0.3 {
            // Cold biomes: more blue/gray tones
            modified_config.base_color[0] = (config.base_color[0] - temp_factor * 0.2).max(0.0); // Less red
            modified_config.base_color[1] = (config.base_color[1] - temp_factor * 0.1).max(0.0); // Less green
            modified_config.base_color[2] = (config.base_color[2] + temp_factor * 0.3).min(1.0); // More blue
        }
    }
    
    // Apply moisture effects
    if biome_config.moisture_effect > 0.0 {
        let moisture_factor = biome_params.moisture * biome_config.moisture_effect;
        
        // Adjust color based on moisture
        if biome_params.moisture > 0.7 {
            // Wet areas: more green/blue tones
            modified_config.base_color[1] = (config.base_color[1] + moisture_factor * 0.2).min(1.0); // More green
            modified_config.base_color[2] = (config.base_color[2] + moisture_factor * 0.1).min(1.0); // More blue
        } else if biome_params.moisture < 0.3 {
            // Dry areas: more brown/red tones
            modified_config.base_color[0] = (config.base_color[0] + moisture_factor * 0.2).min(1.0); // More red
            modified_config.base_color[1] = (config.base_color[1] - moisture_factor * 0.1).max(0.0); // Less green
        }
    }
    
    // Apply height effects
    if biome_config.height_effect > 0.0 {
        let height_factor = biome_params.relative_height * biome_config.height_effect;
        
        // Higher elevations tend to be lighter and more rocky
        let lightness_factor = 0.1 + height_factor * 0.3;
        modified_config.base_color[0] = (config.base_color[0] * lightness_factor).min(1.0);
        modified_config.base_color[1] = (config.base_color[1] * lightness_factor).min(1.0);
        modified_config.base_color[2] = (config.base_color[2] * lightness_factor).min(1.0);
        
        // Increase noise scale for higher elevations (more rocky texture)
        modified_config.noise_scale = config.noise_scale * (1.0 + height_factor * 0.5);
        
        // Increase color variation for more natural look
        modified_config.color_variation = config.color_variation * (1.0 + height_factor * 0.3);
    }
    
    // Adjust noise parameters based on biome
    modified_config.noise_scale = config.noise_scale * (1.0 + biome_params.relative_height * 0.2);
    modified_config.color_variation = config.color_variation * (1.0 + biome_params.temperature * 0.1);
    
    modified_config
}

/// Generate a unique texture key for caching based on biome parameters
pub fn generate_texture_cache_key(block_type: &BlockType, biome_params: &BiomeTextureParams) -> String {
    format!(
        "{:?}-temp{:.2}-moist{:.2}-height{}-biome{}",
        block_type,
        biome_params.temperature,
        biome_params.moisture,
        biome_params.height,
        biome_params.biome_type
    )
}