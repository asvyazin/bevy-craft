use bevy::prelude::*;
use bevy::math::primitives::Cuboid;
use bevy_compute_noise::prelude::*;

mod block;
use block::{Block, BlockType};

mod chunk;
use chunk::{Chunk, ChunkManager, ChunkPosition};

mod chunk_mesh;
use chunk_mesh::{ChunkMesh, ChunkMeshMaterials};

mod texture_atlas;
use texture_atlas::{TextureAtlas, initialize_texture_atlas, load_procedural_textures_into_atlas};

mod texture_gen;
use texture_gen::{TextureGenSettings, generate_procedural_textures, initialize_block_textures, BlockTextures, regenerate_dynamic_textures};

mod noise;
mod test_sophisticated_algorithms;

mod biome_textures;
mod biome_texture_cache;

mod world_gen;
mod player;
use world_gen::{WorldGenSettings, generate_chunks_system};
use crate::noise::NoiseSettings;
use player::PlayerMovementSettings;

mod camera;
use camera::{camera_mouse_control_system, camera_rotation_system, cursor_control_system, spawn_game_camera};

mod block_interaction;
use block_interaction::block_targeting_feedback_system;

mod collision;
use collision::{Collider, collision_detection_system, find_safe_spawn_position};

mod sky;
use sky::{spawn_skybox, spawn_sun_and_moon, update_sky_color, update_sun_and_moon_positions};

mod time;
use time::{GameTime, update_game_time, display_game_time};

fn main() {
    // Create the app first
    let mut app = App::new();
    
    // Add plugins and initialize resources
    app.add_plugins(DefaultPlugins)
        .add_plugins(ComputeNoisePlugin) // Add Perlin noise plugin for world generation
        .init_resource::<ChunkManager>()
        .init_resource::<WorldGenSettings>() // Initialize world generation settings
        .init_resource::<NoiseSettings>() // Initialize noise settings
        .init_resource::<PlayerMovementSettings>() // Initialize player movement settings
        .init_resource::<ChunkMeshMaterials>() // Initialize chunk mesh materials
        .init_resource::<TextureAtlas>() // Initialize texture atlas
        .init_resource::<TextureGenSettings>() // Initialize texture generation settings
        .init_resource::<BlockTextures>() // Initialize block textures resource
        .init_resource::<crate::biome_texture_cache::SharedBiomeTextureCache>() // Initialize biome texture cache
        .init_resource::<GameTime>() // Initialize game time for day/night cycle
        ;
    
    app
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_game_camera)
        .add_systems(Startup, initialize_texture_atlas)
        .add_systems(Startup, initialize_block_textures) // Use standard textures
        .add_systems(Startup, load_procedural_textures_into_atlas.after(initialize_block_textures))
        .add_systems(Startup, initialize_chunk_mesh_materials.after(load_procedural_textures_into_atlas))
        .add_systems(Startup, spawn_skybox) // Add skybox spawning after materials are ready
        .add_systems(Startup, spawn_sun_and_moon) // Add sun and moon spawning
        .add_systems(Startup, test_sophisticated_algorithms::test_sophisticated_algorithms)

        .add_systems(Update, generate_procedural_textures) // Add procedural texture generation
        .add_systems(Update, regenerate_dynamic_textures) // Add dynamic texture regeneration
        .add_systems(Update, update_game_time) // Add game time update system
        .add_systems(Update, display_game_time) // Add game time display system
        .add_systems(Update, update_sky_color) // Add sky color update system
        .add_systems(Update, update_sun_and_moon_positions) // Add sun and moon position update system

        .add_systems(Update, dynamic_chunk_loading_system) // Add dynamic chunk loading system
        .add_systems(Update, generate_chunk_meshes)
        .add_systems(Update, generate_chunks_system) // Add world generation system
        .add_systems(Update, render_chunk_meshes) // Add chunk mesh rendering system
        .add_systems(Startup, spawn_player_safe.after(setup)) // Add safe player spawning system
        .add_systems(Update, player::player_movement_system) // Add player movement system
        .add_systems(Update, collision_detection_system.after(player::player_movement_system)) // Add collision detection system
        .add_systems(Update, cursor_control_system) // Add cursor control system
        .add_systems(Update, camera_mouse_control_system) // Add mouse camera control system
        .add_systems(Update, camera_rotation_system) // Add camera rotation system
        .add_systems(Update, block_targeting_feedback_system) // Add block targeting feedback
        .add_systems(Update, block_interaction::block_interaction_system) // Add block interaction system (breaking and placement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    // Initialize chunk manager
    *chunk_manager = ChunkManager::new(2); // Render distance of 2 chunks

    // Camera is now spawned by the spawn_game_camera system

    // Add light
    commands.spawn((PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        }, Transform::from_xyz(4.0, 8.0, 4.0)));

    // Generate initial chunks around spawn point for dynamic loading system
    generate_initial_chunks(&mut commands, &mut chunk_manager);
}

