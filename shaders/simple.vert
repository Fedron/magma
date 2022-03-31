#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 color;

layout (location = 0) out vec3 outColor;

layout (set = 0, binding = 0) uniform GlobalUbo {
	mat4 projection;
	mat4 view;
	vec3 directionToLight;
} ubo;

layout (push_constant) uniform Push {
	mat4 modelMatrix;
	mat4 normalMatrix;
} push;

const float AMBIENT = 0.02;

void main() {
	gl_Position = ubo.projection * ubo.view * push.modelMatrix * vec4(position, 1.0f);

	vec3 normalWorldSpace = normalize(mat3(push.normalMatrix) * normal);
	float lightIntensity = AMBIENT + max(dot(normalWorldSpace, normalize(ubo.directionToLight)), 0);

	outColor = lightIntensity * color;
}
