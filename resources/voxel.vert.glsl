#version 450

#define CHUNK_SIZE 8

layout(binding = 0) uniform Camera {
	mat4 view;
	mat4 proj;
	mat4 model;
} camera;

layout(binding = 1) uniform Object {
	mat4 model;
} object;

layout(binding = 2) uniform RenderSettings {
	uvec4 resolution;
	uint render_distance;
} settings;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_uvw;
layout(location = 3) in uvec3 in_chunk_position;

layout(location = 0) out vec3 out_clip;
layout(location = 1) out vec3 out_position;
layout(location = 2) out uvec3 out_chunk_position;

void main() {
	mat4 true_model = object.model;

	true_model[3].xyz += vec3(in_chunk_position) * CHUNK_SIZE;
	
	vec3 position = in_position * CHUNK_SIZE / 2;

	vec4 frag_coord = camera.proj * camera.view * true_model * vec4(position, 1.0);
	
	gl_Position = frag_coord;

	out_clip = gl_Position.xyz;
	out_position = position;
	out_chunk_position = in_chunk_position;
}
