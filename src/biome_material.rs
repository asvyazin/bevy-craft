// Enhanced Biome Material System
// This module provides custom biome-aware materials with advanced properties

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::alpha::AlphaMode;
use std::collections::{HashMap, VecDeque};

use crate::block::BlockType;
use crate::biome_textures::BiomeTextureParams;

/// Custom biome material with enhanced properties
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BiomeMaterial {
    /// Base color of the material
    #[uniform(0)]
    pub base_color: Vec4,
    
    /// Base color texture (biome-specific)
    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,
    
    /// Roughness factor (0.0 = smooth, 1.0 = rough)
    #[uniform(3)]
    pub roughness: f32,
    
    /// Metallic factor (0.0 = non-metal, 1.0 = metal)
    #[uniform(4)]
    pub metallic: f32,
    
    /// Reflectance factor (0.0 = no reflection, 1.0 = full reflection)
    #[uniform(5)]
    pub reflectance: f32,
    
    /// Biome-specific normal map intensity
    #[uniform(6)]
    pub normal_map_intensity: f32,
    
    /// Biome-specific ambient occlusion
    #[uniform(7)]
    pub ambient_occlusion: f32,
    
    /// Biome-specific emissive color
    #[uniform(8)]
    pub emissive: Vec4,
    
    /// Biome-specific height variation factor
    #[uniform(9)]
    pub height_variation: f32,
    
    /// Biome-specific moisture effect
    #[uniform(10)]
    pub moisture_effect: f32,
    
    /// Biome-specific temperature effect
    #[uniform(11)]
    pub temperature_effect: f32,
}

impl Default for BiomeMaterial {
    fn default() -> Self {
        Self {
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            base_color_texture: None,
            roughness: 0.8,
            metallic: 0.0,
            reflectance: 0.5,
            normal_map_intensity: 0.5,
            ambient_occlusion: 0.5,
            emissive: Vec4::new(0.0, 0.0, 0.0, 1.0),
            height_variation: 0.0,
            moisture_effect: 0.5,
            temperature_effect: 0.5,
        }
    }
}

impl Material for BiomeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/biome_material.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

/// Biome material properties configuration
#[derive(Debug, Clone)]
pub struct BiomeMaterialProperties {
    pub base_roughness: f32,
    pub base_metallic: f32,
    pub base_reflectance: f32,
    pub normal_intensity: f32,
    pub ambient_occlusion: f32,
    pub emissive_color: Vec4,
    pub height_variation: f32,
}

impl Default for BiomeMaterialProperties {
    fn default() -> Self {
        Self {
            base_roughness: 0.8,
            base_metallic: 0.0,
            base_reflectance: 0.5,
            normal_intensity: 0.5,
            ambient_occlusion: 0.5,
            emissive_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            height_variation: 0.0,
        }
    }
}

/// Biome material configuration for different block types
#[derive(Debug, Clone)]
pub struct BiomeMaterialConfig {
    #[allow(dead_code)]
    pub block_type: BlockType,
    pub base_properties: BiomeMaterialProperties,
    pub biome_effects: HashMap<String, BiomeMaterialProperties>, // Biome-specific overrides
}

impl BiomeMaterialConfig {
    pub fn new(block_type: BlockType) -> Self {
        let base_properties = match block_type {
            BlockType::Grass => BiomeMaterialProperties {
                base_roughness: 0.7,
                base_metallic: 0.0,
                base_reflectance: 0.3,
                normal_intensity: 0.6,
                ambient_occlusion: 0.4,
                emissive_color: Vec4::new(0.1, 0.2, 0.1, 0.0),
                height_variation: 0.1,
            },
            BlockType::Dirt => BiomeMaterialProperties {
                base_roughness: 0.8,
                base_metallic: 0.0,
                base_reflectance: 0.2,
                normal_intensity: 0.4,
                ambient_occlusion: 0.6,
                emissive_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                height_variation: 0.05,
            },
            BlockType::Stone => BiomeMaterialProperties {
                base_roughness: 0.6,
                base_metallic: 0.1,
                base_reflectance: 0.4,
                normal_intensity: 0.7,
                ambient_occlusion: 0.3,
                emissive_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                height_variation: 0.0,
            },
            BlockType::Sand => BiomeMaterialProperties {
                base_roughness: 0.9,
                base_metallic: 0.0,
                base_reflectance: 0.6,
                normal_intensity: 0.3,
                ambient_occlusion: 0.2,
                emissive_color: Vec4::new(0.3, 0.25, 0.2, 0.0),
                height_variation: 0.0,
            },
            BlockType::Wood => BiomeMaterialProperties {
                base_roughness: 0.5,
                base_metallic: 0.0,
                base_reflectance: 0.3,
                normal_intensity: 0.8,
                ambient_occlusion: 0.4,
                emissive_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                height_variation: 0.0,
            },
            _ => BiomeMaterialProperties::default(),
        };

        Self {
            block_type,
            base_properties,
            biome_effects: HashMap::new(),
        }
    }
    
