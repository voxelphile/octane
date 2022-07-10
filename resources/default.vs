#version 450

const uint CHUNK_SIZE = 16;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec2 resolution;
    uint render_distance;
} ubo;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_uvw;

layout(location = 0) out vec3 out_uvw;
layout(location = 1) out vec3 out_position;

void main() {
	float scaling_factor = ubo.render_distance * CHUNK_SIZE;
	vec3 position = in_position * scaling_factor;
	gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
	out_uvw = in_uvw;
	out_position = position;
}
