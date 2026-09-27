#version 440

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float time;
    float bass;
} ubuf;

layout(binding = 1) uniform sampler2D source;

vec2 warped(vec2 uv, float phase)
{
    vec2 centered = uv - 0.5;
    float radius = length(centered);
    float angle = atan(centered.y, centered.x);
    angle += sin(radius * 9.0 - ubuf.time * 0.55 + phase) * 0.42;
    radius *= 0.72 + 0.16 * sin(angle * 3.0 + ubuf.time * 0.35 + phase);
    vec2 bent = vec2(cos(angle), sin(angle)) * radius + 0.5;
    bent.x += sin(uv.y * 7.0 + ubuf.time * 0.42 + phase) * 0.13;
    bent.y += cos(uv.x * 6.0 - ubuf.time * 0.36 + phase) * 0.13;
    return fract(bent);
}

void main()
{
    vec2 uv = qt_TexCoord0;
    float bass = clamp(ubuf.bass, 0.0, 1.0);
    vec2 centerA = vec2(0.26 + 0.05 * sin(ubuf.time * 0.3), 0.35);
    vec2 centerB = vec2(0.72, 0.66 + 0.05 * cos(ubuf.time * 0.25));
    float regionA = 1.0 - smoothstep(0.08, 0.52, distance(uv, centerA));
    float regionB = 1.0 - smoothstep(0.10, 0.48, distance(uv, centerB));

    vec2 localA = centerA + (uv - centerA) * (1.0 - bass * regionA * 0.16);
    vec2 localB = centerB + (uv - centerB) * (1.0 - bass * regionB * 0.13);
    vec3 a = texture(source, warped(localA, 0.0)).rgb;
    vec3 b = texture(source, warped(localB.yx, 2.1)).rgb;
    vec3 c = texture(source, warped(1.0 - uv, 4.2)).rgb;

    float blendA = 0.5 + 0.5 * sin((uv.x + uv.y) * 5.0 + ubuf.time * 0.4);
    float blendB = 0.5 + 0.5 * cos((uv.x - uv.y) * 6.0 - ubuf.time * 0.3);
    vec3 color = mix(a, b.brg, blendA * 0.42);
    color = mix(color, c.gbr, blendB * 0.28);

    float luminance = dot(color, vec3(0.2126, 0.7152, 0.0722));
    color = mix(vec3(luminance), color, 1.14);
    float localPulse = bass * (regionA * 0.20 + regionB * 0.16);
    color = mix(color, color * vec3(1.10, 1.04, 1.13), localPulse);
    color += vec3(0.035, 0.025, 0.045) * localPulse;
    fragColor = vec4(color, 1.0) * ubuf.qt_Opacity;
}
