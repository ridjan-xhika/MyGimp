// Unsharp Mask (Sharpen) Fragment Shader
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: vec4<f32>;

struct FragmentInput {
    @location(0) tex_coord: vec2<f32>,
};

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let sharpen_strength = params.y;
    let texel_size = 1.0 / vec2<f32>(textureDimensions(input_texture));
    
    // Center pixel
    let center = textureSample(input_texture, input_sampler, input.tex_coord);
    
    // Sample neighboring pixels for edge detection
    let up = textureSample(input_texture, input_sampler, input.tex_coord + vec2<f32>(0.0, -texel_size.y));
    let down = textureSample(input_texture, input_sampler, input.tex_coord + vec2<f32>(0.0, texel_size.y));
    let left = textureSample(input_texture, input_sampler, input.tex_coord + vec2<f32>(-texel_size.x, 0.0));
    let right = textureSample(input_texture, input_sampler, input.tex_coord + vec2<f32>(texel_size.x, 0.0));
    
    // Calculate unsharp mask
    let neighbors = (up + down + left + right) / 4.0;
    let high_pass = center - neighbors;
    let sharpened = center + high_pass * sharpen_strength;
    
    return clamp(sharpened, vec4<f32>(0.0), vec4<f32>(1.0));
}
