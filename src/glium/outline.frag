#version 140

in vec2 coord;
in vec2 hole_size;
in vec3 rgb;
out vec4 color;

void main() {
    if (abs(coord.x) < hole_size.x && abs(coord.y) < hole_size.y) {
		discard;
    }
    color = vec4(rgb, 1.0);
}
