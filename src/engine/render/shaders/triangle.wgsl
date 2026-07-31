struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec3<f32>, // pass to fragment
};

@vertex
fn vs_main(
    @location(0) in_pos: vec2<f32>,
    @location(1) in_color: vec3<f32>
) -> VertexOutput {

    var out: VertexOutput;
    out.position = vec4<f32>(in_pos, 0.0, 1.0);
    out.color = in_color;
    return out;
}

@fragment
fn fs_main(
    vertex: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4<f32>(vertex.color, 1.0);
}