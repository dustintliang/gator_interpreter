precision highp float;
uniform mat4 u_Model;
uniform mat4 u_World;
uniform mat4 u_ModelWorldInverseTranspose;
uniform mat4 u_Camera;
uniform vec3 u_Light;
uniform float u_SpecPower;
varying vec3 v_Position;
varying vec3 v_Normal;

void main() {
    vec3 worldPosition = vec3(((u_World * u_Model) * vec4(v_Position, 1.0)));
    vec3 worldNormal = normalize(vec3((u_ModelWorldInverseTranspose * vec4(v_Normal, 0.0))));
    vec3 lightDir = normalize((u_Light - worldPosition));
    float diffuse = max(dot(lightDir, worldNormal), 0.0);
    vec3 reflectDir = normalize(reflect(lightDir, worldNormal));
    vec3 cameraReflectDir = vec3((u_Camera * vec4(reflectDir, 0.0)));
    vec3 cameraSpacePosition = vec3((u_Camera * vec4(worldPosition, 1.0)));
    vec3 cameraDir = normalize((vec3(0.0, 0.0, 0.0) - cameraSpacePosition));
    float angle = max(dot(cameraDir, cameraReflectDir), 0.0);
    float specular = max(pow(angle, u_SpecPower), 0.0);
    vec3 diffuseColor = vec3(1.0, 0.3, 0.7);
    vec3 specularColor = vec3(1.0, 1.0, 1.0);
    vec3 ambientColor = vec3(0.0, 0.0, 0.0);
    vec3 color = ((ambientColor + (diffuse * diffuseColor)) + (specular * specularColor));
    gl_FragColor = vec4(color, 1.0);
}
