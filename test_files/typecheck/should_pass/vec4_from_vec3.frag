precision highp float;
varying Cart3.Point<World> v_Position;
void main() {
    vec4 homPos = vec4(v_Position, 1.0);
    gl_FragColor = homPos;
}