/// Generate initial chunks around spawn point for dynamic loading system
fn generate_initial_chunks(commands: &mut Commands, chunk_manager: &mut ChunkManager) {
    println!("🌍 Generating initial chunks around spawn point...");
    
    // Generate initial chunks around the origin (spawn point)
    // This creates a buffer so the player starts with some terrain
    for x in -1..=1 {
        for z in -1..=1 {
            let chunk_pos = ChunkPosition::new(x, z);
            let chunk_entity = commands.spawn(Chunk::new(chunk_pos)).id();
            
            // Register the chunk in the manager
            chunk_manager.loaded_chunks.insert(chunk_pos, chunk_entity);
            
            println!("🌱 Spawned initial chunk at ({}, {}) - waiting for terrain generation", x, z);
        }
    }
    
    println!("🎮 Initial chunks created, dynamic chunk loading system will handle additional chunks");
}

/// System to initialize chunk mesh materials
fn initialize_chunk_mesh_materials(
    mut mesh_materials: ResMut<ChunkMeshMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_atlas: Res<TextureAtlas>,
) {
    println!("🎨 Initializing chunk mesh materials...");
    mesh_materials.initialize(&mut materials, &texture_atlas);
    println!("✓ Chunk mesh materials initialized");
}

/// System to generate chunk meshes
fn generate_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_materials: Res<ChunkMeshMaterials>,
    chunks: Query<(Entity, &Chunk), Without<ChunkMesh>>,
    all_chunks: Query<&Chunk>,
    chunk_manager: Res<ChunkManager>,
    texture_atlas: Res<TextureAtlas>,

    biome_cache: Res<crate::biome_texture_cache::SharedBiomeTextureCache>,
) {
    for (chunk_entity, chunk) in &chunks {
        if chunk.is_generated && chunk.needs_mesh_update {
            println!("🏗️  Generating mesh for chunk ({}, {})", chunk.position.x, chunk.position.z);
            
            // Generate the mesh for this chunk using neighbor-aware algorithm
            let mesh = chunk_mesh::generate_chunk_mesh(
                &chunk.data,
                &chunk.position,
                &chunk_manager,
                &all_chunks,
                &texture_atlas,
                chunk,
            );
            let mesh_handle = meshes.add(mesh);
            
            // Create the chunk mesh component
            let mut chunk_mesh = ChunkMesh::new();
            chunk_mesh.mesh_handle = mesh_handle;
            
            // Optimized: Collect unique block types first, then get materials
            let mut unique_block_types = std::collections::HashSet::new();
            for local_x in 0..crate::chunk::CHUNK_SIZE {
                for local_z in 0..crate::chunk::CHUNK_SIZE {
                    for y in 0..crate::chunk::CHUNK_HEIGHT {
                        if let Some(block_type) = chunk.data.get_block(local_x, y, local_z) {
                            if block_type != BlockType::Air {
                                unique_block_types.insert(block_type);
                            }
                        }
                    }
                }
            }
            
            // Add materials for unique block types only
            for block_type in unique_block_types {
                if let Some(material_handle) = mesh_materials.get_material(block_type) {
                    chunk_mesh.material_handles.insert(block_type, material_handle);
                }
            }
            
            // Add biome-specific materials if available
            if texture_atlas.has_procedural_textures() {
                use std::collections::HashMap;
                let mut biome_material_cache: HashMap<String, Handle<StandardMaterial>> = HashMap::new();
                
                // Track which biomes we've already processed to avoid duplicates
                let mut processed_biomes: HashMap<String, bool> = HashMap::new();
                
                for local_x in 0..crate::chunk::CHUNK_SIZE {
                    for local_z in 0..crate::chunk::CHUNK_SIZE {
                        if let Some(biome_data) = chunk.biome_data.get_biome_data(local_x, local_z) {
                            // Create a unique biome identifier to avoid processing the same biome multiple times
                            // Use biome type only to drastically reduce unique biome variations
                            // This ensures we only generate one set of textures per biome type
                            let biome_identifier = format!("{}", biome_data.biome_type);
                            
                            // Skip if we've already processed this biome in this chunk
                            if processed_biomes.contains_key(&biome_identifier) {
                                continue;
                            }
                            
                            processed_biomes.insert(biome_identifier, true);
                            
                            // Generate biome parameters once per unique biome
                            let representative_height = match biome_data.biome_type.as_str() {
                                "desert" => 15,
                                "forest" => 25,
                                "mountain" | "snowy_mountain" => 45,
                                "hills" => 20,
                                "plains" => 12,
                                "swamp" => 8,
                                "tundra" => 18,
                                "beach" => 5,
                                _ => 15,
                            };
                            
                            let biome_params = crate::biome_textures::BiomeTextureParams::new(
                                biome_data.biome_type.clone(),
                                biome_data.temperature,
                                biome_data.moisture,
                                representative_height as f32,
                                representative_height as f32 / 100.0, // Use fixed max height for now
                            );
                            
                            // Check all block types that should have biome textures
                            let biome_block_types = [BlockType::Grass, BlockType::Dirt, BlockType::Stone, BlockType::Sand];
                            
                            for block_type in biome_block_types {
                                let texture_key = crate::biome_textures::generate_texture_cache_key(&block_type, &biome_params);
                                
                                // Check if we already have this biome material cached for this chunk
                                if biome_material_cache.contains_key(&texture_key) {
                                    continue;
                                }
                                
                                // Try to get biome-specific material
                                if let Some(biome_material) = mesh_materials.get_biome_material(
                                    block_type, 
                                    &biome_params,

                                    &biome_cache,
                                    &mut materials
                                ) {
                                    // Store biome-specific material in chunk cache
                                    biome_material_cache.insert(texture_key, biome_material.clone());
                                    chunk_mesh.material_handles.insert(block_type, biome_material);
                                    // Reduce logging spam
                                    // println!("🎨 Added biome-specific material for {:?} at biome {}", block_type, biome_params.biome_type);
                                }
                            }
                        }
                    }
                }
            }
            
            // Add the chunk mesh component to the chunk entity
            commands.entity(chunk_entity).insert(chunk_mesh);
            
            println!("✓ Generated mesh for chunk ({}, {})", chunk.position.x, chunk.position.z);
        }
    }
}

