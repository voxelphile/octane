#version 450

#define CHUNK_SIZE 32
#define MAX_STEP_COUNT 512
#define EPSILON 1e-2
#define PI 3.14159265359

layout(early_fragment_tests) in;

layout(binding = 0) uniform UniformBufferObject {
	mat4 model;
	mat4 view;
	mat4 proj;
	vec2 resolution;
	uint render_distance;
} ubo;

struct Node {
	uint child;
	uint valid;
	uint block;
};

layout( binding = 1) buffer OctreeBuffer {
	uint size;
	uint len;
	Node data[];
} octree;

//layout(binding = 2, r16ui) uniform uimage3D cubelet_sdf;

layout(location = 0) in vec3 in_uvw;
layout(location = 1) in vec3 in_position;
layout(location = 2) in flat uvec3 in_chunk_position;

layout(location = 0) out vec4 out_final;

int   seed = 1;
void  srand(int s ) { seed = s; }
int   rand(void)  { seed=seed*0x343fd+0x269ec3; return (seed>>16)&32767; }
float frand(void) { return float(rand())/32767.0; }

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

bool get_voxel(vec3 position, out uint node_index) {
	uint size = 64;
	
	ivec3 map_point = ivec3(floor(position + 0.0)) ;
	
	uint s = size;
	uint h = 0;
	uint pre_node_index = 0;
	uint px,py,pz;
	uint x = map_point.x;
	uint y = map_point.y;
	uint z = map_point.z;

	for (int i = 0; i < octree.size; i++) {
		h = s / 2;

		px = uint(x >= h);
		py = uint(y >= h);
		pz = uint(z >= h);
		uint k = px * 4 + py * 2 + pz;
		uint n = 1 << k;
		uint m = octree.data[pre_node_index].valid & n;
		uint b = bitCount(octree.data[pre_node_index].valid & (n - 1));

		if (m == n)
		{
			pre_node_index = octree.data[pre_node_index].child + b;
		} else {
			return false;
		}

		x -= px * h;
		y -= py * h;
		z -= pz * h;

		s = h;
	}

	node_index = pre_node_index;
	return true;
}

float vertex_ao(vec2 side, float corner) {
	return (side.x + side.y + max(corner, side.x * side.y)) / 3.0;
}

