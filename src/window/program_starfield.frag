#version 320 es

precision mediump float;

layout (location = 0) out vec4 FragColor;

uniform mat3 uInvViewRot;   // inverse camera rotation (no translation)
uniform vec2 uResolution;

// ---------- Hash utilities ----------
// Produce a psuedo random number from the fixed seed of the pixel view ray
float hash1(vec3 p) {
    p = fract(p * 0.3183099 + vec3(0.1, 0.2, 0.3));
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

// Produce a psuedo random vector from the fixed seed of the pixel view ray
vec3 hash3(vec3 p) {
    return vec3(
    hash1(p + 1.0),
    hash1(p + 2.0),
    hash1(p + 3.0)
    );
}

// ---------- Main ----------
void main() {
    // Screen → NDC
    vec2 uv = (gl_FragCoord.xy / uResolution) * 2.0 - 1.0;
    uv.x *= uResolution.x / uResolution.y;

    // View ray
    vec3 dir = normalize(vec3(uv, -1.0));
    dir = uInvViewRot * dir;

    vec3 color = vec3(0.0);

    // ===== Milky Way =====
    // Galactic plane direction (pick something pleasing)
    vec3 galacticNormal = normalize(vec3(0.2, 0.9, 0.4));

    float band = abs(dot(dir, galacticNormal));
    float milky = exp(-band * band * 18.0); // band width

    // Add dusty glow
    vec3 dustColor = vec3(0.9, 0.85, 1.0);
    color += dustColor * milky * 0.15;

    // ===== Stars =====
    // Quantize direction → star cells
    vec3 cell = floor(dir * 900.0);
    float rnd = hash1(cell);

    // Base star density. 5 in 1000 pixels are stars, more in Milky Way
    float threshold = 0.995 - milky * 0.015; // more stars in Milky Way

    if (rnd > threshold) {
        float brightness = smoothstep(threshold, 1.0, rnd);

        // Star color variation (temperature-ish)
        vec3 tint = hash3(cell);
        vec3 starColor = mix(
        vec3(1.0, 0.75, 0.65),   // warm
        vec3(0.65, 0.75, 1.0),   // cool
//        vec3(1.0, 0.9, 0.8),   // warm
//        vec3(0.8, 0.9, 1.0),   // cool
        tint.x
        );

        // Rare bright stars
        brightness *= mix(0.6, 2.5, tint.y * tint.y);

        color += starColor * brightness;
    }

    FragColor = vec4(color, 1.0);
}
