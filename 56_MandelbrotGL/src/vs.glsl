#version 130
in vec2 position;
in float idx;
varying vec2 vCoord;
void main() {
   gl_Position = vec4((position-0.5)*2.0, 0.0, 1.0);
   vCoord = position;
}
