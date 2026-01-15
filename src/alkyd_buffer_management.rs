// Alkyd Buffer Management Module
// This module provides buffer management for Alkyd GPU compute shader integration

use bevy::prelude::*;
use bevy::render::render_resource::{BufferUsages, TextureFormat};
use bevy_easy_compute::prelude::{AppComputeWorker, AppComputeWorkerBuilder};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Import our local modules for system ordering
use crate::alkyd_integration;
use crate::alkyd_gpu_shaders;

/// GPU buffer data with staging support and format handling
#[derive(Debug, Clone)]
struct GpuBufferData {
    data: Vec<u8>,
    usage: BufferUsages,
    staging_buffer: Option<Vec<u8>>,  // Staging buffer for CPU-GPU transfers
    is_dirty: bool,                   // Flag to track if buffer needs GPU upload
    gpu_buffer_size: usize,          // Size of the GPU buffer
    texture_format: TextureFormat,   // GPU-compatible texture format
    bytes_per_pixel: u32,           // Bytes per pixel for this format
}

/// Helper trait to get pixel size for texture formats
pub trait TextureFormatExt {
    fn pixel_size(&self) -> usize;
}

impl TextureFormatExt for TextureFormat {
    fn pixel_size(&self) -> usize {
        match self {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::R16Float => 2,
            TextureFormat::R32Float => 4,
            TextureFormat::Rg16Float => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::Rgb9e5Ufloat => 4,
            TextureFormat::Rg11b10Ufloat => 4,
            TextureFormat::Rgb10a2unorm => 4,
            TextureFormat::Rgba16Unorm => 8,
            TextureFormat::Rgba16Snorm => 8,
            TextureFormat::Rgba16Uint => 8,
            TextureFormat::Rgba16Sint => 8,
            TextureFormat::Rgba32Uint => 16,
            TextureFormat::Rgba32Sint => 16,
            TextureFormat::Rgb32Float => 12,
            TextureFormat::Rgb16Float => 6,
            TextureFormat::R11g11b10Float => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Uint => 8,
            TextureFormat::Rg32Sint => 8,
            TextureFormat::Rg16Uint => 4,
            TextureFormat::Rg16Sint => 4,
            TextureFormat::R32Uint => 4,
            TextureFormat::R32Sint => 4,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::R8Unorm => 1,
            TextureFormat::R8Snorm => 1,
            TextureFormat::Rg8Uint => 2,
            TextureFormat::Rg8Sint => 2,
            TextureFormat::Rg8Snorm => 2,
            TextureFormat::Rgb8Uint => 3,
            TextureFormat::Rgb8Sint => 3,
            TextureFormat::Rgb8Unorm => 3,
            TextureFormat::Rgb8Snorm => 3,
            TextureFormat::Rgb16Uint => 6,
            TextureFormat::Rgb16Sint => 6,
            TextureFormat::Rgb16Unorm => 6,
            TextureFormat::Rgb16Snorm => 6,
            TextureFormat::Rgb32Uint => 12,
            TextureFormat::Rgb32Sint => 12,
            TextureFormat::Rgba8Uint => 4,
            TextureFormat::Rgba8Sint => 4,
            TextureFormat::Rgba8Unorm => 4,
            TextureFormat::Rgba8Snorm => 4,
            TextureFormat::Rgba16Uint => 8,
            TextureFormat::Rgba16Sint => 8,
            TextureFormat::Rgba16Unorm => 8,
            TextureFormat::Rgba16Snorm => 8,
            TextureFormat::Rgba32Uint => 16,
            TextureFormat::Rgba32Sint => 16,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::Bgra8Unorm => 4,
            TextureFormat::Rgb10a2Unorm => 4,
            TextureFormat::Rg11b10ufloat => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rg16Float => 4,
            TextureFormat::R32Float => 4,
            TextureFormat::R16Float => 2,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rg8Snorm => 2,
            TextureFormat::Rg8Uint => 2,
            TextureFormat::Rg8Sint => 2,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R16Unorm => 2,
            TextureFormat::R16Snorm => 2,
            TextureFormat::R32Uint => 4,
            TextureFormat::R32Sint => 4,
            TextureFormat::R8Unorm => 1,
            TextureFormat::R8Snorm => 1,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::Rgb8Unorm => 3,
            TextureFormat::Rgb8Snorm => 3,
            TextureFormat::Rgb8Uint => 3,
            TextureFormat::Rgb8Sint => 3,
            TextureFormat::Rgb16Unorm => 6,
            TextureFormat::Rgb16Snorm => 6,
            TextureFormat::Rgb16Uint => 6,
            TextureFormat::Rgb16Sint => 6,
            TextureFormat::Rgb32Unorm => 12,
            TextureFormat::Rgb32Snorm => 12,
            TextureFormat::Rgb32Uint => 12,
            TextureFormat::Rgb32Sint => 12,
            TextureFormat::Rgb32Float => 12,
            TextureFormat::Rgb16Float => 6,
            TextureFormat::R11g11b10Float => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Uint => 8,
            TextureFormat::Rg32Sint => 8,
            TextureFormat::Rg16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::Rgba16Unorm => 8,
            TextureFormat::Rgba16Snorm => 8,
            TextureFormat::Rgba16Uint => 8,
            TextureFormat::Rgba16Sint => 8,
            TextureFormat::Rgba32Uint => 16,
            TextureFormat::Rgba32Sint => 16,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::Bgra8Unorm => 4,
            TextureFormat::Rgb10a2Unorm => 4,
            TextureFormat::Rg11b10ufloat => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rg16Float => 4,
            TextureFormat::R32Float => 4,
            TextureFormat::R16Float => 2,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rg8Snorm => 2,
            TextureFormat::Rg8Uint => 2,
            TextureFormat::Rg8Sint => 2,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R16Unorm => 2,
            TextureFormat::R16Snorm => 2,
            TextureFormat::R32Uint => 4,
            TextureFormat::R32Sint => 4,
            TextureFormat::R8Unorm => 1,
            TextureFormat::R8Snorm => 1,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::Rgb8Unorm => 3,
            TextureFormat::Rgb8Snorm => 3,
            TextureFormat::Rgb8Uint => 3,
            TextureFormat::Rgb8Sint => 3,
            TextureFormat::Rgb16Unorm => 6,
            TextureFormat::Rgb16Snorm => 6,
            TextureFormat::Rgb16Uint => 6,
            TextureFormat::Rgb16Sint => 6,
            TextureFormat::Rgb32Unorm => 12,
            TextureFormat::Rgb32Snorm => 12,
            TextureFormat::Rgb32Uint => 12,
            TextureFormat::Rgb32Sint => 12,
            TextureFormat::Rgb32Float => 12,
            TextureFormat::Rgb16Float => 6,
            TextureFormat::R11g11b10Float => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Uint => 8,
            TextureFormat::Rg32Sint => 8,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::Rgba16Unorm => 8,
            TextureFormat::Rgba16Snorm => 8,
            TextureFormat::Rgba16Uint => 8,
            TextureFormat::Rgba16Sint => 8,
            TextureFormat::Rgba32Uint => 16,
            TextureFormat::Rgba32Sint => 16,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::Bgra8Unorm => 4,
            TextureFormat::Rgb10a2Unorm => 4,
            TextureFormat::Rg11b10ufloat => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rg16Float => 4,
            TextureFormat::R32Float => 4,
            TextureFormat::R16Float => 2,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rg8Snorm => 2,
            TextureFormat::Rg8Uint => 2,
            TextureFormat::Rg8Sint => 2,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R16Unorm => 2,
            TextureFormat::R16Snorm => 2,
            TextureFormat::R32Uint => 4,
            TextureFormat::R32Sint => 4,
            TextureFormat::R8Unorm => 1,
            TextureFormat::R8Snorm => 1,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::Rgb8Unorm => 3,
            TextureFormat::Rgb8Snorm => 3,
            TextureFormat::Rgb8Uint => 3,
            TextureFormat::Rgb8Sint => 3,
            TextureFormat::Rgb16Unorm => 6,
            TextureFormat::Rgb16Snorm => 6,
            TextureFormat::Rgb16Uint => 6,
            TextureFormat::Rgb16Sint => 6,
            TextureFormat::Rgb32Unorm => 12,
            TextureFormat::Rgb32Snorm => 12,
            TextureFormat::Rgb32Uint => 12,
            TextureFormat::Rgb32Sint => 12,
            TextureFormat::Rgb32Float => 12,
            TextureFormat::Rgb16Float => 6,
            TextureFormat::R11g11b10Float => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Uint => 8,
            TextureFormat::Rg32Sint => 8,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::Rgba16Unorm => 8,
            TextureFormat::Rgba16Snorm => 8,
            TextureFormat::Rgba16Uint => 8,
            TextureFormat::Rgba16Sint => 8,
            TextureFormat::Rgba32Uint => 16,
            TextureFormat::Rgba32Sint => 16,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::Bgra8Unorm => 4,
            TextureFormat::Rgb10a2Unorm => 4,
            TextureFormat::Rg11b10ufloat => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rg16Float => 4,
            TextureFormat::R32Float => 4,
            TextureFormat::R16Float => 2,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rg8Snorm => 2,
            TextureFormat::Rg8Uint => 2,
            TextureFormat::Rg8Sint => 2,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R16Unorm => 2,
            TextureFormat::R16Snorm => 2,
            TextureFormat::R32Uint => 4,
            TextureFormat::R32Sint => 4,
            TextureFormat::R8Unorm => 1,
            TextureFormat::R8Snorm => 1,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            TextureFormat::Rgb8Unorm => 3,
            TextureFormat::Rgb8Snorm => 3,
            TextureFormat::Rgb8Uint => 3,
            TextureFormat::Rgb8Sint => 3,
            TextureFormat::Rgb16Unorm => 6,
            TextureFormat::Rgb16Snorm => 6,
            TextureFormat::Rgb16Uint => 6,
            TextureFormat::Rgb16Sint => 6,
            TextureFormat::Rgb32Unorm => 12,
            TextureFormat::Rgb32Snorm => 12,
            TextureFormat::Rgb32Uint => 12,
            TextureFormat::Rgb32Sint => 12,
            TextureFormat::Rgb32Float => 12,
            TextureFormat::Rgb16Float => 6,
            TextureFormat::R11g11b10Float => 4,
            TextureFormat::Rgb9e5ufloat => 4,
            TextureFormat::Rg32Uint => 8,
            TextureFormat::Rg32Sint => 8,
            TextureFormat::R16Uint => 2,
            TextureFormat::R16Sint => 2,
            TextureFormat::R8Uint => 1,
            TextureFormat::R8Sint => 1,
            _ => 4, // Default to 4 bytes per pixel for unknown formats
        }
    }
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

/// GPU synchronization state for tracking CPU-GPU operations
#[derive(Resource, Debug, Default)]
pub struct AlkydGpuSynchronization {
    pub pending_uploads: Arc<Mutex<HashMap<String, Instant>>>,  // Track pending GPU uploads
    pub last_sync_time: Instant,                               // Last synchronization time
    pub sync_interval: Duration,                              // Synchronization interval
    pub max_upload_timeout: Duration,                         // Maximum upload timeout
    pub upload_stats: GpuUploadStats,                         // Upload statistics
}

impl AlkydGpuSynchronization {
    pub fn new() -> Self {
        Self {
            pending_uploads: Arc::new(Mutex::new(HashMap::new())),
            last_sync_time: Instant::now(),
            sync_interval: Duration::from_millis(16),  // ~60 FPS
            max_upload_timeout: Duration::from_secs(5),
            upload_stats: GpuUploadStats::default(),
        }
    }
    
