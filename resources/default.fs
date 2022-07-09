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

float raycast(vec3 ray_pos, vec3 ray_dir) {
	float t_min = 0;
	float t_max = 100000;

	float bmin = -1;
	float bmax = 1;
	
	vec3 obb = ubo.model[3].xyz;

	vec3 delta = obb - ray_pos;

	vec3 x_axis = ubo.model[0].xyz;

	float x_e = dot(x_axis, delta);
	float x_f = dot(ray_dir, x_axis);
	
	float x_t_1 = (x_e + bmin) / x_f;
	float x_t_2 = (x_e + bmax) / x_f;

	if (x_t_1 > x_t_2) {
		float w = x_t_1;
		x_t_1 = x_t_2;
		x_t_2 = w;
	}

	if (x_t_2 < t_max) {
		t_max = x_t_2;
	}

	if (x_t_1 > t_min) {
		t_min = x_t_1;
	}

	if (t_max < t_min) {
		return -1;
	}

	vec3 y_axis = ubo.model[1].xyz;

	float y_e = dot(y_axis, delta);
	float y_f = dot(ray_dir, y_axis);
	
	float y_t_1 = (y_e + bmin) / y_f;
	float y_t_2 = (y_e + bmax) / y_f;

	if (y_t_1 > y_t_2) {
		float w = y_t_1;
		y_t_1 = y_t_2;
		y_t_2 = w;
	}

	if (y_t_2 < t_max) {
		t_max = y_t_2;
	}

	if (y_t_1 > t_min) {
		t_min = y_t_1;
	}

	if (t_max < t_min) {
		return -1;
	}

	vec3 z_axis = ubo.model[2].xyz;

	float z_e = dot(z_axis, delta);
	float z_f = dot(ray_dir, z_axis);
	
	float z_t_1 = (z_e + bmin) / z_f;
	float z_t_2 = (z_e + bmax) / z_f;

	if (z_t_1 > z_t_2) {
		float w = z_t_1;
		z_t_1 = z_t_2;
		z_t_2 = w;
	}

	if (z_t_2 < t_max) {
		t_max = z_t_2;
	}

	if (z_t_1 > t_min) {
		t_min = z_t_1;
	}

	if (t_max < t_min) {
		return -1;
	}

	return t_min;
}

void main() {
	float cubelet_size = 0.1;

	vec3 camera_position = (inverse(ubo.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz;

	vec3 model_position = (ubo.model * vec4(in_position, 1.0)).xyz;

	//this is backwards because we are projecting onto the backface
	vec3 dir = model_position - camera_position;

	vec3 dir_n = normalize(dir);
	
	vec3 pos2 = (inverse(ubo.model) * vec4(camera_position, 1.0)).xyz;

	float p = raycast(camera_position, dir_n);

	vec3 point = camera_position + dir_n * p; 
	mat4 modelxyzrot = ubo.model;

	modelxyzrot[3].xyz = vec3(0);

	point = (inverse(modelxyzrot) * vec4(point, 1)).xyz;
	dir_n = (inverse(modelxyzrot) * vec4(dir_n, 0)).xyz;
	point /= 2;
	point += 0.5;



	float step = 0.005f;
	
	int step_count = 0;

	int max_step_count = 512;

	vec4 final = vec4(0.1);

	for (; step_count < max_step_count; step_count++)  {
		vec3 ddr = dir_n * step * step_count;

		vec3 pos = point  + ddr;

		vec4 col = texture(cubelet_data, pos);


		if(col.a > 0.5) {
			final = col;
			break;
		}
	}
	
	out_color = final;	
}
