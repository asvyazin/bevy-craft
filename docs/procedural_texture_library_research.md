# Research: Procedural Texture Generation Library Selection

## Overview
This document summarizes the research conducted to select an appropriate procedural texture generation library for the Bevy Craft project. The goal is to enable dynamic texture generation for blocks based on various parameters like biome, height, and environmental conditions.

## Requirements
- Procedural texture generation for block textures
- Good performance for real-time applications
- Easy integration with Bevy ECS
- Compatibility with existing texture atlas system
- Active maintenance and good documentation
- Support for parameterized texture generation (biome, height, etc.)

## Evaluated Libraries

### 1. texture-synthesis
- **Type**: General-purpose texture synthesis library
- **Version**: 0.8.2
- **License**: MIT OR Apache-2.0
- **Features**:
  - Multiresolution Stochastic Texture Synthesis
  - Non-parametric example-based algorithm for image generation
  - Can generate textures from example images
  - Supports various synthesis parameters
- **Performance**: Good (CPU-based, optimized algorithms)
- **Integration**: Requires manual integration with Bevy
- **Repository**: https://github.com/EmbarkStudios/texture-synthesis
- **Best for**: Generating textures from example images, creating variations of existing textures

### 2. funutd
- **Type**: Procedural texture library
- **Version**: 0.16.0
- **License**: MIT OR Apache-2.0
- **Features**:
  - Comprehensive procedural texture generation
  - Multiple noise algorithms (Perlin, Simplex, Worley, etc.)
  - Support for 2D and 3D textures
  - Flexible parameterization and layering
  - Can generate textures from mathematical functions
- **Performance**: Good (CPU-based, optimized)
- **Integration**: Requires manual integration with Bevy
- **Repository**: https://github.com/SamiPerttu/funutd
- **Best for**: Mathematical/procedural texture generation, noise-based textures

### 3. alkyd
- **Type**: Bevy-specific procedural texture library
- **Version**: 0.3.2
- **License**: MIT OR Apache-2.0
- **Features**:
  - Designed specifically for Bevy
  - Procedural textures and shaders
  - GPU-accelerated texture generation
  - Integration with Bevy's ECS and rendering systems
  - Support for dynamic texture updates
- **Performance**: Excellent (GPU-accelerated)
- **Integration**: Native Bevy integration
- **Repository**: https://github.com/KyWinston/alkyd
- **Best for**: Bevy-native procedural textures, GPU-accelerated generation

### 4. wassily-effects
- **Type**: Generative art and procedural textures library
- **Version**: 0.2.0
- **License**: MIT OR Apache-2.0
- **Features**:
  - Visual effects and procedural textures
  - Part of the Wassily generative art framework
  - Various texture generation algorithms
  - Support for artistic/abstract textures
- **Performance**: Good (CPU-based)
- **Integration**: Requires manual integration with Bevy
- **Repository**: https://github.com/jeffreyrosenbluth/wassily/
- **Best for**: Artistic/abstract procedural textures, generative art patterns

### 5. bevy-mutate-image
- **Type**: Bevy image manipulation library
- **Version**: 0.1.0
- **License**: MIT OR Apache-2.0
- **Features**:
  - Create and update images/textures programmatically
  - Manual pixel manipulation
  - Integration with Bevy's image assets
  - Can be used to build custom procedural textures
- **Performance**: Good (CPU-based pixel manipulation)
- **Integration**: Native Bevy integration
- **Repository**: https://github.com/taurr/bevy-mutable-image
- **Best for**: Custom texture generation, pixel-level control

### 6. bevy_compute_noise (already integrated)
- **Type**: Bevy compute shader noise library
- **Version**: 0.1.1 (already in project)
- **License**: MIT OR Apache-2.0
- **Features**:
  - GPU-accelerated noise generation using compute shaders
  - Already integrated for world generation
  - Can be extended for texture generation
  - Excellent performance for real-time applications
