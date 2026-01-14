// Simplified texture generation module that uses only alkyd-generated textures
// This module provides a minimal interface for texture generation using alkyd

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

use crate::alkyd_integration::{AlkydResources, AlkydTextureConfig, EnhancedBlockTextures};

/// Resource to hold texture generation settings
#[derive(Resource, Debug)]
pub struct TextureGenSettings {
    pub texture_size: UVec2,
    pub noise_scale: f32,
    pub noise_octaves: usize,
    pub use_alkyd: bool,
    pub color_scheme: String,
}

impl Default for TextureGenSettings {
    fn default() -> Self {
        Self {
            texture_size: UVec2::new(256, 256),
            noise_scale: 0.05,
            noise_octaves: 4,
            use_alkyd: true,
            color_scheme: "natural".to_string(),
        }
    }
}

impl TextureGenSettings {
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
                use_alkyd: true,
                color_scheme: "stone".to_string(),
            },
            "dirt" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.08,
                noise_octaves: 5,
                use_alkyd: true,
                color_scheme: "dirt".to_string(),
            },
            "grass" => Self {
                texture_size: UVec2::new(128, 128),
                noise_scale: 0.07,
                noise_octaves: 4,
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

/// Component to store an image handle on an entity
#[derive(Component)]
pub struct EntityImageHandle {
    pub handle: Handle<Image>,
}

/// System to generate procedural textures using alkyd
pub fn generate_procedural_textures(
    mut commands: Commands,
    settings: Res<TextureGenSettings>,
    alkyd_resources: Option<Res<AlkydResources>>,
    enhanced_textures: Option<Res<EnhancedBlockTextures>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<Entity, Added<ProceduralTexture>>,
) {
    for entity in &query {
        // Use alkyd-generated textures if available
        let texture_data = if let Some(alkyd) = &alkyd_resources {
            if alkyd.shaders_loaded {
                // Use alkyd-inspired enhanced algorithms
                let alkyd_config = AlkydTextureConfig {
                    texture_size: settings.texture_size,
                    noise_scale: settings.noise_scale,
                    noise_octaves: settings.noise_octaves,
                    use_simplex_noise: true,
                    base_color: [0.5, 0.5, 0.5],
                    color_variation: 0.3,
                    use_gpu_acceleration: true,
                    enable_edge_detection: false,
                    enable_color_blending: false,
                    blend_mode: "normal".to_string(),
                    noise_type: "simplex".to_string(),
                };
                crate::alkyd_integration::generate_alkyd_texture_data(&alkyd_config)
            } else {
                // Fallback to simple gradient if alkyd not available
                generate_fallback_texture_data(&settings)
            }
        } else {
            // Fallback to simple gradient if alkyd not available
            generate_fallback_texture_data(&settings)
        };

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
        
        // Add the image to the entity
        commands.entity(entity).insert(EntityImageHandle {
            handle: image_handle.clone(),
        });
        
        println!("🎨 Generated procedural texture for entity {:?}", entity);
    }
}

/// Generate fallback texture data (simple gradient)
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

    assert_eq!(texture_data.len(), expected_size, "Texture data size mismatch");
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

/// System to initialize block textures resource using alkyd-generated textures
pub fn initialize_block_textures(
    mut commands: Commands,
    settings: Res<TextureGenSettings>,
    alkyd_resources: Option<Res<AlkydResources>>,
    enhanced_textures: Option<Res<EnhancedBlockTextures>>,
    mut images: ResMut<Assets<Image>>,
) {
    println!("🎨 Initializing block textures resource using alkyd-generated textures...");
    
    let mut block_textures = BlockTextures::default();
    
    // If we have enhanced alkyd textures, use those
    if let Some(enhanced) = &enhanced_textures {
        if !enhanced.textures.is_empty() {
            println!("✓ Using pre-generated enhanced alkyd textures");
            // Copy the enhanced textures to our BlockTextures resource
            for (block_type, texture_handle) in &enhanced.textures {
                block_textures.textures.insert(block_type.clone(), texture_handle.clone());
                println!("  ✓ Loaded enhanced alkyd texture for: {}", block_type);
            }
        }
    }
    
    // If no enhanced textures available, generate basic ones using alkyd
    if block_textures.textures.is_empty() {
        println!("ℹ Generating basic alkyd textures (enhanced textures not available)");
        
        // Generate textures for different block types
        let block_types = ["stone", "dirt", "grass", "wood", "sand", "water", "bedrock", "leaves"];
        
        for block_type in block_types {
            // Create settings for this block type
            let block_settings = TextureGenSettings::for_block_type(block_type);
            
            // Generate texture data using alkyd
            let texture_data = if let Some(alkyd) = &alkyd_resources {
                if alkyd.shaders_loaded {
                    let alkyd_config = AlkydTextureConfig::for_block_type(block_type);
                    crate::alkyd_integration::generate_alkyd_texture_data(&alkyd_config)
                } else {
                    generate_fallback_texture_data(&block_settings)
                }
            } else {
                generate_fallback_texture_data(&block_settings)
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
    }
    
    // Replace the existing resource with the new one
    let textures_count = block_textures.textures.len();
    commands.insert_resource(block_textures);
    println!("✓ BlockTextures resource initialized with {} textures", textures_count);
}

/// System to handle dynamic texture regeneration using alkyd
pub fn regenerate_dynamic_textures(
    mut commands: Commands,
    mut query: Query<(Entity, &DynamicTexture)>, 
    alkyd_resources: Option<Res<AlkydResources>>,
    mut images: ResMut<Assets<Image>>,
    mut block_textures: ResMut<BlockTextures>,
) {
    for (entity, dynamic_texture) in &mut query {
        if dynamic_texture.needs_regeneration {
            println!("🔄 Regenerating texture for {:?}", dynamic_texture.block_type);
            
            // Generate new texture data with updated settings using alkyd
            let texture_data = if let Some(alkyd) = &alkyd_resources {
                if alkyd.shaders_loaded {
                    let alkyd_config = AlkydTextureConfig::for_block_type(&dynamic_texture.block_type);
                    crate::alkyd_integration::generate_alkyd_texture_data(&alkyd_config)
                } else {
                    generate_fallback_texture_data(&dynamic_texture.settings)
                }
            } else {
                generate_fallback_texture_data(&dynamic_texture.settings)
            };
            
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
            commands.entity(entity).insert(EntityImageHandle {
                handle: new_image_handle,
            });
            
            // Mark as no longer needing regeneration
            commands.entity(entity).remove::<DynamicTexture>();
            
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
    pub fn new(block_type: &str) -> Self {
        Self {
            block_type: block_type.to_string(),
            settings: TextureGenSettings::for_block_type(block_type),
            needs_regeneration: true,
        }
    }
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
