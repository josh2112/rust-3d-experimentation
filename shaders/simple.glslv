#version 400 core

in vec3 a_Pos;
in vec3 a_Normal;
in vec2 a_Uv;

uniform Transforms {
    mat4 u_Model;
    mat4 u_View;
    mat4 u_Proj;
};

uniform Lights {
    vec4 u_LightPos;
    vec4 u_LightColor;
};

out vec2 v_Uv;
out vec3 v_Normal;
out vec3 v_ToLight;

void main() {
    vec4 worldPos = u_Model * vec4( a_Pos, 1.0 );
    
    gl_Position = u_Proj * u_View * worldPos;
    v_Uv = a_Uv;
    v_Normal = (u_Model * vec4( a_Normal, 1.0 )).xyz;
    v_ToLight = (u_LightPos - worldPos).xyz;
}
