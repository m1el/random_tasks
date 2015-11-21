#version 130
in vec2 position;
in float idx;
varying vec4 color;
void main() {
   gl_Position = vec4(position, 0.0, 1.0);
   if (idx == 0.0) {
       color = vec4(1.0, 0.0, 0.0, 1.0);
   }
   if (idx == 1.0) {
       color = vec4(0.0, 1.0, 0.0, 1.0);
   }
   if (idx == 2.0) {
       color = vec4(0.0, 0.0, 1.0, 1.0);
   }
}
