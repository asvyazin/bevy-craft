// Alkyd Buffer Management Module
// This module provides buffer management for Alkyd GPU compute shader integration

use bevy::prelude::*;
use bevy::render::render_resource::BufferUsages;
use std::collections::HashMap;

// Placeholder for GPU buffer data
#[derive(Debug, Clone)]
struct GpuBufferData {
    data: Vec<u8>,
    usage: BufferUsages,
}

/// Resource for managing Alkyd GPU buffers
#[derive(Resource, Debug, Default)]
pub struct AlkydBufferManager {
    pub texture_buffers: HashMap<String, GpuBufferData>, // Map block type to GPU buffer data
    pub config_buffers: HashMap<String, GpuBufferData>,  // Map block type to config buffer data
    pub buffer_sizes: HashMap<String, u64>,       // Track buffer sizes for each block type
    pub gpu_memory_usage: u64,                    // Total GPU memory usage in bytes
    pub max_memory_usage: u64,                   // Maximum allowed GPU memory usage
}

impl AlkydBufferManager {
    pub fn new(max_memory_mb: u64) -> Self {
        Self {
            texture_buffers: HashMap::new(),
            config_buffers: HashMap::new(),
            buffer_sizes: HashMap::new(),
            gpu_memory_usage: 0,
            max_memory_usage: max_memory_mb * 1024 * 1024, // Convert MB to bytes
        }
    }
    
