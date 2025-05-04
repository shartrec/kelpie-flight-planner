#version 320 es

precision mediump float;

layout (location = 0) in mediump vec4 vColor;
layout (location = 1) in mediump vec3 vWorldPos;
layout (location = 0) out mediump vec4 f_color;

uniform mediump mat4 matrix;
uniform mediump vec3 sun_direction;

void main() {

    vec3 pos = normalize(vWorldPos);
    vec3 sunDir = normalize(sun_direction);

    float lightAmount = max(dot(pos, sunDir), 0.0);

    vec4 dayColor = vec4(0.0, 0.0, 0.0, 0.0);
    vec4 nightColor = vec4(0.0, 0.0, 0.0, 0.55);

    float smoothLight = smoothstep(0.0, 0.10, lightAmount);

    vec4 color = mix(nightColor, dayColor, smoothLight);

    f_color = color;
}
