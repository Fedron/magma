#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 color;

layout (location = 0) out vec3 outColor;

layout (set = 0, binding = 0) uniform GlobalUbo {
	mat4 projection;
	mat4 view;
	
	vec4 ambientLight;
	vec4 lightPosition;
	vec4 lightColor;
} ubo;

layout (push_constant) uniform Push {
	mat4 modelMatrix;
	mat4 normalMatrix;
} push;

void main() {
	vec4 worldPosition = push.modelMatrix * vec4(position, 1.0);
	gl_Position = ubo.projection * ubo.view * worldPosition;

	vec3 directionToLight = ubo.lightPosition.xyz - worldPosition.xyz;
	float attenuation = 1.0 / dot(directionToLight, directionToLight);
	vec3 normalWorldSpace = normalize(mat3(push.normalMatrix) * normal);

	vec3 lightColor = ubo.lightColor.xyz * ubo.lightColor.w * attenuation;
	vec3 ambientLight = ubo.ambientLight.xyz * ubo.ambientLight.w;
	vec3 diffuseLight = lightColor * max(dot(normalWorldSpace, normalize(directionToLight)), 0);

	outColor = (diffuseLight + ambientLight) * color;
}
