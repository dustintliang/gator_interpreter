precision highp float;
uniform Cart3.Point<World> u_Light;
varying Cart3.Point<World> v_Position;
void main() {
    in World {
        Cart3.Point<World> lightPos = u_Light;
        auto brightness = dot(lightPos, v_Position);
    }
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}
