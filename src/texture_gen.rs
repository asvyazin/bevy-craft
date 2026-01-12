use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

use crate::alkyd_integration::{AlkydResources, AlkydTextureConfig};


#[cfg(test)]
mod texture_gen_test;

/// Enum representing different noise types for texture generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Worley,
    Value,
    Fractal,
}

/// Resource to hold texture generation settings
#[derive(Resource, Debug)]
pub struct TextureGenSettings {
    pub texture_size: UVec2,
    pub noise_scale: f32,
    pub noise_octaves: usize,
    pub noise_type: NoiseType,
    pub noise_seed: u32,
    pub use_alkyd: bool,
    pub color_scheme: String,
}

impl Default for TextureGenSettings {
    fn default() -> Self {
        Self {
            texture_size: UVec2::new(256, 256),
            noise_scale: 0.05,
            noise_octaves: 4,
            noise_type: NoiseType::Perlin,
            noise_seed: 42,
            use_alkyd: true,
            color_scheme: "natural".to_string(),
        }
    }
}

impl TextureGenSettings {
    pub fn new() -> Self {
        Self {
            texture_size: UVec2::new(256, 256),
            noise_scale: 0.05,
            noise_octaves: 4,
            noise_type: NoiseType::Perlin,
            noise_seed: 42,
            use_alkyd: true,
            color_scheme: "natural".to_string(),
        }
    }
    
    /// Create settings for a specific block type
    pub fn for_block_type(block_type: &str) -> Self {
        match block_type {
            "stone" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.1,
                noise_octaves: 6,
                noise_type: NoiseType::Fractal,
                noise_seed: 123,
                use_alkyd: true,
                color_scheme: "stone".to_string(),
            },
            "dirt" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.08,
                noise_octaves: 5,
                noise_type: NoiseType::Perlin,
                noise_seed: 456,
                use_alkyd: true,
                color_scheme: "dirt".to_string(),
            },
            "grass" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.07,
                noise_octaves: 4,
                noise_type: NoiseType::Simplex,
                noise_seed: 789,
                use_alkyd: true,
                color_scheme: "grass".to_string(),
            },
            _ => Self::default(),
        }
    }
}

/// Component to mark entities that should have procedural textures
#[derive(Component)]
pub struct ProceduralTexture;

