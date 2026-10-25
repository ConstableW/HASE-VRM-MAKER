//! Additive Merkmale für Avatar-Basen: Ohren (Katze, Hund, Fuchs, Hase, Elf), Schweif, Hörner. Jedes Merkmal wird parametrisch als Mesh erzeugt,
//! an Landmarken der (bereits verformten) Basis verankert und am Skelett befestigt (Ohren/Hörner am Kopf-Bone, Schweif als eigene Bone-Kette).
use crate::avatar::Landmarks;
use crate::export::ExBone;
use crate::vrm_model::VrmModel;

pub struct PDef { pub id: &'static str, pub label: &'static str, pub min: f32, pub max: f32, pub def: f32 }
const fn p(id: &'static str, label: &'static str, min: f32, max: f32, def: f32) -> PDef { PDef { id, label, min, max, def } }

pub const EARS: &[PDef] = &[p("size", "Größe", 0.3, 2.5, 1.0), p("width", "Breite", 0.3, 2.0, 1.0), p("point", "Spitze", 0.0, 1.0, 0.7), p("tilt", "Neigung nach außen (°)", -20.0, 85.0, 12.0), p("fwd", "Neigung nach vorn (°)", -30.0, 60.0, 8.0),
    p("height", "Position: Höhe", -1.0, 1.0, 0.6), p("spread", "Position: Abstand", -1.0, 1.0, 0.0), p("bend", "Knick / Schlappohr", -1.0, 1.0, 0.0), p("thick", "Dicke", 0.3, 2.5, 1.0)];
pub const EARS_PRESETS: &[(&str, &[f32])] = &[
    ("Katze", &[1.0, 1.0, 0.8, 12.0, 10.0, 0.7, 0.0, 0.0, 1.0]), ("Hund (Schlappohr)", &[1.2, 1.1, 0.2, 30.0, 0.0, 0.5, 0.2, -0.9, 1.2]), ("Fuchs", &[1.6, 0.85, 0.9, 10.0, 8.0, 0.8, -0.1, 0.0, 1.0]),
    ("Hase", &[2.4, 0.6, 0.35, 8.0, 5.0, 0.7, -0.4, 0.15, 0.9]), ("Elf (lang, seitlich)", &[0.95, 0.5, 1.0, 6.0, -8.0, -0.9, 0.9, 0.0, 0.7]), ("Wolf", &[1.3, 0.95, 0.75, 14.0, 12.0, 0.65, 0.0, 0.0, 1.1]),
];
pub const TAIL: &[PDef] = &[p("length", "Länge", 0.1, 1.6, 0.7), p("thick", "Dicke", 0.3, 3.0, 1.0), p("taper", "Spitze (Verjüngung)", 0.0, 1.0, 0.6), p("fluff", "Buschigkeit", 0.0, 1.5, 0.0), p("up", "Krümmung nach oben", -1.0, 1.0, 0.3), p("side", "Krümmung seitlich", -1.0, 1.0, 0.0), p("lift", "Ansatz: Höhe", -1.0, 1.0, 0.0)];
pub const TAIL_PRESETS: &[(&str, &[f32])] = &[("Katze", &[0.9, 0.7, 0.4, 0.0, 0.5, 0.0, 0.0]), ("Fuchs (buschig)", &[0.9, 1.2, 0.5, 1.0, 0.2, 0.0, 0.0]), ("Hund", &[0.6, 1.0, 0.7, 0.2, 0.8, 0.0, 0.2]), ("Echse", &[1.2, 1.2, 1.0, 0.0, -0.1, 0.0, -0.3]), ("Kurz (Stummel)", &[0.18, 1.5, 0.3, 0.6, 0.0, 0.0, 0.1])];
pub const HORNS: &[PDef] = &[p("length", "Länge", 0.1, 2.0, 0.7), p("thick", "Dicke", 0.3, 2.5, 1.0), p("curl", "Krümmung", -1.0, 1.5, 0.6), p("spread", "Abstand", -1.0, 1.0, 0.2), p("fwd", "Richtung nach vorn", -1.0, 1.0, 0.2), p("height", "Position: Höhe", -1.0, 1.0, 0.0), p("taper", "Spitze", 0.0, 1.0, 0.8)];
pub const HORNS_PRESETS: &[(&str, &[f32])] = &[("Dämon", &[0.8, 1.0, 0.7, 0.3, 0.3, 0.0, 0.9]), ("Widder (eingerollt)", &[1.2, 1.6, 1.4, 0.8, -0.2, -0.4, 0.5]), ("Kurz", &[0.35, 1.0, 0.2, 0.1, 0.0, 0.2, 0.9]), ("Hirsch-artig (gerade)", &[1.4, 0.7, 0.1, 0.4, 0.0, 0.2, 0.9])];

