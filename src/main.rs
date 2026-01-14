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

mod alkyd_integration;



mod world_gen;
mod player;
use world_gen::{WorldGenSettings, generate_chunks_system};
use player::PlayerMovementSettings;

mod camera;
use camera::{camera_mouse_control_system, camera_rotation_system, cursor_control_system, spawn_game_camera};

mod block_interaction;
use block_interaction::block_targeting_feedback_system;

mod collision;
use collision::{Collider, collision_detection_system, find_safe_spawn_position};

mod sky;
use sky::spawn_skybox;

fn main() {
    // Create the app first
    let mut app = App::new();
    
    // Add plugins and initialize resources
    app.add_plugins(DefaultPlugins)
        .add_plugins(ComputeNoisePlugin) // Add Perlin noise plugin for world generation
        .init_resource::<ChunkManager>()
        .init_resource::<WorldGenSettings>() // Initialize world generation settings
        .init_resource::<PlayerMovementSettings>() // Initialize player movement settings
        .init_resource::<ChunkMeshMaterials>() // Initialize chunk mesh materials
        .init_resource::<TextureAtlas>() // Initialize texture atlas
        .init_resource::<TextureGenSettings>() // Initialize texture generation settings
        .init_resource::<BlockTextures>() // Initialize block textures resource
        ;
    
    // Setup Alkyd integration before adding systems
    alkyd_integration::setup_alkyd_integration(&mut app);
    
    app
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_game_camera)
        .add_systems(Startup, initialize_texture_atlas)
        .add_systems(Startup, initialize_block_textures.after(alkyd_integration::generate_all_block_textures)) // Use alkyd textures
        .add_systems(Startup, load_procedural_textures_into_atlas.after(initialize_block_textures))
        .add_systems(Startup, initialize_chunk_mesh_materials.after(load_procedural_textures_into_atlas))
        .add_systems(Startup, spawn_skybox) // Add skybox spawning after materials are ready

        .add_systems(Update, generate_procedural_textures) // Add procedural texture generation
        .add_systems(Update, regenerate_dynamic_textures) // Add dynamic texture regeneration
        .add_systems(Update, alkyd_integration::generate_alkyd_textures) // Add alkyd texture generation
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

    // Generate some chunks for demonstration
    generate_demo_chunks(&mut commands, &mut chunk_manager);
}

/// Generate demo chunks for testing the chunk system
fn generate_demo_chunks(commands: &mut Commands, chunk_manager: &mut ChunkManager) {
    println!("🌍 Generating demo chunks...");
    
    // Generate a few chunks around the origin
    for x in -1..=1 {
        for z in -1..=1 {
            let chunk_pos = ChunkPosition::new(x, z);
            let chunk_entity = commands.spawn(Chunk::new(chunk_pos)).id();
            
            // Register the chunk in the manager
            chunk_manager.loaded_chunks.insert(chunk_pos, chunk_entity);
            
            println!("🌱 Spawned chunk at ({}, {}) - waiting for terrain generation", x, z);
        }
    }
    
    println!("🎮 Chunks created, terrain generation will be handled by the world generation system");
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
    mesh_materials: Res<ChunkMeshMaterials>,
    chunks: Query<(Entity, &Chunk), Without<ChunkMesh>>,
    all_chunks: Query<&Chunk>,
    chunk_manager: Res<ChunkManager>,
    texture_atlas: Res<TextureAtlas>,
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
