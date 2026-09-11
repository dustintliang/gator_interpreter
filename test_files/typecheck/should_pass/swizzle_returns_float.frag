precision highp float;
varying Cart3.Point<World> v_Position;
void main() {
    float x = v_Position.x;
    float y = v_Position.y;
    float sum = x + y;
    gl_FragColor = vec4(sum, sum, sum, 1.0);
}
