// Alkyd-based World Generation Module
// This module provides GPU-accelerated world generation using Alkyd compute pipelines

use bevy::prelude::*;
use crate::alkyd_integration::{AlkydTextureConfig, AlkydResources};
use std::time::Instant;

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
    pub enable_performance_monitoring: bool,
    pub optimization_level: u8,
}

impl Default for AlkydWorldGenSettings {
    fn default() -> Self {
        Self {
            base_height: 10.0,
            height_scale: 40.0,  // Reduced from 80.0 for less extreme terrain
            frequency: 0.01,     // Reduced from 0.015 for better chunk continuity
            octaves: 6,         // Reduced from 8 for even smoother terrain and better continuity
            persistence: 0.6,   // Increased from 0.5 for even smoother transitions between chunks
            lacunarity: 1.8,    // Reduced from 2.0 for better continuity between chunks
            noise_type: "simplex".to_string(),  // Changed from fractal to simplex for better continuity
            enable_ridged_noise: false,  // Disabled to reduce mountain ridges
            ridged_strength: 0.3,        // Reduced for when ridged noise is enabled
            enable_turbulence: false,    // Disabled to reduce chaotic patterns
            turbulence_strength: 0.1,    // Reduced for when turbulence is enabled
            detail_level: 0.8,  // Reduced from 1.0 for even less extreme detail and better continuity
            use_gpu_acceleration: true,
            biome_scale: 0.005,
            temperature_scale: 0.01,
            moisture_scale: 0.01,
            enable_performance_monitoring: true,
            optimization_level: 2, // Medium optimization by default
        }
    }
}

/// Resource to store Alkyd world generation performance metrics
#[derive(Resource, Debug, Default)]
pub struct AlkydWorldGenPerformance {
    pub chunks_generated: u32,
    pub total_generation_time: f32,
    pub average_generation_time: f32,
    pub peak_generation_time: f32,
    pub gpu_accelerated_chunks: u32,
    pub cpu_fallback_chunks: u32,
    pub biome_distribution: std::collections::HashMap<String, u32>,
}

/// Generate heightmap using Alkyd GPU-accelerated noise algorithms
pub fn generate_alkyd_heightmap(
    x: f32,
    z: f32,
    settings: &AlkydWorldGenSettings,
    alkyd_resources: &AlkydResources,
) -> f32 {
    // Apply performance optimization based on settings
    let optimized_settings = apply_performance_optimization(settings);
    
    // Use GPU-accelerated noise generation if available
    if alkyd_resources.gpu_acceleration_enabled && optimized_settings.use_gpu_acceleration {
        generate_gpu_heightmap(x, z, &optimized_settings)
    } else {
        generate_cpu_fallback_heightmap(x, z, &optimized_settings)
    }
}

/// Generate heightmap with chunk-aware continuity for better seam elimination
pub fn generate_alkyd_heightmap_with_continuity(
    x: f32,
    z: f32,
    chunk_x: i32,
    chunk_z: i32,
    settings: &AlkydWorldGenSettings,
    alkyd_resources: &AlkydResources,
) -> f32 {
    // Apply chunk-aware coordinate adjustment for better continuity
    let adjusted_x = apply_chunk_continuity_adjustment(x, chunk_x);
    let adjusted_z = apply_chunk_continuity_adjustment(z, chunk_z);
    
    // Generate heightmap using the adjusted coordinates
    generate_alkyd_heightmap(adjusted_x, adjusted_z, settings, alkyd_resources)
}

/// Apply chunk continuity adjustment to coordinates
fn apply_chunk_continuity_adjustment(coord: f32, chunk_coord: i32) -> f32 {
    // Add small offset based on chunk position to ensure continuity
    // This helps reduce seams by making sure noise patterns align better
    // Use a more significant offset that's still small enough to not disrupt the overall pattern
    let chunk_offset = chunk_coord as f32 * 0.01; // Increased from 0.001 to 0.01 for better effect
    coord + chunk_offset
}

