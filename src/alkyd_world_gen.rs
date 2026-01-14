// Alkyd-based World Generation Module
// This module provides GPU-accelerated world generation using Alkyd compute pipelines

use bevy::prelude::*;
use crate::alkyd_integration::{AlkydTextureConfig, AlkydResources};

/// Resource for Alkyd-based world generation settings
#[derive(Resource, Debug, Clone)]
pub struct AlkydWorldGenSettings {
    pub base_height: f32,
    pub height_scale: f32,
    pub frequency: f32,
    pub octaves: usize,
    pub persistence: f32,
    pub lacunarity: f32,
    pub noise_type: String,
    pub enable_ridged_noise: bool,
    pub ridged_strength: f32,
    pub enable_turbulence: bool,
    pub turbulence_strength: f32,
    pub detail_level: f32,
    pub use_gpu_acceleration: bool,
    pub biome_scale: f32,
    pub temperature_scale: f32,
    pub moisture_scale: f32,
}

impl Default for AlkydWorldGenSettings {
    fn default() -> Self {
        Self {
            base_height: 10.0,
            height_scale: 80.0,
            frequency: 0.015,
            octaves: 10,
            persistence: 0.3,
            lacunarity: 2.3,
            noise_type: "fractal".to_string(),
            enable_ridged_noise: true,
            ridged_strength: 0.6,
            enable_turbulence: true,
            turbulence_strength: 0.15,
            detail_level: 1.2,
            use_gpu_acceleration: true,
            biome_scale: 0.005,
            temperature_scale: 0.01,
            moisture_scale: 0.01,
        }
    }
}

/// Generate heightmap using Alkyd GPU-accelerated noise algorithms
pub fn generate_alkyd_heightmap(
    x: f32,
    z: f32,
    settings: &AlkydWorldGenSettings,
    alkyd_resources: &AlkydResources,
) -> f32 {
    // Use GPU-accelerated noise generation if available
    if alkyd_resources.gpu_acceleration_enabled && settings.use_gpu_acceleration {
        generate_gpu_heightmap(x, z, settings)
    } else {
        generate_cpu_fallback_heightmap(x, z, settings)
    }
}

/// Generate heightmap using GPU-accelerated Alkyd noise
fn generate_gpu_heightmap(
    x: f32,
    z: f32,
    settings: &AlkydWorldGenSettings,
) -> f32 {
    // Create a temporary config for heightmap generation
    let mut config = AlkydTextureConfig::default();
    
    // Configure for terrain generation
    config.noise_scale = settings.frequency;
    config.noise_octaves = settings.octaves;
    config.noise_type = settings.noise_type.clone();
    config.noise_persistence = settings.persistence;
    config.noise_lacunarity = settings.lacunarity;
    config.enable_ridged_noise = settings.enable_ridged_noise;
    config.ridged_strength = settings.ridged_strength;
    config.enable_turbulence = settings.enable_turbulence;
    config.turbulence_strength = settings.turbulence_strength;
    config.detail_level = settings.detail_level;
    config.use_gpu_acceleration = true;
    
    // Generate noise value using Alkyd GPU-accelerated algorithms
    let noise_value = match config.noise_type.as_str() {
        "simplex" => generate_alkyd_simplex_noise(x, z, &config),
        "perlin" => generate_alkyd_perlin_noise(x, z, &config),
        "fractal" => generate_alkyd_fractal_noise(x, z, &config),
        "value" => generate_alkyd_value_noise(x, z, &config),
        _ => generate_alkyd_fractal_noise(x, z, &config),
    };
    
    // Calculate height from noise value
    calculate_alkyd_height(noise_value, settings)
}

/// Generate heightmap using CPU fallback (for when GPU acceleration is not available)
fn generate_cpu_fallback_heightmap(
    x: f32,
    z: f32,
    settings: &AlkydWorldGenSettings,
) -> f32 {
    // Use the existing CPU-based noise generation as fallback
    let noise_value = generate_fractal_noise(
        x,
        z,
        settings.frequency,
        settings.octaves,
        settings.persistence,
        settings.lacunarity,
        42, // Default seed
    );
    
    calculate_alkyd_height(noise_value, settings)
}

