#version 450

layout(location = 0) in vec3 inColor;
layout(location = 1) in vec3 position;
layout(location = 2) in vec3 normal;
layout(location = 0) out vec4 outColor;

void main() {
    float ambient = 0.1;
    vec3 lightPos = vec3(-10.0, -10.0, 10.0);

    vec3 normal = normalize(normal);
    vec3 light_dir = normalize(lightPos - position);
    float diffuse_strength = max(dot(normal, light_dir), 0.0);

    vec3 color = inColor * (diffuse_strength + ambient);
    outColor = vec4(color, 1.0);
}