#version 450

out gl_PerVertex {
    vec4 gl_Position;
}; 

layout(location=0) in vec2 pos;
layout(location=1) in vec2 size;
layout(location=2) in vec2 uvPos;
layout(location=3) in vec2 uvSize;
layout(location=4) in vec4 color;

layout(location=0) out vec4 fragColor;
layout(location=1) out vec2 fragUvs;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 viewProjMatrix;
};

void main() {
    vec2 fragPos = vec2(0.0, 0.0);
    
    if(gl_VertexIndex == 0){
        fragPos = vec2(pos);
        fragUvs = vec2(uvPos);
    }else if(gl_VertexIndex == 1 || gl_VertexIndex == 3) {
        fragPos = vec2(pos.x, pos.y + size.y);
        fragUvs = vec2(uvPos.x, uvPos.y + uvSize.y);
    }else if(gl_VertexIndex == 2 || gl_VertexIndex == 5) {
        fragPos = vec2(pos.x + size.x, pos.y);
        fragUvs = vec2(uvPos.x + uvSize.x, uvPos.y);
    }else if(gl_VertexIndex == 4) {
        fragPos = vec2(pos.x + size.x, pos.y + size.y);
        fragUvs = vec2(uvPos.x + uvSize.x, uvPos.y + uvSize.y);
    }

    gl_Position = viewProjMatrix * vec4(fragPos, 0.0, 1.0);
    fragColor = color;
}