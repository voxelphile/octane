#version 450

#define SCALE 4

#define ENABLE_NEAREST_NEIGHBOR false
#define ENABLE_HQ4X true

layout(binding = 0) uniform RenderSettings {
	vec2 resolution;
	uint render_distance;
} settings;

layout(binding = 1, rgba32f) uniform image2D source_color;
layout(binding = 2) uniform sampler2D look_up_table;

layout(location = 0) out vec4 out_final;

const mat3 yuv_matrix = mat3(0.299, 0.587, 0.114, -0.169, -0.331, 0.5, 0.5, -0.419, -0.081);
const vec3 yuv_threshold = vec3(48.0/255.0, 7.0/255.0, 6.0/255.0);
const vec3 yuv_offset = vec3(0, 0.5, 0.5);

bool diff(vec3 yuv1, vec3 yuv2)
{
	bvec3 res = greaterThan(abs((yuv1 + yuv_offset) - (yuv2 + yuv_offset)), yuv_threshold);
	return res.x || res.y || res.z;
}

vec3 hq4x(vec4 tex_coords[4], vec2 texture_size) {
	vec2 ps = 1.0 / texture_size;

	float dx = ps.x;
	float dy = ps.y;
	
	vec2 fp = fract(tex_coords[0].xy*texture_size);
	vec2 quad = sign(-0.5 + fp);
	mat3 yuv = transpose(yuv_matrix);
	
	vec3 p1  = imageLoad(source_color, ivec2(tex_coords[0].xy  * texture_size)).rgb;
	vec3 p2  = imageLoad(source_color, ivec2((tex_coords[0].xy + vec2(dx, dy) * quad)  * texture_size)).rgb;
	vec3 p3  = imageLoad(source_color, ivec2((tex_coords[0].xy + vec2(dx, 0) * quad)  * texture_size)).rgb;
	vec3 p4  = imageLoad(source_color, ivec2((tex_coords[0].xy + vec2(0, dy) * quad)  * texture_size)).rgb;
	mat4x3 pixels = mat4x3(p1, p2, p3, p4);

	vec3 w1  = yuv * imageLoad(source_color, ivec2(tex_coords[1].xw * texture_size)).rgb;
	vec3 w2  = yuv * imageLoad(source_color, ivec2(tex_coords[1].yw * texture_size)).rgb;
	vec3 w3  = yuv * imageLoad(source_color, ivec2(tex_coords[1].zw * texture_size)).rgb;

	vec3 w4  = yuv * imageLoad(source_color, ivec2(tex_coords[2].xw * texture_size)).rgb;
	vec3 w5  = yuv * p1;
	vec3 w6  = yuv * imageLoad(source_color, ivec2(tex_coords[2].zw * texture_size)).rgb;

	vec3 w7  = yuv * imageLoad(source_color, ivec2(tex_coords[3].xw * texture_size)).rgb;
	vec3 w8  = yuv * imageLoad(source_color, ivec2(tex_coords[3].yw * texture_size)).rgb;
	vec3 w9  = yuv * imageLoad(source_color, ivec2(tex_coords[3].zw * texture_size)).rgb;

	bvec3 pattern[3];
	pattern[0] =  bvec3(diff(w5, w1), diff(w5, w2), diff(w5, w3));
	pattern[1] =  bvec3(diff(w5, w4), false       , diff(w5, w6));
	pattern[2] =  bvec3(diff(w5, w7), diff(w5, w8), diff(w5, w9));
	bvec4 cross = bvec4(diff(w4, w2), diff(w2, w6), diff(w8, w4), diff(w6, w8));

	vec2 index;
	index.x = dot(vec3(pattern[0]), vec3(1, 2, 4)) +
		dot(vec3(pattern[1]), vec3(8, 0, 16)) +
		dot(vec3(pattern[2]), vec3(32, 64, 128));
	index.y = dot(vec4(cross), vec4(1, 2, 4, 8)) * (SCALE * SCALE) +
		dot(floor(fp * SCALE), vec2(1, SCALE));

	vec2 step = 1.0 / vec2(256.0, 16.0 * (SCALE * SCALE));
	vec2 offset = step / 2.0;
	vec4 weights = texture(look_up_table, index * step + offset);	
	float sum = dot(weights, vec4(1));
	vec3 res = pixels * (weights / sum);
	return res;
}

void main() {

	vec2 uv_coords = gl_FragCoord.xy / settings.resolution;

	vec4 tex_coords[4];

	vec2 texture_size = settings.resolution / SCALE;
	
	vec2 ps = 1.0 / texture_size;
	
	float dx = ps.x;
	float dy = ps.y;

	tex_coords[0].zw = ps;
	tex_coords[0].xy = uv_coords.xy;
	tex_coords[1] = uv_coords.xxxy + vec4(-dx, 0, dx, -dy);
	tex_coords[2] = uv_coords.xxxy + vec4(-dx, 0, dx,   0);
	tex_coords[3] = uv_coords.xxxy + vec4(-dx, 0, dx,  dy);

	vec3 res;

	if (ENABLE_NEAREST_NEIGHBOR) {
		res = imageLoad(source_color, ivec2(gl_FragCoord.xy / SCALE)).rgb;
	} else if (ENABLE_HQ4X) {
		res = hq4x(tex_coords, texture_size);
	} else {
		res = vec3(1, 0, 1);
	}


	out_final = vec4(res, 1.0);
}
