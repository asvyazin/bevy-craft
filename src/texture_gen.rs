// Simplified texture generation module
// This module provides a minimal interface for texture generation

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

/// Resource to hold texture generation settings
#[derive(Resource, Debug)]
pub struct TextureGenSettings {
    pub texture_size: UVec2,
    pub noise_scale: f32,
    pub noise_octaves: usize,
    #[allow(dead_code)]
    pub color_scheme: String,
}

impl Default for TextureGenSettings {
    fn default() -> Self {
        Self {
            texture_size: UVec2::new(256, 256),
            noise_scale: 0.05,
            noise_octaves: 4,
            color_scheme: "natural".to_string(),
        }
    }
}

impl TextureGenSettings {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create settings for a specific block type
    pub fn for_block_type(block_type: &str) -> Self {
        match block_type {
            "stone" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.1,
                noise_octaves: 6,
                color_scheme: "stone".to_string(),
            },
            "dirt" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.08,
                noise_octaves: 5,
                color_scheme: "dirt".to_string(),
            },
            "grass" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.07,
                noise_octaves: 4,
                color_scheme: "grass".to_string(),
            },
            _ => Self::default(),
        }
    }
}

/// Component to mark entities that should have procedural textures
#[derive(Component)]
pub struct ProceduralTexture;

/// Component to store an image handle on an entity
#[derive(Component)]
pub struct EntityImageHandle {
    #[allow(dead_code)]
    pub handle: Handle<Image>,
}

/// System to generate procedural textures
pub fn generate_procedural_textures(
    mut commands: Commands,
    settings: Res<TextureGenSettings>,
    mut images: ResMut<Assets<Image>>,
    query: Query<Entity, Added<ProceduralTexture>>,
) {
    for entity in &query {
        // Generate simple procedural texture
        let texture_data = generate_simple_texture_data(
            settings.texture_size.x,
            settings.texture_size.y,
            settings.noise_scale,
            settings.noise_octaves,
        );

        // Create a new image for the procedural texture
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

        // Add the image to the entity with existence check
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            // Double-check entity exists before inserting
            if entity_commands.id() == entity {
                entity_commands.insert(EntityImageHandle {
                    handle: image_handle.clone(),
                });
                println!("🎨 Generated procedural texture for entity {:?}", entity);
            }
        }
    }
}

/// Generate fallback texture data (simple gradient)
#[allow(dead_code)]
fn generate_fallback_texture_data(settings: &TextureGenSettings) -> Vec<u8> {
    let expected_size = (settings.texture_size.x * settings.texture_size.y * 4) as usize;
    let mut texture_data = Vec::with_capacity(expected_size);

    for y in 0..settings.texture_size.y {
        for x in 0..settings.texture_size.x {
            // Simple gradient pattern as fallback
            let nx = x as f32 / settings.texture_size.x as f32;
            let ny = y as f32 / settings.texture_size.y as f32;
            let noise_value = (nx * ny * 10.0).sin() * 0.5 + 0.5;

            // Convert to color based on color scheme
            let color = match settings.color_scheme.as_str() {
                "stone" => stone_color(noise_value),
                "dirt" => dirt_color(noise_value),
                "grass" => grass_color(noise_value),
                _ => natural_color(noise_value),
            };

            texture_data.extend_from_slice(&color);
        }
    }

    assert_eq!(
        texture_data.len(),
        expected_size,
        "Texture data size mismatch"
    );
    texture_data
}

