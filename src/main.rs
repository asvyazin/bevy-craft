use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_compute_noise::prelude::*;

mod block;
use block::{Block, BlockType};

mod chunk;
use chunk::{Chunk, ChunkManager, ChunkPosition, ChunkPriority};

mod chunk_mesh;
use chunk_mesh::{ChunkMesh, ChunkMeshMaterials};

mod texture_atlas;
use texture_atlas::{initialize_texture_atlas, load_procedural_textures_into_atlas, TextureAtlas};

mod texture_gen;
use texture_gen::{initialize_block_textures, BlockTextures, TextureGenSettings};

mod noise;
mod test_sophisticated_algorithms;

mod biome_debug;
mod biome_material;
mod biome_texture_cache;
mod biome_textures;
use biome_debug::{BiomeDebugSettings, BiomeDebugStats};
use biome_material::BiomeMaterial;

mod player;
use player::{FoodConsumedEvent, PlayerDamageEvent, PlayerDeathEvent};
mod world_gen;
use crate::noise::NoiseSettings;
use player::PlayerMovementSettings;
use world_gen::WorldGenSettings;

mod camera;
use camera::{
    camera_mouse_control_system, camera_rotation_system, cursor_control_system, spawn_game_camera,
};

mod block_interaction;
use block_interaction::{
    block_breaking_system, block_placement_system, block_targeting_feedback_system,
    mouse_button_input_system,
};

mod collision;
use collision::{collision_detection_system, find_safe_spawn_position, Collider};

mod sky;
use sky::AtmosphericScatteringParams;

mod weather;
use weather::{
    display_weather_info, initialize_weather_system, spawn_cloud_layers, spawn_weather_particles,
    spawn_weather_particles_dynamic, update_cloud_rendering, update_lightning_effects,
    update_weather_particles, update_weather_system,
};

mod time;
use time::GameTime;

mod inventory;
use inventory::Inventory;

mod hotbar_ui;
use hotbar_ui::{
    initialize_item_texture_atlas, render_hotbar_item_images, spawn_hotbar_ui,
    update_hotbar_item_icons, update_hotbar_ui, ItemTextureAtlas,
};

mod crafting;
use crafting::{CraftItemEvent, CraftingFailEvent, CraftingSuccessEvent, RecipeBook};

