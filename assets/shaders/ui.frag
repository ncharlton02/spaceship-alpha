#version 450

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec2 uv;
layout(location = 0) out vec4 color;

layout(set = 1, binding = 0) uniform texture2D texture_;
layout(set = 1, binding = 1) uniform sampler sampler_;

void main() {
    color = inColor * texture(sampler2D(texture_, sampler_), uv);
}