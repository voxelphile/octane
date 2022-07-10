#version 450

const uint CHUNK_SIZE = 16;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec2 resolution;
    uint render_distance;
} ubo;

layout(binding = 1) uniform sampler3D cubelet_data;
layout(binding = 2) uniform sampler3D cubelet_sdf;

layout(location = 0) in vec3 in_uvw;
layout(location = 1) in vec3 in_position;

layout(location = 0) out vec4 out_final;

float raycast(vec3 ray_pos, vec3 dir) {
	float t_min = 0;
	float t_max = 100000;

	float bmin = -16;
	float bmax = 16;
	
	vec3 obb = ubo.model[3].xyz;

	vec3 delta = obb - ray_pos;

	vec3 x_axis = ubo.model[0].xyz;

	float x_e = dot(x_axis, delta);
	float x_f = dot(dir, x_axis);
	
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
	float y_f = dot(dir, y_axis);
	
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
	float z_f = dot(dir, z_axis);
	
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
	uint cubelet_size = 2 * ubo.render_distance * CHUNK_SIZE;	
	
	vec4 near_plane = vec4((gl_FragCoord.xy / ubo.resolution) * 2 - 1, 0.1, 1.0);

	near_plane = vec4((inverse(ubo.proj) * near_plane).xy, 0.0, 1.0);

	vec3 camera_position = (inverse(ubo.view) * near_plane).xyz;

	vec3 model_position = (ubo.model * vec4(in_position, 1.0)).xyz;

	vec3 dir = normalize(model_position - camera_position);

	float obb_dist = raycast(camera_position, dir);

	vec3 point = camera_position + dir * obb_dist; 
	
	mat4 model_rot = ubo.model;

	model_rot[3].xyz = vec3(0);

	point = (inverse(model_rot) * vec4(point, 1)).xyz;

	point += cubelet_size / 2;

	dir = (inverse(model_rot) * vec4(dir, 0)).xyz;
	dir = normalize(dir);

	vec4 final = vec4(0.0);

	ivec3 map_point = ivec3(floor(point + 0.));
    	vec3 side_dist;
    	bvec3 mask;
    	vec3 delta_dist;
    	{
        	delta_dist = 1.0 / abs(dir);
        	ivec3 rayStep = ivec3(sign(dir));
        	side_dist = (sign(dir) * (vec3(map_point) - point) + (sign(dir) * 0.5) + 0.5) * delta_dist; 

        	for (int i = 0; i < 64; i++)
        	{
            		
			vec3 pos = (vec3(map_point) + vec3(0.5)) / cubelet_size;
		
	 	   	vec4 col = texture(cubelet_data, pos);
	    
	    		if (col.a == 1) { 
				final = col;
				break;
	    		}

            		mask = lessThanEqual(side_dist.xyz, min(side_dist.yzx, side_dist.zxy));
            		side_dist += vec3(mask) * delta_dist;
            		map_point += ivec3(vec3(mask)) * rayStep;
        	}
    	}

	out_final = final;	
}
