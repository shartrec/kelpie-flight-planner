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

    // Fetch base texture color
    vec4 texColor = texture(texture1, vTexCoord);

    // Strengthen the perceived day/night contrast by both darkening nightside and brightening dayside.
    // 'shadow_strength' now acts like a contrast control in the [0..1] range.
    float contrast = clamp(shadow_strength, 0.0, 1.0);

    // Night side: dim the texture; Day side: boost the texture
    vec3 nightColor = texColor.rgb * (1.0 - contrast);
    vec3 dayColor   = texColor.rgb * (1.0 + contrast);

    // Blend based on smoothed lighting amount
    vec3 litColor = mix(nightColor, dayColor, smoothLight);

    // Mild gamma lift to keep midtones visible on dark textures
    litColor = pow(litColor, vec3(1.0/1.1));

    f_color = vec4(clamp(litColor, 0.0, 1.0), texColor.a);
}
