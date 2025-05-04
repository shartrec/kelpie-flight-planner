#version 320 es

precision mediump float;

layout (location = 0) in  vec4 vColor;
layout (location = 1) in  vec3 vWorldPos;
layout (location = 0) out  vec4 f_color;

uniform  float shadow_strength;
uniform  mat4 matrix;
uniform  vec3 sun_direction;

void main() {

    vec3 pos = normalize(vWorldPos);
    vec3 sunDir = normalize(sun_direction);

    float lightAmount = max(dot(pos, sunDir), 0.0);
    float smoothLight = smoothstep(0.0, 0.10, lightAmount);

    vec4 color = mix(vColor, vec4(0.0, 0.0, 0.0, 1.0), shadow_strength * (1.0 - smoothLight));

    f_color = color;
}
