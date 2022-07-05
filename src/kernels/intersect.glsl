#version 430
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(std430) buffer MyBlock {
    vec4 values[];
};

float map(vec3 p) {
    return length(p - vec3(0., 0., 1.)) - 0.3;
}

void main() {
    vec4 val = values[gl_GlobalInvocationID.x];
    if (length(val.xy) < 0.5) {
        values[gl_GlobalInvocationID.x] = val * 2;
    }
}