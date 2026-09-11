precision highp float;
uniform float u_Value;
void main() {
    int count = 4;
    float bad = count + u_Value;
    gl_FragColor = vec4(bad, bad, bad, 1.0);
}
