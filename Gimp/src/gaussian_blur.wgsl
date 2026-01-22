// Gaussian Blur Fragment Shader
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: vec4<f32>;

struct FragmentInput {
    @location(0) tex_coord: vec2<f32>,
};

fn gaussian(x: f32, sigma: f32) -> f32 {
    let pi = 3.14159265359;
    let numerator = exp(-(x * x) / (2.0 * sigma * sigma));
    let denominator = sqrt(2.0 * pi) * sigma;
    return numerator / denominator;
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let blur_radius = params.x;
    let texel_size = 1.0 / vec2<f32>(textureDimensions(input_texture));
    
    var color = vec4<f32>(0.0);
    var weight_sum = 0.0;
    
    let radius = i32(blur_radius);
    for (var x = -radius; x <= radius; x = x + 1) {
        for (var y = -radius; y <= radius; y = y + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_coord = input.tex_coord + offset;
            let distance = sqrt(f32(x * x + y * y));
            let weight = gaussian(distance, blur_radius);
            
            color += textureSample(input_texture, input_sampler, sample_coord) * weight;
            weight_sum += weight;
        }
    }
    
    return color / weight_sum;
}