/// System to render chunk meshes
fn render_chunk_meshes(
    mut commands: Commands,
    chunk_meshes: Query<(Entity, &ChunkMesh, &Chunk)>, 
) {
    for (entity, chunk_mesh, chunk) in &chunk_meshes {
        // Use a default material (grass) if no specific materials are available
        // This ensures all chunks are rendered even if material assignment is incomplete
        let material_handle = chunk_mesh.material_handles.values().next()
            .cloned()
            .unwrap_or_else(|| Handle::default());
            
        commands.entity(entity).insert((Mesh3d(chunk_mesh.mesh_handle.clone()),
                                        MeshMaterial3d(material_handle),
                                        Transform::from_translation(chunk.position.min_block_position().as_vec3())));
    }
}

/// System to spawn the player at a safe position
fn spawn_player_safe(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    blocks: Query<&Block>,
    chunks: Query<&Chunk>,
    chunk_manager: Res<ChunkManager>,
) {
    // Desired spawn position (where we want the player to spawn)
    let desired_spawn_position = Vec3::new(0.0, 3.0, 0.0);
    
    // Find a safe spawn position that doesn't intersect with blocks
    let safe_spawn_position = find_safe_spawn_position(
        &blocks, 
        &chunks, 
        &chunk_manager, 
        desired_spawn_position
    );
    
    println!("🎮 Spawning player at safe position: {:?}", safe_spawn_position);
    
    // Create player mesh
    let player_mesh = meshes.add(Mesh::from(Cuboid { half_size: Vec3::new(0.3, 0.8, 0.3) }));
    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.5, 0.9),
        ..default()
    });

    // Spawn the player with collider
    commands.spawn(player::Player::new(safe_spawn_position))
        .insert((Mesh3d(player_mesh), MeshMaterial3d(player_material)))
        .insert(Collider::player());
}

/// System for dynamic chunk loading based on player position
fn dynamic_chunk_loading_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<player::Player>>,
    mut chunk_manager: ResMut<ChunkManager>,
    chunks: Query<Entity, With<Chunk>>,
    time: Res<Time>,
) {
    // Get player position
    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;
        let player_chunk_pos = ChunkPosition::from_block_position(IVec3::new(
            player_position.x as i32,
            player_position.y as i32,
            player_position.z as i32
        ));
        
        // Only log player position occasionally to reduce spam
        if time.elapsed_secs_f64() % 5.0 < 0.1 {
            println!("🎮 Player is at chunk position ({}, {})", player_chunk_pos.x, player_chunk_pos.z);
        }
        
        // Calculate the loading boundary (render distance)
        let loading_boundary = chunk_manager.render_distance;
        
        // Check if player is near the edge of loaded chunks
        let mut chunks_loaded = 0;
        const MAX_CHUNKS_PER_FRAME: usize = 2; // Limit chunks loaded per frame for performance
        
        // Check all directions within render distance
        for dx in -loading_boundary..=loading_boundary {
            for dz in -loading_boundary..=loading_boundary {
                let chunk_pos = ChunkPosition::new(player_chunk_pos.x + dx, player_chunk_pos.z + dz);
                
                // Check if this chunk should be loaded but isn't
                if chunk_manager.should_load_chunk(chunk_pos, player_chunk_pos) {
                    if !chunk_manager.loaded_chunks.contains_key(&chunk_pos) {
                        println!("🌱 Loading new chunk at ({}, {})", chunk_pos.x, chunk_pos.z);
                        
                        // Spawn the new chunk
                        let chunk_entity = commands.spawn(Chunk::new(chunk_pos)).id();
                        chunk_manager.loaded_chunks.insert(chunk_pos, chunk_entity);
                        chunks_loaded += 1;
                        
                        // Stop if we've loaded enough chunks for this frame
                        if chunks_loaded >= MAX_CHUNKS_PER_FRAME {
                            println!("🔄 Reached chunk loading limit for this frame");
                            return;
                        }
                    }
                }
            }
        }
        
        if chunks_loaded > 0 {
            println!("🔄 Loaded {} new chunks", chunks_loaded);
        }
    }
}
