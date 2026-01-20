// Enhanced Biome Texture Caching System
// This module provides LRU-based caching for biome textures with efficient reuse

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::{HashMap, VecDeque};

use std::sync::{Arc, Mutex};

use crate::block::BlockType;
use crate::biome_textures::BiomeTextureParams;

/// LRU Cache entry for biome textures
#[derive(Debug, Clone)]
pub struct BiomeTextureCacheEntry {
    pub texture_handle: Handle<Image>,
    pub config: crate::noise::NoiseSettings,
    pub last_used: std::time::Instant,
    pub access_count: u32,
    pub size_bytes: usize,  // Estimated memory size
}

/// Enhanced biome texture cache with LRU eviction
#[derive(Resource, Debug, Default)]
pub struct BiomeTextureCache {
    /// Main cache storage
    cache: HashMap<String, BiomeTextureCacheEntry>,
    /// LRU queue - most recently used at front, least recently used at back
    lru_queue: VecDeque<String>,
    /// Configuration for cache behavior
    pub config: BiomeTextureCacheConfig,
    /// Cache statistics
    pub stats: BiomeTextureCacheStats,
}

/// Configuration for biome texture cache
#[derive(Debug, Clone)]
pub struct BiomeTextureCacheConfig {
    pub max_textures: usize,
    pub max_memory_mb: usize,
    pub enable_lru_eviction: bool,
    pub enable_similarity_reuse: bool,
    pub similarity_threshold: f32,
    pub log_cache_operations: bool,
}

impl Default for BiomeTextureCacheConfig {
    fn default() -> Self {
        Self {
            max_textures: 1024,  // Increased for better coverage
            max_memory_mb: 1024, // Increased memory limit
            enable_lru_eviction: true,
            enable_similarity_reuse: true,
            similarity_threshold: 0.85,  // Lowered threshold for better matching
            log_cache_operations: false, // Reduced logging for production
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Default)]
pub struct BiomeTextureCacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub textures_generated: u64,
    pub textures_evicted: u64,
    pub memory_used_bytes: usize,
    pub peak_memory_used_bytes: usize,
    pub current_textures: usize,
    pub peak_textures: usize,
}