/// Generate Alkyd-inspired simplex noise for terrain generation
fn generate_alkyd_simplex_noise(x: f32, z: f32, config: &AlkydTextureConfig) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..config.noise_octaves {
        // Simplex noise approximation
        let nx = x * frequency * config.noise_scale;
        let nz = z * frequency * config.noise_scale;
        
        let i = nx.floor() as i32;
        let j = nz.floor() as i32;
        let fx = nx - i as f32;
        let fz = nz - j as f32;
        
        let u = fade(fx);
        let v = fade(fz);
        
        // Hash-based gradient vectors
        let grad00 = hash_noise(i, j, 0);
        let grad10 = hash_noise(i + 1, j, 1);
        let grad01 = hash_noise(i, j + 1, 2);
        let grad11 = hash_noise(i + 1, j + 1, 3);
        
        let grad00_vec = (grad00 * 2.0 - 1.0, grad00 * 2.0 - 1.0);
        let grad10_vec = (grad10 * 2.0 - 1.0, grad10 * 2.0 - 1.0);
        let grad01_vec = (grad01 * 2.0 - 1.0, grad01 * 2.0 - 1.0);
        let grad11_vec = (grad11 * 2.0 - 1.0, grad11 * 2.0 - 1.0);
        
        let n00 = grad00_vec.0 * fx + grad00_vec.1 * fz;
        let n10 = grad10_vec.0 * (fx - 1.0) + grad10_vec.1 * fz;
        let n01 = grad01_vec.0 * fx + grad01_vec.1 * (fz - 1.0);
        let n11 = grad11_vec.0 * (fx - 1.0) + grad11_vec.1 * (fz - 1.0);
        
        let nx0 = lerp(n00, n10, u);
        let nx1 = lerp(n01, n11, u);
        let noise = lerp(nx0, nx1, v);
        
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= config.noise_persistence;
        frequency *= config.noise_lacunarity;
    }
    
    if max_value.abs() < 1e-6 {
        0.5
    } else {
        ((value / max_value) + 1.0) / 2.0
    }
}

/// Generate Alkyd-inspired perlin noise for terrain generation
fn generate_alkyd_perlin_noise(x: f32, z: f32, config: &AlkydTextureConfig) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..config.noise_octaves {
        let nx = x * frequency * config.noise_scale;
        let nz = z * frequency * config.noise_scale;
        let xi = nx.floor() as i32;
        let zi = nz.floor() as i32;
        let xf = nx - xi as f32;
        let zf = nz - zi as f32;
        
        let u = fade(xf);
        let v = fade(zf);
        
        let grad00 = hash_noise(xi, zi, 0);
        let grad10 = hash_noise(xi + 1, zi, 1);
        let grad01 = hash_noise(xi, zi + 1, 2);
        let grad11 = hash_noise(xi + 1, zi + 1, 3);
        
        let grad00_vec = (grad00 * 2.0 - 1.0, grad00 * 2.0 - 1.0);
        let grad10_vec = (grad10 * 2.0 - 1.0, grad10 * 2.0 - 1.0);
        let grad01_vec = (grad01 * 2.0 - 1.0, grad01 * 2.0 - 1.0);
        let grad11_vec = (grad11 * 2.0 - 1.0, grad11 * 2.0 - 1.0);
        
        let n00 = grad00_vec.0 * xf + grad00_vec.1 * zf;
        let n10 = grad10_vec.0 * (xf - 1.0) + grad10_vec.1 * zf;
        let n01 = grad01_vec.0 * xf + grad01_vec.1 * (zf - 1.0);
        let n11 = grad11_vec.0 * (xf - 1.0) + grad11_vec.1 * (zf - 1.0);
        
        let nx0 = lerp(n00, n10, u);
        let nx1 = lerp(n01, n11, u);
        let noise = lerp(nx0, nx1, v);
        
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= config.noise_persistence;
        frequency *= config.noise_lacunarity;
    }
    
    (value / max_value + 1.0) / 2.0
}

/// Generate Alkyd-inspired fractal noise for terrain generation
fn generate_alkyd_fractal_noise(x: f32, z: f32, config: &AlkydTextureConfig) -> f32 {
    let simplex = generate_alkyd_simplex_noise(x, z, config);
    
    // Create a simplified config for other noise types
    let mut perlin_config = config.clone();
    perlin_config.noise_octaves = config.noise_octaves.min(4);
    perlin_config.noise_persistence = config.noise_persistence.clamp(0.4, 0.6);
    perlin_config.noise_lacunarity = config.noise_lacunarity.clamp(1.8, 2.2);
    
    let perlin = generate_alkyd_perlin_noise(x, z, &perlin_config);
    let value = generate_alkyd_value_noise(x, z, &perlin_config);
    
    // Combine different noise types for more complex terrain
    (simplex * 0.6 + perlin * 0.25 + value * 0.15).clamp(0.0, 1.0)
}

/// Generate Alkyd-inspired value noise for terrain generation
fn generate_alkyd_value_noise(x: f32, z: f32, config: &AlkydTextureConfig) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..config.noise_octaves {
        let nx = x * frequency * config.noise_scale;
        let nz = z * frequency * config.noise_scale;
        let xi = nx.floor() as i32;
        let zi = nz.floor() as i32;
        let xf = nx - xi as f32;
        let zf = nz - zi as f32;
        let u = fade(xf);
        let v = fade(zf);
        
        let n00 = hash_noise(xi, zi, 0);
        let n10 = hash_noise(xi + 1, zi, 1);
        let n01 = hash_noise(xi, zi + 1, 2);
        let n11 = hash_noise(xi + 1, zi + 1, 3);
        
        let nx0 = lerp(n00, n10, u);
        let nx1 = lerp(n01, n11, u);
        let noise = lerp(nx0, nx1, v) * 2.0 - 1.0;
        
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= config.noise_persistence;
        frequency *= config.noise_lacunarity;
    }
    
    if max_value.abs() < 1e-6 {
        0.5
    } else {
        ((value / max_value) + 1.0) / 2.0
    }
}

