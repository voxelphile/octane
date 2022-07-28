#version 450

#extension GL_ARB_gpu_shader_int64 : enable

#define CHUNK_SIZE 8
#define MAX_STEP_COUNT 512
#define EPSILON 1e-2

struct Node {
	uint child;
	uint valid;
	uint block;
	uint64_t morton;
};

layout(binding = 0) uniform Camera {
	mat4 model;
	mat4 view;
	mat4 proj;
	mat4 camera;
} camera;

layout(binding = 1) uniform RenderSettings {
	vec2 resolution;
	uint render_distance;
} settings;

layout( binding = 2) buffer Octree {
	uint size;
	uint len;
	Node data[];
} octree;

layout(location = 0) in vec3 in_uvw;
layout(location = 1) in vec3 in_position;
layout(location = 2) in flat uvec3 in_chunk_position;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_occlusion;
layout(location = 2) out vec4 out_depth;

int   seed = 1;
void  srand(int s ) { seed = s; }
int   rand(void)  { seed=seed*0x343fd+0x269ec3; return (seed>>16)&32767; }
float frand(void) { return float(rand())/32767.0; }

vec4 get_albedo(uint block) {
	vec4 albedo = vec4(0);

	if (block == 1) {
		albedo = vec4(0.25, 0.5, 0.1, 1);
	} else if (block == 2) {
		albedo = vec4(0, 0.41, 0.58, 0.1);
	} else if (block == 3) {
		albedo = vec4(.72, .39, .12, 1);
	}

	return albedo;
}

float get_refraction(uint block) {
	float refraction;

	if (block == 1) {
		refraction = 1.5;
	} else if (block == 2) {
		refraction = 1.3;
	} else if (block == 42069) {
		refraction = 1.000;
	} else if (block == 3) {
		refraction = 1.5;
	}

	return refraction;
}

float get_reflectivity(uint block) {
	float reflectivity;

	if (block == 1) {
		reflectivity = 0.0;
	} else if (block == 2) {
		reflectivity = 0.2;
	} else if (block == 42069) {
		reflectivity = 0.0;
	} else if (block == 3) {
		reflectivity = 0.0;
	}

	return reflectivity;
}


uint get_position_mask(ivec3 pos, uint level) {
	float h = pow(2, octree.size);

	uint hierarchy[64];

	for (int i = 0; i < octree.size; i++) {
		h = h / 2;

		bool px = float(pos.x) >= h;
		bool py = float(pos.y) >= h;
		bool pz = float(pos.z) >= h;

		uint node_index = 0;

		if (px) {
			node_index += 4;
			pos.x -= int(h);
		}

		if (py) {
			node_index += 2;
			pos.y -= int(h);
		}

		if (pz) {
			node_index += 1;
			pos.z -= int(h);
		}

		hierarchy[i] = 1 << node_index;
	}

	return hierarchy[level];
}

bool get_voxel(vec3 position, out uint node_index, out uint node_depth, bool ignore_transparent) {
	int size = int(2 * settings.render_distance * CHUNK_SIZE);

	if (any(lessThan(ceil(position), vec3(0))) || any(greaterThan(floor(position), vec3(size)))) {
		return false;
	}

	ivec3 p = ivec3(floor(position + 0.0)) ;
	
	int s = size;
	int h = 0;
	int px,py,pz;
	int x = p.x;
	int y = p.y;
	int z = p.z;

	

	node_index = 0;

	for (int i = 0; i < octree.size; i++) {
		h = s / 2;

		px = int(x >= h);
		py = int(y >= h);
		pz = int(z >= h);
		uint k = px * 4 + py * 2 + pz;
		uint n = 1 << k;
		uint m = octree.data[node_index].valid & n;
		uint b = bitCount(octree.data[node_index].valid & (n - 1));

		if (m == n)
		{
			node_index = octree.data[node_index].child + b;
		} else {
			node_depth = i;
			return false;
		}

		x -= px * h;
		y -= py * h;
		z -= pz * h;

		s = h;
	}

	node_depth = octree.size - 1;

	Node node = octree.data[node_index];

	return !(get_albedo(node.block).a != 1 && ignore_transparent);
}


float vertex_ao(vec2 side, float corner) {
	return (side.x + side.y + max(corner, side.x * side.y)) / 3.0;
}

