// Inventory system for Bevy Craft
// This module handles player inventory management, item storage, and hotbar functionality

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::block::BlockType;

/// Enum representing different types of items in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Block(BlockType),
    Tool(ToolType),
    Resource(ResourceType),
    Food(FoodType),
    // Add more item types as needed
}

impl ItemType {
    pub fn name(&self) -> &str {
        match self {
            ItemType::Block(block_type) => block_type.name(),
            ItemType::Tool(tool_type) => tool_type.name(),
            ItemType::Resource(resource_type) => resource_type.name(),
            ItemType::Food(food_type) => food_type.name(),
        }
    }
}

/// Enum representing different types of tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Sword,
}

impl ToolType {
    pub fn name(&self) -> &str {
        match self {
            ToolType::Pickaxe => "Pickaxe",
            ToolType::Axe => "Axe",
            ToolType::Shovel => "Shovel",
            ToolType::Sword => "Sword",
        }
    }
}

/// Enum representing different types of resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Stick,
    String,
    Coal,
    IronIngot,
    GoldIngot,
}

impl ResourceType {
    pub fn name(&self) -> &str {
        match self {
            ResourceType::Stick => "Stick",
            ResourceType::String => "String",
            ResourceType::Coal => "Coal",
            ResourceType::IronIngot => "Iron Ingot",
            ResourceType::GoldIngot => "Gold Ingot",
        }
    }
}

/// Enum representing different types of food
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodType {
    Apple,
    Bread,
    MeatCooked,
    MeatRaw,
    Carrot,
    Potato,
    Mushroom,
}

impl FoodType {
    pub fn name(&self) -> &str {
        match self {
            FoodType::Apple => "Apple",
            FoodType::Bread => "Bread",
            FoodType::MeatCooked => "Cooked Meat",
            FoodType::MeatRaw => "Raw Meat",
            FoodType::Carrot => "Carrot",
            FoodType::Potato => "Potato",
            FoodType::Mushroom => "Mushroom",
        }
    }

    pub fn hunger_restore(&self) -> f32 {
        match self {
            FoodType::Apple => 10.0,
            FoodType::Bread => 15.0,
            FoodType::MeatCooked => 25.0,
            FoodType::MeatRaw => 10.0,
            FoodType::Carrot => 8.0,
            FoodType::Potato => 10.0,
            FoodType::Mushroom => 12.0,
        }
    }

    pub fn thirst_restore(&self) -> f32 {
        match self {
            FoodType::Apple => 5.0,
            FoodType::Bread => 0.0,
            FoodType::MeatCooked => 5.0,
            FoodType::MeatRaw => 3.0,
            FoodType::Carrot => 4.0,
            FoodType::Potato => 0.0,
            FoodType::Mushroom => 6.0,
        }
    }
}

/// Struct representing an item stack in the inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub quantity: u32,
}

impl ItemStack {
    pub fn new(item_type: ItemType, quantity: u32) -> Self {
        Self {
            item_type,
            quantity,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.quantity == 0
    }

    pub fn can_add(&self, amount: u32, max_stack_size: u32) -> bool {
        self.quantity + amount <= max_stack_size
    }

    pub fn add(&mut self, amount: u32, max_stack_size: u32) -> u32 {
        let remaining_space = max_stack_size.saturating_sub(self.quantity);
        let amount_to_add = amount.min(remaining_space);
        self.quantity += amount_to_add;
        amount - amount_to_add
    }

    pub fn remove(&mut self, amount: u32) -> u32 {
        let amount_to_remove = amount.min(self.quantity);
        self.quantity -= amount_to_remove;
        amount_to_remove
    }
}

/// Struct representing the player's inventory
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
    pub hotbar_slots: Vec<ItemStack>,
    pub selected_hotbar_slot: usize,
    pub max_stack_size: u32,
    pub inventory_size: usize,
    pub hotbar_size: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        let inventory_size = 27; // 3 rows of 9 slots
        let hotbar_size = 9;
        let max_stack_size = 64;

        Self {
            slots: vec![ItemStack::new(ItemType::Block(BlockType::Air), 0); inventory_size],
            hotbar_slots: vec![ItemStack::new(ItemType::Block(BlockType::Air), 0); hotbar_size],
            selected_hotbar_slot: 0,
            max_stack_size,
            inventory_size,
            hotbar_size,
        }
    }
}

impl Inventory {
    #[allow(dead_code)]
    pub fn new(inventory_size: usize, hotbar_size: usize, max_stack_size: u32) -> Self {
        Self {
            slots: vec![ItemStack::new(ItemType::Block(BlockType::Air), 0); inventory_size],
            hotbar_slots: vec![ItemStack::new(ItemType::Block(BlockType::Air), 0); hotbar_size],
            selected_hotbar_slot: 0,
            max_stack_size,
            inventory_size,
            hotbar_size,
        }
    }

