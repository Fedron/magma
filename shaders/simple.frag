#version 450

layout (location = 0) in vec3 fragColor;
layout (location = 1) in vec3 fragWorldPosition;
layout (location = 2) in vec3 fragWorldNormal;

layout (location = 0) out vec4 outColor;

layout (set = 0, binding = 0) uniform GlobalUbo {
	mat4 projection;
	mat4 view;
	
	vec4 ambientLight;
	vec4 lightPosition;
	vec4 lightColor;
} ubo;

void main() {
    vec3 directionToLight = ubo.lightPosition.xyz - fragWorldPosition;
	float attenuation = 1.0 / dot(directionToLight, directionToLight);

	vec3 lightColor = ubo.lightColor.xyz * ubo.lightColor.w * attenuation;
	vec3 ambientLight = ubo.ambientLight.xyz * ubo.ambientLight.w;
	vec3 diffuseLight = lightColor * max(dot(normalize(fragWorldNormal), normalize(directionToLight)), 0);

    outColor = vec4((diffuseLight + ambientLight) * fragColor, 1.0);
}