/// Resource to store generated block textures
#[derive(Resource, Debug)]
pub struct BlockTextures {
    pub textures: HashMap<String, Handle<Image>>, // Map block type names to texture handles
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
    _commands: Commands,
    _settings: Res<TextureGenSettings>,
    mut images: ResMut<Assets<Image>>,
    mut block_textures: ResMut<BlockTextures>,
) {
    println!("🎨 Initializing block textures resource...");

    // Generate basic textures
    if block_textures.textures.is_empty() {
        println!("ℹ Generating basic textures");

        // Generate textures for different block types
        let block_types = [
            "stone", "dirt", "grass", "wood", "sand", "water", "bedrock", "leaves",
        ];

        for block_type in block_types {
            // Create settings for this block type
            let block_settings = TextureGenSettings::for_block_type(block_type);

            // Generate texture data
            let texture_data = generate_simple_texture_data(
                block_settings.texture_size.x,
                block_settings.texture_size.y,
                block_settings.noise_scale,
                block_settings.noise_octaves,
            );

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
            block_textures
                .textures
                .insert(block_type.to_string(), image_handle);

            println!(
                "✓ Generated {} texture with size {:?}",
                block_type, block_settings.texture_size
            );
        }
    }

    // Replace the existing resource with the new one
    let textures_count = block_textures.textures.len();
    // No need to insert block_textures as it's already a resource
    println!(
        "✓ BlockTextures resource initialized with {} textures",
        textures_count
    );
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
            println!(
                "🔄 Regenerating texture for {:?}",
                dynamic_texture.block_type
            );

            // Generate new texture data with updated settings
            let texture_data = generate_simple_texture_data(
                dynamic_texture.settings.texture_size.x,
                dynamic_texture.settings.texture_size.y,
                dynamic_texture.settings.noise_scale,
                dynamic_texture.settings.noise_octaves,
            );

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
            block_textures
                .textures
                .insert(dynamic_texture.block_type.clone(), new_image_handle.clone());

            // Update the entity with the new texture with existence check
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                // Double-check entity exists before updating
                if entity_commands.id() == entity {
                    entity_commands.insert(EntityImageHandle {
                        handle: new_image_handle,
                    });

                    // Mark as no longer needing regeneration
                    entity_commands.remove::<DynamicTexture>();
                }
            }

            println!("✓ Texture regenerated for {:?}", dynamic_texture.block_type);
        }
    }
}

/// Component to mark entities that need dynamic texture regeneration
#[derive(Component, Debug)]
pub struct DynamicTexture {
    pub block_type: String,
    pub settings: TextureGenSettings,
    pub needs_regeneration: bool,
}

impl DynamicTexture {
    #[allow(dead_code)]
    pub fn new(block_type: &str) -> Self {
        Self {
            block_type: block_type.to_string(),
            settings: TextureGenSettings::for_block_type(block_type),
            needs_regeneration: true,
        }
    }
}

/// System to create a demo entity with procedural texture
#[allow(dead_code)]
pub fn spawn_procedural_texture_demo(mut commands: Commands, settings: Res<TextureGenSettings>) {
    println!("🎨 Spawning procedural texture demo...");

    // Spawn a quad to display the procedural texture
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(
                settings.texture_size.x as f32,
                settings.texture_size.y as f32,
            )),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        ProceduralTexture, // Mark this entity for procedural texture generation
    ));
}

/// Generate simple procedural texture data
fn generate_simple_texture_data(width: u32, height: u32, scale: f32, octaves: usize) -> Vec<u8> {
    let mut texture_data = vec![0; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            // Simple noise calculation
            let nx = x as f32 * scale;
            let ny = y as f32 * scale;

            // Basic hash-based noise
            let mut value = 0.0;
            let mut amplitude = 1.0;
            let mut max_value = 0.0;

            for _ in 0..octaves {
                let i = nx.floor() as i32;
                let j = ny.floor() as i32;
                let fx = nx - i as f32;
                let fy = ny - j as f32;

                // Simple fade function
                let u = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
                let v = fy * fy * fy * (fy * (fy * 6.0 - 15.0) + 10.0);

                // Hash-based gradient values
                let grad00 = hash_noise(i, j, 0);
                let grad10 = hash_noise(i + 1, j, 1);
                let grad01 = hash_noise(i, j + 1, 2);
                let grad11 = hash_noise(i + 1, j + 1, 3);

                // Simple interpolation
                let lerp1 = grad00 + (grad10 - grad00) * u;
                let lerp2 = grad01 + (grad11 - grad01) * u;
                let noise_value = lerp1 + (lerp2 - lerp1) * v;

                value += noise_value * amplitude;
                max_value += amplitude;
                amplitude *= 0.5;
                // frequency *= 2.0; // Unused assignment
            }

            let noise_value = value / max_value;
            let color = noise_to_color(noise_value);

            let index = ((y * width + x) * 4) as usize;
            texture_data[index] = color[0];
            texture_data[index + 1] = color[1];
            texture_data[index + 2] = color[2];
            texture_data[index + 3] = color[3];
        }
    }

    texture_data
}

