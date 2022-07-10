#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec2 resolution;
} ubo;

layout(binding = 1) uniform sampler3D cubelet_data;
layout(binding = 2) uniform sampler3D cubelet_sdf;

layout(location = 0) in vec3 in_uvw;
layout(location = 1) in vec3 in_position;

layout(location = 0) out vec4 out_color;

float raycast(vec3 ray_pos, vec3 dir_n) {
	float t_min = 0;
	float t_max = 100000;

	float bmin = -1;
	float bmax = 1;
	
	vec3 obb = ubo.model[3].xyz;

	vec3 delta = obb - ray_pos;

	vec3 x_axis = ubo.model[0].xyz;

	float x_e = dot(x_axis, delta);
	float x_f = dot(dir_n, x_axis);
	
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
	float y_f = dot(dir_n, y_axis);
	
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
	float z_f = dot(dir_n, z_axis);
	
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
	int cubelet_size = 10;

	float half_height = tan(radians(90) / 2.0);

	float half_width = (960.0 / 540.0) * half_height;

	vec4 near = vec4((gl_FragCoord.xy / ubo.resolution) * 2 - 1, 0.0, 1.0);

	near = inverse(ubo.proj) * near;

	vec3 camera_position = (inverse(ubo.view) * near).xyz;

	vec3 model_position = (ubo.model * vec4(in_position, 1.0)).xyz;

	vec3 dir = model_position - camera_position;

	vec3 dir_n = normalize(dir);

	float p = raycast(camera_position, dir_n);

	vec3 point = camera_position + dir_n * p; 
	mat4 modelxyzrot = ubo.model;

	modelxyzrot[3].xyz = vec3(0);

	point = (inverse(modelxyzrot) * vec4(point, 1)).xyz;
	dir_n = (inverse(modelxyzrot) * vec4(dir_n, 0)).xyz;
	dir_n = normalize(dir_n);

	point /= 2;
	point += 0.5;

	point *= cubelet_size;

	vec4 final = vec4(0.0);

ivec3 mapPos = ivec3(floor(point + 0.));

    vec3 color = vec3(1.0);
    vec3 sideDist;
    bvec3 mask;
    // core of https://www.shadertoy.com/view/4dX3zl Branchless Voxel Raycasting by fb39ca4 (somewhat reduced)
    vec3 deltaDist;
    {
        deltaDist = 1.0 / abs(dir_n);
        ivec3 rayStep = ivec3(sign(dir_n));
        sideDist = (sign(dir_n) * (vec3(mapPos) - point) + (sign(dir_n) * 0.5) + 0.5) * deltaDist; 

        for (int i = 0; i < 64; i++)
        {
            vec3 pos = (vec3(mapPos) + vec3(0.5)) / cubelet_size;
		
	    vec4 col = texture(cubelet_data, pos);
	    
	    if (col.a == 1) { 
		final = col;
		break;
	    }// forked shader used continue here

            //Thanks kzy for the suggestion!
            mask = lessThanEqual(sideDist.xyz, min(sideDist.yzx, sideDist.zxy));
            sideDist += vec3(mask) * deltaDist;
            mapPos += ivec3(vec3(mask)) * rayStep;
        }
    }

	out_color = final;	
}
