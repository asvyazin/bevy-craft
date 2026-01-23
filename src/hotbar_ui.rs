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
                Node {
                    width: Val::Px(48.0),
                    height: Val::Px(48.0),
                    ..default()
                },
                HotbarItemIcon { slot_index },
                HotbarItemTexture {
                    texture_handle: Handle::default(),
                },
            ));
        });
}

/// System to initialize the item texture atlas
pub fn initialize_item_texture_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
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

    // Load and convert empty slot and unknown textures
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
/// System to fix texture formats for UI compatibility
/// Converts any images with incompatible formats to Rgba8UnormSrgb
pub fn fix_ui_texture_formats(mut images: ResMut<Assets<Image>>) {
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::TextureDimension;
    use bevy::render::render_resource::TextureFormat;

    let mut converted_count = 0;

    for (handle_id, image) in images.iter_mut() {
        let size = image.size();
        let expected_rgba8_size = (size.x * size.y * 4) as usize;

        // Check if image has 16-bit format (data size suggests 2 bytes per pixel)
        let is_16bit = image.data.len() == (size.x * size.y * 2) as usize
            || image.data.len() == (size.x * size.y * 4) as usize
            || image.data.len() == (size.x * size.y * 6) as usize
            || image.data.len() == (size.x * size.y * 8) as usize;

        if is_16bit && expected_rgba8_size * 2 == image.data.len() {
            // R16 or RG16 format (2 bytes per pixel expected, but 4 bytes actual)
            continue;
        }

        if is_16bit {
            info!(
                "Converting image {:?} (size: {}x{}, data: {} bytes) to Rgba8UnormSrgb for UI compatibility",
                handle_id, size.x, size.y, image.data.len()
            );

            // Get original data
            let original_data = image.data.clone();

            // Simple heuristic: convert 16-bit to 8-bit based on data size
            let converted_data = if image.data.len() == (size.x * size.y * 2) as usize {
                // Likely R16 format (1 channel, 2 bytes per pixel)
                convert_r16_to_rgba8(&original_data, size)
            } else if image.data.len() == (size.x * size.y * 4) as usize {
                // Could be RG16 (2 channels, 2 bytes each) or RGB565
                convert_rg16_to_rgba8(&original_data, size)
            } else if image.data.len() == (size.x * size.y * 6) as usize {
                convert_rgb16_to_rgba8(&original_data, size)
            } else if image.data.len() == (size.x * size.y * 8) as usize {
                convert_rgba16_to_rgba8(&original_data, size)
            } else {
                original_data
            };

            // Create new image with correct format
            *image = Image::new(
                bevy::render::render_resource::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                converted_data,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );

            converted_count += 1;
        }
    }

    if converted_count > 0 {
        info!(
            "🔧 Converted {} textures to UI-compatible format",
            converted_count
        );
    }
}

// Conversion functions for different 16-bit formats
fn convert_r16_to_rgba8(data: &[u8], size: bevy::math::UVec2) -> Vec<u8> {
    let mut result = Vec::with_capacity((size.x * size.y * 4) as usize);
    for chunk in data.chunks_exact(2) {
        let val = u16::from_le_bytes([chunk[0], chunk[1]]);
        let byte = (val >> 8) as u8;
        result.extend_from_slice(&[byte, byte, byte, 255]);
    }
    result
}

fn convert_rg16_to_rgba8(data: &[u8], size: bevy::math::UVec2) -> Vec<u8> {
    let mut result = Vec::with_capacity((size.x * size.y * 4) as usize);
    for chunk in data.chunks_exact(4) {
        let r = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g = u16::from_le_bytes([chunk[2], chunk[3]]);
        result.extend_from_slice(&[(r >> 8) as u8, (g >> 8) as u8, 0, 255]);
    }
    result
}

fn convert_rgba16_to_rgba8(data: &[u8], _size: bevy::math::UVec2) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(8) {
        let r = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g = u16::from_le_bytes([chunk[2], chunk[3]]);
        let b = u16::from_le_bytes([chunk[4], chunk[5]]);
        let a = u16::from_le_bytes([chunk[6], chunk[7]]);
        result.extend_from_slice(&[
            (r >> 8) as u8,
            (g >> 8) as u8,
            (b >> 8) as u8,
            (a >> 8) as u8,
        ]);
    }
    result
}

