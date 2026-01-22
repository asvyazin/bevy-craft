use crate::inventory::{Inventory, ItemStack, ItemType, ToolType};
use bevy::prelude::{
    info, warn, ButtonInput, Event, EventReader, EventWriter, KeyCode, Res, ResMut, Resource,
};
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

/// Event sent when a player attempts to craft an item
#[derive(Event, Debug)]
pub struct CraftItemEvent {
    pub recipe_id: RecipeId,
    pub quantity: u32,
}

impl CraftItemEvent {
    pub fn new(recipe_id: RecipeId, quantity: u32) -> Self {
        Self {
            recipe_id,
            quantity,
        }
    }
}

/// Event sent when crafting succeeds
#[derive(Event, Debug)]
pub struct CraftingSuccessEvent {
    pub recipe_id: RecipeId,
    pub recipe_name: String,
    pub output_item: ItemType,
    pub output_quantity: u32,
}

/// Event sent when crafting fails
#[derive(Event, Debug)]
pub struct CraftingFailEvent {
    pub recipe_id: RecipeId,
    pub reason: String,
}

/// System to handle crafting requests
pub fn handle_crafting_requests(
    mut craft_events: EventReader<CraftItemEvent>,
    mut success_events: EventWriter<CraftingSuccessEvent>,
    mut fail_events: EventWriter<CraftingFailEvent>,
    recipe_book: Res<RecipeBook>,
    mut inventory: ResMut<Inventory>,
) {
    for craft_event in craft_events.read() {
        let recipe_id = &craft_event.recipe_id;
        let quantity = craft_event.quantity;

        if let Some(recipe) = recipe_book.get_recipe(recipe_id) {
            let mut all_ingredients_available = true;
            let mut missing_ingredients = Vec::new();

            for ingredient in &recipe.ingredients {
                let available_count = inventory.get_item_count(ingredient.item_type);
                let required_count = ingredient.quantity * quantity;

                if available_count < required_count {
                    all_ingredients_available = false;
                    missing_ingredients.push(format!(
                        "{} (need {}, have {})",
                        ingredient.item_type.name(),
                        required_count,
                        available_count
                    ));
                }
            }

            if all_ingredients_available {
                for ingredient in &recipe.ingredients {
                    inventory.remove_item(ingredient.item_type, ingredient.quantity * quantity);
                }

                let output_item = recipe.output.item_type;
                let output_quantity = recipe.output.quantity * quantity;

                if inventory.add_item(output_item, output_quantity) {
                    success_events.send(CraftingSuccessEvent {
                        recipe_id: recipe.id.clone(),
                        recipe_name: recipe.name.clone(),
                        output_item,
                        output_quantity,
                    });

                    info!(
                        "✅ Crafted: {} x{} ({} x{})",
                        recipe.name,
                        output_quantity,
                        recipe.output.item_type.name(),
                        output_quantity
                    );
                } else {
                    fail_events.send(CraftingFailEvent {
                        recipe_id: recipe.id.clone(),
                        reason: "Inventory full".to_string(),
                    });

                    for ingredient in &recipe.ingredients {
                        inventory.add_item(ingredient.item_type, ingredient.quantity * quantity);
                    }

                    warn!("❌ Crafting failed for {}: Inventory full", recipe.name);
                }
            } else {
                let missing_str = missing_ingredients.join(", ");
                fail_events.send(CraftingFailEvent {
                    recipe_id: recipe.id.clone(),
                    reason: format!("Missing ingredients: {}", missing_str),
                });

                warn!(
                    "❌ Crafting failed for {}: Missing ingredients - {}",
                    recipe.name, missing_str
                );
            }
        } else {
            fail_events.send(CraftingFailEvent {
                recipe_id: recipe_id.clone(),
                reason: format!("Recipe not found: {}", recipe_id),
            });

            warn!("❌ Crafting failed: Recipe not found - {}", recipe_id);
        }
    }
}

/// System to handle crafting success events
pub fn handle_crafting_success_events(mut success_events: EventReader<CraftingSuccessEvent>) {
    for event in success_events.read() {
        info!(
            "🎉 Crafting success: {} x{} ({})",
            event.recipe_name,
            event.output_quantity,
            event.output_item.name()
        );
    }
}

/// System to handle crafting fail events
pub fn handle_crafting_fail_events(mut fail_events: EventReader<CraftingFailEvent>) {
    for event in fail_events.read() {
        warn!("❌ Crafting failed: {}", event.reason);
    }
}

/// System to handle keyboard input for testing crafting
pub fn handle_crafting_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    recipe_book: Res<RecipeBook>,
    inventory: Res<Inventory>,
    mut craft_events: EventWriter<CraftItemEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        info!("📋 Available recipes (can craft):");
        let mut found_any = false;
        for recipe in recipe_book.get_all_recipes() {
            let can_craft = recipe.can_craft_from_inventory(&inventory);
            if can_craft {
                found_any = true;
                info!("📝 Recipe: {} (ID: {})", recipe.name, recipe.id);
                info!("   Ingredients:");
                for ingredient in &recipe.ingredients {
                    info!(
                        "     - {} x{}",
                        ingredient.item_type.name(),
                        ingredient.quantity
                    );
                }
                info!(
                    "   Output: {} x{}",
                    recipe.output.item_type.name(),
                    recipe.output.quantity
                );
            }
        }
        if !found_any {
            info!("❌ No recipes can be crafted with current inventory");
        }
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        craft_events.send(CraftItemEvent::new("stick".to_string(), 1));
    }

    if keyboard.just_pressed(KeyCode::KeyQ) {
        craft_events.send(CraftItemEvent::new("wooden_pickaxe".to_string(), 1));
    }

    if keyboard.just_pressed(KeyCode::KeyZ) {
        craft_events.send(CraftItemEvent::new("stone_pickaxe".to_string(), 1));
    }

    if keyboard.just_pressed(KeyCode::KeyX) {
        craft_events.send(CraftItemEvent::new("wooden_axe".to_string(), 1));
    }
}

impl CraftingRecipe {
    pub fn can_craft_from_inventory(&self, inventory: &Inventory) -> bool {
        for ingredient in &self.ingredients {
            if !inventory.has_item(ingredient.item_type, ingredient.quantity) {
                return false;
            }
        }
        true
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
