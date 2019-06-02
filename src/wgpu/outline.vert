#version 450

layout(location = 0) in vec4 a_Pos;
layout(location = 1) in vec2 a_TexCoord;
layout(location = 2) in vec2 a_HoleSize;
layout(location = 3) in vec3 a_Rgb;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec2 v_HoleSize;
layout(location = 2) out vec3 v_Rgb;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_Transform;
};

void main() {
    v_TexCoord = a_TexCoord;
    v_HoleSize = a_HoleSize;
    v_Rgb = a_Rgb;
    gl_Position = u_Transform * a_Pos;
    // convert from -1,1 Z to 0,1
}