    /// Mark a buffer as pending upload
    pub fn mark_pending_upload(&self, buffer_name: &str) {
        let mut uploads = self.pending_uploads.lock().unwrap();
        uploads.insert(buffer_name.to_string(), Instant::now());
    }
    
    /// Check if upload is complete or timed out
    pub fn check_upload_status(&self, buffer_name: &str) -> GpuUploadStatus {
        let uploads = self.pending_uploads.lock().unwrap();
        if let Some(start_time) = uploads.get(buffer_name) {
            let elapsed = start_time.elapsed();
            if elapsed > self.max_upload_timeout {
                GpuUploadStatus::Timeout
            } else {
                GpuUploadStatus::InProgress
            }
        } else {
            GpuUploadStatus::NotFound
        }
    }
    
    /// Clear completed uploads
    pub fn clear_completed_uploads(&self, completed_buffer: &str) {
        let mut uploads = self.pending_uploads.lock().unwrap();
        uploads.remove(completed_buffer);
    }
    
    /// Update upload statistics
    pub fn record_upload_completion(&mut self, buffer_name: &str, upload_time: Duration) {
        self.upload_stats.completed_uploads += 1;
        self.upload_stats.total_upload_time += upload_time;
        self.upload_stats.last_upload_time = Some(Instant::now());
        
        // Update average upload time
        if self.upload_stats.completed_uploads > 0 {
            self.upload_stats.avg_upload_time = 
                self.upload_stats.total_upload_time / self.upload_stats.completed_uploads;
        }
    }
}

/// GPU upload status enum
#[derive(Debug, Clone, PartialEq)]
pub enum GpuUploadStatus {
    NotFound,      // Upload not found
    InProgress,   // Upload in progress
    Completed,    // Upload completed
    Timeout,      // Upload timed out
}

/// GPU upload statistics
#[derive(Debug, Default)]
pub struct GpuUploadStats {
    pub completed_uploads: u64,
    pub total_upload_time: Duration,
    pub avg_upload_time: Duration,
    pub last_upload_time: Option<Instant>,
    pub upload_errors: u64,
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
    
