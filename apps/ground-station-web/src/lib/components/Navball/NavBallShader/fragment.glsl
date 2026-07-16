uniform sampler2D uTexture;

varying vec2 vUv;
varying vec3 vNormal;
varying vec3 vViewPosition;

void main() {
    vec3 viewDirection = normalize(-vViewPosition);

    float fresnelEffect = pow(1.0 - dot(viewDirection, normalize(vNormal)), 2.0);
    fresnelEffect = clamp(fresnelEffect, 0.0, 1.0);


    vec4 textureColor = texture2D(uTexture, vUv) ;
    vec4 edgeColor = vec4(0.0, 0.0, 0.0, 1.0);

    vec4 finalColor = mix(textureColor, edgeColor, fresnelEffect);

    gl_FragColor = finalColor;
}