    /// Get the currently selected item from the hotbar
    pub fn get_selected_item(&self) -> Option<&ItemStack> {
        self.hotbar_slots.get(self.selected_hotbar_slot)
    }

    /// Get the currently selected item from the hotbar (mutable)
    #[allow(dead_code)]
    pub fn get_selected_item_mut(&mut self) -> Option<&mut ItemStack> {
        self.hotbar_slots.get_mut(self.selected_hotbar_slot)
    }

    /// Select a different hotbar slot
    pub fn select_hotbar_slot(&mut self, slot: usize) {
        if slot < self.hotbar_size {
            self.selected_hotbar_slot = slot;
        }
    }

    /// Cycle to the next hotbar slot
    pub fn next_hotbar_slot(&mut self) {
        self.selected_hotbar_slot = (self.selected_hotbar_slot + 1) % self.hotbar_size;
    }

    /// Cycle to the previous hotbar slot
    pub fn previous_hotbar_slot(&mut self) {
        if self.selected_hotbar_slot == 0 {
            self.selected_hotbar_slot = self.hotbar_size - 1;
        } else {
            self.selected_hotbar_slot -= 1;
        }
    }

    /// Add an item to the inventory, trying to stack first
    pub fn add_item(&mut self, item_type: ItemType, mut quantity: u32) -> bool {
        // First try to add to existing stacks in hotbar
        for stack in &mut self.hotbar_slots {
            if stack.item_type == item_type && stack.can_add(quantity, self.max_stack_size) {
                let remaining = stack.add(quantity, self.max_stack_size);
                if remaining == 0 {
                    return true;
                }
                quantity = remaining;
            }
        }

        // Then try to add to existing stacks in main inventory
        for stack in &mut self.slots {
            if stack.item_type == item_type && stack.can_add(quantity, self.max_stack_size) {
                let remaining = stack.add(quantity, self.max_stack_size);
                if remaining == 0 {
                    return true;
                }
                quantity = remaining;
            }
        }

        // If we still have items left, try to find empty slots
        // First check hotbar
        for stack in &mut self.hotbar_slots {
            if stack.is_empty() {
                stack.item_type = item_type;
                stack.quantity = quantity;
                return true;
            }
        }

        // Then check main inventory
        for stack in &mut self.slots {
            if stack.is_empty() {
                stack.item_type = item_type;
                stack.quantity = quantity;
                return true;
            }
        }

        false // Inventory is full
    }

    /// Remove an item from the inventory
    pub fn remove_item(&mut self, item_type: ItemType, quantity: u32) -> u32 {
        let mut remaining_quantity = quantity;

        // First try to remove from hotbar
        for stack in &mut self.hotbar_slots {
            if stack.item_type == item_type {
                remaining_quantity = stack.remove(remaining_quantity);
                if remaining_quantity == 0 {
                    break;
                }
            }
        }

        // If we still need more, try main inventory
        if remaining_quantity > 0 {
            for stack in &mut self.slots {
                if stack.item_type == item_type {
                    remaining_quantity = stack.remove(remaining_quantity);
                    if remaining_quantity == 0 {
                        break;
                    }
                }
            }
        }

        quantity - remaining_quantity
    }

    /// Get the total count of a specific item type in the inventory
    #[allow(dead_code)]
    pub fn get_item_count(&self, item_type: ItemType) -> u32 {
        let mut count = 0;

        // Count in hotbar
        for stack in &self.hotbar_slots {
            if stack.item_type == item_type {
                count += stack.quantity;
            }
        }

        // Count in main inventory
        for stack in &self.slots {
            if stack.item_type == item_type {
                count += stack.quantity;
            }
        }

        count
    }

    /// Check if the inventory contains at least the specified quantity of an item
    #[allow(dead_code)]
    pub fn has_item(&self, item_type: ItemType, quantity: u32) -> bool {
        self.get_item_count(item_type) >= quantity
    }

    /// Clear a specific slot
    #[allow(dead_code)]
    pub fn clear_slot(&mut self, slot_index: usize, is_hotbar: bool) {
        if is_hotbar && slot_index < self.hotbar_size {
            self.hotbar_slots[slot_index] = ItemStack::new(ItemType::Block(BlockType::Air), 0);
        } else if !is_hotbar && slot_index < self.inventory_size {
            self.slots[slot_index] = ItemStack::new(ItemType::Block(BlockType::Air), 0);
        }
    }

