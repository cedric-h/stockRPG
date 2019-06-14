#version 450

in vec4 position;
in vec2 tex_coords;
in vec2 hole_size;
in vec3 rgb;
out vec2 out_tex_coords;
out vec2 out_hole_size;
out vec3 out_rgb;

mat4 matrix;

void main() {
    out_tex_coords = tex_coords;
    out_hole_size = hole_size;
    out_rgb = rgb;

    gl_Position = matrix * position;
	gl_Position.y = -gl_Position.y;
}
