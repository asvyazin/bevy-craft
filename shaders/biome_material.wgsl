// Biome Material Shader
// Enhanced material shader with biome-specific properties

varying vec3 v_Normal;
varying vec3 v_Position;
varying vec2 v_Uv;

struct BiomeMaterial {
    base_color: vec4<f32>,
    base_color_texture: texture_2d<f32>,
    roughness: f32,
    metallic: f32,
    reflectance: f32,
    normal_map_intensity: f32,
    ambient_occlusion: f32,
    emissive: vec4<f32>,
    height_variation: f32,
    moisture_effect: f32,
    temperature_effect: f32,
};

@group(0) @binding(0)
var<uniform> material: BiomeMaterial;

@group(0) @binding(1)
var base_color_tex: texture_2d<f32>;

@group(0) @binding(2)
var base_color_sampler: sampler;

struct FragmentInput {
    @location(0) normal: vec3<f32>,
    @location(1) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@fragment
fn fragment_main(input: FragmentInput) -> FragmentOutput {
    // Sample base color texture
    var base_color = textureSample(base_color_tex, base_color_sampler, input.uv).rgb;
    
    // Apply biome-specific modifications
    base_color = mix(base_color, material.base_color.rgb, material.height_variation);
    
    // Moisture effect - makes colors more saturated in wet biomes
    base_color = mix(base_color, base_color * vec3<f32>(1.0, 1.1, 1.0), material.moisture_effect);
    
    // Temperature effect - makes colors warmer in hot biomes, cooler in cold biomes
    if (material.temperature_effect > 0.7) {
        // Hot biome - add red/yellow tones
        base_color = mix(base_color, base_color * vec3<f32>(1.2, 0.9, 0.8), 0.3);
    } else if (material.temperature_effect < 0.3) {
        // Cold biome - add blue tones
        base_color = mix(base_color, base_color * vec3<f32>(0.8, 0.9, 1.2), 0.3);
    }
    
    // Apply emissive color
    var final_color = base_color + material.emissive.rgb;
    
    // Basic lighting calculation
    var normal = normalize(input.normal);
    var view_dir = normalize(-input.position);
    
    // Simple diffuse lighting
    var light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    var diffuse = max(dot(normal, light_dir), 0.0);
    
    // Apply ambient occlusion
    diffuse = mix(diffuse, diffuse * 0.5, material.ambient_occlusion);
    
    // Combine with base color
    final_color = final_color * (0.3 + diffuse * 0.7);
    
    // Apply reflectance
    var reflectance = material.reflectance;
    if (material.metallic > 0.0) {
        // Simple specular highlight for metallic surfaces
        var half_vec = normalize(light_dir + view_dir);
        var specular = pow(max(dot(normal, half_vec), 0.0), 32.0);
        final_color += specular * material.metallic * 0.5;
    }
    
    // Apply roughness to diffuse lighting
    final_color = mix(final_color, final_color * 0.8, material.roughness);
    
    return FragmentOutput {
        color: vec4<f32>(final_color, 1.0)
    };
}