pub const HAIR: &[PDef] = &[p("fringe", "Pony / Stirnhaar (Länge)", 0.0, 4.0, 0.7), p("side", "Seitenlänge", 0.3, 7.0, 2.0), p("back", "Hinterkopf-Länge", 0.3, 8.0, 2.4), p("volume", "Volumen", 0.0, 1.0, 0.45), p("width", "Strähnenbreite", 0.4, 2.5, 1.0), p("density", "Dichte", 0.5, 2.0, 1.0), p("wave", "Welle", 0.0, 1.0, 0.1), p("part", "Scheitel (links/rechts)", -1.0, 1.0, 0.0), p("gravity", "Fall (Schwerkraft)", 0.1, 1.0, 0.7)];
pub const HAIR_PRESETS: &[(&str, &[f32])] = &[
    ("Kurz", &[0.45, 0.8, 0.8, 0.55, 1.0, 1.2, 0.0, 0.0, 0.7]), ("Bob", &[0.7, 2.0, 2.0, 0.45, 1.1, 1.2, 0.05, 0.0, 0.8]), ("Lang", &[0.7, 4.5, 6.0, 0.4, 1.1, 1.3, 0.1, 0.0, 0.75]),
    ("Lang, gewellt", &[0.6, 4.5, 5.5, 0.55, 1.2, 1.4, 0.7, 0.3, 0.6]), ("Fransig (Anime)", &[1.0, 2.6, 3.2, 0.6, 1.4, 1.2, 0.15, 0.2, 0.7]),
];

pub const KINDS: &[(&str, &str)] = &[("ears", "Ohren"), ("tail", "Schweif"), ("horns", "Hörner"), ("hair", "Frisur (generiert)")];

pub fn defs(kind: &str) -> &'static [PDef] { match kind { "ears" => EARS, "tail" => TAIL, "hair" => HAIR, _ => HORNS } }
pub fn presets(kind: &str) -> &'static [(&'static str, &'static [f32])] { match kind { "ears" => EARS_PRESETS, "tail" => TAIL_PRESETS, "hair" => HAIR_PRESETS, _ => HORNS_PRESETS } }

#[derive(Clone, Debug, PartialEq)]
pub struct FeatureState { pub enabled: bool, pub vals: Vec<f32>, pub color: [f32; 3], pub color2: [f32; 3] }
impl FeatureState {
    pub fn new(kind: &str) -> FeatureState { FeatureState { enabled: false, vals: presets(kind)[0].1.to_vec(), color: [0.45, 0.30, 0.20], color2: [0.95, 0.65, 0.65] } }
}

pub struct GenPart { pub name: String, pub pos: Vec<[f32; 3]>, pub nrm: Vec<[f32; 3]>, pub uv: Vec<[f32; 2]>, pub idx: Vec<u32>, pub joints: Vec<[u16; 4]>, pub weights: Vec<[f32; 4]>, pub color: [f32; 3] }
#[derive(Default)]
pub struct Generated { pub parts: Vec<GenPart>, pub bones: Vec<ExBone>, pub springs: Vec<(String, Vec<usize>)> }

type V3 = [f32; 3];
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn mul(a: V3, k: f32) -> V3 { [a[0] * k, a[1] * k, a[2] * k] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn norm(a: V3) -> V3 { let l = dot(a, a).sqrt().max(1e-9); mul(a, 1.0 / l) }
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

/// Rotation um eine Achse (Rodrigues).
fn rot(v: V3, axis: V3, ang: f32) -> V3 { let a = norm(axis); let (c, s) = (ang.cos(), ang.sin()); add(add(mul(v, c), mul(cross(a, v), s)), mul(a, dot(a, v) * (1.0 - c))) }

fn smooth_normals(pos: &[V3], idx: &[u32]) -> Vec<V3> {
    let mut n = vec![[0f32; 3]; pos.len()];
    for t in idx.chunks_exact(3) { let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]); let fnm = cross(sub(b, a), sub(c, a)); for &i in t { n[i as usize] = add(n[i as usize], fnm); } }
    n.into_iter().map(norm).collect()
}

