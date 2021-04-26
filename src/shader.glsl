varying vec3 vFragPos;

#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    vFragPos = vec3(uModel * position);
    return transform_projection * vertex_position;
}
    #endif

#ifdef FRAGMENT
uniform vec3 lightPos;

vec4 effect(vec4 color, Image texture, vec2 st, vec2 screen_coords) {
    vec3 fdx = vec3( dFdx( vFragPos.x ), dFdx( vFragPos.y ), dFdx( vFragPos.z ) );
    vec3 fdy = vec3( dFdy( vFragPos.x ), dFdy( vFragPos.y ), dFdy( vFragPos.z ) );
    vec3 norm = normalize( cross( fdx, fdy ) );

    vec3 c = (norm + 1.) / 2.;
    return vec4(c, 1.);
}
    #endif