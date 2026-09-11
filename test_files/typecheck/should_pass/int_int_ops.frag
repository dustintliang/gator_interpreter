precision highp float;
uniform float u_Scale;
void main() {
    int count = 4;
    int doubled = count + count;
    float result = u_Scale * u_Scale;
    gl_FragColor = vec4(result, result, result, 1.0);
}