fn main() {
    // Create the app first
    let mut app = App::new();

    // Add plugins and initialize resources
    app        .add_plugins(DefaultPlugins)
        .add_event::<player::PlayerDeathEvent>() // Register player death event
        .add_event::<player::PlayerDamageEvent>() // Register player damage event
        .add_event::<player::FoodConsumedEvent>() // Register food consumed event
        .add_event::<CraftItemEvent>() // Register crafting event
        .add_event::<CraftingSuccessEvent>() // Register crafting success event
        .add_event::<CraftingFailEvent>() // Register crafting fail event
        .add_plugins(ComputeNoisePlugin) // Add Perlin noise plugin for world generation
        .add_plugins(bevy::pbr::MaterialPlugin::<sky::AtmosphericScatteringMaterial>::default()) // Add atmospheric scattering material plugin
        .init_resource::<ChunkManager>()
        .init_resource::<WorldGenSettings>() // Initialize world generation settings
        .init_resource::<NoiseSettings>() // Initialize noise settings
        .init_resource::<PlayerMovementSettings>() // Initialize player movement settings
        .init_resource::<ChunkMeshMaterials>() // Initialize chunk mesh materials
        .init_resource::<TextureAtlas>() // Initialize texture atlas
        .init_resource::<TextureGenSettings>() // Initialize texture generation settings
        .init_resource::<BlockTextures>() // Initialize block textures resource
        .init_resource::<crate::biome_texture_cache::SharedBiomeTextureCache>() // Initialize biome texture cache
        .init_resource::<crate::biome_material::SharedBiomeMaterialCache>() // Initialize biome material cache
        .init_resource::<GameTime>() // Initialize game time for day/night cycle
        .init_resource::<AtmosphericScatteringParams>() // Initialize atmospheric scattering parameters
        .init_resource::<BiomeDebugSettings>() // Initialize biome debug settings
        .init_resource::<BiomeDebugStats>() // Initialize biome debug statistics
        .init_resource::<Inventory>() // Initialize inventory system
        .init_resource::<ItemTextureAtlas>() // Initialize item texture atlas
        .init_resource::<block_interaction::BlockBreakingProgress>() // Initialize block breaking progress
        .init_resource::<block_interaction::LeftMouseButtonState>() // Initialize left mouse button state
        .init_resource::<block_interaction::RightMouseButtonState>() // Initialize right mouse button state
        .init_resource::<RecipeBook>() // Initialize recipe book with default recipes
        .add_plugins(bevy::pbr::MaterialPlugin::<weather::CloudMaterial>::default()) // Add cloud material plugin
        .add_plugins(bevy::pbr::MaterialPlugin::<crate::biome_material::BiomeMaterial>::default()) // Add biome material plugin
        ;

    app.add_systems(Startup, setup)
        .add_systems(Startup, spawn_game_camera)
        .add_systems(Startup, initialize_texture_atlas)
        .add_systems(Startup, initialize_block_textures) // Use standard textures
        .add_systems(
            Startup,
            load_procedural_textures_into_atlas.after(initialize_block_textures),
        )
        .add_systems(Startup, spawn_player_safe.after(setup)) // Add safe player spawning system
        .add_systems(Update, player::player_movement_system) // Add player movement system
        .add_systems(
            Update,
            player::player_take_damage_system.after(player::player_movement_system),
        ) // Add player damage system
        .add_systems(Update, player::player_death_system) // Add player death system
        .add_systems(Update, player::handle_damage_events) // Add damage event handling
        .add_systems(Update, player::hunger_thirst_decay_system) // Add hunger/thirst decay system
        .add_systems(Update, player::food_consumption_system) // Add food consumption system
        .add_systems(Update, player::display_hunger_thirst_status) // Add hunger/thirst status display
        .add_systems(
            Update,
            collision_detection_system.after(player::player_movement_system),
        ) // Add collision detection system
        .add_systems(Update, cursor_control_system) // Add cursor control system
        .add_systems(Update, camera_mouse_control_system) // Add mouse camera control system
        .add_systems(Update, camera_rotation_system) // Add camera rotation system
        .add_systems(Update, block_targeting_feedback_system) // Add block targeting feedback
        .add_systems(Update, mouse_button_input_system) // Add mouse button input system
        .add_systems(Update, block_breaking_system) // Add block breaking system
        .add_systems(Update, block_placement_system) // Add block placement system
        .add_systems(Update, crafting::handle_crafting_requests) // Add crafting request handling system
        .add_systems(Update, crafting::handle_crafting_success_events) // Add crafting success event handling system
        .add_systems(Update, crafting::handle_crafting_fail_events) // Add crafting fail event handling system
        .add_systems(Update, crafting::handle_crafting_keyboard_input) // Add crafting keyboard input system
        .run();
}

fn setup(mut commands: Commands, mut chunk_manager: ResMut<ChunkManager>) {
    // Initialize chunk manager
    *chunk_manager = ChunkManager::new(2); // Render distance of 2 chunks

    // Camera is now spawned by the spawn_game_camera system

    // Add light
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

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

            // Register the chunk in the manager using spatial partitioning (with existence check)
            if let Some(entity_commands) = commands.get_entity(chunk_entity) {
                if entity_commands.id() == chunk_entity {
                    chunk_manager.insert_chunk(chunk_pos, chunk_entity);
                    println!(
                        "🌱 Spawned initial chunk at ({}, {}) - waiting for terrain generation",
                        x, z
                    );
                }
            }
        }
    }

    println!(
        "🎮 Initial chunks created, dynamic chunk loading system will handle additional chunks"
    );
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

/// System to initialize biome material cache
fn initialize_biome_material_cache(
    mut biome_material_cache: ResMut<crate::biome_material::SharedBiomeMaterialCache>,
) {
    println!("🎨 Initializing biome material cache...");

    // Configure the biome material cache
    let config = crate::biome_material::BiomeMaterialCacheConfig {
        max_materials: 1024,
        max_memory_mb: 512,
        enable_lru_eviction: true,
        log_operations: false, // Set to true for debugging
    };

    *biome_material_cache = crate::biome_material::SharedBiomeMaterialCache::new(config.clone());

    println!(
        "✓ Biome material cache initialized with {} max materials and {} MB limit",
        config.max_materials, config.max_memory_mb
    );
}