    /// Add biome-specific material properties
    #[allow(dead_code)]
    pub fn add_biome_effect(&mut self, biome_type: &str, properties: BiomeMaterialProperties) {
        self.biome_effects.insert(biome_type.to_string(), properties);
    }
    
    /// Get material properties for a specific biome
    pub fn get_properties_for_biome(&self, biome_params: &BiomeTextureParams) -> BiomeMaterialProperties {
        // Check for biome-specific properties first
        if let Some(biome_properties) = self.biome_effects.get(&biome_params.biome_type) {
            return biome_properties.clone();
        }
        
        // Apply biome parameter modifications to base properties
        let mut properties = self.base_properties.clone();
        
        // Temperature effects
        properties.base_roughness = (properties.base_roughness * (1.0 + biome_params.temperature * 0.2 - 0.1)).clamp(0.1, 1.0);
        properties.base_reflectance = (properties.base_reflectance * (1.0 + biome_params.temperature * 0.3)).clamp(0.0, 1.0);
        
        // Moisture effects
        properties.base_roughness = (properties.base_roughness * (1.0 - biome_params.moisture * 0.3)).clamp(0.1, 1.0);
        properties.ambient_occlusion = (properties.ambient_occlusion * (1.0 + biome_params.moisture * 0.4)).clamp(0.1, 1.0);
        
        // Height effects
        properties.normal_intensity = (properties.normal_intensity * (1.0 + biome_params.relative_height * 0.5)).clamp(0.1, 1.0);
        
        properties
    }
}

/// Enhanced biome material cache entry
#[derive(Debug, Clone)]
pub struct BiomeMaterialCacheEntry {
    pub material_handle: Handle<BiomeMaterial>,
    #[allow(dead_code)]
    pub texture_handle: Handle<Image>,
    #[allow(dead_code)]
    pub biome_params: BiomeTextureParams,
    pub last_used: std::time::Instant,
    pub access_count: u32,
}

/// Enhanced biome material cache
#[derive(Resource, Debug, Default)]
pub struct BiomeMaterialCache {
    cache: HashMap<String, BiomeMaterialCacheEntry>,
    lru_queue: VecDeque<String>,
    pub config: BiomeMaterialCacheConfig,
    pub stats: BiomeMaterialCacheStats,
}

#[derive(Debug, Clone, Copy)]
pub struct BiomeMaterialCacheConfig {
    pub max_materials: usize,
    pub max_memory_mb: usize,
    pub enable_lru_eviction: bool,
    pub log_operations: bool,
}

