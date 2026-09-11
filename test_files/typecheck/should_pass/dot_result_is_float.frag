precision highp float;
uniform Cart3.Direction<World> u_Normal;
uniform Cart3.Direction<World> u_LightDir;
void main() {
    float diffuse = dot(u_Normal, u_LightDir);
    float clamped = max(diffuse, 0.0);
    gl_FragColor = vec4(clamped, clamped, clamped, 1.0);
}
