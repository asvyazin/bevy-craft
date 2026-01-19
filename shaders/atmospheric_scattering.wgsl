// Atmospheric scattering shader for realistic sky rendering
// Based on simplified atmospheric scattering models

// Uniform parameters for atmospheric scattering
struct AtmosphericParameters {
    // Rayleigh scattering parameters
    rayleigh_coefficient: vec3<f32>;
    rayleigh_scale_height: f32;
    
    // Mie scattering parameters
    mie_coefficient: vec3<f32>;
    mie_scale_height: f32;
    mie_phase_g: f32;
    
    // Sun parameters
    sun_intensity: f32;
    sun_angular_radius: f32;
    
    // Atmosphere parameters
    planet_radius: f32;
    atmosphere_radius: f32;
    
    // Time of day parameters
    sun_direction: vec3<f32>;
    
    // Camera position
    camera_position: vec3<f32>;
    
    // Padding
    _padding: f32;
};

struct CameraParameters {
    view_proj: mat4x4<f32>;
    position: vec3<f32>;
    _padding: f32;
};

@group(0) @binding(0)
var<uniform> atmospheric_params: AtmosphericParameters;

@group(0) @binding(1)
var<uniform> camera_params: CameraParameters;

// Vertex shader for skybox
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> @builtin(position) vec4<f32> {
    // Skybox vertices (cube)
    let positions = array<vec3<f32>, 8>(
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(1.0, 1.0, -1.0),
        vec3<f32>(-1.0, 1.0, -1.0),
        vec3<f32>(-1.0, -1.0, 1.0),
        vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(-1.0, 1.0, 1.0)
    );
    
    let position = positions[vertex_index];
    return camera_params.view_proj * vec4<f32>(position, 1.0);
}

// Fragment shader for atmospheric scattering
@fragment
fn fs_main(
    @builtin(position) position: vec4<f32>
) -> @location(0) vec4<f32> {
    // Calculate ray direction from camera to skybox
    // For a skybox, we can use the normalized position as the view direction
    let view_dir = normalize(position.xyz);
    
    // Calculate atmospheric scattering
    let color = calculate_atmospheric_scattering(view_dir);
    
    return vec4<f32>(color, 1.0);
}

// Calculate atmospheric scattering for a given view direction
fn calculate_atmospheric_scattering(view_dir: vec3<f32>) -> vec3<f32> {
    // Calculate Rayleigh and Mie scattering
    let rayleigh = calculate_rayleigh_scattering(view_dir);
    let mie = calculate_mie_scattering(view_dir);
    
    // Combine scattering effects
    let scattering = rayleigh + mie;
    
    // Add sun contribution
    let sun = calculate_sun_contribution(view_dir);
    
    return scattering + sun;
}

// Calculate Rayleigh scattering (molecular scattering - creates blue sky)
fn calculate_rayleigh_scattering(view_dir: vec3<f32>) -> vec3<f32> {
    // Simplified Rayleigh scattering calculation
    // Rayleigh scattering is proportional to 1/(wavelength^4)
    // So blue light scatters more than red light
    
    let sun_dot = dot(view_dir, atmospheric_params.sun_direction);
    
    // Calculate scattering based on angle from sun
    let rayleigh_phase = 0.75 * (1.0 + sun_dot * sun_dot);
    
    // Apply Rayleigh scattering coefficients (blue > green > red)
    return atmospheric_params.rayleigh_coefficient * rayleigh_phase;
}

// Calculate Mie scattering (particle scattering - creates haze, sun halos)
fn calculate_mie_scattering(view_dir: vec3<f32>) -> vec3<f32> {
    // Simplified Mie scattering calculation
    // Mie scattering is more uniform across wavelengths
    
    let sun_dot = dot(view_dir, atmospheric_params.sun_direction);
    
    // Henyey-Greenstein phase function for Mie scattering
    let g = atmospheric_params.mie_phase_g;
    let g_squared = g * g;
    let denominator = 1.0 + g_squared - 2.0 * g * sun_dot;
    let mie_phase = (1.0 - g_squared) / (4.0 * 3.14159265 * denominator * sqrt(denominator));
    
    // Apply Mie scattering coefficients
    return atmospheric_params.mie_coefficient * mie_phase;
}

// Calculate sun contribution
fn calculate_sun_contribution(view_dir: vec3<f32>) -> vec3<f32> {
    // Calculate angle between view direction and sun direction
    let sun_dot = dot(view_dir, atmospheric_params.sun_direction);
    
    // Check if we're looking near the sun
    let sun_angle = acos(sun_dot);
    
    // Calculate sun intensity based on angle
    if sun_angle < atmospheric_params.sun_angular_radius {
        // Looking at the sun - bright light
        let intensity = smoothstep(atmospheric_params.sun_angular_radius, 0.0, sun_angle);
        return vec3<f32>(atmospheric_params.sun_intensity) * intensity;
    }
    
    return vec3<f32>(0.0);
}