#version 450

const uint CHUNK_SIZE = 32;

layout (local_size_x = 8, local_size_y = 8, local_size_z = 8) in;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec2 resolution;
    uint render_distance;
} ubo;

layout(binding = 1, r16ui) uniform uimage3D cubelet_sdf_source;
layout(binding = 2, r16ui) uniform uimage3D cubelet_sdf_result;

layout(binding = 3) buffer JFAI
{
    uint step_size;
    uint seed_amount;
    uvec3 seeds[];
} jfai;

void get_min_distance_point(vec3 pos, ivec4 info, inout vec4 data) {
	if (info.w > 0) {
		float dst = distance(pos, vec3(info.xyz));
		if (dst < data.w) {
			data = vec4(info.xyz, dst);
		}
	}
}

void main() {
	uvec3 id = gl_GlobalInvocationID.xyz;
	vec4 data = imageLoad(cubelet_sdf_result, ivec3(id));
	if (data == vec4(0)) {
		data = vec4(0, 0, 0, 42069);
	}

	int step_amount = int(log2(float(jfai.step_size)));

	for (int i = 0; i < step_amount; i++) {
		int step = int(pow(2, step_amount - i - 1));
		for (int x = -1; x <= 1; x += 1) 
		{
			for (int y = -1; y <= 1; y += 1) 
			{
				for (int z = -1; z <= 1; z += 1) 
				{
					ivec3 step_size = ivec3(x,y,z) * step;
					uint block = imageLoad(cubelet_sdf_source, ivec3(id) + step_size).x;
					ivec4 info;
					info.xyz = ivec3(id) + step_size;
					info.w = int(block);
					get_min_distance_point(vec3(id), info, data);
				}

			}
		}
	}

	imageStore(cubelet_sdf_result, ivec3(id), uvec4(data.w));
}
