#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec2 normal;
layout(location = 3) in vec2 uv;

layout(push_constant) uniform Push {
    mat4 transform;
} push;

void main() {
    gl_Position = push.transform * vec4(position, 1.0);
}
