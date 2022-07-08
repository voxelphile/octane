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


	vec4 model_o = ubo.model * vec4(in_position, 1);

	vec3 model_t = model_o.xyz / model_o.w;

	vec3 camera_p = inverse(ubo.view)[3].xyz;

	vec3 dir_v = normalize(camera_p - model_t);

	vec3 dir_q = dir_v;

mat4 MV = ubo.view * ubo.model;
vec3 eye_pos = camera_p - (-transpose(mat3(MV)) * vec3(MV[3]));

	vec3 dir = eye_pos - model_o.xyz;

	float step = 0.01f;
	int step_count = 0;

	float max = 2.0f;

	vec4 final = vec4(0);

	while(true) {
		vec3 pos = in_uvw + dir * (step * step_count);

		vec4 col = texture(cubelet_data, pos);

		if(col.a >= 0.9) {
			final = col;
			break;
		}

		if(step_count > int(max / step)) {
			final = vec4(0);
			break;
		}

		step_count += 1;
	}

	out_color = final;	
}