vec4 voxel_ao(vec3 pos, vec3 d1, vec3 d2) {
	uint _;

	vec4 side = vec4(
			float(get_voxel(pos + d1, _)), 
			float(get_voxel(pos + d2, _)), 
			float(get_voxel(pos - d1, _)), 
			float(get_voxel(pos - d2, _))
			);

	vec4 corner = vec4(
			float(get_voxel(pos + d1 + d2, _)), 
			float(get_voxel(pos - d1 + d2, _)), 
			float(get_voxel(pos - d1 - d2, _)), 
			float(get_voxel(pos + d1 - d2, _))
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
	float bmin = -16;
	float bmax =  16;

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
};

struct RayHit {
	uint node;
	vec3 destination;
	vec3 back_step;
	vec3 normal;
	vec3 reflection;
	vec2 uv;
	float dist;
};

bool ray_cast(Ray ray, out RayHit hit) {
	ray.direction = normalize(ray.direction);

	ivec3 map_point = ivec3(floor(ray.origin + 0.0)) ;
	vec3 delta_dist = 1.0 / abs(ray.direction);
	ivec3 ray_step = ivec3(sign(ray.direction));
	vec3 side_dist = (sign(ray.direction) * (vec3(map_point) - ray.origin) + (sign(ray.direction) * 0.5) + 0.5) * delta_dist; 
	bvec3 mask;

	map_point += CHUNK_SIZE * ivec3(in_chunk_position);

	uint size = 64;

	for (int step_count = 0; step_count < MAX_STEP_COUNT; step_count++) {
		bool in_bounds = all(greaterThanEqual(map_point, ivec3(0))) && all(lessThan(map_point, ivec3(size)));
		bool rough_in_bounds = all(greaterThanEqual(map_point, ivec3(-1))) && all(lessThan(map_point, ivec3(size + 1)));
		if (!rough_in_bounds) {
			return false;
		}


		float dist = length(vec3(mask) * (side_dist - delta_dist));

		if (dist > ray.max_dist) {
			return false;
		}

		uint node_index;

		bool voxel_found = get_voxel(vec3(map_point), node_index);

		if (voxel_found) {
			Node current = octree.data[node_index];

			if (current.block != 42069 && in_bounds) {
				hit.node = node_index;	
				hit.destination = ray.origin + ray.direction * dist;
				hit.back_step = map_point - ray_step * vec3(mask);
				hit.normal = vec3(mask) * sign(-ray.direction);
				hit.reflection = ray.direction - 2 * dot(ray.direction, hit.normal) * hit.normal;
				hit.uv = mod(vec2(dot(vec3(mask) * hit.destination.yzx, vec3(1.0)), dot(vec3(mask) * hit.destination.zxy, vec3(1.0))), vec2(1.0));
				hit.dist = dist;
				return true;
			}
		}

		mask = lessThanEqual(side_dist.xyz, min(side_dist.yzx, side_dist.zxy));
		side_dist += vec3(mask) * delta_dist;
		map_point += ivec3(vec3(mask)) * ray_step;
	}

	return false;
}

vec3 hemisphere_point(vec3 normal)
{
	float theta = 2.0 * PI * frand();
	float cosPhi = frand();
	float sinPhi = sqrt(1.0-cosPhi*cosPhi);

	vec3 zAxis = normal;
	vec3 xAxis = normalize(cross(normal, vec3(1.0, 0.0, 0.0)));
	vec3 yAxis = normalize(cross(normal, xAxis));

	vec3 x = cos(theta) * xAxis;
	vec3 y = sin(theta) * yAxis;
	vec3 horizontal = normalize(x + y) * sinPhi;
	vec3 z = cosPhi * zAxis;
	vec3 p = horizontal + z;

	return p;
}

void main() {
	mat4 true_model = ubo.model;

	true_model[3].xyz += in_chunk_position * CHUNK_SIZE;

	vec4 near_plane = vec4((gl_FragCoord.xy / (ubo.resolution / 4)) * 2 - 1, 0.1, 1.0);

	near_plane = vec4((inverse(ubo.proj) * near_plane).xy, 0.0, 1.0);

	vec3 camera_position = (inverse(ubo.view) * near_plane).xyz;

	vec3 model_position = (true_model * vec4(in_position, 1.0)).xyz;

	vec3 dir = normalize(model_position - camera_position);

	float obb_dist = jump_cast(true_model, camera_position, dir);

	vec3 point = camera_position + dir * (obb_dist - 1);

	point = (inverse(true_model) * vec4(point, 1)).xyz;

	point += CHUNK_SIZE / 2;

	dir = (inverse(true_model) * vec4(dir, 0)).xyz;
	dir = normalize(dir);

	vec4 final = vec4(0.1);

	Ray initial_ray = Ray(point, dir, 200);
	RayHit initial_ray_hit;

	bool initial_success = ray_cast(initial_ray, initial_ray_hit);

	//Albedo
	if (initial_success) {
		final = vec4(0, 1, 0, 1);
	} else {
		out_final = final;
		return;
	}

	//Ambient Occlusion
	vec4 ambient = voxel_ao(
			initial_ray_hit.back_step, 
			abs(initial_ray_hit.normal.zxy), 
			abs(initial_ray_hit.normal.yzx)
			);

	float ao = mix(mix(ambient.z, ambient.w, initial_ray_hit.uv.x), mix(ambient.y, ambient.x, initial_ray_hit.uv.x), initial_ray_hit.uv.y);

	final.xyz = final.xyz - vec3(1 - ao) * 0.25;

	out_final = final;	
}
