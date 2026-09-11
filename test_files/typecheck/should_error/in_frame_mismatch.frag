precision highp float;
uniform Cart3.Point<Object> u_ModelPos;
varying Cart3.Point<World> v_WorldPos;
void main() {
    in World {
        Cart3.Point<Object> localPos = u_ModelPos;
    }
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}
