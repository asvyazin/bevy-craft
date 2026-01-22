// Hotbar UI system for Bevy Craft
// This module handles the visual display of the hotbar and selected items

use bevy::prelude::*;
use bevy::ui::Val;
use std::collections::HashMap;

use crate::block::BlockType;
use crate::inventory::{Inventory, ItemType};

/// Marker component for the hotbar UI root node
#[derive(Component)]
pub struct HotbarUI;

/// Marker component for individual hotbar slots
#[derive(Component)]
pub struct HotbarSlot {
    pub slot_index: usize,
}

/// Marker component for item icons in hotbar slots
#[derive(Component)]
pub struct HotbarItemIcon {
    pub slot_index: usize,
}

/// Component that stores the texture handle for a hotbar item icon
#[derive(Component)]
pub struct HotbarItemTexture {
    pub texture_handle: Handle<Image>,
}

/// Resource that maps item types to their icon texture paths
#[derive(Resource, Default)]
pub struct ItemTextureAtlas {
    pub texture_handles: HashMap<ItemType, Handle<Image>>,
}

/// System to spawn the hotbar UI
pub fn spawn_hotbar_ui(mut commands: Commands) {
    // Spawn the root UI node for the hotbar
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
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
fn spawn_hotbar_slot(parent: &mut ChildBuilder, slot_index: usize) {
    parent
        .spawn((
            Node {
                width: Val::Px(60.0),
                height: Val::Px(60.0),
                margin: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            HotbarSlot { slot_index },
        ))
        .with_children(|slot_parent| {
            // Item icon - will be updated with actual textures
            slot_parent.spawn((
                HotbarItemIcon { slot_index },
                HotbarItemTexture {
                    texture_handle: Handle::default(),
                },
            ));
        });
}

/// System to initialize the item texture atlas
pub fn initialize_item_texture_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut texture_handles = HashMap::new();

    // Load textures for block items
    texture_handles.insert(
        ItemType::Block(BlockType::Grass),
        asset_server.load("textures/grass_icon.png"),
    );
    texture_handles.insert(
        ItemType::Block(BlockType::Dirt),
        asset_server.load("textures/dirt_icon.png"),
    );
    texture_handles.insert(
        ItemType::Block(BlockType::Stone),
        asset_server.load("textures/stone_icon.png"),
    );
    texture_handles.insert(
        ItemType::Block(BlockType::Wood),
        asset_server.load("textures/wood_icon.png"),
    );
    texture_handles.insert(
        ItemType::Block(BlockType::Leaves),
        asset_server.load("textures/leaves_icon.png"),
    );
    texture_handles.insert(
        ItemType::Block(BlockType::Sand),
        asset_server.load("textures/sand_icon.png"),
    );
    texture_handles.insert(
        ItemType::Block(BlockType::Bedrock),
        asset_server.load("textures/bedrock_icon.png"),
    );
    texture_handles.insert(
        ItemType::Block(BlockType::Water),
        asset_server.load("textures/water_icon.png"),
    );

    // Load textures for tools
    texture_handles.insert(
        ItemType::Tool(crate::inventory::ToolType::Pickaxe),
        asset_server.load("textures/pickaxe_icon.png"),
    );
    texture_handles.insert(
        ItemType::Tool(crate::inventory::ToolType::Axe),
        asset_server.load("textures/axe_icon.png"),
    );
    texture_handles.insert(
        ItemType::Tool(crate::inventory::ToolType::Shovel),
        asset_server.load("textures/shovel_icon.png"),
    );
    texture_handles.insert(
        ItemType::Tool(crate::inventory::ToolType::Sword),
        asset_server.load("textures/sword_icon.png"),
    );

    // Load textures for resources
    texture_handles.insert(
        ItemType::Resource(crate::inventory::ResourceType::Stick),
        asset_server.load("textures/stick_icon.png"),
    );
    texture_handles.insert(
        ItemType::Resource(crate::inventory::ResourceType::String),
        asset_server.load("textures/string_icon.png"),
    );
    texture_handles.insert(
        ItemType::Resource(crate::inventory::ResourceType::Coal),
        asset_server.load("textures/coal_icon.png"),
    );
    texture_handles.insert(
        ItemType::Resource(crate::inventory::ResourceType::IronIngot),
        asset_server.load("textures/iron_ingot_icon.png"),
    );
    texture_handles.insert(
        ItemType::Resource(crate::inventory::ResourceType::GoldIngot),
        asset_server.load("textures/gold_ingot_icon.png"),
    );

    // Add empty slot texture
    // Load empty slot and unknown textures (unused variables kept for reference)
    let _empty_slot_texture: Handle<Image> = asset_server.load("textures/empty_slot.png");
    let _unknown_texture: Handle<Image> = asset_server.load("textures/unknown_icon.png");

    let texture_count = texture_handles.len();
    commands.insert_resource(ItemTextureAtlas {
        texture_handles: texture_handles.clone(),
    });

    info!(
        "🎨 Initialized item texture atlas with {} textures",
        texture_count
    );

    // Debug: Print all loaded textures
    for (item_type, handle) in &texture_handles {
        info!("Loaded texture for {:?}: {:?}", item_type, handle);
    }
}

/// System to update the hotbar item icons based on inventory contents
pub fn update_hotbar_item_icons(
    inventory: Res<Inventory>,
    item_textures: Res<ItemTextureAtlas>,
    asset_server: Res<AssetServer>,
    mut item_textures_query: Query<(&HotbarItemIcon, &mut HotbarItemTexture)>,
) {
    let empty_slot_texture: Handle<Image> = asset_server.load("textures/empty_slot.png");

    for (item_icon, mut item_texture) in &mut item_textures_query {
        if item_icon.slot_index < inventory.hotbar_slots.len() {
            let item_stack = &inventory.hotbar_slots[item_icon.slot_index];

            info!(
                "Updating hotbar slot {}: {:?} x{}",
                item_icon.slot_index, item_stack.item_type, item_stack.quantity
            );

            if item_stack.is_empty() {
                // Empty slot - use empty slot texture
                item_texture.texture_handle = empty_slot_texture.clone();
                info!("Slot {} is empty", item_icon.slot_index);
            } else {
                // Find the appropriate texture for this item type
                if let Some(texture_handle) =
                    item_textures.texture_handles.get(&item_stack.item_type)
                {
                    item_texture.texture_handle = texture_handle.clone();
                    info!(
                        "Slot {}: Found texture for {:?}",
                        item_icon.slot_index, item_stack.item_type
                    );
                } else {
                    // Fallback to unknown texture if no specific texture found
                    item_texture.texture_handle = asset_server.load("textures/unknown_icon.png");
                    info!(
                        "Slot {}: Using unknown texture for {:?}",
                        item_icon.slot_index, item_stack.item_type
                    );
                }
            }
        }
    }
}

/// System to render UI images for hotbar item icons
pub fn render_hotbar_item_images(
    item_textures: Query<(Entity, &HotbarItemTexture, &HotbarItemIcon), Changed<HotbarItemTexture>>,
    mut commands: Commands,
) {
    for (entity, item_texture, item_icon) in &item_textures {
        info!(
            "Rendering hotbar item image for slot {} with texture: {:?}",
            item_icon.slot_index, item_texture.texture_handle
        );

        // Remove any existing UI image children
        commands.entity(entity).despawn_descendants();

        // Spawn new UI image with the current texture
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(48.0),
                    height: Val::Px(48.0),
                    ..default()
                },
                ImageNode {
                    image: item_texture.texture_handle.clone(),
                    ..default()
                },
            ));
        });
    }
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
pub fn display_hotbar_info(inventory: Res<Inventory>) {
    // Display hotbar contents in console
    let mut hotbar_info = String::from("🎮 Hotbar: [");

    for (i, slot) in inventory.hotbar_slots.iter().enumerate() {
        if i == inventory.selected_hotbar_slot {
            hotbar_info.push_str(&format!(
                "[{}]",
                if slot.is_empty() {
                    "∅"
                } else {
                    slot.item_type.name()
                }
            ));
        } else {
            hotbar_info.push_str(if slot.is_empty() {
                "∅"
            } else {
                slot.item_type.name()
            });
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
            info!(
                "🔧 Selected: {} x{}",
                selected_item.item_type.name(),
                selected_item.quantity
            );
        }
    }
}