fn convert_rgb16_to_rgba8(data: &[u8], _size: bevy::math::UVec2) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(6) {
        let r = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g = u16::from_le_bytes([chunk[2], chunk[3]]);
        let b = u16::from_le_bytes([chunk[4], chunk[5]]);
        result.extend_from_slice(&[(r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8, 255]);
    }
    result
}

fn convert_r16i_to_rgba8(data: &[u8], size: bevy::math::UVec2) -> Vec<u8> {
    let mut result = Vec::with_capacity((size.x * size.y * 4) as usize);
    for chunk in data.chunks_exact(2) {
        let val = i16::from_le_bytes([chunk[0], chunk[1]]);
        let byte = ((val as i32 + 32768) * 255 / 65535) as u8;
        result.extend_from_slice(&[byte, byte, byte, 255]);
    }
    result
}

fn convert_rg16i_to_rgba8(data: &[u8], size: bevy::math::UVec2) -> Vec<u8> {
    let mut result = Vec::with_capacity((size.x * size.y * 4) as usize);
    for chunk in data.chunks_exact(4) {
        let r = i16::from_le_bytes([chunk[0], chunk[1]]);
        let g = i16::from_le_bytes([chunk[2], chunk[3]]);
        let rb = ((r as i32 + 32768) * 255 / 65535) as u8;
        let gb = ((g as i32 + 32768) * 255 / 65535) as u8;
        result.extend_from_slice(&[rb, gb, 0, 255]);
    }
    result
}

fn convert_rgba16i_to_rgba8(data: &[u8], _size: bevy::math::UVec2) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(8) {
        let r = i16::from_le_bytes([chunk[0], chunk[1]]);
        let g = i16::from_le_bytes([chunk[2], chunk[3]]);
        let b = i16::from_le_bytes([chunk[4], chunk[5]]);
        let a = i16::from_le_bytes([chunk[6], chunk[7]]);
        result.extend_from_slice(&[
            ((r as i32 + 32768) * 255 / 65535) as u8,
            ((g as i32 + 32768) * 255 / 65535) as u8,
            ((b as i32 + 32768) * 255 / 65535) as u8,
            ((a as i32 + 32768) * 255 / 65535) as u8,
        ]);
    }
    result
}

/// System to update the hotbar item icons based on inventory contents
pub fn update_hotbar_item_icons(
    inventory: Res<Inventory>,
    item_textures: Res<ItemTextureAtlas>,
    asset_server: Res<AssetServer>,
    mut item_textures_query: Query<(&HotbarItemIcon, &mut HotbarItemTexture)>,
) {
    for (item_icon, mut item_texture) in &mut item_textures_query {
        if item_icon.slot_index < inventory.hotbar_slots.len() {
            let item_stack = &inventory.hotbar_slots[item_icon.slot_index];

            debug!(
                "Updating hotbar slot {}: {:?} x{}",
                item_icon.slot_index, item_stack.item_type, item_stack.quantity
            );

            if item_stack.is_empty() {
                // Empty slot - use default/empty handle (will not render)
                item_texture.texture_handle = Handle::default();
            } else {
                // Find the appropriate texture for this item type
                if let Some(texture_handle) =
                    item_textures.texture_handles.get(&item_stack.item_type)
                {
                    item_texture.texture_handle = texture_handle.clone();
                } else {
                    // Fallback to unknown texture if no specific texture found
                    item_texture.texture_handle = asset_server.load("textures/unknown_icon.png");
                }
            }
        }
    }
}

/// System to render UI images for hotbar item icons
pub fn render_hotbar_item_images(
    mut item_textures: Query<
        (Entity, &HotbarItemTexture, &HotbarItemIcon),
        Changed<HotbarItemTexture>,
    >,
    mut commands: Commands,
) {
    for (entity, item_texture, item_icon) in &mut item_textures {
        debug!(
            "Rendering hotbar item image for slot {} with texture: {:?}",
            item_icon.slot_index, item_texture.texture_handle
        );

        if item_texture.texture_handle.is_weak() {
            // Empty slot - remove ImageNode component if present
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<ImageNode>();
            }
        } else {
            // Non-empty slot - add/update ImageNode component
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(ImageNode {
                    image: item_texture.texture_handle.clone(),
                    ..default()
                });
            }
        }
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
    debug!("{}", hotbar_info);

    // Display selected item details
    if let Some(selected_item) = inventory.get_selected_item() {
        if !selected_item.is_empty() {
            debug!(
                "🔧 Selected: {} x{}",
                selected_item.item_type.name(),
                selected_item.quantity
            );
        }
    }
}
