struct Globals {
    vp: mat4x4<f32>,
    eye: vec4<f32>,
    key_dir: vec4<f32>,
    fill_dir: vec4<f32>,
    sky: vec4<f32>,
    ground: vec4<f32>,
    flags: vec4<f32>, // x: Gamma-Encode (1 = Ziel nicht sRGB), y: Gitter-Zellgröße, z: Boden/Raster sichtbar
    key_col: vec4<f32>,
    bg_top: vec4<f32>,
    bg_bot: vec4<f32>,
};
@group(0) @binding(0) var<uniform> G: Globals;

struct Mat { tint: vec4<f32>, flags: vec4<f32>, p: array<vec4<f32>, 6> }; // flags: x Alpha-Maske, y hat Textur, z Art (0 Standard, 1 Auge, 2 Haar, 3 Haut), w Glanz
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

fn hash11(x: f32) -> f32 { return fract(sin(x * 127.1) * 43758.5453); }

const EYE_C0: vec2<f32> = vec2<f32>(0.705, 0.296);
const EYE_C1: vec2<f32> = vec2<f32>(0.290, 0.710);

@fragment
fn fs_main(in: VOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let kind = i32(M.flags.z + 0.5);
    var base = M.tint;
    var spec = 0.0;
    var spec_pow = 32.0;
    var n = normalize(in.nrm);
    if (!front) { n = -n; }
    let v = normalize(G.eye.xyz - in.world);
    let kd = normalize(G.key_dir.xyz);
    let h = normalize(kd + v);
    if (kind == 1) {
        // Auge: Iris-/Pupillensteuerung je Auge (links = Welt-x > 0), Texturraum um das nächste Iriszentrum
        let c = select(EYE_C1, EYE_C0, distance(in.uv, EYE_C0) < distance(in.uv, EYE_C1));
        let left = in.world.x > 0.0;
        let inner = select(M.p[2].rgb, M.p[0].rgb, left);
        let psc = select(M.p[2].w, M.p[0].w, left);
        let outer = select(M.p[3].rgb, M.p[1].rgb, left);
        let custom = select(M.p[3].w, M.p[1].w, left);
        let ri = M.p[4].x; let rp = M.p[4].y;
        let d = in.uv - c;
        let r = length(d);
        var r2 = r;
        let rps = rp * psc;
        if (r < ri) {
            if (r < rps) { r2 = r / max(psc, 0.05); } else { r2 = rp + (r - rps) * (ri - rp) / max(ri - rps, 1e-4); }
        }
        let uv2 = select(in.uv, c + d / max(r, 1e-6) * r2, r > 1e-6);
        let t = textureSample(tex, samp, uv2);
        var col = t.rgb;
        if (custom > 0.5) {
            let l = clamp(dot(col, vec3<f32>(0.30, 0.59, 0.11)) / 0.26, 0.25, 1.9);
            let irisc = mix(inner, outer, smoothstep(rp * 1.1, ri, r2)) * l;
            let m = 1.0 - smoothstep(ri * 0.93, ri * 1.02, r2);
            let pupil = 1.0 - smoothstep(rp * 0.9, rp * 1.12, r2);
            col = mix(col, mix(irisc, vec3<f32>(0.01), pupil), m);
        }
        // Limbus (dunkler Ring) für Tiefe
        let limb = smoothstep(ri * 0.80, ri * 1.0, r2) * (1.0 - smoothstep(ri * 1.0, ri * 1.1, r2)) * M.p[5].x;
        col = col * (1.0 - limb * 0.6);
        base = vec4<f32>(col * base.rgb, t.a);
        if (M.flags.x > 0.5 && t.a < 0.5) { discard; }
        spec = M.p[4].w; spec_pow = 220.0;
    } else {
        if (M.flags.y > 0.5) {
            let t = textureSample(tex, samp, in.uv);
            base = vec4<f32>(base.rgb * t.rgb, t.a);
        }
        if (M.flags.x > 0.5 && base.a < 0.5) { discard; }
        if (kind == 2 && M.flags.y > 0.5) {
            // Haar: Verlauf Ansatz→Spitze (Welt-Höhe), Strähnen-Highlights, Textur als Helligkeitsdetail
            let t = textureSample(tex, samp, in.uv);
            let luma = dot(t.rgb, vec3<f32>(0.30, 0.59, 0.11));
            let tt = clamp((M.p[0].w - in.world.y) / max(M.p[0].w - M.p[1].w, 1e-3), 0.0, 1.0);
            var g = mix(M.p[0].rgb, M.p[1].rgb, smoothstep(0.0, 1.0, tt));
            let strand = floor(in.uv.x * 90.0 + floor(in.uv.y * 3.0) * 7.0);
            if (hash11(strand) > 0.82) { g = mix(g, M.p[2].rgb, M.p[2].w); }
            base = vec4<f32>(g * (0.30 + luma * 1.25), t.a);
            if (M.flags.x > 0.5 && base.a < 0.5) { discard; }
            spec = M.flags.w * 0.9; spec_pow = 28.0;
        } else if (kind == 3) {
            spec = M.flags.w * 0.35; spec_pow = 18.0;
        }
    }
    let key = max(dot(n, kd), 0.0);
    let kw = key * 0.85 + 0.15 * (dot(n, kd) * 0.5 + 0.5);
    let fill = max(dot(n, normalize(G.fill_dir.xyz)), 0.0);
    let hemi = mix(G.ground.rgb * 0.5, G.sky.rgb, n.y * 0.5 + 0.5);
    let rim = pow(1.0 - max(dot(n, v), 0.0), 3.0) * 0.18;
    let lit = hemi * 0.55 + G.key_col.rgb * kw * 0.75 + vec3<f32>(0.55, 0.65, 0.85) * fill * 0.25 + vec3<f32>(rim);
    let sp = pow(max(dot(n, h), 0.0), spec_pow) * spec * step(0.0, key);
    return vec4<f32>(encode(base.rgb * lit + G.key_col.rgb * sp), 1.0);
}

// ---- Hintergrundverlauf (Vollbilddreieck) ----
@vertex
fn vs_bg(@builtin(vertex_index) i: u32) -> VOut {
    var o: VOut;
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    o.clip = vec4<f32>(p * 2.0 - 1.0, 0.999, 1.0);
    o.world = vec3<f32>(0.0); o.nrm = vec3<f32>(0.0, 1.0, 0.0); o.uv = vec2<f32>(p.x, 1.0 - p.y);
    return o;
}
@fragment
fn fs_bg(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(encode(mix(G.bg_top.rgb, G.bg_bot.rgb, in.uv.y)), 1.0);
}

// ---- Bodenraster ----
@fragment
fn fs_ground(in: VOut) -> @location(0) vec4<f32> {
    let p = in.world.xz;
    let cell = G.flags.y;
    let g = abs(fract(p / cell - 0.5) - 0.5) / fwidth(p / cell);
    let line = 1.0 - min(min(g.x, g.y), 1.0);
    let r = length(p);
    let fade = clamp(1.0 - r / 6.0, 0.0, 1.0) * G.flags.z;
    let base = vec3<f32>(0.20, 0.215, 0.235);
    let col = mix(base, vec3<f32>(0.34, 0.36, 0.40), line * 0.7);
    let axis = select(0.0, 1.0, abs(p.x) < 0.01 || abs(p.y) < 0.01) * 0.0;
    if (G.flags.z < 0.5) { discard; }
    return vec4<f32>(encode(mix(G.bg_bot.rgb * 0.9, col, fade) + vec3<f32>(axis)), 1.0);
}