vec4 voxel_ao(vec3 pos, vec3 d1, vec3 d2) {
	uint _;

	vec4 side = vec4(
			float(get_voxel(pos + d1, _, _, true)), 
			float(get_voxel(pos + d2, _, _, true)), 
			float(get_voxel(pos - d1, _, _, true)), 
			float(get_voxel(pos - d2, _, _, true))
			);

	vec4 corner = vec4(
			float(get_voxel(pos + d1 + d2, _, _, true)), 
			float(get_voxel(pos - d1 + d2, _, _, true)), 
			float(get_voxel(pos - d1 - d2, _, _, true)), 
			float(get_voxel(pos + d1 - d2, _, _, true))
			);

	vec4 ao;
	ao.x = vertex_ao(side.xy, corner.x);
	ao.y = vertex_ao(side.yz, corner.y);
	ao.z = vertex_ao(side.zw, corner.z);
	ao.w = vertex_ao(side.wx, corner.w);
	return 1.0 - ao;
}

float jump_cast(mat4 true_model, vec3 ray_pos, vec3 dir) {
float t_min = 0;
	float t_max = 100000;

	//no clue why but this only works if 8 is hardcoded
	//CHUNK_SIZE / 2 = 16
	//try to fix at your peril
	float bmin = -4;
	float bmax =  4;

	vec3 obb = true_model[3].xyz;

	vec3 delta = obb - ray_pos;

	vec3 x_axis = true_model[0].xyz;

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

	vec3 y_axis = true_model[1].xyz;

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

	vec3 z_axis = true_model[2].xyz;

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

	return t_min;
}


struct Ray {
	vec3 origin;
	vec3 direction;
	float max_dist;
	uint medium;
	bool bounded;
};

struct RayHit {
	uint node;
	vec3 destination;
	vec3 back_step;
	vec3 normal;
	vec3 reflection;
	vec3 refraction;
	vec2 uv;
	float dist;
};

float sign11(float x)
{
    return x<0. ? -1. : 1.;
}

bool ray_cast(Ray ray, out RayHit hit) {
	ray.direction = normalize(ray.direction);
	ray.origin += ray.direction * pow(EPSILON, 3);

	vec3 p = ray.origin;
	vec3 s = vec3(sign11(ray.direction.x), sign11(ray.direction.y), sign11(ray.direction.z));
	vec3 s01 = max(s, 0.);
	vec3 ird = 1.0 / ray.direction;
	
	bvec3 mask;

	uint size = 2 * settings.render_distance * CHUNK_SIZE;

	bool ignore_transparent = false;

	float pre_dist = 0;
	vec3 post;

	uint node_index;
	uint node_depth;

	float dist = 0;

	
	//vec3 chunk_min = vec3(in_chunk_position * CHUNK_SIZE);
	//vec3 chunk_max = chunk_min + vec3(CHUNK_SIZE);

	vec3 chunk_min = CHUNK_SIZE * vec3(in_chunk_position);
	vec3 chunk_max = CHUNK_SIZE + chunk_min;
	
	for (int step_count = 0; step_count < MAX_STEP_COUNT; step_count++) {
		bool in_object = all(greaterThanEqual(p, vec3(0))) && all(lessThan(p, vec3(size)));
		bool rough_in_object = all(greaterThanEqual(p, vec3(-1))) && all(lessThan(p, vec3(size + 1)));

		bool in_bounds = all(greaterThanEqual(p, chunk_min )) && all(lessThan(p, chunk_max));
		bool rough_in_bounds = all(greaterThanEqual(p, chunk_min - 1)) && all(lessThan(p, chunk_max + 1));

		in_bounds = in_bounds || !ray.bounded;
		rough_in_bounds = rough_in_bounds || !ray.bounded;

		if (!rough_in_bounds || !rough_in_object) {
			break;
		}

		uint node_index;
		uint node_depth;


		bool voxel_found = get_voxel(p, node_index, node_depth, false);

		uint lod = octree.size - node_depth;

		if (voxel_found) {
			Node current = octree.data[node_index];

			if (in_bounds && in_object && ray.medium != current.block) {
				vec3 destination = ray.origin + ray.direction * (dist - 1e-4);
				vec3 back_step = p - s * vec3(mask);
				vec3 normal = vec3(mask) * sign(-ray.direction);
				vec2 uv = mod(vec2(dot(vec3(mask) * destination.yzx, vec3(1.0)), dot(vec3(mask) * destination.zxy, vec3(1.0))), vec2(1.0));
				vec3 reflection = reflect(ray.direction, normal);
				float eta = get_refraction(ray.medium) / get_refraction(current.block);
				vec3 refraction = refract(ray.direction, normal, eta);

				hit.node = node_index;	
				hit.destination = destination;
				hit.back_step = back_step;
				hit.normal = normal;
				hit.reflection = reflection;
				hit.refraction = refraction;
				hit.uv = uv;
				hit.dist = dist;
				return true;
			} 
		}

		float voxel = exp2(lod - 1);
		vec3 t_max = ird * (voxel * s01 - mod(p, voxel));

		mask = lessThanEqual(t_max.xyz, min(t_max.yzx, t_max.zxy));

		float c_dist = min(min(t_max.x, t_max.y), t_max.z);
		p += c_dist * ray.direction;
		dist += c_dist;

		p += 4e-4 * ray.direction * vec3(mask);

	}

	return false;
}

