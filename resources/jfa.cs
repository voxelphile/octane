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

void get_min_distance_point(vec3 pos, vec4 info, inout vec4 data) {
	if (info.w > 0) {
		float dst = distance(pos, info.xyz);
		if (dst < data.w) {
			data = vec4(info.xyz, dst);
		}
	}
}

void main() {
	uvec3 id = gl_GlobalInvocationID.xyz;
	vec4 data = vec4(0, 0, 0, 42069);

	for (int x = -1; x <= 1; x += 2) 
	{
		for (int y = -1; y <= 1; y += 2) 
		{
			for (int z = -1; z <= 1; z += 2) 
			{
				ivec3 step_size = ivec3(x,y,z) * int(jfai.step_size);
				get_min_distance_point(vec3(id), imageLoad(cubelet_sdf_source, ivec3(id) + step_size), data);
			}

		}
	}

	imageStore(cubelet_sdf_result, ivec3(id), data);
}
