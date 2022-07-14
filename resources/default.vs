#version 450

const uint CHUNK_SIZE = 32;

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
layout(location = 3) in vec3 in_chunk_position;

layout(location = 0) out vec3 out_uvw;
layout(location = 1) out vec3 out_position;
layout(location = 2) out vec3 out_chunk_position;

void main() {
	mat4 true_model = ubo.model;

	true_model[3].xyz += in_chunk_position * CHUNK_SIZE;
	
	vec3 position = in_position * CHUNK_SIZE / 2;
	
	gl_Position = ubo.proj * ubo.view * true_model * vec4(position, 1.0);
	
	out_uvw = in_uvw;
	out_position = position;
	out_chunk_position = in_chunk_position;
}
