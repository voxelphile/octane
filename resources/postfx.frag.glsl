#version 450

layout(origin_upper_left) in vec4 gl_FragCoord;

layout(binding = 0) uniform UniformBufferObject {
	mat4 model;
	mat4 view;
	mat4 proj;
	vec2 resolution;
	uint render_distance;
} ubo;

layout(binding = 1, rgba32f) uniform image2D source_color;
layout(binding = 2, rgba32f) uniform image2D source_occlusion;
layout(binding = 3) uniform sampler2D source_depth;

layout(location = 0) out vec4 out_color;

void main() {
	vec2 uv_coords = gl_FragCoord.xy / (ubo.resolution / 4);
	ivec2 pixel = ivec2(gl_FragCoord.xy);

	vec4 color = imageLoad(source_color, pixel);

	float occlusion = clamp(imageLoad(source_occlusion, pixel).x, 0, 1); 

	float depth = clamp(texture(source_depth, uv_coords).x, 0.1, 1);

	//SHADOWS
	int sample_size = int(((1 - occlusion) * 5) / depth);

	vec2 nearest_lit = vec2(sample_size * sample_size);
	float nearest_lit_depth = 0;

	for (int i = -sample_size / 2; i < sample_size / 2; i++) {
		for (int j = -sample_size / 2; j < sample_size / 2; j++) {
			ivec2 pos = ivec2(pixel) + ivec2(i, j);

			float pos_occ = imageLoad(source_occlusion, pos).x;

			float ijdist = distance(vec2(0), vec2(i, j));
			float middledist = distance(vec2(0), nearest_lit);

			if (pos_occ == 1 && (nearest_lit == vec2(sample_size * sample_size) || ijdist < middledist)){ 
				nearest_lit = vec2(i, j);

				nearest_lit_depth = texture(source_depth, vec2(pos)).x;
			}
		}
	}

	sample_size /= 2;

	float depth_offset = (depth - nearest_lit_depth) * (depth - nearest_lit_depth);
	float sample_max = (sample_size * sample_size) + (sample_size * sample_size) + depth_offset;
	float sample_value = (nearest_lit.x * nearest_lit.x) + (nearest_lit.y * nearest_lit.y) + depth_offset;

	
	float shadow = sqrt(sample_value) / sqrt(sample_max);

	if (occlusion == 1) {
		shadow = 0;
	}

	color.xyz *= max(1 - shadow, 0) * 0.5 + 0.5;
	out_color = color;
}
