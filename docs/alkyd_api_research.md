# Alkyd API Research and Integration Guide

## Overview

This document provides detailed research on the Alkyd library's API and integration examples for enhancing Bevy Craft's procedural texture generation system.

## Alkyd Core Features

### 1. GPU-Accelerated Noise Generation

Alkyd provides GPU-accelerated noise generation through compute shaders. Key components:

- **`NOISE_COMPUTE_HANDLE`**: Main noise compute shader
- **`NOISE_FUNCTIONS_HANDLE`**: General noise functions
- **`SIMPLEX_HANDLE`**: 3D Simplex noise shader
- **`SIMPLEX_4D_HANDLE`**: 4D Simplex noise shader
- **`NOISE_GEN_UTILS_HANDLE`**: Noise generation utilities

### 2. Shader Utilities

- **`GLOBAL_VALUES_HANDLE`**: Global constants and utilities
- **`BLEND_MODES_HANDLE`**: Color blending functions
- **`CONVERTERS_HANDLE`**: Color space converters
- **`SOBEL_HANDLE`**: Edge detection filter
- **`SPRITELY_HANDLE`**: Sprite rotation utilities

### 3. Compute Worker System

Alkyd includes a compute worker system for efficient GPU processing:

- **`NoiseComputeWorker`**: Pre-configured worker for noise generation
- **`AppComputeWorker`**: Generic compute worker interface
- **Workgroup-based processing**: Configurable workgroup sizes

## Integration Examples

### Basic Alkyd Setup

```rust
use bevy::prelude::*;
use alkyd::AlkydPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AlkydPlugin::default()) // Enable alkyd with default settings
        .add_systems(Startup, setup_alkyd_textures)
        .run();
}
```

### Using Alkyd Noise Shaders

```rust
use bevy::prelude::*;
use alkyd::{NOISE_COMPUTE_HANDLE, NOISE_FUNCTIONS_HANDLE};

#[derive(Resource)]
struct AlkydTextures {
    noise_compute_shader: Handle<Shader>,
    noise_functions_shader: Handle<Shader>,
}

fn setup_alkyd_textures(
    mut commands: Commands,
    shaders: Res<Assets<Shader>>,
) {
    // Access alkyd's built-in shaders
    let noise_compute = NOISE_COMPUTE_HANDLE;
    let noise_functions = NOISE_FUNCTIONS_HANDLE;
    
    // Verify shaders are loaded
    if shaders.contains(&noise_compute) && shaders.contains(&noise_functions) {
        commands.insert_resource(AlkydTextures {
            noise_compute_shader: noise_compute,
            noise_functions_shader: noise_functions,
        });
        println!("✓ Alkyd shaders loaded successfully");
    } else {
        println!("⚠ Alkyd shaders not yet loaded");
    }
}
```

### GPU-Accelerated Texture Generation

```rust
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use alkyd::{NOISE_COMPUTE_HANDLE, NOISE_FUNCTIONS_HANDLE};

#[derive(Resource)]
struct GpuTextureGenerator {
    noise_shader: Handle<Shader>,
    texture_size: UVec2,
}

fn generate_gpu_texture(
    generator: Res<GpuTextureGenerator>,
    mut images: ResMut<Assets<Image>>,
) -> Handle<Image> {
    // Create a texture with initial data (will be processed by GPU)
    let mut texture_data = vec![0u8; (generator.texture_size.x * generator.texture_size.y * 4) as usize];
    
    // Initialize with some base data
    for y in 0..generator.texture_size.y {
        for x in 0..generator.texture_size.x {
            let idx = ((y * generator.texture_size.x + x) * 4) as usize;
            // Simple gradient for initialization
            let value = (x as f32 / generator.texture_size.x as f32 * 255.0) as u8;
            texture_data[idx] = value;     // R
            texture_data[idx + 1] = value; // G  
            texture_data[idx + 2] = value; // B
            texture_data[idx + 3] = 255;   // A
        }
    }
    
    // Create image that can be processed by GPU shaders
    let image = Image::new(
        Extent3d {
            width: generator.texture_size.x,
            height: generator.texture_size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    
    images.add(image)
}
```

### Advanced Noise Generation System

