#version 320 es

layout (location = 0) in mediump vec4 vColor;
layout (location = 0) out mediump vec4 f_color;
void main() {
    f_color = vColor;
}
