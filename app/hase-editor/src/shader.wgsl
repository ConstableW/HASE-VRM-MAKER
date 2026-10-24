struct Globals {
    vp: mat4x4<f32>,
    eye: vec4<f32>,
    key_dir: vec4<f32>,
    fill_dir: vec4<f32>,
    sky: vec4<f32>,
    ground: vec4<f32>,
    flags: vec4<f32>, // x: Gamma-Encode (1 = Ziel nicht sRGB), y: Gitter-Zellgröße
};
@group(0) @binding(0) var<uniform> G: Globals;

struct Mat { tint: vec4<f32>, flags: vec4<f32> }; // flags.x: Alpha-Maske, flags.y: hat Textur
@group(1) @binding(0) var<uniform> M: Mat;
@group(1) @binding(1) var tex: texture_2d<f32>;
@group(1) @binding(2) var samp: sampler;

struct VIn { @location(0) pos: vec3<f32>, @location(1) nrm: vec3<f32>, @location(2) uv: vec2<f32> };
struct VOut { @builtin(position) clip: vec4<f32>, @location(0) world: vec3<f32>, @location(1) nrm: vec3<f32>, @location(2) uv: vec2<f32> };

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    o.clip = G.vp * vec4<f32>(v.pos, 1.0);
    o.world = v.pos; o.nrm = v.nrm; o.uv = v.uv;
    return o;
}

fn encode(c: vec3<f32>) -> vec3<f32> {
    if (G.flags.x > 0.5) { return pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2)); }
    return c;
}

@fragment
fn fs_main(in: VOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    var base = M.tint;
    if (M.flags.y > 0.5) {
        let t = textureSample(tex, samp, in.uv);
        base = vec4<f32>(base.rgb * t.rgb, t.a);
    }
    if (M.flags.x > 0.5 && base.a < 0.5) { discard; }
    var n = normalize(in.nrm);
    if (!front) { n = -n; }
    let v = normalize(G.eye.xyz - in.world);
    let key = max(dot(n, normalize(G.key_dir.xyz)), 0.0);
    let kw = key * 0.85 + 0.15 * (dot(n, normalize(G.key_dir.xyz)) * 0.5 + 0.5);
    let fill = max(dot(n, normalize(G.fill_dir.xyz)), 0.0);
    let hemi = mix(G.ground.rgb * 0.5, G.sky.rgb, n.y * 0.5 + 0.5);
    let rim = pow(1.0 - max(dot(n, v), 0.0), 3.0) * 0.18;
    let lit = hemi * 0.55 + vec3<f32>(1.0, 0.97, 0.92) * kw * 0.75 + vec3<f32>(0.55, 0.65, 0.85) * fill * 0.25 + vec3<f32>(rim);
    return vec4<f32>(encode(base.rgb * lit), 1.0);
}

// ---- Bodenraster ----
@fragment
fn fs_ground(in: VOut) -> @location(0) vec4<f32> {
    let p = in.world.xz;
    let cell = G.flags.y;
    let g = abs(fract(p / cell - 0.5) - 0.5) / fwidth(p / cell);
    let line = 1.0 - min(min(g.x, g.y), 1.0);
    let r = length(p);
    let fade = clamp(1.0 - r / 6.0, 0.0, 1.0);
    let base = vec3<f32>(0.20, 0.215, 0.235);
    let col = mix(base, vec3<f32>(0.34, 0.36, 0.40), line * 0.7);
    let axis = select(0.0, 1.0, abs(p.x) < 0.01 || abs(p.y) < 0.01) * 0.0;
    return vec4<f32>(encode(mix(vec3<f32>(0.13, 0.14, 0.16), col, fade) + vec3<f32>(axis)), 1.0);
}
