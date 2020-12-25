#version 450

layout(location = 0) in vec3 inColor;
layout(location = 1) in vec3 position;
layout(location = 2) in vec3 normal;
layout(location = 0) out vec4 outColor;

void main() {
    float ambientStrength = 0.25;
    vec3 lightPos = vec3(3.0, 3.0, 3.0);

    vec3 normal = normalize(normal);
    vec3 lightDirection = normalize(lightPos - position);
    float diffuseStrength = max(dot(normal, lightDirection), 0.0);

    vec3 color = inColor * clamp(diffuseStrength + ambientStrength, 0.4, 1.0);
    outColor = vec4(color, 1.0);
}