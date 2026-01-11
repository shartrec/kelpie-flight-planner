#version 320 es

precision mediump float;

layout (location = 0) in  vec4 vColor;
layout (location = 1) in  vec2 vTexCoord;
layout (location = 2) in  vec3 vWorldPos;
layout (location = 0) out  vec4 f_color;

uniform  float shadow_strength;
uniform  mat4 matrix;
uniform  vec3 sun_direction;
uniform sampler2D texture1;

void main() {

    vec3 pos = normalize(vWorldPos);
    vec3 sunDir = normalize(sun_direction);

    float lightAmount = max(dot(pos, sunDir), 0.0);
    float smoothLight = smoothstep(0.0, 0.10, lightAmount);

    vec4 color = mix(texture(texture1, vTexCoord), vec4(0.0, 0.0, 0.0, 1.0), shadow_strength * (1.0 - smoothLight));
//    vec4 color = texture(texture1, vTexCoord);

    f_color = color;
}