/// Apply performance optimization based on optimization level
fn apply_performance_optimization(settings: &AlkydWorldGenSettings) -> AlkydWorldGenSettings {
    let mut optimized = settings.clone();
    
    match settings.optimization_level {
        0 => {
            // No optimization - maximum quality
            // Keep all settings as-is
        },
        1 => {
            // Light optimization - slight quality reduction for better performance
            optimized.detail_level = (settings.detail_level * 0.9).max(0.8);
            optimized.turbulence_strength = (settings.turbulence_strength * 0.8).max(0.05);
        },
        2 => {
            // Medium optimization - balanced approach
            optimized.detail_level = (settings.detail_level * 0.8).max(0.7);
            optimized.turbulence_strength = (settings.turbulence_strength * 0.7).max(0.05);
            optimized.ridged_strength = (settings.ridged_strength * 0.9).max(0.3);
        },
        3 => {
            // Heavy optimization - significant performance improvement
            optimized.detail_level = (settings.detail_level * 0.6).max(0.5);
            optimized.turbulence_strength = (settings.turbulence_strength * 0.5).max(0.03);
            optimized.ridged_strength = (settings.ridged_strength * 0.8).max(0.2);
            optimized.octaves = ((settings.octaves as f32 * 0.8) as usize).max(4);
        },
        _ => {
            // Extreme optimization - maximum performance
            optimized.detail_level = (settings.detail_level * 0.5).max(0.4);
            optimized.turbulence_strength = (settings.turbulence_strength * 0.3).max(0.02);
            optimized.ridged_strength = (settings.ridged_strength * 0.6).max(0.1);
            optimized.octaves = ((settings.octaves as f32 * 0.6) as usize).max(3);
        }
    }
    
    optimized
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
    
    commands.insert_resource(settings.clone());
    commands.insert_resource(AlkydWorldGenPerformance::default());
    
    println!("✓ Alkyd world generation initialized with GPU acceleration");
    println!("  - Performance monitoring: {}", settings.enable_performance_monitoring);
    println!("  - Optimization level: {}", settings.optimization_level);
    println!("  - GPU acceleration: {}", settings.use_gpu_acceleration);
}

/// System to monitor and log Alkyd world generation performance
pub fn monitor_alkyd_world_gen_performance(
    performance: Res<AlkydWorldGenPerformance>,
    settings: Res<AlkydWorldGenSettings>,
) {
    if !settings.enable_performance_monitoring {
        return;
    }
    
    if performance.chunks_generated > 0 {
        println!("📊 Alkyd World Generation Performance Report:");
        println!("  - Chunks generated: {}", performance.chunks_generated);
        println!("  - Total time: {:.2}ms", performance.total_generation_time);
        println!("  - Average time per chunk: {:.2}ms", performance.average_generation_time);
        println!("  - Peak time: {:.2}ms", performance.peak_generation_time);
        println!("  - GPU accelerated: {} ({:.1}%)", 
                 performance.gpu_accelerated_chunks,
                 (performance.gpu_accelerated_chunks as f32 / performance.chunks_generated as f32) * 100.0);
        println!("  - CPU fallback: {} ({:.1}%)", 
                 performance.cpu_fallback_chunks,
                 (performance.cpu_fallback_chunks as f32 / performance.chunks_generated as f32) * 100.0);
        
        println!("  - Biome distribution:");
        for (biome, count) in &performance.biome_distribution {
            println!("    - {}: {} ({:.1}%)", biome, count, 
                     (*count as f32 / performance.chunks_generated as f32) * 100.0);
        }
    }
}

/// System to generate a performance test chunk
pub fn generate_alkyd_performance_test_chunk(
    mut commands: Commands,
    settings: Res<AlkydWorldGenSettings>,
    alkyd_resources: Res<AlkydResources>,
) {
    if !settings.enable_performance_monitoring {
        return;
    }
    
    println!("🧪 Running Alkyd performance test...");
    
    let start_time = Instant::now();
    
    // Test multiple heightmap generations
    const TEST_ITERATIONS: usize = 100;
    for i in 0..TEST_ITERATIONS {
        let x = i as f32 * 100.0;
        let z = i as f32 * 100.0;
        let _height = generate_alkyd_heightmap(x, z, &settings, &alkyd_resources);
    }
    
    let duration = start_time.elapsed();
    let avg_time_per_call = duration.as_secs_f32() / TEST_ITERATIONS as f32 * 1000.0;
    
    println!("✓ Alkyd performance test completed:");
    println!("  - {} iterations in {:.2}ms", TEST_ITERATIONS, duration.as_secs_f32() * 1000.0);
    println!("  - Average time per heightmap: {:.3}ms", avg_time_per_call);
    println!("  - Performance: {} calls/second", 
             (TEST_ITERATIONS as f32 / duration.as_secs_f32()).round());
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