impl Default for BiomeMaterialCacheConfig {
    fn default() -> Self {
        Self {
            max_materials: 512,
            max_memory_mb: 512,
            enable_lru_eviction: true,
            log_operations: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct BiomeMaterialCacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub materials_generated: u64,
    pub materials_evicted: u64,
    pub memory_used_bytes: usize,
    pub current_materials: usize,
}

impl BiomeMaterialCache {
    pub fn new(config: BiomeMaterialCacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            lru_queue: VecDeque::new(),
            config,
            stats: BiomeMaterialCacheStats::default(),
        }
    }
    
    /// Generate a cache key for biome material
    pub fn generate_cache_key(block_type: &BlockType, biome_params: &BiomeTextureParams) -> String {
        format!(
            "{:?}-{}-{}-{}-{}",
            block_type,
            biome_params.biome_type,
            (biome_params.temperature * 10.0).round() as i32,
            (biome_params.moisture * 10.0).round() as i32,
            (biome_params.height * 10.0).round() as i32
        )
    }
    
    /// Get or generate biome material
    pub fn get_or_generate<
        F: FnOnce(&BiomeTextureParams) -> (Handle<Image>, BiomeMaterialProperties),
    >(
        &mut self,
        block_type: &BlockType,
        biome_params: &BiomeTextureParams,
        materials: &mut ResMut<Assets<BiomeMaterial>>,
        _images: &mut ResMut<Assets<Image>>,
        generate_fn: F,
    ) -> Handle<BiomeMaterial> {
        self.stats.total_requests += 1;
        
        let cache_key = Self::generate_cache_key(block_type, biome_params);
        
        // Check cache
        if self.cache.contains_key(&cache_key) {
            self.stats.cache_hits += 1;
            self.update_lru(&cache_key);
            
            if self.config.log_operations {
                println!("📊 BiomeMaterial cache HIT for: {}", cache_key);
            }
            
            // Get the entry after updating LRU to avoid borrow conflict
            return self.cache.get(&cache_key).unwrap().material_handle.clone();
        }
        
        self.stats.cache_misses += 1;
        
        // Generate new material
        if self.config.log_operations {
            println!("🎨 Generating new biome material: {}", cache_key);
        }
        
        let (texture_handle, material_properties) = generate_fn(biome_params);
        
        // Create biome material
        let biome_material = BiomeMaterial {
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            base_color_texture: Some(texture_handle.clone()),
            roughness: material_properties.base_roughness,
            metallic: material_properties.base_metallic,
            reflectance: material_properties.base_reflectance,
            normal_map_intensity: material_properties.normal_intensity,
            ambient_occlusion: material_properties.ambient_occlusion,
            emissive: material_properties.emissive_color,
            height_variation: material_properties.height_variation,
            moisture_effect: biome_params.moisture,
            temperature_effect: biome_params.temperature,
        };
        
        let material_handle = materials.add(biome_material);
        
        // Add to cache
        self.add_to_cache(cache_key.clone(), material_handle.clone(), texture_handle, biome_params.clone());
        
        self.stats.materials_generated += 1;
        
        material_handle
    }
    
    /// Add material to cache
    fn add_to_cache(
        &mut self,
        key: String,
        material_handle: Handle<BiomeMaterial>,
        texture_handle: Handle<Image>,
        biome_params: BiomeTextureParams,
    ) {
        let entry = BiomeMaterialCacheEntry {
            material_handle,
            texture_handle,
            biome_params,
            last_used: std::time::Instant::now(),
            access_count: 1,
        };
        
        self.cache.insert(key.clone(), entry);
        self.lru_queue.push_front(key);
        
        self.stats.current_materials += 1;
        self.stats.memory_used_bytes += 128 * 128 * 4; // Estimate texture size
        
        self.check_and_evict();
    }
    
    /// Update LRU position
    fn update_lru(&mut self, key: &str) {
        if let Some(pos) = self.lru_queue.iter().position(|k| k == key) {
            self.lru_queue.remove(pos);
        }
        self.lru_queue.push_front(key.to_string());
        
        if let Some(entry) = self.cache.get_mut(key) {
            entry.access_count += 1;
            entry.last_used = std::time::Instant::now();
        }
    }
    
    /// Check and evict materials if needed
    fn check_and_evict(&mut self) {
        if !self.config.enable_lru_eviction {
            return;
        }
        
        // Check material count limit
        while self.stats.current_materials > self.config.max_materials && !self.lru_queue.is_empty() {
            self.evict_lru_material();
        }
        
        // Check memory limit
        let max_memory_bytes = self.config.max_memory_mb * 1024 * 1024;
        while self.stats.memory_used_bytes > max_memory_bytes && !self.lru_queue.is_empty() {
            self.evict_lru_material();
        }
    }
    
    /// Evict least recently used material
    fn evict_lru_material(&mut self) {
        if let Some(key) = self.lru_queue.pop_back() {
            if let Some(_entry) = self.cache.remove(&key) {
                self.stats.current_materials -= 1;
                self.stats.materials_evicted += 1;
                self.stats.memory_used_bytes -= 128 * 128 * 4;
                
                if self.config.log_operations {
                    println!("🗑️  Evicted biome material: {}", key);
                }
            }
        }
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> &BiomeMaterialCacheStats {
        &self.stats
    }
    
    /// Print cache statistics
    #[allow(dead_code)]
    pub fn print_stats(&self) {
        println!("📈 Biome Material Cache Statistics:");
        println!("  Total Requests: {}", self.stats.total_requests);
        println!("  Cache Hits: {} ({:.1}%)", 
            self.stats.cache_hits,
            (self.stats.cache_hits as f64 / self.stats.total_requests as f64 * 100.0));
        println!("  Cache Misses: {} ({:.1}%)", 
            self.stats.cache_misses,
            (self.stats.cache_misses as f64 / self.stats.total_requests as f64 * 100.0));
        println!("  Materials Generated: {}", self.stats.materials_generated);
        println!("  Materials Evicted: {}", self.stats.materials_evicted);
        println!("  Current Materials: {}", self.stats.current_materials);
        println!("  Memory Used: {:.2} MB", self.stats.memory_used_bytes as f64 / 1024.0 / 1024.0);
    }
    
    /// Clear cache
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_queue.clear();
        self.stats.current_materials = 0;
        self.stats.memory_used_bytes = 0;
        
        if self.config.log_operations {
            println!("🧹 Cleared biome material cache");
        }
    }
}

/// Thread-safe wrapper for biome material cache
#[derive(Resource, Default)]
pub struct SharedBiomeMaterialCache {
    pub cache: std::sync::Arc<std::sync::Mutex<BiomeMaterialCache>>,
}

impl SharedBiomeMaterialCache {
    pub fn new(config: BiomeMaterialCacheConfig) -> Self {
        Self {
            cache: std::sync::Arc::new(std::sync::Mutex::new(BiomeMaterialCache::new(config))),
        }
    }
}