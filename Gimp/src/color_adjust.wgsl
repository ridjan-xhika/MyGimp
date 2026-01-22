// Color Adjust Fragment Shader (Brightness, Contrast, Saturation)
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: vec4<f32>;

struct FragmentInput {
    @location(0) tex_coord: vec2<f32>,
};

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(rgb.r, max(rgb.g, rgb.b));
    let cmin = min(rgb.r, min(rgb.g, rgb.b));
    let delta = cmax - cmin;
    
    var h = 0.0;
    if (delta != 0.0) {
        if (cmax == rgb.r) {
            h = 60.0 * (fract((rgb.g - rgb.b) / delta / 6.0));
        } else if (cmax == rgb.g) {
            h = 60.0 * ((rgb.b - rgb.r) / delta + 2.0);
        } else {
            h = 60.0 * ((rgb.r - rgb.g) / delta + 4.0);
        }
    }
    
    let s = select(0.0, delta / cmax, cmax != 0.0);
    let v = cmax;
    
    return vec3<f32>(h, s, v);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let c = hsv.z * hsv.y;
    let h_prime = hsv.x / 60.0;
    let x = c * (1.0 - abs(fract(h_prime * 0.5) * 2.0 - 1.0));
    
    var rgb = vec3<f32>(0.0);
    if (h_prime < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h_prime < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h_prime < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h_prime < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h_prime < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    let m = hsv.z - c;
    return rgb + m;
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let brightness = params.z;
    let contrast = params.w;
    let saturation = 1.0; // Could use a 5th parameter if needed
    
    var color = textureSample(input_texture, input_sampler, input.tex_coord);
    let alpha = color.a;
    
    // Apply brightness
    color.rgb += brightness;
    
    // Apply contrast (relative to 0.5)
    color.rgb = (color.rgb - 0.5) * contrast + 0.5;
    
    // Apply saturation
    var hsv = rgb_to_hsv(color.rgb);
    hsv.y *= saturation;
    color.rgb = hsv_to_rgb(hsv);
    
    // Clamp and preserve alpha
    color = clamp(color, vec4<f32>(0.0), vec4<f32>(1.0));
    color.a = alpha;
    
    return color;
}
