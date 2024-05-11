#version 320 es

layout (location = 0) in mediump vec3 vColor;
layout (location = 0) out mediump vec4 f_color;
void main() {
    f_color = vec4(vColor, 1.0);
}