```rust
use bevy::prelude::*;
use alkyd::{NOISE_COMPUTE_HANDLE, NOISE_FUNCTIONS_HANDLE, NOISE_GEN_UTILS_HANDLE};

#[derive(Resource)]
pub struct AdvancedNoiseGenerator {
    pub noise_compute_shader: Handle<Shader>,
    pub noise_functions_shader: Handle<Shader>,
    pub noise_utils_shader: Handle<Shader>,
    pub workgroup_size: u32,
    pub texture_size: UVec2,
}

impl Default for AdvancedNoiseGenerator {
    fn default() -> Self {
        Self {
            noise_compute_shader: NOISE_COMPUTE_HANDLE,
            noise_functions_shader: NOISE_FUNCTIONS_HANDLE,
            noise_utils_shader: NOISE_GEN_UTILS_HANDLE,
            workgroup_size: 8,
            texture_size: UVec2::new(256, 256),
        }
    }
}

fn setup_advanced_noise_generator(mut commands: Commands) {
    commands.insert_resource(AdvancedNoiseGenerator::default());
    println!("🎨 Advanced noise generator initialized");
}
```

### Block-Specific Texture Generation with Alkyd

```rust
use bevy::prelude::*;
use alkyd::{NOISE_COMPUTE_HANDLE, SIMPLEX_HANDLE};

#[derive(Resource)]
pub struct BlockTextureGenerator {
    pub simplex_3d_shader: Handle<Shader>,
    pub noise_compute_shader: Handle<Shader>,
    pub block_configs: HashMap<String, BlockTextureConfig>,
}

#[derive(Debug, Clone)]
pub struct BlockTextureConfig {
    pub base_color: [f32; 3],
    pub noise_scale: f32,
    pub noise_octaves: usize,
    pub use_simplex: bool,
}

impl Default for BlockTextureGenerator {
    fn default() -> Self {
        let mut block_configs = HashMap::new();
        
        // Stone configuration
        block_configs.insert("stone".to_string(), BlockTextureConfig {
            base_color: [0.5, 0.5, 0.5],
            noise_scale: 0.1,
            noise_octaves: 6,
            use_simplex: true,
        });
        
        // Dirt configuration  
        block_configs.insert("dirt".to_string(), BlockTextureConfig {
            base_color: [0.4, 0.3, 0.2],
            noise_scale: 0.08,
            noise_octaves: 5,
            use_simplex: true,
        });
        
        Self {
            simplex_3d_shader: SIMPLEX_HANDLE,
            noise_compute_shader: NOISE_COMPUTE_HANDLE,
            block_configs,
        }
    }
}

fn generate_block_texture_with_alkyd(
    generator: Res<BlockTextureGenerator>,
    block_type: &str,
    mut images: ResMut<Assets<Image>>,
) -> Option<Handle<Image>> {
    if let Some(config) = generator.block_configs.get(block_type) {
        let texture_size = UVec2::new(128, 128);
        let mut texture_data = vec![0u8; (texture_size.x * texture_size.y * 4) as usize];
        
        // Generate texture data using alkyd-inspired approach
        for y in 0..texture_size.y {
            for x in 0..texture_size.x {
                let idx = ((y * texture_size.x + x) * 4) as usize;
                
                // Simulate GPU noise generation with CPU fallback
                let noise_value = if config.use_simplex {
                    generate_simplex_noise_approximation(
                        x as f32 * config.noise_scale,
                        y as f32 * config.noise_scale,
                        config.noise_octaves
                    )
                } else {
                    generate_perlin_noise_approximation(
                        x as f32 * config.noise_scale,
                        y as f32 * config.noise_scale,
                        config.noise_octaves
                    )
                };
                
                // Apply base color with noise variation
                let r = ((config.base_color[0] + noise_value * 0.2).clamp(0.0, 1.0) * 255.0) as u8;
                let g = ((config.base_color[1] + noise_value * 0.2).clamp(0.0, 1.0) * 255.0) as u8;
                let b = ((config.base_color[2] + noise_value * 0.2).clamp(0.0, 1.0) * 255.0) as u8;
                
                texture_data[idx] = r;
                texture_data[idx + 1] = g;
                texture_data[idx + 2] = b;
                texture_data[idx + 3] = 255;
            }
        }
        
        let image = Image::new(
            Extent3d {
                width: texture_size.x,
                height: texture_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        
        Some(images.add(image))
    } else {
        None
    }
}

// Helper functions for noise approximation
fn generate_simplex_noise_approximation(x: f32, y: f32, octaves: usize) -> f32 {
    // Simplified simplex noise approximation
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..octaves {
        // Basic simplex-like pattern
        let nx = x * frequency;
        let ny = y * frequency;
        let s = (nx + ny) * 0.5;
        let i = nx.floor();
        let j = ny.floor();
        
        // Simple hash-based noise
        let mut n = (i as u32).wrapping_mul(1664525) ^ (j as u32).wrapping_mul(1013904223);
        n ^= n >> 13;
        n ^= n << 9;
        n ^= n >> 17;
        
        let noise = (n as f32 / u32::MAX as f32) * 2.0 - 1.0;
        
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    (value / max_value + 1.0) / 2.0 // Normalize to [0, 1]
}

fn generate_perlin_noise_approximation(x: f32, y: f32, octaves: usize) -> f32 {
    // Simplified perlin noise approximation
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..octaves {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - xi as f32;
        let yf = y - yi as f32;
        
        // Simple gradient noise
        let mut n = (xi as u32).wrapping_mul(1664525) ^ (yi as u32).wrapping_mul(1013904223);
        n ^= n >> 13;
        n ^= n << 9;
        n ^= n >> 17;
        
        let noise = (n as f32 / u32::MAX as f32) * 2.0 - 1.0;
        
        value += noise * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    (value / max_value + 1.0) / 2.0 // Normalize to [0, 1]
}
```

