// Cloud vertex shader
// Simple pass-through vertex shader for cloud rendering

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> cloud_uniform: CloudUniform;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transform position to clip space
    output.clip_position = vec4<f32>(input.position, 1.0);
    
    // Pass through UV coordinates
    output.uv = input.uv;
    
    return output;
}