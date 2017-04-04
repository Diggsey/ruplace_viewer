#version 100

varying vec2 v_TexCoord;
uniform sampler2D t_Color;

void main() {
    gl_FragColor = texture2D(t_Color, v_TexCoord);
}