/// System to generate procedural textures using alkyd
pub fn generate_procedural_textures(
    mut commands: Commands,
    settings: Res<TextureGenSettings>,
    alkyd_resources: Option<Res<AlkydResources>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<Entity, Added<ProceduralTexture>>,
) {
    for entity in &query {
        // Generate procedural texture data using alkyd-enhanced algorithms if available
        let texture_data = if let Some(alkyd) = &alkyd_resources {
            if alkyd.shaders_loaded {
                // Use alkyd-inspired enhanced algorithms
                let alkyd_config = AlkydTextureConfig {
                    texture_size: settings.texture_size,
                    noise_scale: settings.noise_scale,
                    noise_octaves: settings.noise_octaves,
                    use_simplex_noise: true, // Use simplex noise by default for alkyd
                    base_color: [0.5, 0.5, 0.5], // Default gray
                    color_variation: 0.3,
                    use_gpu_acceleration: true,
                    enable_edge_detection: false,
                    enable_color_blending: false,
                    blend_mode: "normal".to_string(),
                    noise_type: "simplex".to_string(),
                };
                crate::alkyd_integration::generate_alkyd_texture_data(&alkyd_config)
            } else {
                // Use original algorithm
                generate_procedural_texture_data(&settings)
            }
        } else {
            // Use original algorithm
            generate_procedural_texture_data(&settings)
        };

        // Create a new image for the procedural texture with the correct data
        let image = Image::new(
            Extent3d {
                width: settings.texture_size.x,
                height: settings.texture_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );

        // Add the image to assets
        let image_handle = images.add(image);
        
        // Add the image to the entity
        commands.entity(entity).insert(image_handle);
        
        println!("🎨 Generated procedural texture for entity {:?}", entity);
    }
}

/// Generate procedural texture data using noise
fn generate_procedural_texture_data(settings: &TextureGenSettings) -> Vec<u8> {
    let expected_size = (settings.texture_size.x * settings.texture_size.y * 4) as usize;
    let mut texture_data = Vec::with_capacity(expected_size);

    for y in 0..settings.texture_size.y {
        for x in 0..settings.texture_size.x {
            // Generate noise value for this pixel using the specified noise type
            let noise_value = generate_noise_value(
                x as f32 * settings.noise_scale,
                y as f32 * settings.noise_scale,
                settings.noise_octaves,
                settings.noise_type,
                settings.noise_seed
            );

            // Convert noise value to color based on color scheme
            let color = noise_to_color_with_scheme(noise_value, &settings.color_scheme);
            
            // Add color to texture data (RGBA format)
            texture_data.extend_from_slice(&color);
        }
    }

    // Ensure the texture data has the exact expected size
    assert_eq!(texture_data.len(), expected_size, "Texture data size mismatch");
    texture_data
}

/// Generate noise value using the specified noise type
fn generate_noise_value(x: f32, y: f32, octaves: usize, noise_type: NoiseType, seed: u32) -> f32 {
    match noise_type {
        NoiseType::Perlin => generate_perlin_noise(x, y, octaves, seed),
        NoiseType::Simplex => generate_simplex_noise(x, y, octaves, seed),
        NoiseType::Worley => generate_worley_noise(x, y, octaves, seed),
        NoiseType::Value => generate_value_noise(x, y, octaves, seed),
        NoiseType::Fractal => generate_fractal_noise(x, y, octaves, seed),
    }
}

/// Generate Perlin noise
fn generate_perlin_noise(x: f32, y: f32, octaves: usize, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        // Improved Perlin noise with seed
        let noise = perlin_noise_2d(x * frequency, y * frequency, seed);
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

/// Generate Simplex noise
fn generate_simplex_noise(x: f32, y: f32, octaves: usize, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        // Simplex noise approximation
        let noise = simplex_noise_2d(x * frequency, y * frequency, seed);
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

/// Generate Worley noise (cellular noise)
fn generate_worley_noise(x: f32, y: f32, octaves: usize, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        // Worley noise approximation
        let noise = worley_noise_2d(x * frequency, y * frequency, seed);
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

/// Generate Value noise
fn generate_value_noise(x: f32, y: f32, octaves: usize, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        // Value noise (grid-based)
        let noise = value_noise_2d(x * frequency, y * frequency, seed);
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

/// Generate Fractal noise (combined noise types)
fn generate_fractal_noise(x: f32, y: f32, octaves: usize, seed: u32) -> f32 {
    let perlin = generate_perlin_noise(x, y, octaves, seed);
    let simplex = generate_simplex_noise(x, y, octaves, seed + 1);
    let worley = generate_worley_noise(x, y, octaves, seed + 2);
    
    // Combine different noise types for more complex patterns
    (perlin * 0.5 + simplex * 0.3 + worley * 0.2) / 1.0
}

/// 2D Perlin noise implementation with seed
fn perlin_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    // Grid coordinates
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    
    // Fractional parts
    let xf = x - xi as f32;
    let yf = y - yi as f32;
    
    // Fade curves
    let u = fade(xf);
    let v = fade(yf);
    
    // Hash coordinates with seed
    let a = hash(seed, xi, yi);
    let b = hash(seed, xi + 1, yi);
    let c = hash(seed, xi, yi + 1);
    let d = hash(seed, xi + 1, yi + 1);
    
    // Interpolate
    let x1 = lerp(a, b, u);
    let x2 = lerp(c, d, u);
    let result = lerp(x1, x2, v);
    
    // Map to [-1, 1]
    result * 2.0 - 1.0
}

/// 2D Simplex noise approximation
fn simplex_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    // Simplified Simplex noise - use Perlin as base for now
    perlin_noise_2d(x, y, seed)
}

/// 2D Worley noise (cellular noise)
fn worley_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    // Simplified Worley noise - create cellular pattern
    let cell_size = 0.5;
    let cell_x = (x / cell_size).floor() as i32;
    let cell_y = (y / cell_size).floor() as i32;
    
    // Find distance to nearest feature point
    let mut min_dist = f32::MAX;
    for dx in -1..=1 {
        for dy in -1..=1 {
            let feature_x = (cell_x + dx) as f32 * cell_size + hash(seed, cell_x + dx, cell_y + dy);
            let feature_y = (cell_y + dy) as f32 * cell_size + hash(seed + 1, cell_x + dx, cell_y + dy);
            
            let dist_x = x - feature_x;
            let dist_y = y - feature_y;
            let dist = dist_x * dist_x + dist_y * dist_y;
            
            if dist < min_dist {
                min_dist = dist;
            }
        }
    }
    
    // Normalize distance
    (min_dist.sqrt() / cell_size).clamp(0.0, 1.0)
}

/// 2D Value noise (grid-based)
fn value_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    
    // Get grid cell value
    hash(seed, xi, yi) * 2.0 - 1.0
}

/// Hash function for pseudo-random numbers
fn hash(seed: u32, x: i32, y: i32) -> f32 {
    let mut n = seed;
    n = n.wrapping_mul(1664525).wrapping_add(1013904223);
    n ^= (x as u32).wrapping_mul(314159265).wrapping_add(271828183);
    n ^= (y as u32).wrapping_mul(271828183).wrapping_add(314159265);
    n ^= n >> 16;
    n = n.wrapping_mul(1664525).wrapping_add(1013904223);
    (n as f32) / (u32::MAX as f32)
}

/// Fade function for Perlin noise
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Convert noise value to RGBA color
fn noise_to_color(noise_value: f32) -> [u8; 4] {
    noise_to_color_with_scheme(noise_value, "natural")
}

/// Convert noise value to RGBA color based on color scheme
fn noise_to_color_with_scheme(noise_value: f32, scheme: &str) -> [u8; 4] {
    match scheme {
        "stone" => stone_color(noise_value),
        "dirt" => dirt_color(noise_value),
        "grass" => grass_color(noise_value),
        "wood" => wood_color(noise_value),
        "sand" => sand_color(noise_value),
        "water" => water_color(noise_value),
        "bedrock" => bedrock_color(noise_value),
        "leaves" => leaves_color(noise_value),
        _ => natural_color(noise_value), // Default natural scheme
    }
}

/// Natural color scheme (gradient)
fn natural_color(noise_value: f32) -> [u8; 4] {
    let r = (noise_value * 255.0) as u8;
    let g = ((1.0 - noise_value) * 255.0) as u8;
    let b = ((noise_value * 0.5 + 0.25) * 255.0) as u8;
    [r, g, b, 255]
}

/// Stone color scheme
fn stone_color(noise_value: f32) -> [u8; 4] {
    // Stone colors: grays with some brown variations
    let base_gray = 128 + (noise_value * 64.0) as u8;
    let variation = (noise_value * 32.0) as u8;
    let r = base_gray + variation;
    let g = base_gray + variation / 2;
    let b = base_gray - variation / 2;
    [r, g, b, 255]
}

/// Dirt color scheme
fn dirt_color(noise_value: f32) -> [u8; 4] {
    // Dirt colors: browns with some red variations
    let r = 128 + (noise_value * 64.0) as u8;
    let g = 96 + (noise_value * 48.0) as u8;
    let b = 64 + (noise_value * 32.0) as u8;
    [r, g, b, 255]
}

/// Grass color scheme
fn grass_color(noise_value: f32) -> [u8; 4] {
    // Grass colors: greens with some yellow/brown variations
    let r = 64 + (noise_value * 32.0) as u8;
    let g = 128 + (noise_value * 64.0) as u8;
    let b = 32 + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}

/// Wood color scheme
fn wood_color(noise_value: f32) -> [u8; 4] {
    // Wood colors: browns with grain patterns
    let base = 128 + (noise_value * 32.0) as u8;
    let variation = (noise_value * 48.0) as u8;
    let r = base + variation;
    let g = base - variation / 2;
    let b = base - variation;
    [r, g, b, 255]
}

/// Sand color scheme
fn sand_color(noise_value: f32) -> [u8; 4] {
    // Sand colors: yellows and beiges
    let r = 192 + (noise_value * 32.0) as u8;
    let g = 160 + (noise_value * 32.0) as u8;
    let b = 128 + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}

/// Water color scheme
fn water_color(noise_value: f32) -> [u8; 4] {
    // Water colors: blues with some transparency variations
    let r = 32 + (noise_value * 16.0) as u8;
    let g = 64 + (noise_value * 32.0) as u8;
    let b = 128 + (noise_value * 64.0) as u8;
    [r, g, b, 255]
}

/// Bedrock color scheme
fn bedrock_color(noise_value: f32) -> [u8; 4] {
    // Bedrock colors: dark grays with some purple/blue variations
    let base = 64 + (noise_value * 32.0) as u8;
    let r = base + (noise_value * 16.0) as u8;
    let g = base;
    let b = base + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}

/// Leaves color scheme
fn leaves_color(noise_value: f32) -> [u8; 4] {
    // Leaves colors: greens with some yellow variations
    let r = 32 + (noise_value * 32.0) as u8;
    let g = 128 + (noise_value * 64.0) as u8;
    let b = 32 + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}

/// System to create a demo entity with procedural texture
pub fn spawn_procedural_texture_demo(
    mut commands: Commands,
    settings: Res<TextureGenSettings>,
) {
    println!("🎨 Spawning procedural texture demo...");
    
    // Spawn a quad to display the procedural texture
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(settings.texture_size.x as f32, settings.texture_size.y as f32)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        ProceduralTexture, // Mark this entity for procedural texture generation
    ));
}