impl BiomeTextureCache {
    /// Create a new biome texture cache
    pub fn new(config: BiomeTextureCacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            lru_queue: VecDeque::new(),
            config,
            stats: BiomeTextureCacheStats::default(),
        }
    }

    /// Get a texture from cache or generate if missing
    pub fn get_or_generate<
        F: FnOnce(&BiomeTextureParams) -> (Handle<Image>, crate::noise::NoiseSettings),
    >(
        &mut self,
        block_type: &BlockType,
        biome_params: &BiomeTextureParams,
        images: &mut ResMut<Assets<Image>>,
        generate_fn: F,
    ) -> Handle<Image> {
        self.stats.total_requests += 1;
        
        let texture_key = crate::biome_textures::generate_texture_cache_key(block_type, biome_params);
        
        // Check if texture exists in cache
        if self.cache.contains_key(&texture_key) {
            self.stats.cache_hits += 1;
            
            // Update LRU - move to front
            self.update_lru(&texture_key);
            
            if self.config.log_cache_operations {
                println!("📊 Cache HIT for biome texture: {}", texture_key);
            }
            
            // Get the entry after updating LRU to avoid borrow conflict
            return self.cache.get(&texture_key).unwrap().texture_handle.clone();
        }
        
        self.stats.cache_misses += 1;
        
        // Check for similar textures that can be reused
        if self.config.enable_similarity_reuse {
            if let Some(similar_texture) = self.find_similar_texture(block_type, biome_params) {
                self.stats.cache_hits += 1;  // Count similarity match as cache hit
                self.stats.cache_misses -= 1; // Adjust cache miss count
                
                // Reduce logging spam - only log similarity reuse in debug mode
                if self.config.log_cache_operations && self.config.enable_lru_eviction {
                    // Debug logging disabled for production performance
                    // println!("🔄 Reusing similar biome texture for: {}", texture_key);
                }
                
                // Update LRU for the similar texture
                self.update_lru(&similar_texture);
                
                return self.cache.get(&similar_texture).unwrap().texture_handle.clone();
            }
        }
        
        // Generate new texture using biome-aware generation
        if self.config.log_cache_operations {
            println!("🎨 Generating new biome texture: {}", texture_key);
        }
        
        // Convert block type to string for texture generation
        let block_type_str = match block_type {
            BlockType::Stone => "stone",
            BlockType::Dirt => "dirt",
            BlockType::Grass => "grass",
            BlockType::Wood => "wood",
            BlockType::Leaves => "leaves",
            BlockType::Sand => "sand",
            BlockType::Water => "water",
            BlockType::Bedrock => "bedrock",
            _ => "stone", // Default to stone for unknown types
        };
        
        // Generate biome-aware texture data
        let texture_data = crate::texture_gen::generate_biome_texture_data(
            128, 128, biome_params, block_type_str
        );
        
        // Create image from generated texture data
        let image = Image::new(
            Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        
        let texture_handle = images.add(image);
        
        // Estimate texture size (128x128 RGBA = 128*128*4 bytes)
        let texture_size = 128 * 128 * 4;
        
        // Create a default noise config for this biome
        let biome_config = crate::biome_textures::BiomeTextureConfig::for_block_type(block_type);
        let noise_config = crate::biome_textures::apply_biome_parameters_to_config(
            &biome_config.base_config, biome_params, &biome_config
        );
        
        // Add to cache
        self.add_to_cache(texture_key.clone(), texture_handle.clone(), noise_config, texture_size);
        
        self.stats.textures_generated += 1;
        
        texture_handle
    }

    /// Add a texture to cache
    fn add_to_cache(
        &mut self,
        key: String,
        texture_handle: Handle<Image>,
        config: crate::noise::NoiseSettings,
        size_bytes: usize,
    ) {
        let entry = BiomeTextureCacheEntry {
            texture_handle,
            config,
            last_used: std::time::Instant::now(),
            access_count: 1,
            size_bytes,
        };
        
        // Add to cache and LRU queue
        self.cache.insert(key.clone(), entry);
        self.lru_queue.push_front(key);
        
        // Update statistics
        self.stats.memory_used_bytes += size_bytes;
        self.stats.peak_memory_used_bytes = self.stats.peak_memory_used_bytes.max(self.stats.memory_used_bytes);
        self.stats.current_textures += 1;
        self.stats.peak_textures = self.stats.peak_textures.max(self.stats.current_textures);
        
        // Check if we need to evict textures
        self.check_and_evict();
    }

    /// Update LRU position for a key
    fn update_lru(&mut self, key: &str) {
        // Remove from current position
        if let Some(pos) = self.lru_queue.iter().position(|k| k == key) {
            self.lru_queue.remove(pos);
        }
        
        // Add to front (most recently used)
        self.lru_queue.push_front(key.to_string());
        
        // Update access count
        if let Some(entry) = self.cache.get_mut(key) {
            entry.access_count += 1;
            entry.last_used = std::time::Instant::now();
        }
    }

    /// Find similar texture that can be reused
    fn find_similar_texture(
        &self,
        block_type: &BlockType,
        biome_params: &BiomeTextureParams,
    ) -> Option<String> {
        let target_key = crate::biome_textures::generate_texture_cache_key(block_type, biome_params);
        
        // Look for textures with same block type and similar biome parameters
        for (key, entry) in &self.cache {
            if !key.starts_with(&format!("{:?}-", block_type)) {
                continue;
            }
            
            // Parse the key to extract biome parameters
            if let Some(similarity) = self.calculate_texture_similarity(key, &target_key) {
                if similarity >= self.config.similarity_threshold {
                    return Some(key.clone());
                }
            }
        }
        
        None
    }

    /// Calculate similarity between two texture keys
    fn calculate_texture_similarity(&self, key1: &str, key2: &str) -> Option<f32> {
        // Parse biome parameters from keys
        let parts1: Vec<&str> = key1.split('-').collect();
        let parts2: Vec<&str> = key2.split('-').collect();
        
        if parts1.len() != 5 || parts2.len() != 5 {
            return None;
        }
        
        // Extract parameters
        let temp1: f32 = parts1[1].replace("temp", "").parse().ok()?;
        let moist1: f32 = parts1[2].replace("moist", "").parse().ok()?;
        let biome1 = parts1[4].replace("biome", "");
        
        let temp2: f32 = parts2[1].replace("temp", "").parse().ok()?;
        let moist2: f32 = parts2[2].replace("moist", "").parse().ok()?;
        let biome2 = parts2[4].replace("biome", "");
        
        // Calculate similarity score
        let temp_diff = (temp1 - temp2).abs();
        let moist_diff = (moist1 - moist2).abs();
        let biome_match = if biome1 == biome2 { 1.0 } else { 0.0 };
        
        // Weighted similarity score
        let similarity = 1.0 - (temp_diff * 0.4 + moist_diff * 0.4 + (1.0 - biome_match) * 0.2);
        
        Some(similarity)
    }

    /// Check if cache limits are exceeded and evict textures if needed
    fn check_and_evict(&mut self) {
        if !self.config.enable_lru_eviction {
            return;
        }
        
        // Check texture count limit
        while self.stats.current_textures > self.config.max_textures && !self.lru_queue.is_empty() {
            self.evict_lru_texture();
        }
        
        // Check memory limit (convert MB to bytes)
        let max_memory_bytes = self.config.max_memory_mb * 1024 * 1024;
        while self.stats.memory_used_bytes > max_memory_bytes && !self.lru_queue.is_empty() {
            self.evict_lru_texture();
        }
    }

    /// Evict the least recently used texture
    fn evict_lru_texture(&mut self) {
        if let Some(key) = self.lru_queue.pop_back() {
            if let Some(entry) = self.cache.remove(&key) {
                self.stats.memory_used_bytes -= entry.size_bytes;
                self.stats.current_textures -= 1;
                self.stats.textures_evicted += 1;
                
                if self.config.log_cache_operations {
                    println!("🗑️  Evicted LRU biome texture: {}", key);
                }
            }
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> &BiomeTextureCacheStats {
        &self.stats
    }

    /// Print cache statistics
    pub fn print_stats(&self) {
        println!("📈 Biome Texture Cache Statistics:");
        println!("  Total Requests: {}", self.stats.total_requests);
        println!("  Cache Hits: {} ({:.1}%)", 
            self.stats.cache_hits,
            (self.stats.cache_hits as f64 / self.stats.total_requests as f64 * 100.0));
        println!("  Cache Misses: {} ({:.1}%)", 
            self.stats.cache_misses,
            (self.stats.cache_misses as f64 / self.stats.total_requests as f64 * 100.0));
        println!("  Textures Generated: {}", self.stats.textures_generated);
        println!("  Textures Evicted: {}", self.stats.textures_evicted);
        println!("  Current Textures: {}", self.stats.current_textures);
        println!("  Peak Textures: {}", self.stats.peak_textures);
        println!("  Memory Used: {:.2} MB", self.stats.memory_used_bytes as f64 / 1024.0 / 1024.0);
        println!("  Peak Memory: {:.2} MB", self.stats.peak_memory_used_bytes as f64 / 1024.0 / 1024.0);
    }

    /// Get a texture from cache by key (for backward compatibility)
    pub fn get_texture(&self, key: &str) -> Option<&BiomeTextureCacheEntry> {
        self.cache.get(key)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_queue.clear();
        self.stats.memory_used_bytes = 0;
        self.stats.current_textures = 0;
        
        if self.config.log_cache_operations {
            println!("🧹 Cleared biome texture cache");
        }
    }
}

/// Thread-safe wrapper for biome texture cache
#[derive(Resource, Default)]
pub struct SharedBiomeTextureCache {
    pub cache: Arc<Mutex<BiomeTextureCache>>,
}

impl SharedBiomeTextureCache {
    pub fn new(config: BiomeTextureCacheConfig) -> Self {
        Self {
            cache: Arc::new(Mutex::new(BiomeTextureCache::new(config))),
        }
    }
}