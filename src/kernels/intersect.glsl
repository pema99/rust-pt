#version 430
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

buffer RayDirections { vec4 rayDirections[]; };

float map(vec3 p) {
    // metaball
    float d = length(p + vec3(0.4, 0.5, -0.5)) - 0.5;
    d = min(d, length(p + vec3(0.7, 0.2, -0.3)) - 0.4);

    // ball
    d = min(d, length(p + vec3(-0.5, 0.5, 0.3)) - 0.5);
    
    // walls
    d = min(d, abs(p.y+1.0));
    d = min(d, abs(p.y-1.0));
    d = min(d, abs(p.x+1.5));
    d = min(d, abs(p.x-1.5));
    d = min(d, abs(p.z+3.5));
    d = min(d, abs(p.z-2.0));

    return d;
}

vec2 march(vec3 ro, vec3 rd) {
    float t = 0.;
    vec3 p = ro;
    uint i = 0;
    for (; i < 50; i++) {
        p = ro + rd * t;
        float dist = map(p);
        if (t > 10 || dist < 0.001) break;
        t += dist;
    }
    return vec2(t, i);
}

vec3 normal(vec3 p)
{
    vec2 h = vec2(0, 0.0001);
    return normalize(vec3(
        map(p + h.yxx) - map(p - h.yxx),
        map(p + h.xyx) - map(p - h.xyx),
        map(p + h.xxy) - map(p - h.xxy)
    ));
}

void main() {
    vec4 val = rayDirections[gl_GlobalInvocationID.x];
    vec3 ro = vec3(0, 0, -2);
    vec3 rd = val.xyz;

    vec2 res = march(ro, rd);
    float t = res.x;
    float steps = res.y;
    vec3 hit = ro + rd * t;
    vec3 norm = normal(hit);

    vec3 col = vec3(0., 0., 0.);

    if (t < 10) {
        col = norm * 0.5 + 0.5;
    }

    rayDirections[gl_GlobalInvocationID.x].xyz = col;
}