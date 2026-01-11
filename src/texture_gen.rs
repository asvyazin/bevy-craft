use bevy::prelude::*;
use alkyd::prelude::*;

#[cfg(test)]
mod texture_gen_test;

/// Resource to hold texture generation settings
#[derive(Resource, Default)]
pub struct TextureGenSettings {
    pub texture_size: UVec2,
    pub noise_scale: f32,
    pub noise_octaves: usize,
}

impl TextureGenSettings {
    pub fn new() -> Self {
        Self {
            texture_size: UVec2::new(256, 256),
            noise_scale: 0.05,
            noise_octaves: 4,
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
    mut images: ResMut<Assets<Image>>,
    query: Query<Entity, Added<ProceduralTexture>>,
) {
    for entity in &query {
        // Create a new image for the procedural texture
        let mut image = Image::new_fill(
            Extent3d {
                width: settings.texture_size.x,
                height: settings.texture_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
        );

        // Generate procedural texture data
        let texture_data = generate_procedural_texture_data(&settings);
        
        // Fill the image with generated data
        image.data = texture_data;

        // Add the image to assets
        let image_handle = images.add(image);
        
        // Add the image to the entity
        commands.entity(entity).insert(image_handle);
        
        println!("🎨 Generated procedural texture for entity {:?}", entity);
    }
}

/// Generate procedural texture data using noise
fn generate_procedural_texture_data(settings: &TextureGenSettings) -> Vec<u8> {
    let mut texture_data = Vec::with_capacity(
        (settings.texture_size.x * settings.texture_size.y * 4) as usize
    );

    for y in 0..settings.texture_size.y {
        for x in 0..settings.texture_size.x {
            // Generate noise value for this pixel
            let noise_value = generate_noise_value(
                x as f32 * settings.noise_scale,
                y as f32 * settings.noise_scale,
                settings.noise_octaves
            );

            // Convert noise value to color
            let color = noise_to_color(noise_value);
            
            // Add color to texture data (RGBA format)
            texture_data.extend_from_slice(&color);
        }
    }

    texture_data
}

/// Simple noise generation function (placeholder - in real implementation, use alkyd's noise)
fn generate_noise_value(x: f32, y: f32, octaves: usize) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        // Simple pseudo-random noise (replace with proper noise function)
        let noise = (x * frequency + y * frequency).sin() * 0.5 + 0.5;
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

/// Convert noise value to RGBA color
fn noise_to_color(noise_value: f32) -> [u8; 4] {
    // Map noise value to color gradient
    let r = (noise_value * 255.0) as u8;
    let g = ((1.0 - noise_value) * 255.0) as u8;
    let b = ((noise_value * 0.5 + 0.25) * 255.0) as u8;
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