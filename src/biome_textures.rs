// Biome-based texture parameterization system
// This module handles texture variations based on biome and height parameters


use crate::block::BlockType;
use crate::noise::NoiseSettings;

/// Biome texture parameters that influence texture generation
#[derive(Debug, Clone, PartialEq)]
pub struct BiomeTextureParams {
    pub biome_type: String,
    pub temperature: f32,
    pub moisture: f32,
    pub height: f32,
    pub relative_height: f32,
}

impl BiomeTextureParams {
    pub fn new(biome_type: String, temperature: f32, moisture: f32, height: f32, relative_height: f32) -> Self {
        Self {
            biome_type,
            temperature,
            moisture,
            height,
            relative_height,
        }
    }
}

/// Configuration for how biome parameters affect texture generation
#[derive(Debug, Clone)]
pub struct BiomeTextureConfig {
    pub base_config: NoiseSettings,
    pub temperature_effect: f32,
    pub moisture_effect: f32,
    pub height_effect: f32,
}

impl BiomeTextureConfig {
    /// Create a biome texture config for a specific block type
    pub fn for_block_type(block_type: &BlockType) -> Self {
        let base_config = match block_type {
            BlockType::Grass => NoiseSettings {
                base_height: 0.0,
                height_scale: 1.0,
                scale: 0.07,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
                biome_scale: 0.005,
            },
            BlockType::Dirt => NoiseSettings {
                base_height: 0.0,
                height_scale: 1.0,
                scale: 0.08,
                octaves: 5,
                persistence: 0.5,
                lacunarity: 2.0,
                biome_scale: 0.005,
            },
            BlockType::Stone => NoiseSettings {
                base_height: 0.0,
                height_scale: 1.0,
                scale: 0.1,
                octaves: 6,
                persistence: 0.5,
                lacunarity: 2.0,
                biome_scale: 0.005,
            },
            BlockType::Sand => NoiseSettings {
                base_height: 0.0,
                height_scale: 1.0,
                scale: 0.09,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
                biome_scale: 0.005,
            },
            BlockType::Wood => NoiseSettings {
                base_height: 0.0,
                height_scale: 1.0,
                scale: 0.06,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
                biome_scale: 0.005,
            },
            _ => NoiseSettings::default(),
        };

        // Default biome effects
        let (temperature_effect, moisture_effect, height_effect) = match block_type {
            BlockType::Grass => (0.8, 0.6, 0.4),
            BlockType::Dirt => (0.6, 0.5, 0.3),
            BlockType::Stone => (0.4, 0.3, 0.5),
            BlockType::Sand => (0.9, 0.2, 0.1),
            BlockType::Wood => (0.5, 0.4, 0.2),
            _ => (0.5, 0.5, 0.5),            // Default balanced
        };

        Self {
            base_config,
            temperature_effect,
            moisture_effect,
            height_effect,
        }
    }
}

/// Apply biome parameters to texture configuration
pub fn apply_biome_parameters_to_config(
    config: &NoiseSettings,
    biome_params: &BiomeTextureParams,
    biome_config: &BiomeTextureConfig,
) -> NoiseSettings {
    let mut modified_config = config.clone();
    
    // Simple biome adjustments - just modify noise parameters for variety
    if biome_config.temperature_effect > 0.0 {
        let temp_factor = biome_params.temperature * biome_config.temperature_effect;
        modified_config.scale = (config.scale * (0.8 + temp_factor * 0.4)).max(0.01);
    }
    
    if biome_config.moisture_effect > 0.0 {
        let moisture_factor = biome_params.moisture * biome_config.moisture_effect;
        modified_config.octaves = (config.octaves as f32 * (0.8 + moisture_factor * 0.4)) as usize;
    }
    
    if biome_config.height_effect > 0.0 {
        let height_factor = biome_params.height * biome_config.height_effect;
        modified_config.persistence = (config.persistence * (0.8 + height_factor * 0.4)).max(0.1);
    }
    
    modified_config
}

/// Generate a unique texture key for caching based on biome parameters
pub fn generate_texture_cache_key(block_type: &BlockType, biome_params: &BiomeTextureParams) -> String {
    format!(
        "{:?}-{}-{}-{}-{}",
        block_type,
        biome_params.biome_type,
        (biome_params.temperature * 10.0).round() as i32,
        (biome_params.moisture * 10.0).round() as i32,
        (biome_params.height * 10.0).round() as i32
    )
}

/// Determine biome type based on temperature and moisture
#[allow(dead_code)]
pub fn determine_biome_type(temperature: f32, moisture: f32, height: i32) -> String {
    if height > 60 {
        if temperature < 0.3 {
            "snowy_mountain".to_string()
        } else {
            "rocky_mountain".to_string()
        }
    } else if temperature > 0.7 {
        if moisture < 0.4 {
            "desert".to_string()
        } else {
            "savanna".to_string()
        }
    } else if temperature > 0.5 {
        if moisture > 0.7 {
            "jungle".to_string()
        } else if moisture > 0.4 {
            "forest".to_string()
        } else {
            "grassland".to_string()
        }
    } else {
        if moisture > 0.6 {
            "swamp".to_string()
        } else if moisture > 0.3 {
            "plains".to_string()
        } else {
            "tundra".to_string()
        }
    }
}

/// Get biome texture parameters for a given position
#[allow(dead_code)]
pub fn get_biome_texture_params(
    temperature: f32,
    moisture: f32,
    height: i32,
    max_height: i32,
) -> BiomeTextureParams {
    let biome_type = determine_biome_type(temperature, moisture, height);
    let relative_height = height as f32 / max_height as f32;

    BiomeTextureParams::new(
        biome_type,
        temperature,
        moisture,
        height as f32,
        relative_height,
    )
}