float depth(mat4 true_model, vec3 position) {
	vec4 projected = camera.proj * camera.view * true_model * vec4(position, 1);
	return projected.z / projected.w;
}

void main() {
	out_color = vec4(1);
	out_occlusion = vec4(1);	
	out_depth = vec4(1, 0, 0, 1);

	vec3 sun_pos = vec3(-1000, 2000, 100);
	mat4 true_model = camera.model;

	true_model[3].xyz += vec3(in_chunk_position * CHUNK_SIZE);

	vec4 near_plane = vec4((gl_FragCoord.xy / (settings.resolution / 4)) * 2 - 1, 0, 1.0);

	near_plane = vec4((inverse(camera.proj) * near_plane).xy, 0.0, 1.0);

	vec3 camera_position = (camera.camera * near_plane).xyz;

	vec3 model_position = (true_model * vec4(in_position, 1.0)).xyz;

	vec3 dir = normalize(model_position - camera_position);

	float obb_dist = jump_cast(true_model, camera_position, dir);

	vec3 point = camera_position + dir * (obb_dist - 1);

	point = (inverse(true_model) * vec4(point, 1)).xyz;

	point += CHUNK_SIZE * ivec3(in_chunk_position);	
	point += CHUNK_SIZE / 2;


	dir = (inverse(true_model) * vec4(dir, 0)).xyz;
	dir = normalize(dir);

	vec4 color = vec4(0);

	uint node_index;
	uint node_depth;

	bool voxel_found = get_voxel(point, node_index, node_depth, false);

	uint medium = 42069;
	vec4 albedo = voxel_found ? get_albedo(medium) : vec4(0);

	Ray ray = Ray(point, dir, 4000, medium, true);
	RayHit ray_hit;

	float occlusion = 1;
	
	uint last_medium = medium;
	//Albedo
	for (int c_sample = 0; c_sample < 2;  c_sample++ ) {
		bool success = ray_cast(ray, ray_hit);
		if (success) {
			Node node = octree.data[ray_hit.node];
		
			albedo.xyz *= max(pow(ray_hit.dist, 2), 1);
			albedo += get_albedo(node.block);
			float reflectivity = get_reflectivity(node.block);
			if (reflectivity > 0) {
				Ray ref = Ray(ray_hit.destination, ray_hit.reflection, 4000, node.block, false);
				RayHit ref_hit;

				bool ref_success = ray_cast(ref, ref_hit);

				if (ref_success) {
					Node node = octree.data[ref_hit.node];
					albedo += get_albedo(node.block) * reflectivity;
				}
			}
			if (albedo.a < 1) {
				//I don't know how to fix transparency other than making the ray unbounded.
				ray = Ray(ray_hit.destination + ray_hit.refraction * EPSILON, ray_hit.refraction, 4000, node.block, false);
				last_medium = ray.medium;
			} else {
				break;
			}
		} else {
			out_color = vec4(0.0);
			out_occlusion = vec4(0.0);
			return;
		}
	}

	color = vec4(normalize(albedo.xyz), 1);

	//Ambient Occlusion
	vec4 ambient = voxel_ao(
			ray_hit.back_step, 
			abs(ray_hit.normal.zxy), 
			abs(ray_hit.normal.yzx)
			);

	float ao = mix(mix(ambient.z, ambient.w, ray_hit.uv.x), mix(ambient.y, ambient.x, ray_hit.uv.x), ray_hit.uv.y);

	color.xyz = color.xyz - vec3(1 - ao) * 0.25;	


	//Lighting
	vec3 pos = ray_hit.destination;
	vec3 sun_dir = normalize(sun_pos - pos);
	Ray light = Ray(pos, sun_dir, 4000, last_medium, false);
	RayHit light_hit;

	bool light_success = ray_cast(light, light_hit);

	if (light_success) {
		occlusion = light_hit.dist / distance(ray_hit.destination, sun_pos);
	}

	out_color = color;	
	out_occlusion = vec4(occlusion, 0, 0, 1);
}
