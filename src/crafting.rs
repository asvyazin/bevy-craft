use crate::inventory::{ItemType, ToolType};
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Unique identifier for a crafting recipe
pub type RecipeId = String;

/// Represents a single item requirement in a recipe
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub item_type: ItemType,
    pub quantity: u32,
}

impl RecipeIngredient {
    pub fn new(item_type: ItemType, quantity: u32) -> Self {
        Self {
            item_type,
            quantity,
        }
    }
}

/// Represents the result of crafting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeOutput {
    pub item_type: ItemType,
    pub quantity: u32,
}

impl RecipeOutput {
    pub fn new(item_type: ItemType, quantity: u32) -> Self {
        Self {
            item_type,
            quantity,
        }
    }
}

/// Size of the crafting grid required for this recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftingGridSize {
    Size1x1,
    Size2x2,
    Size3x3,
}

impl CraftingGridSize {
    pub fn size(&self) -> usize {
        match self {
            CraftingGridSize::Size1x1 => 1,
            CraftingGridSize::Size2x2 => 4,
            CraftingGridSize::Size3x3 => 9,
        }
    }
}

/// A complete crafting recipe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingRecipe {
    pub id: RecipeId,
    pub name: String,
    pub grid_size: CraftingGridSize,
    pub ingredients: Vec<RecipeIngredient>,
    pub output: RecipeOutput,
}

impl CraftingRecipe {
    pub fn new(
        id: RecipeId,
        name: String,
        grid_size: CraftingGridSize,
        ingredients: Vec<RecipeIngredient>,
        output: RecipeOutput,
    ) -> Self {
        Self {
            id,
            name,
            grid_size,
            ingredients,
            output,
        }
    }

    /// Check if this recipe can be crafted with the given ingredients
    /// Returns true if all required ingredients are present in sufficient quantity
    pub fn can_craft(&self, available_items: &[(ItemType, u32)]) -> bool {
        for required in &self.ingredients {
            let mut found = false;
            for (available_type, available_quantity) in available_items {
                if *available_type == required.item_type {
                    if *available_quantity >= required.quantity {
                        found = true;
                    }
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

/// Collection of all available crafting recipes
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct RecipeBook {
    pub recipes: Vec<CraftingRecipe>,
}

impl RecipeBook {
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    pub fn add_recipe(&mut self, recipe: CraftingRecipe) {
        self.recipes.push(recipe);
    }

    pub fn get_recipe(&self, id: &RecipeId) -> Option<&CraftingRecipe> {
        self.recipes.iter().find(|r| &r.id == id)
    }

    pub fn get_all_recipes(&self) -> &[CraftingRecipe] {
        &self.recipes
    }

    /// Find all recipes that can be crafted with the given items
    pub fn find_craftable_recipes(
        &self,
        available_items: &[(ItemType, u32)],
    ) -> Vec<&CraftingRecipe> {
        self.recipes
            .iter()
            .filter(|recipe| recipe.can_craft(available_items))
            .collect()
    }
}

/// Initialize the recipe book with default recipes
pub fn initialize_recipe_book() -> RecipeBook {
    let mut recipe_book = RecipeBook::new();

    use crate::block::BlockType;
    use crate::inventory::ResourceType;

    recipe_book.add_recipe(CraftingRecipe::new(
        "wooden_pickaxe".to_string(),
        "Wooden Pickaxe".to_string(),
        CraftingGridSize::Size3x3,
        vec![
            RecipeIngredient::new(ItemType::Block(BlockType::Wood), 3),
            RecipeIngredient::new(ItemType::Resource(ResourceType::Stick), 2),
        ],
        RecipeOutput::new(ItemType::Tool(ToolType::Pickaxe), 1),
    ));

    recipe_book.add_recipe(CraftingRecipe::new(
        "stone_pickaxe".to_string(),
        "Stone Pickaxe".to_string(),
        CraftingGridSize::Size3x3,
        vec![
            RecipeIngredient::new(ItemType::Block(BlockType::Stone), 3),
            RecipeIngredient::new(ItemType::Resource(ResourceType::Stick), 2),
        ],
        RecipeOutput::new(ItemType::Tool(ToolType::Pickaxe), 1),
    ));

    recipe_book.add_recipe(CraftingRecipe::new(
        "wooden_axe".to_string(),
        "Wooden Axe".to_string(),
        CraftingGridSize::Size3x3,
        vec![
            RecipeIngredient::new(ItemType::Block(BlockType::Wood), 3),
            RecipeIngredient::new(ItemType::Resource(ResourceType::Stick), 2),
        ],
        RecipeOutput::new(ItemType::Tool(ToolType::Axe), 1),
    ));

    recipe_book.add_recipe(CraftingRecipe::new(
        "wooden_shovel".to_string(),
        "Wooden Shovel".to_string(),
        CraftingGridSize::Size3x3,
        vec![
            RecipeIngredient::new(ItemType::Block(BlockType::Wood), 1),
            RecipeIngredient::new(ItemType::Resource(ResourceType::Stick), 2),
        ],
        RecipeOutput::new(ItemType::Tool(ToolType::Shovel), 1),
    ));

    recipe_book.add_recipe(CraftingRecipe::new(
        "stick".to_string(),
        "Stick".to_string(),
        CraftingGridSize::Size2x2,
        vec![RecipeIngredient::new(ItemType::Block(BlockType::Wood), 2)],
        RecipeOutput::new(ItemType::Resource(ResourceType::Stick), 4),
    ));

    recipe_book
}

impl Default for RecipeBook {
    fn default() -> Self {
        initialize_recipe_book()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    #[test]
    fn test_ingredient_creation() {
        let ingredient = RecipeIngredient::new(ItemType::Block(BlockType::Stone), 2);
        assert_eq!(ingredient.quantity, 2);
    }

    #[test]
    fn test_crafting_grid_size() {
        assert_eq!(CraftingGridSize::Size1x1.size(), 1);
        assert_eq!(CraftingGridSize::Size2x2.size(), 4);
        assert_eq!(CraftingGridSize::Size3x3.size(), 9);
    }

    #[test]
    fn test_recipe_can_craft() {
        let recipe = CraftingRecipe::new(
            "stone_pickaxe".to_string(),
            "Stone Pickaxe".to_string(),
            CraftingGridSize::Size3x3,
            vec![
                RecipeIngredient::new(ItemType::Block(BlockType::Stone), 3),
                RecipeIngredient::new(ItemType::Tool(ToolType::Pickaxe), 1),
            ],
            RecipeOutput::new(ItemType::Tool(ToolType::Pickaxe), 1),
        );

        let available_items = vec![
            (ItemType::Block(BlockType::Stone), 3),
            (ItemType::Tool(ToolType::Pickaxe), 1),
        ];

        assert!(recipe.can_craft(&available_items));
    }

    #[test]
    fn test_recipe_cannot_craft_insufficient() {
        let recipe = CraftingRecipe::new(
            "stone_pickaxe".to_string(),
            "Stone Pickaxe".to_string(),
            CraftingGridSize::Size3x3,
            vec![
                RecipeIngredient::new(ItemType::Block(BlockType::Stone), 3),
                RecipeIngredient::new(ItemType::Tool(ToolType::Pickaxe), 1),
            ],
            RecipeOutput::new(ItemType::Tool(ToolType::Pickaxe), 1),
        );

        let available_items = vec![(ItemType::Block(BlockType::Stone), 2)];

        assert!(!recipe.can_craft(&available_items));
    }
}
