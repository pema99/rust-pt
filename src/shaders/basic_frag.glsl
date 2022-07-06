#version 140

in vec2 v_tex_coords;
out vec4 color;
uniform sampler2D framebuffer;

void main() {
    vec2 uv = v_tex_coords.xy*0.5+0.5;
    color = texture(framebuffer, uv);
}