/// System to display biome material cache statistics
fn display_biome_material_stats(
    biome_material_cache: Res<crate::biome_material::SharedBiomeMaterialCache>,
    time: Res<Time>,
) {
    // Only display stats every 5 seconds to avoid spam
    if time.elapsed_secs_f64() % 5.0 > 0.1 {
        return;
    }

    let cache = biome_material_cache.cache.lock().unwrap();
    let stats = cache.get_stats();

    if stats.total_requests > 0 {
        println!("📈 Biome Material Cache Stats:");
        println!(
            "  Requests: {} | Hits: {} ({:.1}%) | Misses: {} ({:.1}%)",
            stats.total_requests,
            stats.cache_hits,
            (stats.cache_hits as f64 / stats.total_requests as f64 * 100.0),
            stats.cache_misses,
            (stats.cache_misses as f64 / stats.total_requests as f64 * 100.0)
        );
        println!(
            "  Materials: {} generated, {} evicted | Memory: {:.1} MB",
            stats.materials_generated,
            stats.materials_evicted,
            stats.memory_used_bytes as f64 / 1024.0 / 1024.0
        );
    }
}

/// System to generate chunk meshes
fn generate_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut biome_materials: ResMut<Assets<BiomeMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mesh_materials: Res<ChunkMeshMaterials>,
    chunks_without_mesh: Query<(Entity, &Chunk), Without<ChunkMesh>>,
    all_chunks: Query<&Chunk>,
    chunk_manager: Res<ChunkManager>,
    texture_atlas: Res<TextureAtlas>,
    biome_cache: Res<crate::biome_texture_cache::SharedBiomeTextureCache>,
    biome_material_cache: Res<crate::biome_material::SharedBiomeMaterialCache>,
) {
    for (chunk_entity, chunk) in chunks_without_mesh.iter().filter(|(_, c)| c.is_generated) {
        println!(
            "🏗️  Generating mesh for chunk ({}, {})",
            chunk.position.x, chunk.position.z
        );

        // Generate the mesh for this chunk using neighbor-aware algorithm
        let mesh = chunk_mesh::generate_chunk_mesh(
            &chunk.data,
            &chunk.position,
            &chunk_manager,
            &all_chunks,
            &texture_atlas,
            &chunk,
            &biome_cache,
            &mut images,
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
                chunk_mesh
                    .material_handles
                    .insert(block_type, material_handle);
            }
        }

        // Add biome-specific materials if available
        if texture_atlas.has_procedural_textures() {
            use std::collections::HashMap;
            let mut biome_material_local_cache: HashMap<String, Handle<StandardMaterial>> =
                HashMap::new();

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
                        let biome_block_types = [
                            BlockType::Grass,
                            BlockType::Dirt,
                            BlockType::Stone,
                            BlockType::Sand,
                        ];

                        for block_type in biome_block_types {
                            let texture_key = crate::biome_textures::generate_texture_cache_key(
                                &block_type,
                                &biome_params,
                            );

                            // Check if we already have this biome material cached for this chunk
                            if biome_material_local_cache.contains_key(&texture_key) {
                                continue;
                            }

                            // Try to get biome-specific material using enhanced biome material system
                            if let Some(biome_material) = mesh_materials.get_biome_material(
                                block_type,
                                &biome_params,
                                &biome_cache,
                                &biome_material_cache,
                                &mut materials,
                                &mut biome_materials,
                                &mut images,
                            ) {
                                // Store biome-specific material in chunk cache
                                biome_material_local_cache
                                    .insert(texture_key, biome_material.clone());
                                chunk_mesh
                                    .material_handles
                                    .insert(block_type, biome_material);
                                // Reduce logging spam
                                // println!("🎨 Added biome-specific material for {:?} at biome {}", block_type, biome_params.biome_type);
                            }
                        }
                    }
                }
            }
        }

        // Add the chunk mesh component to the chunk entity with existence check
        if let Some(mut entity_commands) = commands.get_entity(chunk_entity) {
            // Double-check that the entity still exists before inserting
            if entity_commands.id() == chunk_entity {
                entity_commands.insert(chunk_mesh);
                println!(
                    "✓ Generated mesh for chunk ({}, {})",
                    chunk.position.x, chunk.position.z
                );
            }
        }
    }
}

