#version 330

layout (location = 0) in vec3 Position;
out vec3 vColor;

uniform mat4 matrix;
uniform vec3 color;

void main()
{
    gl_Position = matrix * vec4(Position, 1.0);
    vColor = color;
}