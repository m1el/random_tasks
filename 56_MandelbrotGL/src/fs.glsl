#version 130
uniform sampler2D uTexture;
varying vec2 vCoord;
out vec4 out_color;
void main() {
   out_color = texture2D(uTexture, vCoord);
}
