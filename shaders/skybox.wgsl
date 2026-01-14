// Skybox shader with procedural sky rendering
// Includes gradient sky, sun, and atmospheric scattering effects

#version 460

struct SkyboxMaterial {
    color_top: vec4<f32>,
    color_bottom: vec4<f32>,
    sun_position: vec3<f32>,
    sun_color: vec4<f32>,
    sun_intensity: f32,
};

@group(0) @binding(0)
var<uniform> material: SkyboxMaterial;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.world_position = input.position;
    output.world_normal = normalize(input.normal);
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize the world position to get a direction vector
    let sky_dir = normalize(input.world_position);
    
    // Calculate the sky gradient based on the Y coordinate
    // Top of sky (Y=1) uses color_top, bottom (Y=-1) uses color_bottom
    let sky_gradient = mix(material.color_bottom, material.color_top, (sky_dir.y + 1.0) * 0.5);
    
    // Calculate sun contribution
    let sun_dir = normalize(material.sun_position);
    let sun_dot = dot(sky_dir, sun_dir);
    let sun_factor = pow(max(0.0, sun_dot), 16.0) * material.sun_intensity;
    let sun_color = material.sun_color.rgb * sun_factor;
    
    // Combine sky gradient with sun
    let final_color = sky_gradient.rgb + sun_color;
    
    // Add some atmospheric scattering effect
    let scattering = pow(max(0.0, sky_dir.y), 2.0) * 0.1;
    let final_color_with_scattering = final_color + vec3<f32>(scattering, scattering * 0.8, scattering * 0.6);
    
    return vec4<f32>(final_color_with_scattering, 1.0);
}