    /// Create a new GPU buffer for texture data
    pub fn create_texture_buffer(&mut self, block_type: &str, texture_data: &[u8]) -> Option<GpuBufferData> {
        let buffer_size = texture_data.len() as u64;
        
        // Check if we have enough memory
        if self.gpu_memory_usage + buffer_size > self.max_memory_usage {
            println!("⚠️  GPU memory limit reached, cannot create buffer for {}", block_type);
            return None;
        }
        
        // Create GPU buffer data (placeholder for actual GPU buffer)
        let buffer_data = GpuBufferData {
            data: texture_data.to_vec(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        };
        
        // Store buffer and update memory usage
        self.texture_buffers.insert(block_type.to_string(), buffer_data.clone());
        self.buffer_sizes.insert(block_type.to_string(), buffer_size);
        self.gpu_memory_usage += buffer_size;
        
        println!("✓ Created GPU texture buffer for {}: {} bytes", block_type, buffer_size);
        
        Some(buffer_data)
    }
    
    /// Create a new GPU buffer for configuration data
    pub fn create_config_buffer(&mut self, block_type: &str, config_data: &[u8]) -> Option<GpuBufferData> {
        let buffer_size = config_data.len() as u64;
        
        // Check if we have enough memory
        if self.gpu_memory_usage + buffer_size > self.max_memory_usage {
            println!("⚠️  GPU memory limit reached, cannot create config buffer for {}", block_type);
            return None;
        }
        
        // Create GPU buffer data (placeholder for actual GPU buffer)
        let buffer_data = GpuBufferData {
            data: config_data.to_vec(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        };
        
        // Store buffer and update memory usage
        self.config_buffers.insert(block_type.to_string(), buffer_data.clone());
        self.buffer_sizes.insert(block_type.to_string(), buffer_size);
        self.gpu_memory_usage += buffer_size;
        
        println!("✓ Created GPU config buffer for {}: {} bytes", block_type, buffer_size);
        
        Some(buffer_data)
    }
    
    /// Get a texture buffer by block type
    pub fn get_texture_buffer(&self, block_type: &str) -> Option<&GpuBufferData> {
        self.texture_buffers.get(block_type)
    }
    
    /// Get a config buffer by block type
    pub fn get_config_buffer(&self, block_type: &str) -> Option<&GpuBufferData> {
        self.config_buffers.get(block_type)
    }
    
    /// Update an existing texture buffer
    pub fn update_texture_buffer(&mut self, block_type: &str, new_data: &[u8]) -> bool {
        if let Some(existing_buffer) = self.texture_buffers.get(block_type) {
            let old_size = self.buffer_sizes.get(block_type).copied().unwrap_or(0);
            let new_size = new_data.len() as u64;
            
            // Update buffer data (in a real implementation, this would use GPU buffer mapping)
            let new_buffer_data = GpuBufferData {
                data: new_data.to_vec(),
                usage: existing_buffer.usage,
            };
            
            self.texture_buffers.insert(block_type.to_string(), new_buffer_data);
            self.buffer_sizes.insert(block_type.to_string(), new_size);
            self.gpu_memory_usage = self.gpu_memory_usage - old_size + new_size;
            
            println!("✓ Updated GPU texture buffer for {}: {} bytes", block_type, new_size);
            true
        } else {
            println!("⚠️  Texture buffer for {} not found", block_type);
            false
        }
    }
    
    /// Clean up buffers for a specific block type
    pub fn cleanup_block_buffers(&mut self, block_type: &str) {
        if let Some(buffer_size) = self.buffer_sizes.remove(block_type) {
            self.texture_buffers.remove(block_type);
            self.config_buffers.remove(block_type);
            self.gpu_memory_usage -= buffer_size;
            println!("✓ Cleaned up GPU buffers for {}: freed {} bytes", block_type, buffer_size);
        }
    }
    
    /// Get current GPU memory usage statistics
    pub fn get_memory_stats(&self) -> AlkydMemoryStats {
        AlkydMemoryStats {
            current_usage: self.gpu_memory_usage,
            max_usage: self.max_memory_usage,
            used_percentage: (self.gpu_memory_usage as f32 / self.max_memory_usage as f32 * 100.0),
            buffer_count: self.texture_buffers.len() as u32,
        }
    }
    
    /// Clean up all buffers (called on shutdown)
    pub fn cleanup_all_buffers(&mut self) {
        let texture_count = self.texture_buffers.len();
        let config_count = self.config_buffers.len();
        
        self.texture_buffers.clear();
        self.config_buffers.clear();
        self.buffer_sizes.clear();
        self.gpu_memory_usage = 0;
        
        println!("✓ Cleaned up all Alkyd GPU buffers: {} texture buffers, {} config buffers", 
                 texture_count, config_count);
    }
}

/// GPU memory usage statistics
#[derive(Debug, Clone)]
pub struct AlkydMemoryStats {
    pub current_usage: u64,      // Current GPU memory usage in bytes
    pub max_usage: u64,         // Maximum allowed GPU memory usage in bytes
    pub used_percentage: f32,   // Percentage of memory used (0-100)
    pub buffer_count: u32,      // Number of active buffers
}

/// System to initialize Alkyd buffer manager
pub fn initialize_alkyd_buffer_manager(
    mut commands: Commands,
) {
    println!("🔧 Initializing Alkyd buffer manager...");
    
    // Create buffer manager with reasonable memory limits
    // 256MB should be sufficient for texture generation
    let buffer_manager = AlkydBufferManager::new(256); // 256MB limit
    
    commands.insert_resource(buffer_manager);
    
    println!("✓ Alkyd buffer manager initialized with 256MB memory limit");
}

/// System to monitor and log Alkyd GPU memory usage
pub fn monitor_alkyd_memory_usage(
    buffer_manager: Res<AlkydBufferManager>,
    time: Res<Time>,
) {
    // Only log every 10 seconds to avoid spam
    if time.elapsed_secs() % 10.0 < 0.1 {
        let stats = buffer_manager.get_memory_stats();
        
        // Log memory usage periodically (every 10 seconds)
        println!("📊 Alkyd GPU Memory Usage:");
        println!("   - Current: {:.2} MB / {:.2} MB ({:.1}%)",
                 stats.current_usage as f32 / 1024.0 / 1024.0,
                 stats.max_usage as f32 / 1024.0 / 1024.0,
                 stats.used_percentage);
        println!("   - Active buffers: {}", stats.buffer_count);
        
        // Warn if memory usage is high
        if stats.used_percentage > 80.0 {
            println!("⚠️  High GPU memory usage! Consider optimizing texture generation.");
        }
    }
}

/// System to create GPU buffers for texture generation
pub fn create_alkyd_gpu_buffers(
    mut buffer_manager: ResMut<AlkydBufferManager>,
    enhanced_textures: Res<crate::alkyd_integration::EnhancedBlockTextures>,
) {
    println!("🎨 Creating Alkyd GPU buffers for texture generation...");
    
    // Create buffers for all available textures
    for (block_type, _) in &enhanced_textures.textures {
        // In a real implementation, we would get the actual texture data from the GPU
        // For now, we'll create a placeholder buffer with the expected size
        
        // Calculate expected texture size (128x128 RGBA = 128*128*4 bytes)
        let texture_size = 128 * 128 * 4;
        let placeholder_data = vec![0u8; texture_size];
        
        // Create texture buffer
        if let Some(_texture_buffer) = buffer_manager.create_texture_buffer(block_type, &placeholder_data) {
            // Create config buffer (placeholder for configuration data)
            let config_data = vec![0u8; 128]; // 128 bytes for config
            buffer_manager.create_config_buffer(block_type, &config_data);
            
            println!("✓ Created GPU buffers for {} texture", block_type);
        }
    }
    
    let stats = buffer_manager.get_memory_stats();
    println!("✓ Alkyd GPU buffers created: {} MB used, {} buffers",
             stats.current_usage as f32 / 1024.0 / 1024.0,
             stats.buffer_count);
}

/// System to cleanup Alkyd GPU buffers on shutdown
pub fn cleanup_alkyd_buffers(
    mut buffer_manager: ResMut<AlkydBufferManager>,
) {
    println!("🧹 Cleaning up Alkyd GPU buffers...");
    buffer_manager.cleanup_all_buffers();
    println!("✓ Alkyd GPU buffers cleaned up");
}

/// System to setup Alkyd buffer management in the app
pub fn setup_alkyd_buffer_management(app: &mut App) {
    println!("🔧 Setting up Alkyd buffer management...");
    app
        .init_resource::<AlkydBufferManager>()
        .add_systems(Startup, initialize_alkyd_buffer_manager)
        .add_systems(Startup, create_alkyd_gpu_buffers.after(initialize_alkyd_buffer_manager))
        .add_systems(Update, monitor_alkyd_memory_usage)
        .add_systems(Last, cleanup_alkyd_buffers);
}

/// Component to mark entities that use Alkyd GPU buffers
#[derive(Component, Debug)]
pub struct AlkydGpuBuffered {
    pub block_type: String,
    pub uses_gpu_buffer: bool,
}

impl AlkydGpuBuffered {
    pub fn new(block_type: &str) -> Self {
        Self {
            block_type: block_type.to_string(),
            uses_gpu_buffer: true,
        }
    }
}

/// System to apply Alkyd GPU buffers to entities
pub fn apply_alkyd_gpu_buffers(
    mut commands: Commands,
    buffer_manager: Res<AlkydBufferManager>,
    query: Query<(Entity, &AlkydGpuBuffered), Added<AlkydGpuBuffered>>,
) {
    for (entity, gpu_buffered) in &query {
        println!("🎨 Applying Alkyd GPU buffer to entity for {}", gpu_buffered.block_type);
        
        // Check if we have a GPU buffer for this block type
        if let Some(_texture_buffer) = buffer_manager.get_texture_buffer(&gpu_buffered.block_type) {
            println!("✓ Applied GPU buffer for {} to entity {:?}", 
                     gpu_buffered.block_type, entity);
            
            // In a real implementation, we would bind the GPU buffer to the entity
            // For now, we just mark it as using GPU acceleration
            commands.entity(entity).insert(crate::alkyd_gpu_shaders::AlkydGpuTexture {
                block_type: gpu_buffered.block_type.clone(),
                config: crate::alkyd_gpu_shaders::AlkydGpuTextureConfig::for_block_type(&gpu_buffered.block_type),
            });
        } else {
            println!("⚠️  No GPU buffer available for {}, using fallback", gpu_buffered.block_type);
            
            // Fallback to regular texture generation
            commands.entity(entity).insert(crate::alkyd_integration::AlkydTexture {
                block_type: gpu_buffered.block_type.clone(),
                config: crate::alkyd_integration::AlkydTextureConfig::for_block_type(&gpu_buffered.block_type),
            });
        }
    }
}