- **Performance**: Excellent (GPU-accelerated)
- **Integration**: Already integrated
- **Repository**: https://github.com/jadedbay/bevy_compute_noise
- **Best for**: Noise-based procedural textures, consistent with existing world generation

## Recommendation

**Selected Library**: **alkyd** with potential extension using **bevy_compute_noise**

### Rationale:

1. **Bevy Integration**: alkyd is designed specifically for Bevy, ensuring smooth integration with the ECS system and existing rendering pipeline

2. **Performance**: GPU acceleration provides better performance for real-time texture generation, crucial for a game environment

3. **Consistency**: Using alkyd alongside the already-integrated bevy_compute_noise provides a consistent GPU-based approach for both world and texture generation

4. **Features**: alkyd provides comprehensive procedural texture capabilities while maintaining compatibility with Bevy's rendering systems

5. **Future-proof**: Being a Bevy-native solution, it's more likely to remain compatible with future Bevy updates

### Implementation Strategy:

1. **Primary Library**: Use alkyd for main procedural texture generation
2. **Noise Integration**: Leverage existing bevy_compute_noise for noise-based texture components
3. **Fallback Option**: Consider funutd as a CPU-based fallback for complex mathematical textures
4. **Custom Extension**: Use bevy-mutate-image for any custom pixel-level manipulation needs

### Implementation Plan:

1. Add alkyd to Cargo.toml dependencies
2. Set up the alkyd plugin in the Bevy app
3. Create texture generation systems that integrate with the existing texture atlas
4. Develop parameterized texture generation based on biome, height, and other environmental factors
5. Integrate with the chunk mesh generation pipeline

## Comparison Table

| Library | Type | Performance | Bevy Integration | Features |
|---------|------|-------------|------------------|----------|
| `alkyd` | Bevy Plugin | Excellent (GPU) | Native | Procedural textures, GPU-accelerated, shader support |
| `texture-synthesis` | General-purpose | Good (CPU) | Manual | Example-based synthesis, texture variation |
| `funutd` | General-purpose | Good (CPU) | Manual | Mathematical textures, multiple noise algorithms |
| `wassily-effects` | Generative Art | Good (CPU) | Manual | Artistic textures, generative patterns |
| `bevy-mutate-image` | Bevy Plugin | Good (CPU) | Native | Pixel manipulation, custom texture generation |
| `bevy_compute_noise` | Bevy Plugin | Excellent (GPU) | Already Integrated | Noise-based textures, compute shaders |

## Next Steps

1. **Dependency Setup**: Add alkyd to the project dependencies
2. **Basic Integration**: Set up alkyd plugin and verify basic functionality
3. **Texture Generation System**: Create systems for generating procedural textures
4. **Parameterization**: Implement biome and height-based texture variation
5. **Integration**: Connect with existing texture atlas and chunk mesh systems
6. **Performance Testing**: Benchmark texture generation performance
7. **Visual Testing**: Verify visual quality of generated textures

## Technical Considerations

### Texture Format Compatibility
- Ensure generated textures use formats compatible with Bevy's rendering pipeline
- Consider using `TextureFormat::Rgba8Unorm` for linear color space compatibility
- Verify texture dimensions and aspect ratios match existing atlas requirements

### Performance Optimization
- Implement texture caching to avoid regenerating identical textures
- Use GPU acceleration where possible for real-time generation
- Consider async texture generation for complex patterns
- Implement LOD (Level of Detail) for distant textures

### Integration Points
- **Texture Atlas**: Modify existing atlas system to support procedural textures
- **Chunk Mesh**: Update mesh generation to use procedural texture UV coordinates
- **Block System**: Add texture parameters to block definitions
- **World Generation**: Integrate texture generation with biome and height data

## Conclusion

The combination of **alkyd** (for comprehensive procedural textures) and **bevy_compute_noise** (for noise-based components) provides the best solution for Bevy Craft's procedural texture generation needs. This approach ensures excellent performance, seamless Bevy integration, and consistency with the existing world generation system.