/// System to generate procedural textures for specific block types
pub fn generate_block_type_textures(
    commands: Commands,
    _settings: Res<TextureGenSettings>,
    mut images: ResMut<Assets<Image>>,
) {
    println!("🎨 Generating procedural textures for block types...");
    
    // Generate textures for different block types
    let block_types = ["stone", "dirt", "grass", "wood", "sand", "water", "bedrock", "leaves"];
    
    for block_type in block_types {
        // Create settings for this block type
        let block_settings = TextureGenSettings::for_block_type(block_type);
        
        // Generate texture data
        let texture_data = generate_procedural_texture_data(&block_settings);
        
        // Create image
        let image = Image::new(
            Extent3d {
                width: block_settings.texture_size.x,
                height: block_settings.texture_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        
        // Add to assets
        let _image_handle = images.add(image);
        
        println!("✓ Generated {} texture with size {:?}", block_type, block_settings.texture_size);
        
        // Store the texture handle in a resource or component for later use
        // This would be used by the texture atlas system
    }
}

/// Resource to store generated block textures
#[derive(Resource, Debug)]
pub struct BlockTextures {
    pub textures: HashMap<String, Handle<Image>>, // Map block type names to texture handles
}

/// Component to mark entities that need dynamic texture regeneration
#[derive(Component, Debug)]
pub struct DynamicTexture {
    pub block_type: String,
    pub settings: TextureGenSettings,
    pub needs_regeneration: bool,
}

impl DynamicTexture {
    pub fn new(block_type: &str) -> Self {
        Self {
            block_type: block_type.to_string(),
            settings: TextureGenSettings::for_block_type(block_type),
            needs_regeneration: true,
        }
    }
}

impl Default for BlockTextures {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }
}

/// System to initialize block textures resource
pub fn initialize_block_textures(
    mut commands: Commands,
    settings: Res<TextureGenSettings>,
    alkyd_resources: Option<Res<AlkydResources>>,
    mut images: ResMut<Assets<Image>>,
) {
    println!("🎨 Initializing block textures resource...");
    
    let mut block_textures = BlockTextures::default();
    
    // Generate textures for different block types
    let block_types = ["stone", "dirt", "grass", "wood", "sand", "water", "bedrock", "leaves"];
    
    for block_type in block_types {
        // Create settings for this block type
        let block_settings = TextureGenSettings::for_block_type(block_type);
        
        // Generate texture data using alkyd-enhanced algorithms if available
        let texture_data = if let Some(alkyd) = &alkyd_resources {
            if alkyd.shaders_loaded {
                // Use alkyd-inspired enhanced algorithms
                let alkyd_config = AlkydTextureConfig::for_block_type(block_type);
                crate::alkyd_integration::generate_alkyd_texture_data(&alkyd_config)
            } else {
                // Use original algorithm
                generate_procedural_texture_data(&block_settings)
            }
        } else {
            // Use original algorithm
            generate_procedural_texture_data(&block_settings)
        };
        
        // Create image
        let image = Image::new(
            Extent3d {
                width: block_settings.texture_size.x,
                height: block_settings.texture_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        
        // Add to assets and store handle
        let image_handle = images.add(image);
        block_textures.textures.insert(block_type.to_string(), image_handle);
        
        println!("✓ Generated {} texture with size {:?}", block_type, block_settings.texture_size);
    }
    
    // Replace the existing resource with the new one
    let textures_count = block_textures.textures.len();
    commands.insert_resource(block_textures);
    println!("✓ BlockTextures resource initialized with {} textures", textures_count);
}

/// System to handle dynamic texture regeneration
pub fn regenerate_dynamic_textures(
    mut commands: Commands,
    mut query: Query<(Entity, &DynamicTexture)>, 
    mut images: ResMut<Assets<Image>>,
    mut block_textures: ResMut<BlockTextures>,
) {
    for (entity, dynamic_texture) in &mut query {
        if dynamic_texture.needs_regeneration {
            println!("🔄 Regenerating texture for {:?}", dynamic_texture.block_type);
            
            // Generate new texture data with updated settings
            let texture_data = generate_procedural_texture_data(&dynamic_texture.settings);
            
            // Create new image
            let image = Image::new(
                Extent3d {
                    width: dynamic_texture.settings.texture_size.x,
                    height: dynamic_texture.settings.texture_size.y,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                texture_data,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            
            // Replace the texture in assets
            let new_image_handle = images.add(image);
            
            // Update the block textures resource
            block_textures.textures.insert(dynamic_texture.block_type.clone(), new_image_handle.clone());
            
            // Update the entity with the new texture
            commands.entity(entity).insert(new_image_handle);
            
            // Mark as no longer needing regeneration
            commands.entity(entity).remove::<DynamicTexture>();
            
            println!("✓ Texture regenerated for {:?}", dynamic_texture.block_type);
        }
    }
}

/// System to create a dynamic texture entity
pub fn spawn_dynamic_texture_entity(
    mut commands: Commands,
    block_type: &str,
) {
    println!("🎨 Spawning dynamic texture entity for {}...", block_type);
    
    let dynamic_texture = DynamicTexture::new(block_type);
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    dynamic_texture.settings.texture_size.x as f32,
                    dynamic_texture.settings.texture_size.y as f32
                )),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        dynamic_texture,
    ));
}

/// System to trigger texture regeneration for all block types
pub fn trigger_texture_regeneration(
    mut commands: Commands,
    current_block_textures: Res<BlockTextures>,
) {
    println!("🔄 Triggering texture regeneration for all block types...");
    
    // Create dynamic texture components for all existing block types
    for (block_type, _) in &current_block_textures.textures {
        let mut settings = TextureGenSettings::for_block_type(block_type);
        // Modify settings slightly to create variation
        settings.noise_seed += 1; // Change seed for different pattern
        
        let dynamic_texture = DynamicTexture {
            block_type: block_type.clone(),
            settings,
            needs_regeneration: true,
        };
        
        commands.spawn(dynamic_texture);
    }
}