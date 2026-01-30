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

// Simple multi-frequency "blotch" noise using hash1
float blotchNoise(vec3 p) {
    float n = 0.0;
    float amp = 0.6;
    float freq = 0.6;
    // 4 octaves, cheap FBM-like
    for (int i = 0; i < 4; ++i) {
        n += amp * hash1(p * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return n;
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
    vec3 baseDustA = vec3(0.9, 0.85, 1.0);
    vec3 baseDustB = vec3(0.65, 0.75, 1.0);   // cool
    vec3 dustColor = mix(baseDustA, baseDustB, milky);

    // ===== Blotchy variation =====
    // Controls: larger scale -> larger blotches; strength -> how much they alter the dust color
    const float blotchScale = 5.0;      // size of blotches on the sky
    const float blotchStrength = 0.6;   // how strong the blotch tint is
    // sample noise in world/view direction; bias by milky so blotches appear mostly in band
    float blot = blotchNoise(dir * blotchScale);
    // remap to a smoother mask
    blot = smoothstep(0.35, 0.85, blot);
    // tint color for dusty patches (slightly darker / cooler)
    vec3 blotchTint = vec3(0.6, 0.6, 0.9);
    // Mix dustColor with blotch tint, scaled by milky and strength
    dustColor = mix(dustColor, mix(dustColor, blotchTint, 0.5), blot * milky * blotchStrength);

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
            tint.x
        );

        // Rare bright stars
        brightness *= mix(0.6, 2.5, tint.y * tint.y);

        color += starColor * brightness;
    }

    FragColor = vec4(color, 1.0);
}
