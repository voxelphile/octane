#version 450

const uint CHUNK_SIZE = 32;

layout (local_size_x = 64, local_size_y = 64, local_size_z = 64) in;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec2 resolution;
    uint render_distance;
} ubo;

layout(binding = 1, rgba32f) uniform image3D cubelet_data;
layout(binding = 2, rgba32f) uniform image3D cubelet_sdf_source;
layout(binding = 3, rgba32f) uniform image3D cubelet_sdf_result;

layout(binding = 4) buffer JFAI
{
    uint step_size;
    uvec3 seeds[];
} jfai;

void main() {
	uint id = gl_GlobalInvocationID.x 
		+ gl_GlobalInvocationID.y * CHUNK_SIZE 
		+ gl_GlobalInvocationID.z * CHUNK_SIZE * CHUNK_SIZE;
	
	uvec3 seed_pos = jfai.seeds[id];
	
	uint px = seed_pos.x % (2 * ubo.render_distance * CHUNK_SIZE);
	uint py = seed_pos.y % (2 * ubo.render_distance * CHUNK_SIZE);
	uint pz = seed_pos.z % (2 * ubo.render_distance * CHUNK_SIZE);
	
	imageStore(cubelet_sdf_source, ivec3(px, py, pz), vec4(px, py, pz, id));
}

