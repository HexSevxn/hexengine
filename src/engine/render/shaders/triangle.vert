// triangle.vert
// Draws a triangle defined by three local-space vertices, transformed by
// a position offset and a rotation angle (radians).
// Requires gl_VertexID -> draw with 3 vertices, no vertex buffer needed.

#version 330 core

uniform vec2 u_position;     // world-space translation
uniform float u_rotation;    // rotation angle in radians
uniform vec2 u_vertices[3];  // triangle vertices (local space)

void main() {
    vec2 local_pos = u_vertices[gl_VertexID];

    float c = cos(u_rotation);
    float s = sin(u_rotation);

    // 2D rotation matrix applied to the local vertex
    vec2 rotated = vec2(
        local_pos.x * c - local_pos.y * s,
        local_pos.x * s + local_pos.y * c
    );

    vec2 world_pos = rotated + u_position;

    gl_Position = vec4(world_pos, 0.0, 1.0);
}
