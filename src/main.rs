use bevy::prelude::*;
use bevy::math::primitives::Cuboid;
use bevy_compute_noise::prelude::*;

mod block;
use block::{Block, BlockType};

mod chunk;
use chunk::{Chunk, ChunkManager, ChunkPosition};

mod noise_demo;
mod world_gen;
use world_gen::{WorldGenSettings, generate_chunks_system};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ComputeNoisePlugin::<Perlin2d>::default()) // Add Perlin noise plugin for world generation
        .init_resource::<ChunkManager>()
        .init_resource::<WorldGenSettings>() // Initialize world generation settings
        .add_systems(Startup, setup)
        .add_systems(Startup, noise_demo::demo_noise_generation)
        .add_systems(Update, update_block_rendering)
        .add_systems(Update, generate_chunks_system) // Add world generation system
        .add_systems(Update, spawn_blocks_from_chunks) // Add chunk rendering system
        .add_systems(Update, update_chunk_mesh_status) // Add mesh status update system
        .run();
}

fn setup(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    // Initialize chunk manager
    *chunk_manager = ChunkManager::new(2); // Render distance of 2 chunks

    // Add a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Add light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Create a simple world with different block types
    generate_simple_world(&mut commands);

    // Generate some chunks for demonstration
    generate_demo_chunks(&mut commands, &mut chunk_manager);
}

/// Generate a simple world with different block types for demonstration
fn generate_simple_world(commands: &mut Commands) {
    // Create a small platform of different blocks
    for x in -2..=2 {
        for z in -2..=2 {
            // Bedrock layer
            commands.spawn(Block::new(BlockType::Bedrock, IVec3::new(x, 0, z)));
            
            // Dirt layer
            commands.spawn(Block::new(BlockType::Dirt, IVec3::new(x, 1, z)));
            
            // Grass layer (top)
            commands.spawn(Block::new(BlockType::Grass, IVec3::new(x, 2, z)));
        }
    }
    
    // Add some stone blocks
    commands.spawn(Block::new(BlockType::Stone, IVec3::new(0, 3, 0)));
    commands.spawn(Block::new(BlockType::Stone, IVec3::new(1, 3, 0)));
    
    // Add some wood blocks
    commands.spawn(Block::new(BlockType::Wood, IVec3::new(-1, 3, 0)));
    
    // Add some leaves
    commands.spawn(Block::new(BlockType::Leaves, IVec3::new(0, 4, 0)));
    
    // Add some sand
    commands.spawn(Block::new(BlockType::Sand, IVec3::new(2, 2, 2)));
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

/// System to render blocks based on their type
fn update_block_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    blocks: Query<(Entity, &Block), Without<Handle<Mesh>>>, // Only blocks without mesh handles
) {
    // Spawn visual representations for all blocks
    for (entity, block) in &blocks {
        let mesh = meshes.add(Mesh::from(Cuboid { half_size: Vec3::ONE * 0.5 }));
        let material = materials.add(StandardMaterial {
            base_color: block.block_type.color(),
            ..default()
        });
        
        // Spawn the visual representation
        commands.entity(entity).insert(PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(block.position.as_vec3()),
            ..default()
        });
    }
}

/// System to spawn block entities from generated chunks
fn spawn_blocks_from_chunks(
    mut commands: Commands,
    chunks: Query<&Chunk>,
) {
    for chunk in &chunks {
        if chunk.is_generated && chunk.needs_mesh_update {
            println!("🏗️  Spawning blocks for chunk ({}, {})", chunk.position.x, chunk.position.z);
            
            // Spawn all non-air blocks in the chunk
            for local_x in 0..crate::chunk::CHUNK_SIZE {
                for local_z in 0..crate::chunk::CHUNK_SIZE {
                    for y in 0..crate::chunk::CHUNK_HEIGHT {
                        if let Some(block_type) = chunk.data.get_block(local_x, y, local_z) {
                            if block_type != BlockType::Air {
                                let world_pos = chunk.position.min_block_position() + IVec3::new(local_x as i32, y as i32, local_z as i32);
                                commands.spawn(Block::new(block_type, world_pos));
                            }
                        }
                    }
                }
            }
            
            // Mark chunk as no longer needing mesh update
            // Note: We can't modify the chunk directly here, so we'll handle this in another system
        }
    }
}

/// System to mark chunks as updated after block spawning
fn update_chunk_mesh_status(
    mut chunks: Query<&mut Chunk>,
) {
    for mut chunk in &mut chunks {
        if chunk.needs_mesh_update {
            // Check if we should mark it as updated
            // For now, we'll just mark it after a delay to allow block spawning
            chunk.needs_mesh_update = false;
        }
    }
}
