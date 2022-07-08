#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(binding = 1) uniform sampler3D cubelet_data;
layout(binding = 2) uniform sampler3D cubelet_sdf;

layout(location = 0) in vec3 in_uvw;
layout(location = 1) in vec3 in_position;

layout(location = 0) out vec4 out_color;

void main() {
	float cubelet_size = 0.1;

	vec3 camera_position = (inverse(ubo.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz;

	vec3 model_position = (ubo.model * vec4(in_position, 1.0)).xyz;

	//this is backwards because we are projecting onto the backface
	vec3 dir = camera_position - model_position;

	vec3 dir_n = normalize(dir);

	float step = 0.005f;
	
	int step_count = 0;

	float max = 1.5f;

	vec4 final = vec4(0);

	while(true) {
		vec3 ddr = dir_n * step * step_count;

		vec3 pos = in_uvw  + ddr;

		vec4 col = texture(cubelet_data, pos);

		if (pos.x < 0 || pos.y < 0 || pos.z < 0 || pos.x > 1 || pos.y > 1 || pos.z > 1) {
			break;
		}

		if(col.a > 0.5) {
			final = col;
		}


		step_count += 1;
	}
	
	out_color = final;	
}
