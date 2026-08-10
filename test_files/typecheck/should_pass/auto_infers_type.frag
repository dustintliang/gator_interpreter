precision highp float;
uniform Cart3.Point<World> u_Light;
varying Cart3.Point<World> v_Position;
void main() {
    auto lightDir = u_Light - v_Position;
    Cart3.Point<World> result = lightDir + v_Position;
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}
