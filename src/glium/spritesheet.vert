#version 140
uniform mat4 matrix;

in vec4 position;
in vec2 tex_coords;

out vec2 v_tex_coords;

void main() {
	gl_Position = matrix * position;
	gl_Position.y = -gl_Position.y;

	v_tex_coords = tex_coords;
	v_tex_coords.y = -v_tex_coords.y;
}
