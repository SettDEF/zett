//!HOOK MAIN
//!BIND HOOKED
//!DESC zett logo: folded fluid
// zett's logo, rendered once: a domain-warped flow (curl-like folds, as ink in water), its value cut into sharp
// colour bands that blur a little where the flow folds tightest. Colours: lavender, blue, pink, peach on base.

float h(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float n(vec2 p) { vec2 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f);
    return mix(mix(h(i), h(i + vec2(1, 0)), f.x), mix(h(i + vec2(0, 1)), h(i + vec2(1, 1)), f.x), f.y); }
float fbm(vec2 p) { float v = 0.0, a = 0.5; mat2 r = mat2(0.8, 0.6, -0.6, 0.8); for (int i = 0; i < 6; i++) { v += a * n(p); p = r * p * 2.02; a *= 0.5; } return v; }

vec4 hook() {
    vec2 uv = HOOKED_pos * 2.0 - 1.0;
    // the flow: warped twice, then swirled around the centre (one big fold, as in a stirred fluid)
    vec2 p = uv * 1.6;
    float r = length(uv);
    p = mat2(cos(1.9 * (1.0 - r)), -sin(1.9 * (1.0 - r)), sin(1.9 * (1.0 - r)), cos(1.9 * (1.0 - r))) * p;
    vec2 q = vec2(fbm(p + vec2(0.0, 0.0)), fbm(p + vec2(5.2, 1.3)));
    vec2 s = vec2(fbm(p + 3.4 * q + vec2(1.7, 9.2)), fbm(p + 3.4 * q + vec2(8.3, 2.8)));
    float v = fbm(p + 3.0 * s);
    // how tightly it folds here: the blur where it is tight
    float fold = clamp(length(fwidth(s)) * 60.0, 0.0, 1.0);
    // sharp bands of the value, softened by the fold
    float bands = 5.0, x = v * bands;
    float k = fract(x), edge = mix(0.02, 0.22, fold);
    float t = floor(x) + smoothstep(0.5 - edge, 0.5 + edge, k);
    vec3 pal[5] = vec3[5](vec3(0.118, 0.118, 0.180), vec3(0.537, 0.706, 0.980), vec3(0.796, 0.651, 0.969), vec3(0.961, 0.761, 0.906), vec3(0.980, 0.702, 0.529));
    int i0 = int(clamp(floor(t), 0.0, 4.0)), i1 = int(clamp(floor(t) + 1.0, 0.0, 4.0));
    vec3 c = mix(pal[i0], pal[i1], fract(t));
    // a soft light from the top left, as on a glossy tile
    c += 0.10 * (1.0 - length(uv - vec2(-0.6, -0.6)) * 0.7);
    return vec4(c, 1.0);
}