/// Generate biome-aware procedural texture data
pub fn generate_biome_texture_data(
    width: u32,
    height: u32,
    biome_params: &crate::biome_textures::BiomeTextureParams,
    block_type: &str,
) -> Vec<u8> {
    let mut texture_data = vec![0; (width * height * 4) as usize];

    // Adjust noise parameters based on biome
    let base_scale = match block_type {
        "stone" => 0.1,
        "dirt" => 0.08,
        "grass" => 0.07,
        "wood" => 0.06,
        "sand" => 0.09,
        _ => 0.05,
    };

    let base_octaves = match block_type {
        "stone" => 6,
        "dirt" => 5,
        "grass" => 4,
        "wood" => 4,
        "sand" => 3,
        _ => 4,
    };

    // Apply biome modifications to noise parameters
    let scale = base_scale * (1.0 + biome_params.temperature * 0.2 - biome_params.moisture * 0.1);
    let octaves = (base_octaves as f32 * (1.0 + biome_params.height * 0.1)) as usize;

    for y in 0..height {
        for x in 0..width {
            // Biome-aware noise calculation
            let nx = x as f32 * scale;
            let ny = y as f32 * scale;

            // Add biome-specific variations
            let biome_variation_x = biome_params.temperature * 100.0;
            let biome_variation_y = biome_params.moisture * 100.0;
            let nx = nx + biome_variation_x;
            let ny = ny + biome_variation_y;

            // Enhanced hash-based noise with biome influence
            let mut value = 0.0;
            let mut amplitude = 1.0;
            let mut max_value = 0.0;

            for _ in 0..octaves {
                let i = nx.floor() as i32;
                let j = ny.floor() as i32;
                let fx = nx - i as f32;
                let fy = ny - j as f32;

                // Biome-influenced fade function
                let u = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
                let v = fy * fy * fy * (fy * (fy * 6.0 - 15.0) + 10.0);

                // Hash-based gradient values with biome seed
                let biome_seed =
                    (biome_params.temperature * 1000.0 + biome_params.moisture * 100.0) as i32;
                let grad00 = hash_noise(i, j, biome_seed);
                let grad10 = hash_noise(i + 1, j, biome_seed + 1);
                let grad01 = hash_noise(i, j + 1, biome_seed + 2);
                let grad11 = hash_noise(i + 1, j + 1, biome_seed + 3);

                // Biome-influenced interpolation
                let lerp1 = grad00 + (grad10 - grad00) * u;
                let lerp2 = grad01 + (grad11 - grad01) * u;
                let noise_value = lerp1 + (lerp2 - lerp1) * v;

                value += noise_value * amplitude;
                max_value += amplitude;
                amplitude *= 0.5;
                // frequency *= 2.0; // Unused assignment
            }

            let noise_value = value / max_value;

            // Apply biome-specific color modifications
            let color = apply_biome_color_modifications(noise_value, biome_params, block_type);

            let index = ((y * width + x) * 4) as usize;
            texture_data[index] = color[0];
            texture_data[index + 1] = color[1];
            texture_data[index + 2] = color[2];
            texture_data[index + 3] = color[3];
        }
    }

    texture_data
}