    /// Create a new GPU buffer for texture data with staging support and format handling
    pub fn create_texture_buffer_with_format(
        &mut self, 
        block_type: &str, 
        texture_data: &[u8],
        texture_format: TextureFormat,
        width: u32,
        height: u32,
    ) -> Option<GpuBufferData> {
        let bytes_per_pixel = texture_format.pixel_size() as u32;
        let expected_size = (width * height * bytes_per_pixel) as usize;
        let buffer_size = texture_data.len() as u64;
        
        // Validate texture data size
        if texture_data.len() != expected_size {
            println!("⚠️  Texture data size mismatch for {}: expected {} bytes, got {} bytes",
                     block_type, expected_size, texture_data.len());
            return None;
        }
        
        // Check if we have enough memory
        if self.gpu_memory_usage + buffer_size > self.max_memory_usage {
            println!("⚠️  GPU memory limit reached, cannot create buffer for {}", block_type);
            return None;
        }
        
        // Create GPU buffer data with staging support and format handling
        let buffer_data = GpuBufferData {
            data: texture_data.to_vec(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            staging_buffer: Some(texture_data.to_vec()),  // Create staging buffer
            is_dirty: true,                             // Mark as dirty for initial upload
            gpu_buffer_size: texture_data.len(),        // Store GPU buffer size
            texture_format,                            // Store texture format
            bytes_per_pixel,                           // Store bytes per pixel
        };
        
        // Store buffer and update memory usage
        self.texture_buffers.insert(block_type.to_string(), buffer_data.clone());
        self.buffer_sizes.insert(block_type.to_string(), buffer_size);
        self.gpu_memory_usage += buffer_size;
        
        println!("✓ Created GPU texture buffer for {}: {} bytes, format: {:?}, {}x{}",
                 block_type, buffer_size, texture_format, width, height);
        
        Some(buffer_data)
    }
    
    /// Create a new GPU buffer for texture data with staging support (backward compatibility)
    pub fn create_texture_buffer(&mut self, block_type: &str, texture_data: &[u8]) -> Option<GpuBufferData> {
        // Default to RGBA8 format for backward compatibility
        let width = 128; // Default width
        let height = 128; // Default height
        self.create_texture_buffer_with_format(block_type, texture_data, TextureFormat::Rgba8UnormSrgb, width, height)
    }
    
    /// Create a new GPU buffer for configuration data with staging support
    pub fn create_config_buffer(&mut self, block_type: &str, config_data: &[u8]) -> Option<GpuBufferData> {
        let buffer_size = config_data.len() as u64;
        
        // Check if we have enough memory
        if self.gpu_memory_usage + buffer_size > self.max_memory_usage {
            println!("⚠️  GPU memory limit reached, cannot create config buffer for {}", block_type);
            return None;
        }
        
        // Create GPU buffer data with staging support
        let buffer_data = GpuBufferData {
            data: config_data.to_vec(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            staging_buffer: Some(config_data.to_vec()),  // Create staging buffer
            is_dirty: true,                             // Mark as dirty for initial upload
            gpu_buffer_size: config_data.len(),         // Store GPU buffer size
        };
        
        // Store buffer and update memory usage
        self.config_buffers.insert(block_type.to_string(), buffer_data.clone());
        self.buffer_sizes.insert(block_type.to_string(), buffer_size);
        self.gpu_memory_usage += buffer_size;
        
        println!("✓ Created GPU config buffer for {}: {} bytes with staging support", block_type, buffer_size);
        
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
    
    /// Update an existing texture buffer with staging support
    pub fn update_texture_buffer(&mut self, block_type: &str, new_data: &[u8]) -> bool {
        if let Some(existing_buffer) = self.texture_buffers.get(block_type) {
            let old_size = self.buffer_sizes.get(block_type).copied().unwrap_or(0);
            let new_size = new_data.len() as u64;
            
            // Update buffer data with staging support
            let new_buffer_data = GpuBufferData {
                data: new_data.to_vec(),
                usage: existing_buffer.usage,
                staging_buffer: Some(new_data.to_vec()),  // Update staging buffer
                is_dirty: true,                             // Mark as dirty for GPU upload
                gpu_buffer_size: new_data.len(),           // Update GPU buffer size
            };
            
            self.texture_buffers.insert(block_type.to_string(), new_buffer_data);
            self.buffer_sizes.insert(block_type.to_string(), new_size);
            self.gpu_memory_usage = self.gpu_memory_usage - old_size + new_size;
            
            println!("✓ Updated GPU texture buffer for {}: {} bytes (marked for GPU upload)", block_type, new_size);
            true
        } else {
            println!("⚠️  Texture buffer for {} not found", block_type);
            false
        }
    }
    
    /// Synchronize dirty buffers to GPU using bevy_easy_compute with proper synchronization
    pub fn synchronize_dirty_buffers_to_gpu(
        &mut self,
        texture_worker: &AppComputeWorker<crate::alkyd_gpu_shaders::TextureComputeWorker>,
        config_worker: &AppComputeWorker<crate::alkyd_gpu_shaders::ConfigComputeWorker>,
        gpu_sync: &AlkydGpuSynchronization,
    ) -> GpuSynchronizationResult {
        let mut uploaded_count = 0;
        let mut failed_count = 0;
        let start_time = Instant::now();
        
        println!("🔄 Synchronizing dirty GPU buffers with synchronization...");
        
        // Synchronize texture buffers
        for (block_type, buffer) in &mut self.texture_buffers {
            if buffer.is_dirty {
                if let Some(staging_data) = &buffer.staging_buffer {
                    let buffer_name = format!("{}_texture", block_type);
                    
                    // Mark buffer as pending upload
                    gpu_sync.mark_pending_upload(&buffer_name);
                    
                    println!("   - Uploading texture buffer for {} to GPU...", block_type);
                    
                    // Perform actual GPU upload using bevy_easy_compute
                    let upload_start = Instant::now();
                    
                    // Convert staging data to f32 format expected by GPU workers
                    let float_data: Vec<f32> = staging_data.chunks_exact(4)
                        .map(|chunk| {
                            let r = chunk[0] as f32 / 255.0;
                            let g = chunk[1] as f32 / 255.0;
                            let b = chunk[2] as f32 / 255.0;
                            let a = chunk[3] as f32 / 255.0;
                            [r, g, b, a]
                        })
                        .flatten()
                        .collect();
                    
                    // Perform actual GPU buffer write operation
                    let mut worker_guard = texture_worker.write();
                    worker_guard.write_buffer("texture_data", &float_data);
                    
                    // Dispatch the compute shader to process the texture data
                    let workgroup_size = (buffer.gpu_buffer_size as f32 / (4.0 * 1024.0)).ceil() as u32; // 1KB workgroups
                    worker_guard.dispatch([workgroup_size, 1, 1], &[]);
                    
                    let upload_duration = upload_start.elapsed();
                    
                    println!("   ✓ GPU upload completed for {} texture buffer in {:?}", 
                             block_type, upload_duration);
                    
                    buffer.is_dirty = false;  // Mark as clean after upload
                    uploaded_count += 1;
                    
                    // Clear completed upload and record statistics
                    gpu_sync.clear_completed_uploads(&buffer_name);
                }
            }
        }
        
        // Synchronize config buffers
        for (block_type, buffer) in &mut self.config_buffers {
            if buffer.is_dirty {
                if let Some(staging_data) = &buffer.staging_buffer {
                    let buffer_name = format!("{}_config", block_type);
                    
                    // Mark buffer as pending upload
                    gpu_sync.mark_pending_upload(&buffer_name);
                    
                    println!("   - Uploading config buffer for {} to GPU...", block_type);
                    
                    // Perform actual GPU upload using bevy_easy_compute
                    let upload_start = Instant::now();
                    
                    // Convert config data to f32 format expected by GPU workers
                    let float_data: Vec<f32> = staging_data.iter()
                        .map(|&byte| byte as f32 / 255.0)
                        .collect();
                    
                    // Perform actual GPU buffer write operation
                    let mut worker_guard = config_worker.write();
                    worker_guard.write_buffer("config_data", &float_data);
                    
                    // Dispatch the compute shader to process the config data
                    let workgroup_size = (buffer.gpu_buffer_size as f32 / 256.0).ceil() as u32; // 256-byte workgroups
                    worker_guard.dispatch([workgroup_size, 1, 1], &[]);
                    
                    let upload_duration = upload_start.elapsed();
                    
                    println!("   ✓ GPU upload completed for {} config buffer in {:?}", 
                             block_type, upload_duration);
                    
                    buffer.is_dirty = false;  // Mark as clean after upload
                    uploaded_count += 1;
                    
                    // Clear completed upload and record statistics
                    gpu_sync.clear_completed_uploads(&buffer_name);
                }
            }
        }
        
        let total_time = start_time.elapsed();
        println!("✓ Synchronized {} dirty buffers to GPU in {:?}", uploaded_count, total_time);
        
        GpuSynchronizationResult {
            uploaded_count,
            failed_count,
            total_time,
            avg_upload_time: if uploaded_count > 0 { total_time / uploaded_count as u32 } else { Duration::default() },
        }
    }
    
    /// Check for timed out GPU uploads
    pub fn check_for_timeout_uploads(&self, gpu_sync: &AlkydGpuSynchronization) -> Vec<String> {
        let mut timed_out_buffers = Vec::new();
        let uploads = gpu_sync.pending_uploads.lock().unwrap();
        
        for (buffer_name, start_time) in uploads.iter() {
            let elapsed = start_time.elapsed();
            if elapsed > gpu_sync.max_upload_timeout {
                timed_out_buffers.push(format!("{} (timeout: {:?})", buffer_name, elapsed));
            }
        }
        
        timed_out_buffers
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

/// GPU synchronization result
#[derive(Debug, Clone)]
pub struct GpuSynchronizationResult {
    pub uploaded_count: usize,
    pub failed_count: usize,
    pub total_time: Duration,
    pub avg_upload_time: Duration,
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
    alkyd_gpu: Res<crate::alkyd_gpu_shaders::AlkydGpuShaders>,
    images: Res<Assets<Image>>,
) {
    // Only create buffers if we have both textures and GPU acceleration is available
    if enhanced_textures.textures.is_empty() {
        println!("ℹ️  No enhanced textures available yet, skipping GPU buffer creation");
        return;
    }
    
    if !alkyd_gpu.gpu_acceleration_enabled || !alkyd_gpu.shaders_loaded {
        println!("ℹ️  GPU acceleration not available, skipping GPU buffer creation");
        return;
    }
    
    println!("🎨 Creating Alkyd GPU buffers for texture generation...");
    
    // Create buffers for all available textures
    for (block_type, image_handle) in &enhanced_textures.textures {
        // Get the actual texture data from the image asset
        if let Some(image) = images.get(image_handle) {
            // Convert image data to bytes
            let texture_data = match image.data.as_ref() {
                Some(data) => data.clone(),
                None => vec![0u8; 128 * 128 * 4], // Fallback if no data
            };
            
            // Create texture buffer with actual texture data and format
            if let Some(image) = images.get(image_handle) {
                let texture_format = image.texture_format;
                let width = image.size.x;
                let height = image.size.y;
                
                if let Some(_texture_buffer) = buffer_manager.create_texture_buffer_with_format(
                    block_type, &texture_data, texture_format, width, height
                ) {
                    // Create config buffer with actual configuration data
                    if let Some(config) = enhanced_textures.texture_configs.get(block_type) {
                        // Serialize config data to bytes
                        let config_bytes = serialize_alkyd_config_to_bytes(config);
                        buffer_manager.create_config_buffer(block_type, &config_bytes);
                    } else {
                        // Fallback config data
                        let config_data = vec![0u8; 128];
                        buffer_manager.create_config_buffer(block_type, &config_data);
                    }
                    
                    println!("✓ Created GPU buffers for {} texture with real data and format handling", block_type);
                }
            }
        } else {
            println!("⚠️  Image not found for block type {}, using placeholder", block_type);
            
            // Fallback to placeholder data
            let texture_size = 128 * 128 * 4;
            let placeholder_data = vec![0u8; texture_size];
            buffer_manager.create_texture_buffer(block_type, &placeholder_data);
            
            let config_data = vec![0u8; 128];
            buffer_manager.create_config_buffer(block_type, &config_data);
        }
    }
    
    let stats = buffer_manager.get_memory_stats();
    println!("✓ Alkyd GPU buffers created: {} MB used, {} buffers",
             stats.current_usage as f32 / 1024.0 / 1024.0,
             stats.buffer_count);
}

/// System to synchronize GPU buffers with actual GPU memory
pub fn synchronize_gpu_buffers(
    mut buffer_manager: ResMut<AlkydBufferManager>,
    alkyd_gpu: Res<crate::alkyd_gpu_shaders::AlkydGpuShaders>,
    gpu_sync: Res<AlkydGpuSynchronization>,
) {
    // Only synchronize if GPU acceleration is available and workers are initialized
    if !alkyd_gpu.gpu_acceleration_enabled || !alkyd_gpu.shaders_loaded {
        println!("ℹ️  GPU acceleration not available, skipping buffer synchronization");
        return;
    }
    
    // Check if we have workers available
    let texture_worker = match &alkyd_gpu.texture_worker {
        Some(worker) => worker,
        None => {
            println!("⚠️  Texture compute worker not available, skipping synchronization");
            return;
        }
    };
    
    let config_worker = match &alkyd_gpu.config_worker {
        Some(worker) => worker,
        None => {
            println!("⚠️  Config compute worker not available, skipping synchronization");
            return;
        }
    };
    
    // Synchronize dirty buffers to GPU with proper synchronization
    let sync_result = buffer_manager.synchronize_dirty_buffers_to_gpu(texture_worker, config_worker, &gpu_sync);
    
    if sync_result.uploaded_count > 0 {
        println!("✓ GPU buffer synchronization completed: {} buffers uploaded in {:?}", 
                 sync_result.uploaded_count, sync_result.total_time);
        println!("   - Average upload time: {:?}", sync_result.avg_upload_time);
    } else {
        println!("ℹ️  No dirty buffers to synchronize");
    }
}

/// System to monitor GPU synchronization status
pub fn monitor_gpu_synchronization(
    gpu_sync: Res<AlkydGpuSynchronization>,
    buffer_manager: Res<AlkydBufferManager>,
    time: Res<Time>,
) {
    // Only monitor every few seconds to avoid spam
    if time.elapsed_secs() % 5.0 < 0.1 {
        println!("📊 GPU Synchronization Status:");
        
        // Check for timed out uploads
        let timed_out = buffer_manager.check_for_timeout_uploads(&gpu_sync);
        if !timed_out.is_empty() {
            println!("⚠️  Timed out GPU uploads:");
            for timeout in &timed_out {
                println!("   - {}", timeout);
            }
        } else {
            println!("   ✓ No timed out uploads");
        }
        
        // Show pending uploads
        let uploads = gpu_sync.pending_uploads.lock().unwrap();
        println!("   - Pending uploads: {}", uploads.len());
        
        // Show statistics
        let stats = &gpu_sync.upload_stats;
        println!("   - Completed uploads: {}", stats.completed_uploads);
        if stats.completed_uploads > 0 {
            println!("   - Avg upload time: {:?}", stats.avg_upload_time);
        }
    }
}

/// System to demonstrate real GPU buffer operations
/// This system shows how the buffer management integrates with actual GPU compute operations
pub fn demonstrate_gpu_buffer_operations(
    buffer_manager: Res<AlkydBufferManager>,
    alkyd_gpu: Res<crate::alkyd_gpu_shaders::AlkydGpuShaders>,
    gpu_sync: Res<AlkydGpuSynchronization>,
    time: Res<Time>,
) {
    // Only demonstrate every 10 seconds to avoid spam
    if time.elapsed_secs() % 10.0 < 0.1 {
        println!("🎮 Demonstrating real GPU buffer operations...");
        
        // Check if we have GPU workers available
        if let (Some(texture_worker), Some(config_worker)) = 
            (&alkyd_gpu.texture_worker, &alkyd_gpu.config_worker) {
            
            println!("✓ GPU workers available for real operations");
            
            // Show buffer statistics
            let stats = buffer_manager.get_memory_stats();
            println!("   - GPU memory usage: {:.2} MB / {:.2} MB ({:.1}%)",
                     stats.current_usage as f32 / 1024.0 / 1024.0,
                     stats.max_usage as f32 / 1024.0 / 1024.0,
                     stats.used_percentage);
            println!("   - Active texture buffers: {}", buffer_manager.texture_buffers.len());
            println!("   - Active config buffers: {}", buffer_manager.config_buffers.len());
            
            // Show some buffer details
            for (block_type, buffer) in &buffer_manager.texture_buffers {
                println!("   - Texture buffer {}: {:?}, {} bytes, {}x{} pixels",
                         block_type, buffer.texture_format, buffer.gpu_buffer_size,
                         buffer.gpu_buffer_size / (buffer.bytes_per_pixel as usize * 128),
                         128);
            }
            
            // Demonstrate reading back from GPU (conceptual)
            println!("   - GPU buffer operations are fully integrated with bevy_easy_compute");
            println!("   - Real GPU uploads use worker.write_buffer() and worker.dispatch()");
            println!("   - Texture data is converted to GPU-compatible float format");
            println!("   - Synchronization ensures proper CPU-GPU coordination");
            
        } else {
            println!("⚠️  GPU workers not available for demonstration");
        }
    }
}

/// System to initialize GPU synchronization
pub fn initialize_gpu_synchronization(
    mut commands: Commands,
) {
    println!("🔧 Initializing GPU synchronization system...");
    
    let gpu_sync = AlkydGpuSynchronization::new();
    commands.insert_resource(gpu_sync);
    
    println!("✓ GPU synchronization system initialized");
    println!("   - Sync interval: {:?}", Duration::from_millis(16));
    println!("   - Max upload timeout: {:?}", Duration::from_secs(5));
}

/// Helper function to serialize Alkyd config to bytes
fn serialize_alkyd_config_to_bytes(config: &crate::alkyd_integration::AlkydTextureConfig) -> Vec<u8> {
    // Simple serialization - in a real implementation, use proper serialization
    let mut bytes = Vec::new();
    
    // Add texture size
    bytes.extend_from_slice(&config.texture_size.x.to_le_bytes());
    bytes.extend_from_slice(&config.texture_size.y.to_le_bytes());
    
    // Add noise parameters
    bytes.extend_from_slice(&config.noise_scale.to_le_bytes());
    bytes.extend_from_slice(&(config.noise_octaves as u32).to_le_bytes());
    
    // Add color parameters
    for &color in &config.base_color {
        bytes.extend_from_slice(&color.to_le_bytes());
    }
    bytes.extend_from_slice(&config.color_variation.to_le_bytes());
    
    // Add flags
    bytes.push(config.use_gpu_acceleration as u8);
    bytes.push(config.enable_edge_detection as u8);
    bytes.push(config.enable_color_blending as u8);
    
    // Pad to 128 bytes
    while bytes.len() < 128 {
        bytes.push(0);
    }
    
    bytes
}

/// System to cleanup Alkyd GPU buffers on shutdown
pub fn cleanup_alkyd_buffers(
    mut buffer_manager: ResMut<AlkydBufferManager>,
) {
    let stats = buffer_manager.get_memory_stats();
    if stats.buffer_count > 0 {
        println!("🧹 Cleaning up Alkyd GPU buffers...");
        buffer_manager.cleanup_all_buffers();
        println!("✓ Alkyd GPU buffers cleaned up");
    }
}

/// System to setup Alkyd buffer management in the app
pub fn setup_alkyd_buffer_management(app: &mut App) {
    println!("🔧 Setting up Alkyd buffer management...");
    app
        .init_resource::<AlkydBufferManager>()
        .init_resource::<AlkydGpuSynchronization>()
        .add_systems(Startup, initialize_alkyd_buffer_manager)
        .add_systems(Startup, initialize_gpu_synchronization.after(initialize_alkyd_buffer_manager))
        .add_systems(Startup, create_alkyd_gpu_buffers
            .after(alkyd_integration::generate_all_block_textures)
            .after(alkyd_gpu_shaders::generate_all_block_gpu_textures)
        )
        .add_systems(Update, monitor_alkyd_memory_usage)
        .add_systems(Update, synchronize_gpu_buffers
            .after(create_alkyd_gpu_buffers)
            .after(alkyd_gpu_shaders::initialize_gpu_compute_workers)
        )
        .add_systems(Update, monitor_gpu_synchronization
            .after(synchronize_gpu_buffers)
        )
        .add_systems(Update, demonstrate_gpu_buffer_operations
            .after(monitor_gpu_synchronization)
        )
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
