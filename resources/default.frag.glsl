#version 450

const uint CHUNK_SIZE = 32;

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

float raycast(mat4 true_model, vec3 ray_pos, vec3 dir) {
	float t_min = 0;
	float t_max = 100000;

	//no clue why but this only works if 8 is hardcoded
	//CHUNK_SIZE / 2 = 8
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

uint get_position_mask(ivec3 pos, uint level) {
	float h = pow(2, octree.size);

	uint hierarchy[64];

	for (int i = 0; i < octree.size; i++) {
		h = h / 2;

		bool px = float(pos.x) >= h;
		bool py = float(pos.y) >= h;
		bool pz = float(pos.z) >= h;

		uint ind = 0;

		if (px) {
			ind += 4;
			pos.x -= int(h);
		}

		if (py) {
			ind += 2;
			pos.y -= int(h);
		}

		if (pz) {
			ind += 1;
			pos.z -= int(h);
		}

		hierarchy[i] = 1 << ind;
	}

	return hierarchy[level];
}

void main() {
	mat4 true_model = ubo.model;

	true_model[3].xyz += in_chunk_position * CHUNK_SIZE;

	vec4 near_plane = vec4((gl_FragCoord.xy / (ubo.resolution / 4)) * 2 - 1, 0.1, 1.0);

	near_plane = vec4((inverse(ubo.proj) * near_plane).xy, 0.0, 1.0);

	vec3 camera_position = (inverse(ubo.view) * near_plane).xyz;

	vec3 model_position = (true_model * vec4(in_position, 1.0)).xyz;

	vec3 dir = normalize(model_position - camera_position);

	float obb_dist = raycast(true_model, camera_position, dir);

	vec3 point = camera_position + dir * (obb_dist - 1);

	point = (inverse(true_model) * vec4(point, 1)).xyz;

	point += CHUNK_SIZE / 2;

	dir = (inverse(true_model) * vec4(dir, 0)).xyz;
	dir = normalize(dir);

	vec4 final = vec4(0.1);

	ivec3 map_point = ivec3(floor(point + 0.0)) ;
	vec3 side_dist;
	bvec3 mask;
	vec3 delta_dist;
	int total = 0;

	delta_dist = 1.0 / abs(dir);
	ivec3 ray_step = ivec3(sign(dir));
	side_dist = (sign(dir) * (vec3(map_point) - point) + (sign(dir) * 0.5) + 0.5) * delta_dist; 
	map_point += ivec3(in_chunk_position) * int(CHUNK_SIZE);

	uint stack[64];
	uint stack_index = 0;
	stack[0] = 0;

	bool traverse = true;
	int trav = 0;

	uint size = 64;

	bool early = false;

	while (traverse) {
		bool in_bounds = all(greaterThanEqual(map_point, ivec3(0))) && all(lessThan(map_point, ivec3(size)));
		bool rough_in_bounds = all(greaterThanEqual(map_point, ivec3(-1))) && all(lessThan(map_point, ivec3(size + 1)));
		if (!rough_in_bounds) {
			traverse = false;
			break;
		}

		uint s = size;
		uint h = 0;
		uint ind = 0;
		uint px,py,pz,childindex;
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
			uint m = octree.data[ind].valid & n;
			uint b = bitCount(octree.data[ind].valid & (n - 1));
			
			if (m == n)
			{
				ind = octree.data[ind].child + b;
			}

			x -= px * h;
			y -= py * h;
			z -= pz * h;

			s = h;
		}


		Node current = octree.data[ind];

		if (current.block == 1 && in_bounds) {
			final = vec4(0.5, 1.0, 0.1, 1.0);
			break;
		}

		mask = lessThanEqual(side_dist.xyz, min(side_dist.yzx, side_dist.zxy));
		side_dist += vec3(mask) * delta_dist;
		map_point += ivec3(vec3(mask)) * ray_step;

		trav += 1;
		if (trav > 200) {
			out_final = final;
			return;
		}
	}

	if (mask.x) {
		final.xyz *= vec3(0.5);
	}
	if (mask.y) {
		final.xyz *= vec3(1.0);
	}
	if (mask.z) {
		final.xyz *= vec3(0.75);
	}

	out_final = final;	
}