/// Rohr entlang einer Mittellinie (Querschnitt = Kreis mit Radien je Ring); Enden geschlossen. Liefert (pos, uv, idx).
fn tube(center: &[V3], radii: &[f32], sides: usize) -> (Vec<V3>, Vec<[f32; 2]>, Vec<u32>) {
    let n = center.len();
    let mut pos = Vec::new(); let mut uv = Vec::new(); let mut idx = Vec::new();
    let mut up = [0.0, 0.0, 1.0];
    for i in 0..n {
        let t = if i + 1 < n { sub(center[i + 1], center[i]) } else { sub(center[i], center[i - 1]) };
        let t = norm(t);
        if dot(t, up).abs() > 0.95 { up = [1.0, 0.0, 0.0]; }
        let s = norm(cross(t, up)); let u = norm(cross(s, t)); up = u;
        for k in 0..=sides { let a = k as f32 / sides as f32 * std::f32::consts::TAU; pos.push(add(center[i], add(mul(s, a.cos() * radii[i]), mul(u, a.sin() * radii[i])))); uv.push([k as f32 / sides as f32, i as f32 / (n - 1) as f32]); }
    }
    let w = sides + 1;
    for i in 0..n - 1 { for k in 0..sides { let (a, b, c, d) = ((i * w + k) as u32, (i * w + k + 1) as u32, ((i + 1) * w + k) as u32, ((i + 1) * w + k + 1) as u32); idx.extend_from_slice(&[a, c, b, b, c, d]); } }
    // Kappen
    for (ring, flip) in [(0usize, true), (n - 1, false)] {
        let c = pos.len() as u32; pos.push(center[ring]); uv.push([0.5, ring as f32 / (n - 1) as f32]);
        for k in 0..sides { let (a, b) = ((ring * w + k) as u32, (ring * w + k + 1) as u32); if flip { idx.extend_from_slice(&[c, b, a]); } else { idx.extend_from_slice(&[c, a, b]); } }
    }
    (pos, uv, idx)
}

fn head_extent(m: &VrmModel, lm: &Landmarks) -> (V3, f32, f32) {
    // Scheitel: höchster Punkt nahe der Mittelebene oberhalb der Augen; Kopfhalbbreite aus den Ohr-Landmarken
    let mut top = (f32::MIN, [0f32, lm.head[1], lm.head[2]]);
    for p in &m.parts { for v in &p.pos { if v[1] > lm.eye_l[1] && v[0].abs() < lm.ipd * 0.6 && v[2] > lm.head[2] - lm.ipd * 3.0 && v[1] > top.0 { top = (v[1], *v); } } }
    let crown = if top.0 > f32::MIN { [0.0, top.1[1], top.1[2]] } else { [0.0, lm.eye_l[1] + lm.ipd * 2.5, lm.head[2]] };
    (crown, lm.ear_l[0].abs().max(lm.ipd * 1.2), (crown[1] - lm.ear_l[1]).max(lm.ipd))
}

