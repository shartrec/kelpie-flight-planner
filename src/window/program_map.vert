#version 320 es

precision mediump float;

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

layout (location = 0) out vec4 vColor;
layout (location = 1) out vec2 vTexCoord;
layout (location = 2) out vec3 vWorldPos;

uniform mat4 matrix;
uniform vec3 color;
uniform float pointSize;

void main()
{
    vec4 worldPos = vec4(aPos, 1.0);
    vWorldPos = worldPos.xyz;
    vColor = vec4(color, 1.0);
    vTexCoord = aTexCoord;
    gl_Position = matrix * vec4(aPos, 1.0);
    gl_PointSize = pointSize;
}