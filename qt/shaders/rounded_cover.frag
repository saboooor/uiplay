#version 440

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float cornerRadius;
} ubuf;

layout(binding = 1) uniform sampler2D source;

void main()
{
    vec2 uv = qt_TexCoord0;
    float radius = clamp(ubuf.cornerRadius, 0.0, 0.5);
    vec2 corner = abs(uv - vec2(0.5)) - vec2(0.5 - radius);
    float distanceToEdge = length(max(corner, vec2(0.0)))
                           + min(max(corner.x, corner.y), 0.0) - radius;
    float feather = max(fwidth(distanceToEdge), 0.001);
    float alpha = 1.0 - smoothstep(-feather, feather, distanceToEdge);
    fragColor = texture(source, uv) * (alpha * ubuf.qt_Opacity);
}
