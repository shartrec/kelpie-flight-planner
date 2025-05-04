#version 320 es

precision mediump float;

layout (location = 0) in vec3 aPos;
layout (location = 0) out vec4 vColor;
layout (location = 1) out vec3 vWorldPos;

uniform mat4 matrix;
uniform vec4 color4;
uniform float pointSize;

void main()
{
    vec4 worldPos = vec4(aPos, 1.0);
    vWorldPos = worldPos.xyz;
    vColor = color4;
    gl_Position = matrix * vec4(aPos, 1.0);
    gl_PointSize = pointSize;
}