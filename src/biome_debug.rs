// Biome Debugging and Visualization Tools
// This module provides tools for visualizing biome boundaries and texture variations

use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::math::primitives::Cuboid;
use crate::biome_textures::BiomeTextureParams;
use crate::block::BlockType;
use crate::chunk::ChunkPosition;

/// Resource for controlling biome debug visualization
#[derive(Resource, Debug, Clone)]
pub struct BiomeDebugSettings {
    /// Enable biome boundary visualization
    pub show_biome_boundaries: bool,
    /// Enable biome texture variation visualization
    pub show_texture_variations: bool,
    /// Enable biome parameter display
    pub show_biome_parameters: bool,
    /// Enable biome cache statistics display
    pub show_cache_stats: bool,
    /// Biome boundary visualization mode
    pub boundary_mode: BiomeBoundaryMode,
    /// Texture variation visualization mode
    pub texture_mode: BiomeTextureMode,
    /// Debug UI position
    pub ui_position: BiomeDebugUIPosition,
    /// Toggle for advanced biome debugging
    pub advanced_debug: bool,
}

impl Default for BiomeDebugSettings {
    fn default() -> Self {
        Self {
            show_biome_boundaries: false,
            show_texture_variations: false,
            show_biome_parameters: false,
            show_cache_stats: false,
            boundary_mode: BiomeBoundaryMode::Wireframe,
            texture_mode: BiomeTextureMode::ColorGradient,
            ui_position: BiomeDebugUIPosition::TopLeft,
            advanced_debug: false,
        }
    }
}

/// Biome boundary visualization modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiomeBoundaryMode {
    Wireframe,
    Solid,
    Transparent,
    HeightMap,
}

/// Biome texture visualization modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiomeTextureMode {
    ColorGradient,
    ParameterOverlay,
    TextureAtlas,
    NormalMap,
}

/// Biome debug UI position
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiomeDebugUIPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Component for marking biome boundary visualization entities
#[derive(Component, Debug)]
pub struct BiomeBoundaryVisualization {
    pub chunk_position: ChunkPosition,
    pub biome_type: String,
}

/// Component for marking biome texture variation entities
#[derive(Component, Debug)]
pub struct BiomeTextureVisualization {
    pub block_type: BlockType,
    pub biome_params: BiomeTextureParams,
}

/// Component for biome debug UI elements
#[derive(Component, Debug)]
pub struct BiomeDebugUI;

/// Biome debug statistics
#[derive(Resource, Debug, Default)]
pub struct BiomeDebugStats {
    pub active_biomes: usize,
    pub biome_transitions: usize,
    pub texture_variations: usize,
    pub cache_hit_rate: f32,
    pub last_updated: f64,
}

/// Initialize biome debug systems
pub fn initialize_biome_debug_system(
    mut commands: Commands,
) {
    // Add default debug settings resource
    commands.insert_resource(BiomeDebugSettings::default());
    commands.insert_resource(BiomeDebugStats::default());
    
    info!("🔍 Biome debug system initialized");
}

/// System to toggle biome debug visualization
pub fn toggle_biome_debug(
    mut debug_settings: ResMut<BiomeDebugSettings>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        debug_settings.show_biome_boundaries = !debug_settings.show_biome_boundaries;
        debug_settings.show_texture_variations = !debug_settings.show_texture_variations;
        debug_settings.show_biome_parameters = !debug_settings.show_biome_parameters;
        
        let status = if debug_settings.show_biome_boundaries {
            "enabled"
        } else {
            "disabled"
        };
        info!("🔍 Biome debug visualization: {}", status);
    }
    
    if keyboard_input.just_pressed(KeyCode::F4) {
        debug_settings.advanced_debug = !debug_settings.advanced_debug;
        info!("🔬 Advanced biome debug: {}", 
            if debug_settings.advanced_debug { "enabled" } else { "disabled" });
    }
}

/// System to display biome debug information
pub fn display_biome_debug_info(
    debug_settings: Res<BiomeDebugSettings>,
    debug_stats: Res<BiomeDebugStats>,
    game_time: Res<crate::time::GameTime>,
) {
    if !debug_settings.show_biome_parameters {
        return;
    }
    
    if game_time.current_time % 3.0 < 0.1 { // Display every 3 seconds
        println!("🌿 Biome Debug Info:");
        println!("   Active Biomes: {}", debug_stats.active_biomes);
        println!("   Biome Transitions: {}", debug_stats.biome_transitions);
        println!("   Texture Variations: {}", debug_stats.texture_variations);
        println!("   Cache Hit Rate: {:.1}%", debug_stats.cache_hit_rate * 100.0);
        println!("   Boundary Mode: {:?}", debug_settings.boundary_mode);
        println!("   Texture Mode: {:?}", debug_settings.texture_mode);
    }
}

