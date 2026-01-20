// Hotbar UI system for Bevy Craft
// This module handles the visual display of the hotbar and selected items

use bevy::prelude::*;

use crate::inventory::Inventory;

/// Simple hotbar UI display system
pub fn display_hotbar_info(
    inventory: Res<Inventory>,
) {
    // Display hotbar contents in console
    let mut hotbar_info = String::from("🎮 Hotbar: [");
    
    for (i, slot) in inventory.hotbar_slots.iter().enumerate() {
        if i == inventory.selected_hotbar_slot {
            hotbar_info.push_str(&format!("[{}]", if slot.is_empty() { "∅" } else { slot.item_type.name() }));
        } else {
            hotbar_info.push_str(if slot.is_empty() { "∅" } else { slot.item_type.name() });
        }
        
        if i < inventory.hotbar_slots.len() - 1 {
            hotbar_info.push_str(" | ");
        }
    }
    
    hotbar_info.push_str("]");
    info!("{}", hotbar_info);
    
    // Display selected item details
    if let Some(selected_item) = inventory.get_selected_item() {
        if !selected_item.is_empty() {
            info!("🔧 Selected: {} x{}", selected_item.item_type.name(), selected_item.quantity);
        }
    }
}

