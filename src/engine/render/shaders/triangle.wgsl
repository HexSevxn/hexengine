// triangle.wgsl
// Draws a triangle defined by three local-space vertices, transformed by
// a position offset and a rotation angle (radians).

struct Instance {
    position: vec2<f32>,    // world-space translation
    rotation: f32,          // rotation angle in radians
    _pad: f32,              // padding to keep vertices 8-byte aligned
    v0: vec2<f32>,          // triangle vertex 0 (local space)
    v1: vec2<f32>,          // triangle vertex 1 (local space)
    v2: vec2<f32>,          // triangle vertex 2 (local space)
    c0: vec4<f32>,          // triangle color 0
    c1: vec4<f32>,          // triangle color 0
    c2: vec4<f32>,          // triangle color 0
};

@group(0) @binding(0)
var<storage, read> instances: array<Instance>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let instance = instances[instance_index];

    var local_pos: vec2<f32>;
    var vertex_color: vec4<f32>;
    switch(vertex_index) {
        case 0u { local_pos = instance.v0; vertex_color = instance.c0; }
        case 1u { local_pos = instance.v1; vertex_color = instance.c1; }
        default { local_pos = instance.v2; vertex_color = instance.c2; }
    }

    let cos_offset = cos(instance.rotation);
    let sin_offset = sin(instance.rotation);

    // 2D rotation matrix applied to the local vertex
    let rotated = vec2<f32>(
        local_pos.x * cos_offset - local_pos.y * sin_offset,
        local_pos.x * sin_offset + local_pos.y * cos_offset
    );
    let world_pos = rotated + instance.position;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(world_pos, 0.0, 1.0);
    out.color = vertex_color;
    return out;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(vertex.color);
}