fn ear_parts(m: &VrmModel, lm: &Landmarks, st: &FeatureState, head_bone: u16, out: &mut Generated) {
    let v = &st.vals;
    let (size, width, point, tilt, fwd, height, spread, bend, thick) = (v[0], v[1], v[2], v[3].to_radians(), v[4].to_radians(), v[5], v[6], v[7], v[8]);
    let (crown, hw, drop) = head_extent(m, lm);
    let u = lm.ipd;
    let (h0, w0) = (u * 1.55 * size, u * 1.3 * width);
    let (nu, nv) = (9usize, 12usize);
    let pexp = 1.0 + 2.6 * (1.0 - point);
    for (si, side) in [1.0f32, -1.0].into_iter().enumerate() {
        // Anker auf der Schädelellipse: θ = Winkel vom Scheitel (Höhe 1 → nahe Scheitel, -1 → Ohrhöhe)
        let th_a = (55.0f32 - 40.0 * height).clamp(8.0, 100.0).to_radians();
        let (sx, cy) = (th_a.sin(), th_a.cos());
        let y = crown[1] - drop * (1.0 - cy);
        let x = side * hw * sx * (0.95 + 0.2 * spread);
        let z = lerp(crown[2], lm.ear_l[2], (1.0 - cy).clamp(0.0, 1.0));
        let anchor = [x, y, z];
        let tilt = tilt + th_a * 0.85;
        // Mittellinie in lokalen Koordinaten (y nach oben, z nach vorn); Knick biegt die Spitze
        let mut cl: Vec<(V3, V3)> = Vec::new(); // (Punkt, Normale)
        let (mut py, mut pz, mut th) = (0.0f32, 0.0f32, 0.0f32);
        for j in 0..=nv { let t = j as f32 / nv as f32; if j > 0 { th = -bend * 2.4 * t * t; py += h0 / nv as f32 * th.cos(); pz += h0 / nv as f32 * th.sin(); } cl.push(([0.0, py, pz], [0.0, -th.sin(), th.cos()])); }
        let build = |scale: f32, off: f32, uvs: bool| -> (Vec<V3>, Vec<[f32; 2]>, Vec<u32>) {
            let (mut pos, mut uv, mut idx) = (Vec::new(), Vec::new(), Vec::new());
            for (surf, sgn) in [(0usize, 1.0f32), (1, -1.0)] {
                let base = pos.len() as u32;
                for j in 0..=nv {
                    let t = j as f32 / nv as f32;
                    let wv = w0 * scale * (1.0 - t.powf(pexp)).max(0.0);
                    let tv = u * 0.16 * thick * (1.0 - t).powf(0.6) * scale;
                    for i in 0..nu { let s = i as f32 / (nu - 1) as f32 * 2.0 - 1.0; let (c, n) = cl[j]; let xl = s * wv * 0.5; let curv = (1.0 - s * s) * 0.0; let p = [xl, c[1] * scale, c[2] * scale + curv]; let q = add(p, mul(n, sgn * (tv + off) * (1.0 - 0.6 * s * s))); pos.push(q); uv.push(if uvs { [(s + 1.0) * 0.5, t] } else { [0.5, 0.5] }); }
                }
                for j in 0..nv { for i in 0..nu - 1 { let a = base + (j * nu + i) as u32; let b = a + 1; let c = a + nu as u32; let dd = c + 1; if surf == 0 { idx.extend_from_slice(&[a, b, c, b, dd, c]); } else { idx.extend_from_slice(&[a, c, b, b, c, dd]); } } }
            }
            // Ränder verbinden (links/rechts und Spitze)
            let nvs = (nv + 1) * nu;
            for j in 0..nv { for &i in &[0usize, nu - 1] { let a = (j * nu + i) as u32; let b = ((j + 1) * nu + i) as u32; let a2 = a + nvs as u32; let b2 = b + nvs as u32; if i == 0 { idx.extend_from_slice(&[a, b, a2, b, b2, a2]); } else { idx.extend_from_slice(&[a, a2, b, b, a2, b2]); } } }
            (pos, uv, idx)
        };
        for (inner, scale, off, col) in [(false, 1.0f32, 0.0f32, st.color), (true, 0.62, u * 0.012, st.color2)] {
            let (lp, uv, idx) = build(scale, off, false);
            // lokal → Welt: Neigung nach außen (um z), nach vorn (um x), Position
            let xf = |q: V3| -> V3 {
                let mut r = rot(q, [0.0, 0.0, 1.0], -side * tilt);
                r = rot(r, [1.0, 0.0, 0.0], -fwd);
                if inner { r = add(r, [0.0, 0.0, u * 0.0]); }
                add(r, anchor)
            };
            let pos: Vec<V3> = lp.into_iter().map(|q| if side < 0.0 { xf([q[0], q[1], q[2]]) } else { xf(q) }).collect();
            let mut idx2 = idx;
            if side < 0.0 { for t in idx2.chunks_exact_mut(3) { t.swap(1, 2); } }
            let nrm = smooth_normals(&pos, &idx2);
            let n = pos.len();
            out.parts.push(GenPart { name: format!("Ear_{}{}", if si == 0 { "L" } else { "R" }, if inner { "_inner" } else { "" }), pos, nrm, uv, idx: idx2, joints: vec![[head_bone, 0, 0, 0]; n], weights: vec![[1.0, 0.0, 0.0, 0.0]; n], color: col });
        }
    }
}

