precision highp float;
uniform Cart3.Point<World> u_Light;
varying Cart3.Point<World> v_Position;
void main() {
    Cart3.Direction<World> lightDir = normalize(u_Light - v_Position);
    float brightness = dot(lightDir, lightDir);
    gl_FragColor = vec4(brightness, brightness, brightness, 1.0);
}