/// System to visualize biome boundaries
pub fn visualize_biome_boundaries(
    debug_settings: Res<BiomeDebugSettings>,
    biome_cache: Res<crate::biome_texture_cache::SharedBiomeTextureCache>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !debug_settings.show_biome_boundaries {
        return;
    }
    
    // Get biome cache statistics for visualization
    let cache = biome_cache.cache.lock().unwrap();
    let stats = cache.get_stats();
    
    // Create a simple visualization mesh to represent biome boundaries
    // This is a placeholder - in a full implementation, this would create
    // actual boundary meshes based on biome transitions
    
    // Create a small debug cube to represent biome boundary points
    let cube_mesh = meshes.add(Mesh::from(Cuboid {
        half_size: Vec3::new(0.25, 0.25, 0.25),
    }));
    
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0), // Green for biome boundaries
        emissive: Color::srgb(0.0, 0.5, 0.0).into(),
        ..default()
    });
    
    // Spawn a few debug cubes to visualize biome boundaries
    // In a real implementation, these would be placed at actual biome transitions
    for i in 0..stats.current_textures.min(10) {
        let x = i as f32 * 2.0 - 5.0;
        let y = 10.0;
        let z = 0.0;
        
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(material.clone()),
            BiomeBoundaryVisualization {
                chunk_position: ChunkPosition { x: 0, z: 0 },
                biome_type: format!("debug_biome_{}", i),
            },
        ));
    }
    
    info!("🎨 Visualized {} biome boundary markers", stats.current_textures.min(10));
}

/// System to visualize biome texture variations
pub fn visualize_biome_texture_variations(
    debug_settings: Res<BiomeDebugSettings>,
    biome_cache: Res<crate::biome_texture_cache::SharedBiomeTextureCache>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !debug_settings.show_texture_variations {
        return;
    }
    
    // Get biome cache statistics for visualization
    let cache = biome_cache.cache.lock().unwrap();
    let stats = cache.get_stats();
    
    // Create visualization for different biome texture variations
    // This shows the variety of textures being used
    
    let cube_mesh = meshes.add(Mesh::from(Cuboid {
        half_size: Vec3::new(0.5, 0.5, 0.5),
    }));
    
    // Create different colored materials to represent texture variations
    let colors = vec![
        Color::srgb(1.0, 0.0, 0.0), // Red
        Color::srgb(0.0, 1.0, 0.0), // Green
        Color::srgb(0.0, 0.0, 1.0), // Blue
        Color::srgb(1.0, 1.0, 0.0), // Yellow
        Color::srgb(1.0, 0.0, 1.0), // Magenta
        Color::srgb(0.0, 1.0, 1.0), // Cyan
    ];
    
    // Spawn cubes to represent different texture variations
    for (i, color) in colors.iter().enumerate() {
        let material = materials.add(StandardMaterial {
            base_color: *color,
            ..default()
        });
        
        let x = i as f32 * 2.0 - 5.0;
        let y = 12.0;
        let z = 2.0;
        
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(material.clone()),
            BiomeTextureVisualization {
                block_type: BlockType::Grass, // Example block type
                biome_params: BiomeTextureParams {
                    biome_type: format!("variation_{}", i),
                    temperature: 0.5,
                    moisture: 0.5,
                    height: 10.0,
                    relative_height: 0.5,
                },
            },
        ));
    }
    
    info!("🎨 Visualized {} biome texture variations", colors.len());
}

/// System to update biome debug statistics
pub fn update_biome_debug_stats(
    mut debug_stats: ResMut<BiomeDebugStats>,
    time: Res<Time>,
) {
    debug_stats.last_updated = time.elapsed_secs_f64();
    // TODO: Update statistics from biome systems
}

/// System to create biome debug UI
pub fn create_biome_debug_ui(
    debug_settings: Res<BiomeDebugSettings>,
    _commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    if !debug_settings.show_biome_parameters {
        return;
    }
    
    // TODO: Implement UI creation using Bevy UI components
    // This will show biome information in a panel
}

/// Helper function to get biome color for visualization
pub fn get_biome_color(biome_type: &str) -> Color {
    match biome_type {
        "desert" => Color::srgb(0.9, 0.8, 0.2),
        "forest" => Color::srgb(0.2, 0.6, 0.2),
        "jungle" => Color::srgb(0.1, 0.5, 0.1),
        "grassland" => Color::srgb(0.3, 0.7, 0.3),
        "swamp" => Color::srgb(0.2, 0.4, 0.2),
        "tundra" => Color::srgb(0.7, 0.8, 0.9),
        "snowy_mountain" => Color::srgb(0.9, 0.9, 0.9),
        "rocky_mountain" => Color::srgb(0.5, 0.5, 0.5),
        "plains" => Color::srgb(0.5, 0.7, 0.3),
        "savanna" => Color::srgb(0.8, 0.7, 0.2),
        _ => Color::srgb(0.5, 0.5, 0.5), // Default gray
    }
}

/// Helper function to get biome display name
pub fn get_biome_display_name(biome_type: &str) -> String {
    biome_type.replace('_', " ").to_title_case()
}

/// Extension trait for string title case conversion
pub trait StringTitleCase {
    fn to_title_case(&self) -> String;
}

impl StringTitleCase for str {
    fn to_title_case(&self) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;
        
        for c in self.chars() {
            if capitalize_next && c.is_alphabetic() {
                result.push(c.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(c.to_lowercase().next().unwrap());
            }
            
            if c == ' ' || c == '_' || c == '-' {
                capitalize_next = true;
            }
        }
        
        result
    }
}