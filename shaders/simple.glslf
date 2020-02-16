#version 400 core

uniform sampler2D t_Texture;

in vec2 v_Uv;
out vec4 Target0;

void main() {
    Target0 = texture( t_Texture, vec2( v_Uv.x, 1-v_Uv.y ));
}