precision highp float;
uniform mat4 u_Matrix;
varying vec4 v_Position;
void main() {
    Hom4.Matrix<Object,World> typed = u_Matrix as! Hom4.Matrix<Object,World>;
    gl_FragColor = v_Position;
}
