// Filter vertex shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var output: VertexOutput;
    
    // Generate a full-screen quad
    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = f32((vertex_index & 2u)) - 1.0;
    
    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.tex_coord = vec2<f32>(x * 0.5 + 0.5, y * -0.5 + 0.5);
    
    return output;
}