/// Apply biome-specific color modifications to base noise value
fn apply_biome_color_modifications(
    noise_value: f32,
    biome_params: &crate::biome_textures::BiomeTextureParams,
    block_type: &str,
) -> [u8; 4] {
    let base_color = match block_type {
        "stone" => stone_color(noise_value),
        "dirt" => dirt_color(noise_value),
        "grass" => grass_color(noise_value),
        "wood" => wood_color(noise_value),
        "sand" => sand_color(noise_value),
        "water" => water_color(noise_value),
        "bedrock" => bedrock_color(noise_value),
        "leaves" => leaves_color(noise_value),
        _ => natural_color(noise_value),
    };

    // Apply biome-specific color modifications
    let mut r = base_color[0] as f32;
    let mut g = base_color[1] as f32;
    let mut b = base_color[2] as f32;

    // Temperature effects
    if biome_params.temperature > 0.7 {
        // Hot biomes - more red/yellow tones
        r = (r as f32 * 1.2).min(255.0);
        g = (g as f32 * 0.9).min(255.0);
    } else if biome_params.temperature < 0.3 {
        // Cold biomes - more blue tones
        b = (b as f32 * 1.3).min(255.0);
        r = (r as f32 * 0.8).min(255.0);
    }

    // Moisture effects
    if biome_params.moisture > 0.7 {
        // Wet biomes - more green/blue tones
        g = (g as f32 * 1.1).min(255.0);
        b = (b as f32 * 1.1).min(255.0);
    } else if biome_params.moisture < 0.3 {
        // Dry biomes - more brown/red tones
        r = (r as f32 * 1.1).min(255.0);
        g = (g as f32 * 0.9).min(255.0);
        b = (b as f32 * 0.8).min(255.0);
    }

    // Height effects
    if biome_params.relative_height > 0.8 {
        // High altitude - lighter colors
        r = (r as f32 * 1.1).min(255.0);
        g = (g as f32 * 1.1).min(255.0);
        b = (b as f32 * 1.1).min(255.0);
    } else if biome_params.relative_height < 0.2 {
        // Low altitude - darker colors
        r = (r as f32 * 0.9).min(255.0);
        g = (g as f32 * 0.9).min(255.0);
        b = (b as f32 * 0.9).min(255.0);
    }

    [r as u8, g as u8, b as u8, 255]
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
    let base_gray = 128 + (noise_value * 64.0) as u8;
    let variation = (noise_value * 32.0) as u8;
    let r = base_gray + variation;
    let g = base_gray + variation / 2;
    let b = base_gray - variation / 2;
    [r, g, b, 255]
}

/// Dirt color scheme
fn dirt_color(noise_value: f32) -> [u8; 4] {
    let r = 128 + (noise_value * 64.0) as u8;
    let g = 96 + (noise_value * 48.0) as u8;
    let b = 64 + (noise_value * 32.0) as u8;
    [r, g, b, 255]
}

/// Grass color scheme
fn grass_color(noise_value: f32) -> [u8; 4] {
    let r = 64 + (noise_value * 32.0) as u8;
    let g = 128 + (noise_value * 64.0) as u8;
    let b = 32 + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}

/// Wood color scheme
fn wood_color(noise_value: f32) -> [u8; 4] {
    let base = 128 + (noise_value * 32.0) as u8;
    let variation = (noise_value * 48.0) as u8;
    let r = base + variation;
    let g = base - variation / 2;
    let b = base - variation;
    [r, g, b, 255]
}

/// Sand color scheme
fn sand_color(noise_value: f32) -> [u8; 4] {
    let r = 192 + (noise_value * 32.0) as u8;
    let g = 160 + (noise_value * 32.0) as u8;
    let b = 128 + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}

/// Water color scheme
fn water_color(noise_value: f32) -> [u8; 4] {
    let r = 32 + (noise_value * 16.0) as u8;
    let g = 64 + (noise_value * 32.0) as u8;
    let b = 128 + (noise_value * 64.0) as u8;
    [r, g, b, 255]
}

/// Bedrock color scheme
fn bedrock_color(noise_value: f32) -> [u8; 4] {
    let base = 64 + (noise_value * 32.0) as u8;
    let r = base + (noise_value * 16.0) as u8;
    let g = base;
    let b = base + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}

/// Leaves color scheme
fn leaves_color(noise_value: f32) -> [u8; 4] {
    let r = 32 + (noise_value * 32.0) as u8;
    let g = 128 + (noise_value * 64.0) as u8;
    let b = 32 + (noise_value * 16.0) as u8;
    [r, g, b, 255]
}