    /// Swap items between two slots
    #[allow(dead_code)]
    pub fn swap_slots(&mut self, slot1: usize, is_hotbar1: bool, slot2: usize, is_hotbar2: bool) {
        // Handle different combinations of slot types
        if is_hotbar1 && is_hotbar2 && slot1 < self.hotbar_size && slot2 < self.hotbar_size {
            // Both slots are in hotbar - use split_at_mut to avoid double borrow
            let (left, right) = self.hotbar_slots.split_at_mut(slot2.max(slot1));
            let (left, right) = if slot1 < slot2 {
                (&mut left[slot1], &mut right[0])
            } else {
                (&mut right[0], &mut left[slot2])
            };
            std::mem::swap(left, right);
        } else if !is_hotbar1
            && !is_hotbar2
            && slot1 < self.inventory_size
            && slot2 < self.inventory_size
        {
            // Both slots are in main inventory - use split_at_mut to avoid double borrow
            let (left, right) = self.slots.split_at_mut(slot2.max(slot1));
            let (left, right) = if slot1 < slot2 {
                (&mut left[slot1], &mut right[0])
            } else {
                (&mut right[0], &mut left[slot2])
            };
            std::mem::swap(left, right);
        } else if is_hotbar1
            && !is_hotbar2
            && slot1 < self.hotbar_size
            && slot2 < self.inventory_size
        {
            // Swap between hotbar and inventory - no double borrow issue here
            let hotbar_item = std::mem::replace(
                &mut self.hotbar_slots[slot1],
                ItemStack::new(ItemType::Block(BlockType::Air), 0),
            );
            let inventory_item = std::mem::replace(&mut self.slots[slot2], hotbar_item);
            self.hotbar_slots[slot1] = inventory_item;
        } else if !is_hotbar1
            && is_hotbar2
            && slot1 < self.inventory_size
            && slot2 < self.hotbar_size
        {
            // Swap between inventory and hotbar - no double borrow issue here
            let inventory_item = std::mem::replace(
                &mut self.slots[slot1],
                ItemStack::new(ItemType::Block(BlockType::Air), 0),
            );
            let hotbar_item = std::mem::replace(&mut self.hotbar_slots[slot2], inventory_item);
            self.slots[slot1] = hotbar_item;
        }
    }
}

/// System to handle inventory updates
pub fn inventory_update_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
) {
    // Handle hotbar selection with number keys
    // In Bevy 0.15, we need to check individual key codes
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        inventory.select_hotbar_slot(0);
        info!("Selected hotbar slot: 0");
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        inventory.select_hotbar_slot(1);
        info!("Selected hotbar slot: 1");
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        inventory.select_hotbar_slot(2);
        info!("Selected hotbar slot: 2");
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        inventory.select_hotbar_slot(3);
        info!("Selected hotbar slot: 3");
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) {
        inventory.select_hotbar_slot(4);
        info!("Selected hotbar slot: 4");
    }
    if keyboard_input.just_pressed(KeyCode::Digit6) {
        inventory.select_hotbar_slot(5);
        info!("Selected hotbar slot: 5");
    }
    if keyboard_input.just_pressed(KeyCode::Digit7) {
        inventory.select_hotbar_slot(6);
        info!("Selected hotbar slot: 6");
    }
    if keyboard_input.just_pressed(KeyCode::Digit8) {
        inventory.select_hotbar_slot(7);
        info!("Selected hotbar slot: 7");
    }
    if keyboard_input.just_pressed(KeyCode::Digit9) {
        inventory.select_hotbar_slot(8);
        info!("Selected hotbar slot: 8");
    }

    // Handle hotbar cycling with arrow keys
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        inventory.next_hotbar_slot();
        info!("Selected hotbar slot: {}", inventory.selected_hotbar_slot);
    }

    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        inventory.previous_hotbar_slot();
        info!("Selected hotbar slot: {}", inventory.selected_hotbar_slot);
    }
}

/// System to display inventory debug information
pub fn display_inventory_info(inventory: Res<Inventory>) {
    // Display selected item information
    if let Some(selected_item) = inventory.get_selected_item() {
        if !selected_item.is_empty() {
            info!(
                "Selected: {} x{}",
                selected_item.item_type.name(),
                selected_item.quantity
            );
        }
    }
}

/// System to initialize the inventory with some starting items
pub fn initialize_inventory(mut inventory: ResMut<Inventory>) {
    // Add some starting items for testing
    inventory.add_item(ItemType::Block(BlockType::Stone), 32);
    inventory.add_item(ItemType::Block(BlockType::Dirt), 16);
    inventory.add_item(ItemType::Block(BlockType::Grass), 8);
    inventory.add_item(ItemType::Block(BlockType::Wood), 16);
    inventory.add_item(ItemType::Block(BlockType::Sand), 8);
    inventory.add_item(ItemType::Resource(ResourceType::Stick), 32);

    // Add some food items for testing
    inventory.add_item(ItemType::Food(FoodType::Apple), 10);
    inventory.add_item(ItemType::Food(FoodType::Bread), 5);
    inventory.add_item(ItemType::Food(FoodType::Carrot), 8);
    inventory.add_item(ItemType::Food(FoodType::Potato), 8);
    inventory.add_item(ItemType::Food(FoodType::Mushroom), 12);
    inventory.add_item(ItemType::Food(FoodType::MeatRaw), 6);
    inventory.add_item(ItemType::Food(FoodType::MeatCooked), 4);

    info!("Inventory initialized with starting items");
}
