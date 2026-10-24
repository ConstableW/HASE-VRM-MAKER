// ---------- Morph-Compute (Gather je Vertex, CSR) ----------
struct MorphParams { nv: u32, copies: u32, nt: u32, pad: u32 };
@group(0) @binding(0) var<uniform> MP: MorphParams;
@group(0) @binding(1) var<storage, read> base_pos: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> v_off: array<u32>;
@group(0) @binding(3) var<storage, read> v_tgt: array<u32>;
@group(0) @binding(4) var<storage, read> v_delta: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> weights: array<f32>;
@group(0) @binding(6) var<storage, read_write> out_pos: array<vec4<f32>>;

@compute @workgroup_size(64)
fn morph(@builtin(global_invocation_id) gid: vec3<u32>) {
    let g = gid.x;
    if (g >= MP.nv * MP.copies) { return; }
    let copy = g / MP.nv;
    let v = g % MP.nv;
    var p = base_pos[v];
    let wbase = copy * MP.nt;
    let e1 = v_off[v + 1u];
    for (var e = v_off[v]; e < e1; e = e + 1u) {
        let w = weights[wbase + v_tgt[e]];
        if (w != 0.0) { p = p + vec4<f32>(w * v_delta[e].xyz, 0.0); }
    }
    out_pos[g] = p;
}

// ---------- Rendering (Vertex Pulling + Skinning) ----------
struct Scene { view_proj: mat4x4<f32>, light_vp: mat4x4<f32>, light_dir: vec4<f32>, n: vec4<u32>, grid: vec4<f32> };
@group(0) @binding(0) var<uniform> S: Scene;
@group(0) @binding(1) var<storage, read> pos: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> joints: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read> skinw: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> bones: array<mat4x4<f32>>;
@group(0) @binding(5) var shadow_tex: texture_depth_2d;
@group(0) @binding(6) var shadow_samp: sampler_comparison;

fn skin_world(vi: u32, inst: u32) -> vec3<f32> {
    let p = vec4<f32>(pos[inst * S.n.x + vi].xyz, 1.0);
    let j = joints[vi];
    let w = skinw[vi];
    let nb = S.n.z;
    var r = vec3<f32>(0.0);
    r = r + w.x * (bones[inst * nb + j.x] * p).xyz;
    r = r + w.y * (bones[inst * nb + j.y] * p).xyz;
    r = r + w.z * (bones[inst * nb + j.z] * p).xyz;
    r = r + w.w * (bones[inst * nb + j.w] * p).xyz;
    let cols = u32(S.grid.z);
    let off = vec3<f32>(f32(inst % cols) * S.grid.x, f32(inst / cols) * S.grid.y, 0.0);
    return r + off;
}

struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) world: vec3<f32> };

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) inst: u32) -> VsOut {
    var o: VsOut;
    let w = skin_world(vi, inst);
    o.world = w;
    o.clip = S.view_proj * vec4<f32>(w, 1.0);
    return o;
}

@vertex
fn vs_shadow(@builtin(vertex_index) vi: u32, @builtin(instance_index) inst: u32) -> @builtin(position) vec4<f32> {
    return S.light_vp * vec4<f32>(skin_world(vi, inst), 1.0);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(cross(dpdx(in.world), dpdy(in.world)));
    let l = normalize(S.light_dir.xyz);
    let lc = S.light_vp * vec4<f32>(in.world, 1.0);
    let uv = vec2<f32>(lc.x * 0.5 + 0.5, 0.5 - lc.y * 0.5);
    var vis = 0.0;
    let texel = 1.0 / 2048.0;
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        for (var dy = -1; dy <= 1; dy = dy + 1) {
            vis = vis + textureSampleCompare(shadow_tex, shadow_samp, uv + vec2<f32>(f32(dx), f32(dy)) * texel, lc.z - 0.002);
        }
    }
    vis = vis / 9.0;
    let diff = max(dot(n, l), 0.0);
    let col = vec3<f32>(0.85, 0.72, 0.65) * (0.25 + 0.75 * diff * vis);
    return vec4<f32>(col, 1.0);
}
