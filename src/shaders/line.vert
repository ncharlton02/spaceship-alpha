#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location=0) in vec3 pos1;
layout(location=1) in vec3 pos2;
layout(location=2) in vec3 color;

layout(location=0) out vec3 fragColor;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 viewProjMatrix;
};

void main() {
    if (gl_VertexIndex == 0) {
        gl_Position = viewProjMatrix * vec4(pos1, 1.0);
    } else {
        gl_Position = viewProjMatrix * vec4(pos2, 1.0);
    }

    fragColor = color;
}