#version 450

layout(location = 0) in vec2 coord;
layout(location = 1) in vec2 hole_size;
layout(location = 2) in vec3 rgb;
layout(location = 0) out vec4 color;

void main() {
    if (abs(coord.x) < hole_size.x && abs(coord.y) < hole_size.y) {
	discard;
    }
    color = vec4(rgb, 1.0);
}
