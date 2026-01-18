// Noise generation module to replace alkyd functionality
// This module provides standard procedural noise algorithms

use bevy::prelude::*;

/// Simple 2D noise generator using basic algorithms
pub fn generate_simple_noise(x: f32, z: f32, settings: &NoiseSettings) -> f32 {
    // Basic hash-based noise for simplicity
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..settings.octaves {
        // Simple hash-based noise
        let nx = x * frequency * settings.scale;
        let nz = z * frequency * settings.scale;
        
        let i = nx.floor() as i32;
        let j = nz.floor() as i32;
        let fx = nx - i as f32;
        let fz = nz - j as f32;
        
        // Simple fade function
        let u = fade(fx);
        let v = fade(fz);
        
        // Hash-based gradient values
        let grad00 = hash_noise(i, j, 0);
        let grad10 = hash_noise(i + 1, j, 1);
        let grad01 = hash_noise(i, j + 1, 2);
        let grad11 = hash_noise(i + 1, j + 1, 3);
        
        // Simple interpolation
        let lerp1 = lerp(grad00, grad10, u);
        let lerp2 = lerp(grad01, grad11, u);
        let noise_value = lerp(lerp1, lerp2, v);
        
        value += noise_value * amplitude;
        max_value += amplitude;
        amplitude *= settings.persistence;
        frequency *= settings.lacunarity;
    }
    
    value / max_value
}

/// Generate heightmap using noise
pub fn generate_heightmap(x: f32, z: f32, settings: &NoiseSettings) -> f32 {
    let noise_value = generate_simple_noise(x, z, settings);
    
    // Convert noise to height
    settings.base_height + noise_value * settings.height_scale
}

/// Generate biome information (temperature and moisture)
pub fn generate_biome_info(x: f32, z: f32, settings: &NoiseSettings) -> (f32, f32) {
    // Temperature noise
    let temp_settings = NoiseSettings {
        scale: settings.biome_scale,
        octaves: 3,
        ..*settings
    };
    let temperature = generate_simple_noise(x, z, &temp_settings);
    
    // Moisture noise
    let moisture = generate_simple_noise(x + 1000.0, z + 1000.0, &temp_settings);
    
    (temperature, moisture)
}

/// Simple fade function for smooth interpolation
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Simple linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Simple hash function for noise generation
fn hash_noise(x: i32, y: i32, seed: i32) -> f32 {
    let mut hash = x ^ y ^ seed;
    hash = hash.wrapping_mul(0x34567891);
    hash = hash.wrapping_add(hash >> 16);
    hash = hash.wrapping_mul(0x01234567);
    hash = hash.wrapping_add(hash >> 16);
    
    (hash as f32) / (std::i32::MAX as f32)
}

/// Noise generation settings
#[derive(Resource, Debug, Clone)]
pub struct NoiseSettings {
    pub base_height: f32,
    pub height_scale: f32,
    pub scale: f32,
    pub octaves: usize,
    pub persistence: f32,
    pub lacunarity: f32,
    pub biome_scale: f32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            base_height: 10.0,
            height_scale: 35.0,
            scale: 0.008,
            octaves: 5,
            persistence: 0.7,
            lacunarity: 1.6,
            biome_scale: 0.005,
        }
    }
}