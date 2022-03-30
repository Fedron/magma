#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 color;

layout (location = 0) out vec3 outColor;

layout (push_constant) uniform Push {
	vec2 push;
} push;

void main() {
	gl_Position = vec4(position, 1.0f);
	outColor = color;
}
