#version 400 core

in vec2 v_Uv;
in vec3 v_Normal;
in vec3 v_ToLight;

uniform sampler2D t_Texture;

uniform Lights {
    vec4 u_LightPos;
    vec4 u_LightColor;
};

out vec4 Target0;

void main() {
    float brightness = max( 0.0, dot( normalize( v_Normal ), normalize( v_ToLight )));
    vec3 diffuse = brightness *  u_LightColor.xyz;
    Target0 = vec4( diffuse, 1.0 ) * texture( t_Texture, vec2( v_Uv.x, 1-v_Uv.y ));
}