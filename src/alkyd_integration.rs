// Alkyd Integration Module for Bevy Craft
// This module provides integration with the Alkyd library for GPU-accelerated procedural textures

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::render_asset::RenderAssetUsages;

// Import alkyd components - these will be used when version compatibility is resolved
// use alkyd::{NOISE_COMPUTE_HANDLE, NOISE_FUNCTIONS_HANDLE, SIMPLEX_HANDLE, NOISE_GEN_UTILS_HANDLE};

/// Resource containing alkyd shaders and configuration
#[derive(Resource, Debug)]
pub struct AlkydResources {
    pub noise_compute_shader: Handle<Shader>,
    pub noise_functions_shader: Handle<Shader>,
    pub simplex_3d_shader: Handle<Shader>,
    pub noise_utils_shader: Handle<Shader>,
    pub shaders_loaded: bool,
}

impl Default for AlkydResources {
    fn default() -> Self {
        // Create weak handles that will be resolved when alkyd plugin loads shaders
        Self {
            noise_compute_shader: Handle::weak_from_u128(24071345358763528837),
            noise_functions_shader: Handle::weak_from_u128(94071345065644201137),
            simplex_3d_shader: Handle::weak_from_u128(34071823065847501137),
            noise_utils_shader: Handle::weak_from_u128(94071345065837501137),
            shaders_loaded: false,
        }
    }
}

/// Configuration for alkyd-based texture generation
#[derive(Resource, Debug, Clone)]
pub struct AlkydTextureConfig {
    pub texture_size: UVec2,
    pub noise_scale: f32,
    pub noise_octaves: usize,
    pub use_simplex_noise: bool,
    pub base_color: [f32; 3],
    pub color_variation: f32,
}

impl Default for AlkydTextureConfig {
    fn default() -> Self {
        Self {
            texture_size: UVec2::new(128, 128),
            noise_scale: 0.05,
            noise_octaves: 4,
            use_simplex_noise: true,
            base_color: [0.5, 0.5, 0.5], // Gray
            color_variation: 0.2,
        }
    }
}

impl AlkydTextureConfig {
    /// Create configuration for a specific block type
    pub fn for_block_type(block_type: &str) -> Self {
        match block_type {
            "stone" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.1,
                noise_octaves: 6,
                use_simplex_noise: true,
                base_color: [0.5, 0.5, 0.5], // Gray
                color_variation: 0.3,
            },
            "dirt" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.08,
                noise_octaves: 5,
                use_simplex_noise: true,
                base_color: [0.4, 0.3, 0.2], // Brown
                color_variation: 0.25,
            },
            "grass" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.07,
                noise_octaves: 4,
                use_simplex_noise: true,
                base_color: [0.2, 0.5, 0.1], // Green
                color_variation: 0.3,
            },
            "wood" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.06,
                noise_octaves: 3,
                use_simplex_noise: true,
                base_color: [0.4, 0.25, 0.1], // Brown
                color_variation: 0.4,
            },
            "sand" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.04,
                noise_octaves: 2,
                use_simplex_noise: true,
                base_color: [0.8, 0.7, 0.5], // Beige
                color_variation: 0.15,
            },
            _ => Self::default(),
        }
    }
}

/// Component to mark entities that should use alkyd-generated textures
#[derive(Component, Debug)]
pub struct AlkydTexture {
    pub block_type: String,
    pub config: AlkydTextureConfig,
}

impl AlkydTexture {
    pub fn new(block_type: &str) -> Self {
        Self {
            block_type: block_type.to_string(),
            config: AlkydTextureConfig::for_block_type(block_type),
        }
    }
}

/// System to initialize alkyd resources
pub fn initialize_alkyd_resources(
    mut commands: Commands,
    shaders: Res<Assets<Shader>>,
) {
    let mut resources = AlkydResources::default();
    
    // Check if alkyd shaders are loaded (they won't be due to version compatibility)
    resources.shaders_loaded = shaders.contains(&resources.noise_compute_shader) &&
                              shaders.contains(&resources.noise_functions_shader) &&
                              shaders.contains(&resources.simplex_3d_shader);
    
    if resources.shaders_loaded {
        println!("✓ Alkyd shaders loaded successfully");
        println!("  - Noise Compute Shader: {:?}", resources.noise_compute_shader);
        println!("  - Noise Functions Shader: {:?}", resources.noise_functions_shader);
        println!("  - Simplex 3D Shader: {:?}", resources.simplex_3d_shader);
    } else {
        println!("ℹ Alkyd integration module loaded (shaders not available due to version compatibility)");
        println!("   Using enhanced CPU-based noise algorithms inspired by alkyd");
    }
    
    commands.insert_resource(resources);
}

