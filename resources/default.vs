#version 450

vec3 colors[3] = vec3[](
	vec3(1.0, 0.0, 0.0),
	vec3(0.0, 1.0, 0.0),
	vec3(0.0, 0.0, 1.0)
);

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 frag_color;

void main() {
	gl_Position = vec4(position, 1.0);
	frag_color = colors[gl_VertexIndex];
}
