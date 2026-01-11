use bevy::prelude::*;
use bevy::math::primitives::Cuboid;

mod block;
use block::{Block, BlockType};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_block_rendering)
        .run();
}

fn setup(
    mut commands: Commands,
) {
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
