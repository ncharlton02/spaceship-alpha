#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;

layout(location = 3) in vec4 model0;
layout(location = 4) in vec4 model1;
layout(location = 5) in vec4 model2;
layout(location = 6) in vec4 model3;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 fPosition;
layout(location = 2) out vec3 fNormal;


layout(set = 0, binding = 0) uniform Transforms {
    mat4 transformMatrix;
};

void main() {
    mat4 model = mat4(model0, model1, model2, model3);
    vec4 position = model * vec4(pos, 1.0);
    gl_Position = transformMatrix * position;
    
    fragColor = color;
    fPosition = pos;
    fNormal = normal;
}