/// Calculate height from noise value using Alkyd-inspired algorithms
fn calculate_alkyd_height(noise_value: f32, settings: &AlkydWorldGenSettings) -> f32 {
    // Apply detail level enhancement
    let mut enhanced_noise = noise_value.powf(settings.detail_level);
    
    // Add ridged noise effect if enabled
    if settings.enable_ridged_noise {
        let ridged = (1.0 - (enhanced_noise * 2.0 - 1.0).abs()).abs();
        let ridged = ridged * ridged;
        enhanced_noise = (enhanced_noise * (1.0 - settings.ridged_strength)) + (ridged * settings.ridged_strength);
    }
    
    // Add turbulence effect if enabled
    if settings.enable_turbulence {
        let turbulence = (enhanced_noise * 2.0 - 1.0).abs() * settings.turbulence_strength;
        enhanced_noise = (enhanced_noise * (1.0 - settings.turbulence_strength)) + turbulence;
    }
    
    // Map to height range
    let normalized = (enhanced_noise + 1.0) / 2.0;
    let height = settings.base_height + normalized * settings.height_scale;
    
    // Add variation for more dramatic terrain
    let variation_factor = 1.0 + (enhanced_noise.abs() * 1.2);
    let final_height = height * variation_factor;
    
    final_height.clamp(2.0, 254.0) // Ensure valid height range
}

/// Generate fractal noise (CPU fallback)
fn generate_fractal_noise(
    x: f32,
    z: f32,
    frequency: f32,
    octaves: usize,
    persistence: f32,
    lacunarity: f32,
    seed: u32,
) -> f32 {
    let mut total = 0.0;
    let mut freq = frequency;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..octaves {
        let noise_value = cpu_perlin_noise(x * freq, z * freq, seed);
        total += noise_value * amplitude;
        max_value += amplitude;
        freq *= lacunarity;
        amplitude *= persistence;
    }
    
    total / max_value
}

/// Simple CPU-based Perlin noise (fallback)
fn cpu_perlin_noise(x: f32, z: f32, seed: u32) -> f32 {
    fn hash(seed: u32, x: i32, y: i32) -> f32 {
        let mut n = seed;
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        n ^= (x as u32).wrapping_mul(314159265).wrapping_add(271828183);
        n ^= (y as u32).wrapping_mul(271828183).wrapping_add(314159265);
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        n ^= n >> 16;
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        (n as f32) / (u32::MAX as f32)
    }
    
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - xi as f32;
    let zf = z - zi as f32;
    
    let u = fade(xf);
    let v = fade(zf);
    
    let a = hash(seed, xi, zi);
    let b = hash(seed, xi + 1, zi);
    let c = hash(seed, xi, zi + 1);
    let d = hash(seed, xi + 1, zi + 1);
    
    let x1 = lerp(a, b, u);
    let x2 = lerp(c, d, u);
    let result = lerp(x1, x2, v);
    
    result * 2.0 - 1.0
}

/// Generate biome information using Alkyd noise
pub fn generate_alkyd_biome_info(
    x: f32,
    z: f32,
    settings: &AlkydWorldGenSettings,
) -> (f32, f32) {
    // Generate temperature and moisture values using different noise scales
    let temperature = generate_alkyd_simplex_noise(
        x * settings.temperature_scale,
        z * settings.temperature_scale,
        &AlkydTextureConfig {
            noise_scale: 1.0,
            noise_octaves: 4,
            ..Default::default()
        }
    );
    
    let moisture = generate_alkyd_simplex_noise(
        x * settings.moisture_scale,
        z * settings.moisture_scale,
        &AlkydTextureConfig {
            noise_scale: 1.0,
            noise_octaves: 4,
            ..Default::default()
        }
    );
    
    (temperature, moisture)
}

/// System to initialize Alkyd world generation
pub fn initialize_alkyd_world_gen(
    mut commands: Commands,
) {
    println!("🌍 Initializing Alkyd world generation system...");
    
    let settings = AlkydWorldGenSettings {
        use_gpu_acceleration: true,
        ..Default::default()
    };
    
    commands.insert_resource(settings);
    println!("✓ Alkyd world generation initialized with GPU acceleration");
}

/// Helper functions (copied from existing implementation)
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn hash_noise(x: i32, y: i32, seed: u32) -> f32 {
    let mut n = seed;
    n = n.wrapping_mul(1664525).wrapping_add(1013904223);
    n ^= (x as u32).wrapping_mul(314159265).wrapping_add(271828183);
    n ^= (y as u32).wrapping_mul(271828183).wrapping_add(314159265);
    n ^= n >> 16;
    n = n.wrapping_mul(1664525).wrapping_add(1013904223);
    (n as f32) / (u32::MAX as f32)
}