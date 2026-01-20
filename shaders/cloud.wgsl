// Cloud fragment shader
// This shader renders procedural clouds using noise functions

// Import standard WGSL functions
fn hash22(p: vec2<f32>) -> vec2<f32> {
    let p3 = fract(vec3<p>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn noise2D(p: vec2<f32>) -> f32 {
    const K1: f32 = 0.366025404;
    const K2: f32 = 0.211324865;
    
    let i = floor(p + (p.x + p.y) * K1);
    let a = p - i + (i.x + i.y) * K2;
    
    let mut o: f32 = 0.5 - a.x * a.x - a.y * a.y;
    if (o > 0.0) {
        o = o * o;
        return o * o * dot(a, hash22(i) - 0.5);
    }
    return 0.0;
}

fn fbm2D(p: vec2<f32>, octaves: i32, frequency: f32, persistence: f32) -> f32 {
    var value: f32 = 0.0;
    var amplitude: f32 = 1.0;
    var freq: f32 = frequency;
    
    for (var i: i32 = 0; i < octaves; i = i + 1) {
        value = value + noise2D(p * freq) * amplitude;
        amplitude = amplitude * persistence;
        freq = freq * 2.0;
    }
    
    return value;
}

// Cloud rendering function
fn render_clouds(
    uv: vec2<f32>,
    layer_altitude: f32,
    layer_scale: f32,
    layer_density: f32,
    layer_speed: f32,
    layer_direction: vec2<f32>,
    noise_octaves: i32,
    noise_frequency: f32,
    noise_persistence: f32,
    animation_time: f32,
    global_coverage: f32,
    global_density: f32,
    camera_position: vec3<f32>,
    sun_direction: vec3<f32>,
    weather_type: u32,
    precipitation_intensity: f32,
) -> vec4<f32> {
    // Apply animation and movement
    let animated_uv = uv + layer_direction * animation_time * layer_speed;
    
    // Generate cloud noise
    let cloud_noise = fbm2D(animated_uv * layer_scale, noise_octaves, noise_frequency, noise_persistence);
    
    // Apply global coverage and density
    let cloud_density = clamp(cloud_noise * global_density * layer_density, 0.0, 1.0);
    
    // Apply global coverage
    let cloud_coverage = smoothstep(0.0, global_coverage, cloud_density);
    
    // Early exit if no cloud
    if (cloud_coverage <= 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    // Calculate lighting based on sun direction
    let light_intensity = max(0.0, dot(sun_direction, vec3<f32>(0.0, 1.0, 0.0)));
    
    // Base cloud color (white with some blue tint)
    let base_color = vec3<f32>(0.8, 0.85, 0.9);
    
    // Darken clouds based on density and weather
    let cloud_color = mix(
        base_color * 0.5,
        base_color,
        cloud_coverage
    );
    
    // Apply lighting
    let final_color = cloud_color * (0.7 + 0.3 * light_intensity);
    
    // Adjust alpha based on weather conditions
    let alpha = cloud_coverage * (1.0 + precipitation_intensity * 0.2);
    
    return vec4<f32>(final_color, alpha);
}

@group(0) @binding(0)
var<uniform> cloud_uniform: CloudUniform;

@fragment
fn fragment_main(in: FragmentInput) -> @location(0) vec4<f32> {
    // Get UV coordinates
    let uv = in.uv;
    
    // Calculate cloud contribution from each layer
    var final_color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    // Render each cloud layer
    for (var i: i32 = 0; i < 4; i = i + 1) {
        if (cloud_uniform.layer_densities[i] > 0.0) {
            let layer_color = render_clouds(
                uv,
                cloud_uniform.layer_altitudes[i],
                cloud_uniform.layer_scales[i],
                cloud_uniform.layer_densities[i],
                cloud_uniform.layer_speeds[i],
                cloud_uniform.layer_directions[i],
                4, // Default octaves
                0.001, // Default frequency
                0.5, // Default persistence
                cloud_uniform.animation_time,
                cloud_uniform.global_coverage,
                cloud_uniform.global_density,
                cloud_uniform.camera_position,
                cloud_uniform.sun_direction,
                cloud_uniform.weather_type,
                cloud_uniform.precipitation_intensity
            );
            
            // Blend layers additively
            final_color = mix(final_color, layer_color, layer_color.a);
        }
    }
    
    // Apply weather-specific effects
    if (cloud_uniform.weather_type == 5u) { // Rain
        final_color.rgb = mix(final_color.rgb, vec3<f32>(0.6, 0.7, 0.8), 0.2);
    } else if (cloud_uniform.weather_type == 6u) { // Heavy Rain
        final_color.rgb = mix(final_color.rgb, vec3<f32>(0.5, 0.6, 0.7), 0.3);
    } else if (cloud_uniform.weather_type == 7u) { // Thunderstorm
        final_color.rgb = mix(final_color.rgb, vec3<f32>(0.4, 0.5, 0.6), 0.4);
    }
    
    return final_color;
}