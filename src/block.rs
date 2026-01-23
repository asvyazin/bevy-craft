use bevy::prelude::*;

/// Enum representing different types of blocks in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BlockType {
    Air,
    Dirt,
    Stone,
    Grass,
    Wood,
    Leaves,
    Sand,
    Water,
    Bedrock,
    Lava,
}

impl BlockType {
    /// Get the display name of the block type
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            BlockType::Air => "Air",
            BlockType::Dirt => "Dirt",
            BlockType::Stone => "Stone",
            BlockType::Grass => "Grass",
            BlockType::Wood => "Wood",
            BlockType::Leaves => "Leaves",
            BlockType::Sand => "Sand",
            BlockType::Water => "Water",
            BlockType::Bedrock => "Bedrock",
            BlockType::Lava => "Lava",
        }
    }

    /// Get the color associated with this block type
    pub fn color(&self) -> Color {
        match self {
            BlockType::Air => Color::NONE,
            BlockType::Dirt => Color::srgb(0.5, 0.3, 0.2),
            BlockType::Stone => Color::srgb(0.7, 0.7, 0.7),
            BlockType::Grass => Color::srgb(0.2, 0.8, 0.2),
            BlockType::Wood => Color::srgb(0.6, 0.4, 0.2),
            BlockType::Leaves => Color::srgb(0.1, 0.7, 0.1),
            BlockType::Sand => Color::srgb(0.9, 0.8, 0.5),
            BlockType::Water => Color::srgb(0.1, 0.1, 0.9),
            BlockType::Bedrock => Color::srgb(0.3, 0.3, 0.3),
            BlockType::Lava => Color::srgb(1.0, 0.5, 0.0),
        }
    }

    /// Check if the block is solid (not air or water)
    pub fn is_solid(&self) -> bool {
        match self {
            BlockType::Air | BlockType::Water => false,
            _ => true,
        }
    }

    /// Check if the block is transparent
    pub fn is_transparent(&self) -> bool {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Leaves => true,
            _ => false,
        }
    }

    /// Get the hardness of this block type (higher = harder to break)
    /// Returns None for unbreakable blocks
    pub fn hardness(&self) -> Option<f32> {
        match self {
            BlockType::Air => None,
            BlockType::Dirt => Some(1.5),
            BlockType::Stone => Some(3.0),
            BlockType::Grass => Some(1.2),
            BlockType::Wood => Some(2.0),
            BlockType::Leaves => Some(0.5),
            BlockType::Sand => Some(1.0),
            BlockType::Water => None,
            BlockType::Bedrock => None,
            BlockType::Lava => Some(100.0),
        }
    }
}

/// Component representing a block in the game world
#[derive(Component, Debug)]
pub struct Block {
    pub block_type: BlockType,
    pub position: IVec3, // Integer position in the world grid
}

impl Block {
    #[allow(dead_code)]
    pub fn new(block_type: BlockType, position: IVec3) -> Self {
        Self {
            block_type,
            position,
        }
    }
}