fn horn_parts(m: &VrmModel, lm: &Landmarks, st: &FeatureState, head_bone: u16, out: &mut Generated) {
    let v = &st.vals;
    let (length, thick, curl, spread, fwd, height, taper) = (v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
    let (crown, hw, _) = head_extent(m, lm);
    let u = lm.ipd;
    for (si, side) in [1.0f32, -1.0].into_iter().enumerate() {
        let y = crown[1] - u * (0.55 - 0.7 * height);
        let x = side * hw * (0.45 + 0.3 * spread).clamp(0.1, 0.95);
        let anchor = [x, y, crown[2] + u * 0.3 + u * 0.4 * fwd];
        let n = 14usize;
        let len = u * 3.2 * length;
        let mut dir = norm([side * (0.25 + 0.7 * spread.max(0.0)), 1.0, 0.1 + 0.8 * fwd]);
        let mut c = vec![anchor]; let mut radii = vec![u * 0.3 * thick];
        for j in 1..n {
            let t = j as f32 / (n - 1) as f32;
            // Krümmung: Richtung dreht sich in der Ebene (nach innen/vorn)
            let axis = norm([0.0, 0.3, 1.0]);
            dir = rot(dir, axis, -side * curl * 0.22);
            dir = norm(add(dir, mul([0.0, 0.0, 1.0], curl * 0.02 * (1.0 - t))));
            c.push(add(*c.last().unwrap(), mul(dir, len / (n - 1) as f32)));
            radii.push(u * 0.3 * thick * (1.0 - taper * t.powf(0.8)).max(0.04));
        }
        let (pos, uv, mut idx) = tube(&c, &radii, 10);
        if side < 0.0 { for t in idx.chunks_exact_mut(3) { t.swap(1, 2); } }
        let nrm = smooth_normals(&pos, &idx);
        let nn = pos.len();
        out.parts.push(GenPart { name: format!("Horn_{}", if si == 0 { "L" } else { "R" }), pos, nrm, uv, idx, joints: vec![[head_bone, 0, 0, 0]; nn], weights: vec![[1.0, 0.0, 0.0, 0.0]; nn], color: st.color });
    }
}

fn tail_part(m: &VrmModel, lm: &Landmarks, st: &FeatureState, hips_bone: usize, base_bones: usize, out: &mut Generated) {
    let v = &st.vals;
    let (length, thick, taper, fluff, up, side, lift) = (v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
    let hips = m.bones[hips_bone].pos;
    let u = lm.ipd;
    let body_back = lm.glute_l[2];
    let root = [0.0, hips[1] - u * 0.35 + lift * u * 0.8, body_back + u * 0.15];
    let nseg = 10usize;
    let total = (m.height * 0.55).max(0.3) * length;
    let r0 = u * 0.45 * thick;
    let mut dir = norm([0.0, -0.25 + 0.3 * lift, -1.0]);
    let mut c = vec![root]; let mut radii = vec![r0];
    for j in 1..=nseg {
        let t = j as f32 / nseg as f32;
        // Krümmung nach oben (um die x-Achse) und seitlich (um die y-Achse)
        dir = rot(dir, [1.0, 0.0, 0.0], up * 0.16);
        dir = rot(dir, [0.0, 1.0, 0.0], side * 0.12);
        c.push(add(*c.last().unwrap(), mul(dir, total / nseg as f32)));
        let prof = (1.0 - taper * t.powf(1.2)).max(0.05);
        radii.push(r0 * prof * (1.0 + fluff * (std::f32::consts::PI * t).sin() * 0.9));
    }
    // Bones: eine je Mittellinienpunkt (Kette unter der Hüfte)
    let first = base_bones + out.bones.len();
    let mut chain = Vec::new();
    for (j, p) in c.iter().enumerate().take(nseg) { out.bones.push(ExBone { name: format!("J_Sec_Tail_{:02}", j + 1), parent: Some(if j == 0 { hips_bone } else { first + j - 1 }), pos: *p }); chain.push(first + j); }
    out.springs.push(("tail".into(), chain));
    let (pos, uv, idx) = tube(&c, &radii, 12);
    let nrm = smooth_normals(&pos, &idx);
    let w = 13usize;
    let nb = nseg;
    let (mut joints, mut weights) = (Vec::new(), Vec::new());
    for i in 0..pos.len() {
        let ring = (i / w).min(c.len() - 1);
        let t = (ring as f32).min((nb - 1) as f32);
        let (b0, fr) = (t.floor() as usize, t - t.floor());
        let b1 = (b0 + 1).min(nb - 1);
        joints.push([(first + b0) as u16, (first + b1) as u16, 0, 0]);
        weights.push(if b0 == b1 { [1.0, 0.0, 0.0, 0.0] } else { [1.0 - fr, fr, 0.0, 0.0] });
    }
    out.parts.push(GenPart { name: "Tail".into(), pos, nrm, uv, idx, joints, weights, color: st.color });
}

/// Frisur: Strähnenbänder, auf der Schädelellipse verwurzelt, fallen unter Schwerkraft herab und halten Abstand zum Kopf.
fn hair_parts(m: &VrmModel, lm: &Landmarks, st: &FeatureState, head_bone: u16, out: &mut Generated) {
    let v = &st.vals;
    let (fringe, side_l, back_l, vol, wscale, dens, wave, part, grav) = (v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8]);
    let (crown, hw, drop) = head_extent(m, lm);
    let hu = drop; // Längeneinheit: Scheitel bis Ohrhöhe
    let c = [0.0, crown[1] - drop, lm.ear_l[2]];
    let (ax, ay, az) = (hw * 1.02, drop * 1.02, hw * 1.12);
    let margin = hu * (0.04 + 0.14 * vol);
    let ell = |q: V3| -> f32 { ((q[0] - c[0]) / (ax + margin)).powi(2) + ((q[1] - c[1]) / (ay + margin)).powi(2) + ((q[2] - c[2]) / (az + margin)).powi(2) };
    let (mut pos, mut nrm, mut uv, mut idx) = (Vec::<V3>::new(), Vec::<V3>::new(), Vec::<[f32; 2]>::new(), Vec::<u32>::new());
    let nphi = (78.0 * dens) as usize; let nth = (15.0 * dens.sqrt()) as usize;
    let mut seed = 12345u32;
    let mut rnd = move || { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); (seed >> 8) as f32 / 16777216.0 };
    let nstep = 14usize;
    for it in 0..nth {
        let th = (4.0 + (it as f32 + 0.5) / nth as f32 * 98.0).to_radians();
        let ring_c = th.sin();
        let nph = ((nphi as f32 * ring_c.max(0.12)).round() as usize).max(3);
        for ip in 0..nph {
            let phi = (ip as f32 + rnd() * 0.6) / nph as f32 * std::f32::consts::TAU; // 0 = vorn
            let (cf, sf) = (phi.cos(), phi.sin());
            // Gesichtsbereich freihalten: vorn nur oberhalb der Haarlinie; Nacken nur hinten/seitlich
            let th_deg = th.to_degrees();
            if cf > 0.35 && th_deg > 52.0 { continue; }
            if cf > -0.2 && th_deg > 88.0 && sf.abs() < 0.8 { continue; }
            let root = [c[0] + sf * th.sin() * ax, c[1] + th.cos() * ay, c[2] + cf * th.sin() * az];
            let nrm0 = norm([sf * th.sin() / ax, th.cos() / ay, cf * th.sin() / az]);
            let (wf, wb) = (cf.max(0.0).powi(2), (-cf).max(0.0).powi(2));
            let len = (wf * fringe + wb * back_l + (1.0 - wf - wb).max(0.0) * side_l) * hu * (0.85 + 0.3 * rnd());
            // Scheitel: Pony/Seitenscheitel verschiebt Richtung nach links/rechts
            let tdown = norm([sf * th.cos() * ax, -th.sin() * ay, cf * th.cos() * az]);
            let mut d = norm(add(add(mul(tdown, 1.0), mul(nrm0, 0.15 + vol * 0.9)), [part * 0.35 * (if cf > 0.0 { 1.0 } else { 0.0 }), 0.0, 0.0]));
            let lat = (if sf >= part * 0.35 { 1.0f32 } else { -1.0 }) * wf * (fringe / 4.0 + 0.15).min(0.6);
            let mut p = add(root, mul(nrm0, margin * 0.6));
            let step = len / nstep as f32;
            let sidev = norm(cross(d, [0.0, 1.0, 0.0]));
            let (mut pts, mut dirs) = (vec![p], vec![d]);
            let phase = rnd() * 6.28;
            for k in 0..nstep {
                let t = (k + 1) as f32 / nstep as f32;
                // Schwerkraft zieht Richtung nach unten, Steifigkeit am Ansatz
                let gstr = grav * (0.45 + 1.3 * t);
                d = norm(add(add(d, mul([0.0, -1.0, 0.0], gstr * 0.55)), [lat * (1.0 - t) * 0.5, 0.0, 0.0]));
                let w = wave * hu * 0.35 * (t * 7.0 + phase).sin() * step / (len.max(1e-4)) * nstep as f32 * 0.25;
                let mut q = add(add(p, mul(d, step)), mul(sidev, w));
                // außerhalb des Kopfes halten
                let e = ell(q); if e < 1.0 { let r = sub(q, c); let rn = norm([r[0] / (ax + margin), r[1] / (ay + margin), r[2] / (az + margin)]); q = [c[0] + rn[0] * (ax + margin), c[1] + rn[1] * (ay + margin), c[2] + rn[2] * (az + margin)]; }
                // Oberkörper (Schultern) als Ellipse im Grundriss freihalten
                if q[1] < c[1] - ay * 0.95 {
                    let (cz, rx, rz) = (c[2] - hw * 0.35, hw * 2.5 + margin * 0.5, hw * 1.25 + margin * 0.5);
                    let (ex, ez) = ((q[0]) / rx, (q[2] - cz) / rz);
                    let e2 = ex * ex + ez * ez;
                    if e2 < 1.0 { let k = 1.0 / e2.sqrt().max(1e-3); q[0] = ex * k * rx; q[2] = cz + ez * k * rz; }
                }
                dirs.push(norm(sub(q, p))); p = q; pts.push(p);
            }
            // Band: Breite verjüngt zur Spitze
            let w0 = hu * 0.075 * wscale / dens.sqrt();
            let base = pos.len() as u32;
            for (j, (q, dd)) in pts.iter().zip(&dirs).enumerate() {
                let t = j as f32 / nstep as f32;
                let sv = norm(cross(*dd, nrm0)); let sv = if dot(sv, sv) < 0.5 { sidev } else { sv };
                let ww = w0 * (1.0 - 0.8 * t.powf(1.5));
                let n = norm(cross(sv, *dd));
                for sg in [-1.0f32, 1.0] { pos.push(add(*q, mul(sv, sg * ww))); nrm.push(n); uv.push([(sg + 1.0) * 0.5, t]); }
            }
            for j in 0..nstep as u32 { let a = base + j * 2; idx.extend_from_slice(&[a, a + 1, a + 2, a + 1, a + 3, a + 2]); }
            // Rückseite (doppelseitig)
            let nb = pos.len() as u32 - base;
            for j in 0..nb { let (q, n) = (pos[(base + j) as usize], nrm[(base + j) as usize]); pos.push(q); nrm.push(mul(n, -1.0)); uv.push(uv[(base + j) as usize]); }
            for j in 0..nstep as u32 { let a = base + nb + j * 2; idx.extend_from_slice(&[a, a + 2, a + 1, a + 1, a + 2, a + 3]); }
        }
    }
    if pos.is_empty() { return; }
    let n = pos.len();
    out.parts.push(GenPart { name: "Hair".into(), pos, nrm, uv, idx, joints: vec![[head_bone, 0, 0, 0]; n], weights: vec![[1.0, 0.0, 0.0, 0.0]; n], color: st.color });
}

/// Erzeugt alle aktiven Merkmale auf der (verformten) Basis. `base_bones` = Anzahl der Basis-Bones (Indizes der Zusatz-Bones beginnen dort).
pub fn build(m: &VrmModel, lm: &Landmarks, feats: &std::collections::BTreeMap<String, FeatureState>) -> Generated {
    let mut out = Generated::default();
    let base = m.bones.len();
    let Some(&head) = m.humanoid.get("head") else { return out };
    for (kind, _) in KINDS {
        let Some(st) = feats.get(*kind) else { continue };
        if !st.enabled { continue; }
        match *kind {
            "ears" => ear_parts(m, lm, st, head as u16, &mut out),
            "horns" => horn_parts(m, lm, st, head as u16, &mut out),
            "hair" => hair_parts(m, lm, st, head as u16, &mut out),
            "tail" => { if let Some(&h) = m.humanoid.get("hips") { tail_part(m, lm, st, h, base, &mut out); } }
            _ => {}
        }
    }
    out
}