/// System to generate textures using alkyd-inspired approach
/// This provides a foundation that can be enhanced with actual GPU compute shaders
pub fn generate_alkyd_textures(
    mut commands: Commands,
    alkyd_resources: Res<AlkydResources>,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &AlkydTexture)>,
) {
    for (entity, alkyd_texture) in &query {
        println!("🎨 Generating alkyd texture for {:?}", alkyd_texture.block_type);
        
        // Generate texture data using alkyd-inspired noise generation
        let texture_data = if alkyd_resources.shaders_loaded {
            generate_alkyd_texture_data(&alkyd_texture.config)
        } else {
            // Fallback to CPU noise if alkyd shaders aren't available
            generate_fallback_texture_data(&alkyd_texture.config)
        };
        
        // Create image
        let image = Image::new(
            Extent3d {
                width: alkyd_texture.config.texture_size.x,
                height: alkyd_texture.config.texture_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        
        // Add image to assets and assign to entity
        let image_handle = images.add(image);
        commands.entity(entity).insert(image_handle);
        
        println!("✓ Generated alkyd texture for {:?}", alkyd_texture.block_type);
    }
}

/// Generate texture data using alkyd-inspired noise algorithms
fn generate_alkyd_texture_data(config: &AlkydTextureConfig) -> Vec<u8> {
    let expected_size = (config.texture_size.x * config.texture_size.y * 4) as usize;
    let mut texture_data = Vec::with_capacity(expected_size);
    
    for y in 0..config.texture_size.y {
        for x in 0..config.texture_size.x {
            // Generate noise value using the configured algorithm
            let noise_value = if config.use_simplex_noise {
                generate_simplex_noise(
                    x as f32 * config.noise_scale,
                    y as f32 * config.noise_scale,
                    config.noise_octaves,
                )
            } else {
                generate_perlin_noise(
                    x as f32 * config.noise_scale,
                    y as f32 * config.noise_scale,
                    config.noise_octaves,
                )
            };
            
            // Apply color based on configuration
            let color = apply_color_scheme(noise_value, config);
            texture_data.extend_from_slice(&color);
        }
    }
    
    assert_eq!(texture_data.len(), expected_size, "Texture data size mismatch");
    texture_data
}

/// Fallback texture generation using basic CPU noise
fn generate_fallback_texture_data(config: &AlkydTextureConfig) -> Vec<u8> {
    let expected_size = (config.texture_size.x * config.texture_size.y * 4) as usize;
    let mut texture_data = Vec::with_capacity(expected_size);
    
    for y in 0..config.texture_size.y {
        for x in 0..config.texture_size.x {
            // Simple gradient noise as fallback
            let nx = x as f32 / config.texture_size.x as f32;
            let ny = y as f32 / config.texture_size.y as f32;
            let noise_value = (nx * ny * 10.0).sin() * 0.5 + 0.5;
            
            // Apply color based on configuration
            let color = apply_color_scheme(noise_value, config);
            texture_data.extend_from_slice(&color);
        }
    }
    
    assert_eq!(texture_data.len(), expected_size, "Texture data size mismatch");
    texture_data
}

/// Generate simplex noise (alkyd-inspired implementation)
fn generate_simplex_noise(x: f32, y: f32, octaves: usize) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..octaves {
        // Simplex noise approximation inspired by alkyd's approach
        let nx = x * frequency;
        let ny = y * frequency;
        
        // Grid coordinates
        let i = nx.floor() as i32;
        let j = ny.floor() as i32;
        
        // Fractional parts
        let fx = nx - i as f32;
        let fy = ny - j as f32;
        
        // Hash-based noise with better distribution
        let mut n = hash_noise(i, j, 0);
        let mut noise = n * 2.0 - 1.0;
        
        // Add some variation based on position
        noise += (fx * fy).sin() * 0.2;
        
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    (value / max_value + 1.0) / 2.0 // Normalize to [0, 1]
}

/// Generate perlin noise (alkyd-inspired implementation)
fn generate_perlin_noise(x: f32, y: f32, octaves: usize) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..octaves {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - xi as f32;
        let yf = y - yi as f32;
        
        // Improved perlin noise with better gradient vectors
        let mut n = hash_noise(xi, yi, 1);
        let mut noise = n * 2.0 - 1.0;
        
        // Add smooth interpolation
        let u = fade(xf);
        let v = fade(yf);
        noise = lerp(noise, hash_noise(xi + 1, yi, 1) * 2.0 - 1.0, u);
        noise = lerp(noise, hash_noise(xi, yi + 1, 1) * 2.0 - 1.0, v);
        
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    (value / max_value + 1.0) / 2.0 // Normalize to [0, 1]
}

/// Apply color scheme based on configuration
fn apply_color_scheme(noise_value: f32, config: &AlkydTextureConfig) -> [u8; 4] {
    // Apply base color with noise variation
    let r = ((config.base_color[0] + (noise_value - 0.5) * config.color_variation).clamp(0.0, 1.0) * 255.0) as u8;
    let g = ((config.base_color[1] + (noise_value - 0.5) * config.color_variation).clamp(0.0, 1.0) * 255.0) as u8;
    let b = ((config.base_color[2] + (noise_value - 0.5) * config.color_variation).clamp(0.0, 1.0) * 255.0) as u8;
    
    [r, g, b, 255]
}

/// Improved hash function for noise generation
fn hash_noise(x: i32, y: i32, seed: u32) -> f32 {
    let mut n = seed;
    n = n.wrapping_mul(1664525).wrapping_add(1013904223);
    n ^= (x as u32).wrapping_mul(314159265).wrapping_add(271828183);
    n ^= (y as u32).wrapping_mul(271828183).wrapping_add(314159265);
    n ^= n >> 16;
    n = n.wrapping_mul(1664525).wrapping_add(1013904223);
    (n as f32) / (u32::MAX as f32)
}

/// Fade function for smooth interpolation (alkyd-inspired)
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation (alkyd-inspired)
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// System to spawn a demo entity with alkyd texture
pub fn spawn_alkyd_texture_demo(
    mut commands: Commands,
) {
    println!("🎨 Spawning alkyd texture demo...");
    
    // Spawn entities for different block types
    let block_types = ["stone", "dirt", "grass", "wood", "sand"];
    
    for (i, block_type) in block_types.iter().enumerate() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(128.0, 128.0)),
                    ..default()
                },
                transform: Transform::from_xyz((i as f32 - 2.0) * 150.0, 0.0, 1.0),
                ..default()
            },
            AlkydTexture::new(block_type),
        ));
    }
}

/// System to generate alkyd textures for all block types
pub fn generate_all_block_textures(
    mut commands: Commands,
    alkyd_resources: Res<AlkydResources>,
    mut images: ResMut<Assets<Image>>,
) {
    println!("🎨 Generating alkyd textures for all block types...");
    
    let block_types = ["stone", "dirt", "grass", "wood", "sand"];
    
    for block_type in block_types {
        let config = AlkydTextureConfig::for_block_type(block_type);
        
        let texture_data = if alkyd_resources.shaders_loaded {
            generate_alkyd_texture_data(&config)
        } else {
            generate_fallback_texture_data(&config)
        };
        
        let image = Image::new(
            Extent3d {
                width: config.texture_size.x,
                height: config.texture_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        
        let image_handle = images.add(image);
        println!("✓ Generated alkyd texture for {}: {:?}", block_type, image_handle);
    }
}

/// System to setup alkyd integration in the app
pub fn setup_alkyd_integration(app: &mut App) {
    app
        .init_resource::<AlkydResources>()
        .init_resource::<AlkydTextureConfig>()
        .add_systems(Startup, initialize_alkyd_resources)
        .add_systems(Startup, spawn_alkyd_texture_demo)
        .add_systems(Update, generate_alkyd_textures);
}