/// System to update existing chunk meshes when blocks change
fn update_chunk_meshes(
    mut commands: Commands,
    mut chunks_with_mesh: Query<(Entity, &mut Chunk), With<ChunkMesh>>,
) {
    for (chunk_entity, mut chunk) in &mut chunks_with_mesh {
        if chunk.is_generated && chunk.needs_mesh_update {
            println!(
                "🔄 Regenerating mesh for chunk ({}, {})",
                chunk.position.x, chunk.position.z
            );

            // Remove ChunkMesh and Mesh3d components so render_chunk_meshes will recreate them
            commands
                .entity(chunk_entity)
                .remove::<ChunkMesh>()
                .remove::<Mesh3d>()
                .remove::<MeshMaterial3d<StandardMaterial>>();

            // Reset the flag to prevent infinite regeneration
            chunk.needs_mesh_update = false;
        }
    }
}

/// System to render chunk meshes
fn render_chunk_meshes(
    mut commands: Commands,
    chunk_meshes: Query<(Entity, &ChunkMesh, &Chunk)>,
    existing_meshes: Query<Entity, With<Mesh3d>>,
) {
    for (entity, chunk_mesh, chunk) in &chunk_meshes {
        // Check if this entity already has a mesh rendered to avoid duplicate insertion
        if existing_meshes.get(entity).is_ok() {
            continue; // Skip if already rendered
        }

        // Multiple existence checks to prevent race conditions
        // First check: verify the entity exists
        if commands.get_entity(entity).is_some() {
            // Second check: can we get entity commands for it?
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                // Third check: verify the entity hasn't been despawned
                if entity_commands.id() == entity {
                    // Use a default material (grass) if no specific materials are available
                    // This ensures all chunks are rendered even if material assignment is incomplete
                    let material_handle = chunk_mesh
                        .material_handles
                        .values()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| Handle::default());

                    // Only insert components if all checks pass
                    entity_commands.insert((
                        Mesh3d(chunk_mesh.mesh_handle.clone()),
                        MeshMaterial3d(material_handle),
                        Transform::from_translation(chunk.position.min_block_position().as_vec3()),
                    ));
                }
            }
        }
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
    let safe_spawn_position =
        find_safe_spawn_position(&blocks, &chunks, &chunk_manager, desired_spawn_position);

    println!(
        "🎮 Spawning player at safe position: {:?}",
        safe_spawn_position
    );

    // Create player mesh
    let player_mesh = meshes.add(Mesh::from(Cuboid {
        half_size: Vec3::new(0.3, 0.8, 0.3),
    }));
    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.5, 0.9),
        ..default()
    });

    // Spawn the player with collider
    commands
        .spawn(player::Player::new(safe_spawn_position))
        .insert((Mesh3d(player_mesh), MeshMaterial3d(player_material)))
        .insert(Collider::player());
}

