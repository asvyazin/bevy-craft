// Hotbar UI system for Bevy Craft
// This module handles the visual display of the hotbar and selected items

use bevy::prelude::*;
use bevy::ui::Val;

use crate::inventory::Inventory;

/// Marker component for the hotbar UI root node
#[derive(Component)]
pub struct HotbarUI;

/// Marker component for individual hotbar slots
#[derive(Component)]
pub struct HotbarSlot {
    pub slot_index: usize,
}

/// System to spawn the hotbar UI
pub fn spawn_hotbar_ui(
    mut commands: Commands,
) {
    // Spawn the root UI node for the hotbar
    commands.spawn((
        NodeBundle {
            node: Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
            ..default()
        },
        HotbarUI,
    ))
    .with_children(|parent| {
        // Spawn individual hotbar slots
        for slot_index in 0..9 {
            spawn_hotbar_slot(parent, slot_index);
        }
    });
}

/// Helper function to spawn an individual hotbar slot
fn spawn_hotbar_slot(
    parent: &mut ChildBuilder,
    slot_index: usize,
) {
    parent.spawn((
        NodeBundle {
            node: Node {
                width: Val::Px(60.0),
                height: Val::Px(60.0),
                margin: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            ..default()
        },
        HotbarSlot { slot_index },
    ))
    .with_children(|slot_parent| {
        // Item icon placeholder (will be updated dynamically)
        slot_parent.spawn((
            NodeBundle {
                node: Node {
                    width: Val::Px(48.0),
                    height: Val::Px(48.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.5)),
                ..default()
            },
        ));
    });
}

/// System to update the hotbar UI based on inventory contents
pub fn update_hotbar_ui(
    inventory: Res<Inventory>,
    mut hotbar_slots: Query<(&HotbarSlot, &mut BackgroundColor)>, 
) {
    // Update the background color based on selection (simple visual feedback)
    for (hotbar_slot, mut bg_color) in &mut hotbar_slots {
        if hotbar_slot.slot_index == inventory.selected_hotbar_slot {
            bg_color.0 = Color::srgba(0.3, 0.5, 0.8, 0.9); // Highlight selected slot
        } else {
            bg_color.0 = Color::srgba(0.2, 0.2, 0.2, 0.8); // Normal slot color
        }
    }
}

/// Simple hotbar UI display system (console fallback)
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

