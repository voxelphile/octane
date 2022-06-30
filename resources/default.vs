#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

vec3 colors[8] = vec3[](
	vec3(0.0, 0.0, 0.0),
	vec3(1.0, 0.0, 0.0),
	vec3(1.0, 0.0, 1.0),
	vec3(0.0, 0.0, 1.0),
	vec3(0.0, 1.0, 0.0),
	vec3(1.0, 1.0, 0.0),
	vec3(1.0, 1.0, 1.0),
	vec3(0.0, 1.0, 1.0)
);

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 frag_color;

void main() {
	gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
	frag_color = colors[gl_VertexIndex];
}