/// System for dynamic chunk loading and unloading based on player position
fn dynamic_chunk_loading_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<player::Player>>,
    mut chunk_manager: ResMut<ChunkManager>,
    chunks: Query<&Chunk>,
    time: Res<Time>,
) {
    // Get player position
    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;
        let player_chunk_pos = ChunkPosition::from_block_position(IVec3::new(
            player_position.x as i32,
            player_position.y as i32,
            player_position.z as i32,
        ));

        // Only log player position occasionally to reduce spam
        if time.elapsed_secs_f64() % 5.0 < 0.1 {
            println!(
                "🎮 Player is at chunk position ({}, {})",
                player_chunk_pos.x, player_chunk_pos.z
            );
        }

        // Update chunk priorities based on player position
        chunk_manager.update_chunk_priorities(&player_chunk_pos);

        // Calculate the loading boundary (render distance)
        let loading_boundary = chunk_manager.render_distance;

        // Log performance statistics occasionally
        if time.elapsed_secs_f64() % 10.0 < 0.1 {
            let loaded_chunks = chunk_manager.get_loaded_chunk_count();
            let memory_usage_kb = chunk_manager.estimate_memory_usage() / 1024;
            let cache_stats = chunk_manager.get_cache_stats();

            println!("📊 Chunk System Stats:");
            println!("   Loaded chunks: {}", loaded_chunks);
            println!("   Memory usage: {} KB", memory_usage_kb);
            println!(
                "   Cache: {}/{} chunks",
                cache_stats.cached_chunks, cache_stats.max_cache_size
            );
            println!("   Render distance: {}", loading_boundary);
        }

        // First, unload chunks that are too far from the player using spatial partitioning
        let mut chunks_unloaded = 0;
        const MAX_CHUNKS_UNLOADED_PER_FRAME: usize = 4; // Limit chunks unloaded per frame for performance

        // Use spatial partitioning to get chunks in the player's region and neighboring regions
        let chunks_to_check = chunk_manager
            .get_chunks_within_distance_spatial(&player_chunk_pos, loading_boundary + 1);

        for chunk_pos in chunks_to_check {
            if chunk_manager.should_unload_chunk(chunk_pos, player_chunk_pos) {
                if let Some(chunk_entity) = chunk_manager.remove_chunk(&chunk_pos) {
                    println!("🗑️  Unloading chunk at ({}, {})", chunk_pos.x, chunk_pos.z);

                    // Before despawning, cache the chunk data for potential reuse
                    if let Some(mut entity_commands) = commands.get_entity(chunk_entity) {
                        // Double-check entity exists before caching and despawning
                        if entity_commands.id() == chunk_entity {
                            if let Ok(chunk_component) = chunks.get(chunk_entity) {
                                chunk_manager.cache_chunk(
                                    chunk_pos,
                                    chunk_component.data.clone(),
                                    chunk_component.biome_data.clone(),
                                    chunk_component.is_generated,
                                    time.elapsed_secs_f64(),
                                );
                                println!(
                                    "💾 Cached chunk data for ({}, {})",
                                    chunk_pos.x, chunk_pos.z
                                );
                            }

                            // Despawn the chunk entity
                            entity_commands.despawn();
                        }
                    }

                    chunks_unloaded += 1;

                    // Stop if we've unloaded enough chunks for this frame
                    if chunks_unloaded >= MAX_CHUNKS_UNLOADED_PER_FRAME {
                        println!("🔄 Reached chunk unloading limit for this frame");
                        break;
                    }
                }
            }
        }

        if chunks_unloaded > 0 {
            println!("🔄 Unloaded {} chunks", chunks_unloaded);
        }

        // Then, load new chunks that are within render distance using priority-based spatial partitioning
        let mut chunks_loaded = 0;
        const MAX_CHUNKS_PER_FRAME: usize = 2; // Limit chunks loaded per frame for performance

        // Get chunks sorted by priority to load the most important chunks first
        let chunks_by_priority = chunk_manager.get_chunks_sorted_by_priority(&player_chunk_pos);

        // First, try to load chunks in priority order from existing spatial grid
        for (chunk_pos, priority) in chunks_by_priority {
            // Only consider chunks that should be loaded but aren't
            if chunk_manager.should_load_chunk(chunk_pos, player_chunk_pos)
                && !chunk_manager.loaded_chunks.contains_key(&chunk_pos)
            {
                // Check if we have cached data for this chunk
                if let Some(cached_data) =
                    chunk_manager.get_cached_chunk(&chunk_pos, time.elapsed_secs_f64())
                {
                    println!(
                        "🔄 Restoring cached chunk at ({}, {}) with priority {:?}",
                        chunk_pos.x, chunk_pos.z, priority
                    );

                    // Spawn the new chunk with cached data
                    let mut chunk = Chunk::new(chunk_pos);
                    chunk.data = cached_data.data;
                    chunk.biome_data = cached_data.biome_data;
                    chunk.is_generated = cached_data.is_generated;
                    chunk.needs_mesh_update = true; // Ensure mesh is regenerated

                    let chunk_entity = commands.spawn(chunk).id();
                    chunk_manager.insert_chunk(chunk_pos, chunk_entity);
                } else {
                    println!(
                        "🌱 Loading new chunk at ({}, {}) with priority {:?}",
                        chunk_pos.x, chunk_pos.z, priority
                    );

                    // Spawn the new chunk
                    let chunk_entity = commands.spawn(Chunk::new(chunk_pos)).id();
                    // Register the chunk in the manager with existence check
                    if let Some(entity_commands) = commands.get_entity(chunk_entity) {
                        if entity_commands.id() == chunk_entity {
                            chunk_manager.insert_chunk(chunk_pos, chunk_entity);
                        }
                    }
                }

                chunks_loaded += 1;

                // Stop if we've loaded enough chunks for this frame
                if chunks_loaded >= MAX_CHUNKS_PER_FRAME {
                    println!("🔄 Reached chunk loading limit for this frame");
                    break;
                }
            }
        }

        // If we still haven't loaded enough chunks, fall back to spatial partitioning
        if chunks_loaded < MAX_CHUNKS_PER_FRAME {
            let center_region = chunk_manager.chunk_pos_to_grid_region(&player_chunk_pos);
            let mut regions_checked = 0;
            let max_regions_to_check = 4; // Limit regions checked per frame for performance

            // Spiral outward from the center region
            let mut region_radius = 0;
            while chunks_loaded < MAX_CHUNKS_PER_FRAME && regions_checked < max_regions_to_check {
                for dx in -region_radius..=region_radius {
                    for dz in -region_radius..=region_radius {
                        // Skip if this is not on the perimeter of the current radius
                        if (dx as i32).abs() != region_radius && (dz as i32).abs() != region_radius
                        {
                            continue;
                        }

                        let region_x = center_region.0 + dx;
                        let region_z = center_region.1 + dz;
                        let _region_coords = (region_x, region_z);
                        regions_checked += 1;

                        // Check if we need to load chunks in this region
                        let min_chunk_x = region_x * chunk_manager.grid_region_size;
                        let min_chunk_z = region_z * chunk_manager.grid_region_size;

                        for local_dx in 0..chunk_manager.grid_region_size {
                            for local_dz in 0..chunk_manager.grid_region_size {
                                let chunk_pos = ChunkPosition::new(
                                    min_chunk_x + local_dx,
                                    min_chunk_z + local_dz,
                                );

                                // Check if this chunk should be loaded
                                if chunk_manager.should_load_chunk(chunk_pos, player_chunk_pos) {
                                    if !chunk_manager.loaded_chunks.contains_key(&chunk_pos) {
                                        // Calculate priority for this chunk
                                        let dx = (chunk_pos.x - player_chunk_pos.x).abs();
                                        let dz = (chunk_pos.z - player_chunk_pos.z).abs();
                                        let distance = (dx.max(dz)) as i32;
                                        let priority =
                                            if distance <= chunk_manager.render_distance / 2 {
                                                ChunkPriority::Near
                                            } else {
                                                ChunkPriority::Far
                                            };

                                        println!(
                                            "🌱 Loading new chunk at ({}, {}) with priority {:?}",
                                            chunk_pos.x, chunk_pos.z, priority
                                        );

                                        // Spawn the new chunk
                                        let chunk_entity =
                                            commands.spawn(Chunk::new(chunk_pos)).id();
                                        // Register the chunk in the manager with existence check
                                        if let Some(entity_commands) =
                                            commands.get_entity(chunk_entity)
                                        {
                                            if entity_commands.id() == chunk_entity {
                                                chunk_manager.insert_chunk(chunk_pos, chunk_entity);
                                            }
                                        }
                                        chunks_loaded += 1;

                                        // Stop if we've loaded enough chunks for this frame
                                        if chunks_loaded >= MAX_CHUNKS_PER_FRAME {
                                            println!(
                                                "🔄 Reached chunk loading limit for this frame"
                                            );
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                region_radius += 1;
            }
        }

        // If we still haven't loaded enough chunks, fall back to the original method
        if chunks_loaded < MAX_CHUNKS_PER_FRAME {
            // Check chunks in a spiral pattern around the player for better cache locality
            let mut distance = 1;
            while chunks_loaded < MAX_CHUNKS_PER_FRAME {
                for dx in -distance..=distance {
                    for dz in -distance..=distance {
                        // Only check the perimeter of the current distance
                        if (dx as i32).abs() == distance || (dz as i32).abs() == distance {
                            let chunk_pos = ChunkPosition::new(
                                player_chunk_pos.x + dx,
                                player_chunk_pos.z + dz,
                            );

                            // Check if this chunk should be loaded but isn't
                            if chunk_manager.should_load_chunk(chunk_pos, player_chunk_pos) {
                                if !chunk_manager.loaded_chunks.contains_key(&chunk_pos) {
                                    println!(
                                        "🌱 Loading new chunk at ({}, {})",
                                        chunk_pos.x, chunk_pos.z
                                    );

                                    // Spawn the new chunk
                                    let chunk_entity = commands.spawn(Chunk::new(chunk_pos)).id();
                                    // Register the chunk in the manager with existence check
                                    if let Some(entity_commands) = commands.get_entity(chunk_entity)
                                    {
                                        if entity_commands.id() == chunk_entity {
                                            chunk_manager.insert_chunk(chunk_pos, chunk_entity);
                                        }
                                    }
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
                }
                distance += 1;

                // Don't go beyond render distance
                if distance > loading_boundary {
                    break;
                }
            }
        }

        if chunks_loaded > 0 {
            println!("🔄 Loaded {} new chunks", chunks_loaded);
        }
    }
}