## Integration Strategy

### Phase 1: Basic Alkyd Integration
1. **Enable Alkyd Plugin**: Uncomment the alkyd plugin in `main.rs`
2. **Verify Shader Loading**: Ensure all alkyd shaders are properly loaded
3. **Create Test System**: Develop a simple system that uses alkyd shaders

### Phase 2: GPU-Accelerated Texture Generation
1. **Replace CPU Noise**: Modify `generate_procedural_texture_data` to use GPU shaders
2. **Add Shader Binding**: Create shader bindings for texture generation parameters
3. **Implement Compute Pipeline**: Use alkyd's compute worker for efficient processing

### Phase 3: Advanced Features
1. **Add Blend Modes**: Use alkyd's blend mode shaders for complex textures
2. **Implement Filters**: Add sobel edge detection for texture enhancement
3. **Color Space Conversion**: Use alkyd's converters for better color handling

## Performance Considerations

### GPU vs CPU Comparison

| Approach | Performance | Quality | Integration Complexity |
|----------|-------------|---------|-------------------------|
| CPU Noise (Current) | Good | Basic | Low |
| GPU Noise (Alkyd) | Excellent | High | Medium |
| Hybrid Approach | Very Good | High | High |

### Optimization Strategies

1. **Texture Caching**: Cache generated textures to avoid redundant computation
2. **Batch Processing**: Generate multiple textures in a single compute pass
3. **LOD System**: Use lower resolution textures for distant blocks
4. **Async Generation**: Offload texture generation to background threads

## Migration Plan

### Step 1: Enable Alkyd Plugin
```rust
// In main.rs
.add_plugins(AlkydPlugin::default()) // Enable alkyd plugin
```

### Step 2: Create Alkyd Integration Module
```rust
// src/alkyd_integration.rs
pub mod alkyd_integration {
    use bevy::prelude::*;
    use alkyd::*;
    
    pub fn setup_alkyd_systems(app: &mut App) {
        app.add_systems(Startup, initialize_alkyd_resources)
           .add_systems(Update, generate_alkyd_textures);
    }
    
    // Implementation goes here
}
```

### Step 3: Gradual Replacement
1. Start with one block type (e.g., stone)
2. Compare visual quality and performance
3. Expand to other block types
4. Optimize based on profiling results

## Conclusion

Alkyd provides powerful GPU-accelerated features that can significantly enhance Bevy Craft's procedural texture generation:

- **Performance**: GPU acceleration for real-time texture generation
- **Quality**: Advanced noise algorithms and shader effects
- **Integration**: Native Bevy support for smooth integration
- **Extensibility**: Comprehensive shader library for future enhancements

The migration should be done gradually, starting with basic integration and expanding to advanced features while maintaining compatibility with the existing texture atlas system.