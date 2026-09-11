precision highp float;
uniform float u_R;
uniform float u_G;
uniform float u_B;
void main() {
    vec3 color = vec3(u_R, u_G, u_B);
    gl_FragColor = vec4(color, 1.0);
}
