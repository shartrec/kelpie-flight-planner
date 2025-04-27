#version 320 es

layout (location = 0) in mediump vec3 Position;
layout (location = 0) out mediump vec3 vColor;

uniform mediump mat4 matrix;
uniform mediump vec3 color;
uniform mediump float pointSize;

void main()
{
    gl_Position = matrix * vec4(Position, 1.0);
    gl_PointSize = pointSize;
    vColor = color;
}