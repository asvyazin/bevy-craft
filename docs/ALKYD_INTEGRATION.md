# Alkyd Integration Guide

This document describes the integration of the Alkyd library for procedural texture generation in Bevy Craft.

## Overview

Alkyd is a Bevy crate for handling procedural textures and shaders. This integration provides the foundation for dynamic texture generation in the game.

## Integration Details

### 1. Dependency Setup

Added to `Cargo.toml`:
```toml
[dependencies]
alkyd = "0.3.2"
```

### 2. Plugin Configuration

In `src/main.rs`, the Alkyd plugin is added to the Bevy app:
```rust
.add_plugins(alkyd::AlkydPlugin) // Add alkyd plugin for procedural texture generation
```

### 3. Texture Generation Module

Created `src/texture_gen.rs` with:
- `TextureGenSettings` resource for configuration
- `ProceduralTexture` component marker
- `generate_procedural_textures` system for texture generation
- `spawn_procedural_texture_demo` system for demonstration

### 4. System Integration

Added to the Bevy app setup:
```rust
.init_resource::<TextureGenSettings>() // Initialize texture generation settings
.add_systems(Startup, spawn_procedural_texture_demo) // Add procedural texture demo
.add_systems(Update, generate_procedural_textures) // Add procedural texture generation
```

## Current Implementation

The current implementation includes:

1. **Basic Noise Generation**: Simple procedural noise for texture patterns
2. **Color Mapping**: Converts noise values to RGB colors
3. **Demo System**: Shows a 256x256 procedural texture
4. **Configuration**: Adjustable texture size, noise scale, and octaves

## Usage

To use procedural textures on an entity:

```rust
commands.spawn((
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(256.0, 256.0)),
            ..default()
        },
        ..default()
    },
    ProceduralTexture, // This marker triggers texture generation
));
```

## Future Enhancements

The current implementation uses basic noise generation as a foundation. Future work should:

1. **Use Alkyd's Advanced Features**: Replace basic noise with Alkyd's GPU-accelerated shaders
2. **Block Texture Integration**: Generate textures for different block types
3. **Performance Optimization**: Use Alkyd's compute shaders for faster generation
4. **Texture Variants**: Create multiple texture patterns for the same block type

## Testing

Run the verification script to check integration:
```bash
./verify_integration.sh
```

Run the game to see the procedural texture demo:
```bash
cargo run
```

## Documentation

- [Alkyd Crate Documentation](https://docs.rs/alkyd/0.3.2)
- [Alkyd GitHub Repository](https://github.com/KyWinston/alkyd)

## Troubleshooting

If you encounter issues:

1. **Build Errors**: Ensure all dependencies are properly specified in `Cargo.toml`
2. **Plugin Issues**: Verify the Alkyd plugin is added before other systems that depend on it
3. **Texture Display**: Check that the entity has the `ProceduralTexture` component

## License

Alkyd is licensed under MIT OR Apache-2.0, compatible with Bevy Craft's licensing.