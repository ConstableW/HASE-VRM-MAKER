//! Avatar-Erstellung auf einer VRM-/VRoid-artigen Basis: parametrische Verformung ohne vorgefertigte Morph-Targets.
//! Gesicht/Körper werden über Landmarken (Augen-Bones, Mund aus dem „aa“-Ausdruck, Nase/Kinn/Ohren aus der Geometrie) und weich ausklingende
//! Bereichsoperationen verformt; Proportionen über Knochenlängen/-skalierung mit Skinning (jede Basis mit humanoider Zuordnung funktioniert).
use crate::export::ExBone;
use crate::vrm_model::VrmModel;
use std::collections::BTreeMap;

pub struct AvDef { pub id: &'static str, pub label: &'static str, pub region: &'static str, pub min: f32, pub max: f32, pub def: f32 }

macro_rules! d { ($id:expr, $l:expr, $r:expr) => { AvDef { id: $id, label: $l, region: $r, min: -1.0, max: 1.0, def: 0.0 } }; }

/// Regionsnamen entsprechen `REGIONS` im Editor (Navigation/Kamerafokus).
pub const DEFS: &[AvDef] = &[
    d!("head-size", "Kopfgröße", "Gesamtkopf"), d!("head-width", "Kopfbreite", "Gesamtkopf"), d!("face-length", "Gesichtslänge", "Gesamtkopf"),
    d!("brow-height", "Brauenhöhe", "Brauen"),
    d!("eye-size-l", "Augengröße (links)", "Augen"), d!("eye-size-r", "Augengröße (rechts)", "Augen"),
    d!("eye-width-l", "Augenbreite (links)", "Augen"), d!("eye-width-r", "Augenbreite (rechts)", "Augen"),
    d!("eye-height-l", "Augenhöhe (links)", "Augen"), d!("eye-height-r", "Augenhöhe (rechts)", "Augen"),
    d!("eye-spacing-l", "Augenabstand (links)", "Augen"), d!("eye-spacing-r", "Augenabstand (rechts)", "Augen"),
    d!("eye-tilt-l", "Augenwinkel (links)", "Augen"), d!("eye-tilt-r", "Augenwinkel (rechts)", "Augen"),
    d!("cheek-width", "Wangenbreite", "Wangen"),
    d!("nose-size", "Nasengröße", "Nase"), d!("nose-height", "Nasenhöhe", "Nase"), d!("nose-width", "Nasenbreite", "Nase"), d!("nose-depth", "Nasenvorsprung", "Nase"),
    d!("mouth-width", "Mundbreite", "Mund & Lippen"), d!("mouth-height", "Mundhöhe", "Mund & Lippen"), d!("mouth-size", "Mundgröße", "Mund & Lippen"),
    d!("chin-length", "Kinnlänge", "Kiefer & Kinn"), d!("chin-width", "Kinnbreite", "Kiefer & Kinn"), d!("jaw-width", "Kieferbreite", "Kiefer & Kinn"),
    d!("ear-size", "Ohrgröße", "Ohren"), d!("ear-length", "Ohrlänge (spitz)", "Ohren"),
    d!("height", "Körpergröße", "Gesamtkörper"), d!("hand-size", "Handgröße", "Hände"), d!("foot-size", "Fußgröße", "Füße"),
    d!("neck-length", "Halslänge", "Hals"), d!("neck-thickness", "Halsdicke", "Hals"),
    d!("shoulder-width", "Schulterbreite", "Schultern & Torso"), d!("torso-length", "Oberkörperlänge", "Schultern & Torso"), d!("torso-depth", "Oberkörpertiefe", "Schultern & Torso"),
    d!("breast-size", "Brustgröße", "Brust"), d!("breast-height", "Brusthöhe", "Brust"), d!("breast-spacing", "Brustabstand", "Brust"),
    d!("belly", "Bauch", "Bauch"), d!("waist", "Taille", "Bauch"),
    d!("hip-width", "Hüftbreite", "Becken & Hüfte"), d!("glute-size", "Gesäßgröße", "Gesäß"),
    d!("arm-length", "Armlänge", "Arme"), d!("arm-thickness", "Armumfang", "Arme"),
    d!("leg-length", "Beinlänge", "Beine"), d!("leg-thickness", "Beinumfang", "Beine"),
];

pub fn index_of(id: &str) -> Option<usize> { DEFS.iter().position(|d| d.id == id) }
pub fn defaults() -> Vec<f32> { DEFS.iter().map(|d| d.def).collect() }
/// Gegenseite eines links/rechts-Parameters.
pub fn mirror(i: usize) -> Option<usize> {
    let id = DEFS[i].id;
    let other = if let Some(s) = id.strip_suffix("-l") { format!("{s}-r") } else if let Some(s) = id.strip_suffix("-r") { format!("{s}-l") } else { return None };
    index_of(&other)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Hair, Eye, Face, Body, Other }

pub fn kind_of(mat: &str) -> Kind {
    let m = mat.to_lowercase();
    if m.contains("hair") { Kind::Hair } else if m.contains("eye") && !m.contains("eyeline") { Kind::Eye } else if m.contains("face") { Kind::Face } else if m.contains("body") { Kind::Body } else { Kind::Other }
}

#[derive(Clone, Debug)]
pub struct Landmarks { pub eye_l: [f32; 3], pub eye_r: [f32; 3], pub mouth: [f32; 3], pub nose: [f32; 3], pub chin: [f32; 3], pub ear_l: [f32; 3], pub ear_r: [f32; 3], pub ipd: f32, pub head: [f32; 3], pub neck: [f32; 3], pub breast_l: [f32; 3], pub breast_r: [f32; 3], pub glute_l: [f32; 3], pub glute_r: [f32; 3], pub belly: [f32; 3] }

pub struct Prep { pub lm: Landmarks, pub kinds: Vec<Kind>, pub ok: bool }

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dist(a: [f32; 3], b: [f32; 3]) -> f32 { let d = sub(a, b); (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() }
/// Weich ausklingende Gewichtung (1 im Zentrum, 0 ab Radius `r`).
fn fall(d: f32, r: f32) -> f32 { if d >= r { 0.0 } else { let t = 1.0 - (d / r) * (d / r); t * t } }

fn bone_pos(m: &VrmModel, name: &str) -> Option<[f32; 3]> { m.humanoid.get(name).map(|&i| m.bones[i].pos) }

pub fn prepare(m: &VrmModel) -> Prep {
    let kinds: Vec<Kind> = m.parts.iter().map(|p| kind_of(&p.mat_name)).collect();
    let head = bone_pos(m, "head").unwrap_or([0.0, m.height * 0.9, 0.0]);
    let neck = bone_pos(m, "neck").unwrap_or([head[0], head[1] - 0.08, head[2]]);
    let (mut el, mut er) = (bone_pos(m, "leftEye"), bone_pos(m, "rightEye"));
    if el.is_none() || er.is_none() { let w = m.height * 0.03; el = Some([head[0] + w, head[1] + 0.08, head[2] + 0.07]); er = Some([head[0] - w, head[1] + 0.08, head[2] + 0.07]); }
    let (mut el, mut er) = (el.unwrap(), er.unwrap());
    if el[0] < er[0] { std::mem::swap(&mut el, &mut er); }
    let ipd = dist(el, er).max(0.02);
    // Mund: Schwerpunkt der „aa“-Verformung
    let mut acc = ([0f32; 3], 0f32);
    if let Some((_, binds)) = m.expressions.iter().find(|e| e.0 == "aa") {
        for (pi, ti, w) in binds { let part = &m.parts[*pi]; let t = &part.targets[*ti].1; for (k, d) in t.iter().enumerate() { let mag = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() * w.abs(); if mag > 1e-5 { let p = part.pos[k]; for c in 0..3 { acc.0[c] += p[c] * mag; } acc.1 += mag; } }
        }
    }
    let mouth = if acc.1 > 0.0 { [0.0, acc.0[1] / acc.1, acc.0[2] / acc.1] } else { [0.0, (el[1] + head[1] - 0.12) * 0.5 - ipd * 0.6, el[2]] };
    // Nase/Kinn/Ohren aus der Gesichtsgeometrie (ohne Haare)
    let mid_y = (el[1] + mouth[1]) * 0.5;
    let (mut nose, mut nose_z) = ([0.0, mid_y, el[2]], f32::MIN);
    let (mut chin, mut chin_y) = ([0.0, mouth[1] - ipd * 0.8, mouth[2]], f32::MAX);
    let (mut max_x, mut ear_pts): (f32, Vec<[f32; 3]>) = (0.0, vec![]);
    let eye_y = el[1];
    for (pi, p) in m.parts.iter().enumerate() {
        if kinds[pi] == Kind::Hair { continue; }
        for v in &p.pos {
            if v[1] > mouth[1] - ipd * 1.6 && v[1] < eye_y + ipd * 1.2 && v[2] > head[2] - ipd * 3.0 && v[0].abs() < ipd * 0.55 {
                if v[0].abs() < ipd * 0.12 && v[1] > mouth[1] + ipd * 0.1 && v[1] < eye_y - ipd * 0.05 && v[2] > nose_z { nose_z = v[2]; nose = *v; }
                if v[0].abs() < ipd * 0.2 && v[1] < mouth[1] - ipd * 0.1 && v[2] > head[2] && v[1] < chin_y { chin_y = v[1]; chin = *v; }
            }
            if v[1] > eye_y - ipd * 1.0 && v[1] < eye_y + ipd * 0.3 && v[2] > head[2] - ipd * 1.2 { max_x = max_x.max(v[0].abs()); }
        }
    }
    for (pi, p) in m.parts.iter().enumerate() {
        if kinds[pi] == Kind::Hair { continue; }
        for v in &p.pos { if v[1] > eye_y - ipd * 1.0 && v[1] < eye_y + ipd * 0.3 && v[0].abs() > max_x * 0.93 && v[2] > head[2] - ipd * 1.2 { ear_pts.push(*v); } }
    }
    let avg = |side: f32| { let l: Vec<&[f32; 3]> = ear_pts.iter().filter(|p| p[0] * side > 0.0).collect(); if l.is_empty() { [side * max_x, eye_y - ipd * 0.3, head[2]] } else { let n = l.len() as f32; [l.iter().map(|p| p[0]).sum::<f32>() / n, l.iter().map(|p| p[1]).sum::<f32>() / n, l.iter().map(|p| p[2]).sum::<f32>() / n] } };
    let (ear_l, ear_r) = (avg(1.0), avg(-1.0));
    // Körper-Landmarken
    let chest = bone_pos(m, "upperChest").or_else(|| bone_pos(m, "chest")).unwrap_or([0.0, neck[1] - 0.2, 0.0]);
    let hips = bone_pos(m, "hips").unwrap_or([0.0, m.height * 0.5, 0.0]);
    let (ual, uar) = (bone_pos(m, "leftUpperArm").unwrap_or([0.17, chest[1], 0.0]), bone_pos(m, "rightUpperArm").unwrap_or([-0.17, chest[1], 0.0]));
    let sw = (ual[0] - uar[0]).abs();
    let (body_front, body_back) = {
        let mut best = ((f32::MIN, f32::MAX), ());
        let _ = &mut best;
        let (mut fz, mut bz) = (f32::MIN, f32::MAX);
        for (pi, p) in m.parts.iter().enumerate() { if kinds[pi] == Kind::Hair || kinds[pi] == Kind::Face || kinds[pi] == Kind::Eye { continue; } for v in &p.pos { if v[1] > hips[1] - 0.12 && v[1] < chest[1] + 0.05 && v[0].abs() < sw * 0.35 { fz = fz.max(v[2]); bz = bz.min(v[2]); } } }
        (fz, bz)
    };
    let by = chest[1] - sw * 0.20;
    let breast = |side: f32| [side * sw * 0.20, by, body_front.max(chest[2]) - 0.01];
    let glute = |side: f32| [side * sw * 0.13, hips[1] - sw * 0.10, body_back.min(hips[2]) + 0.01];
    let navel_y = hips[1] + (chest[1] - hips[1]) * 0.25;
    let lm = Landmarks { eye_l: el, eye_r: er, mouth, nose, chin, ear_l, ear_r, ipd, head, neck, breast_l: breast(1.0), breast_r: breast(-1.0), glute_l: glute(1.0), glute_r: glute(-1.0), belly: [0.0, navel_y, body_front] };
    Prep { lm, kinds, ok: !m.humanoid.is_empty() }
}

pub struct Deformed { pub pos: Vec<Vec<[f32; 3]>>, pub bones: Vec<[f32; 3]> }

/// Wendet die Parameter auf die Ruheform an (Bereichsoperationen, dann Knochen-Skinning).
pub fn deform(m: &VrmModel, prep: &Prep, orig: &[Vec<[f32; 3]>], ob: &[[f32; 3]], vals: &[f32]) -> Deformed {
    let v = |id: &str| index_of(id).map(|i| vals[i]).unwrap_or(0.0);
    let lm = &prep.lm;
    let u = lm.ipd;
    let mut out: Vec<Vec<[f32; 3]>> = orig.to_vec();
    let nz = |x: f32| x.abs() > 1e-4;
    let face_active = ["head-width", "face-length", "brow-height", "cheek-width", "nose-size", "nose-height", "nose-width", "nose-depth", "mouth-width", "mouth-height", "mouth-size", "chin-length", "chin-width", "jaw-width", "ear-size", "ear-length", "eye-size-l", "eye-size-r", "eye-width-l", "eye-width-r", "eye-height-l", "eye-height-r", "eye-spacing-l", "eye-spacing-r", "eye-tilt-l", "eye-tilt-r"].iter().any(|k| nz(v(k)));
    let body_active = ["breast-size", "breast-height", "breast-spacing", "belly", "waist", "glute-size", "torso-depth"].iter().any(|k| nz(v(k)));
    if face_active || body_active {
        for (pi, part) in m.parts.iter().enumerate() {
            let kind = prep.kinds[pi];
            let dst = &mut out[pi];
            let _ = part;
            for (k, p0) in orig[pi].iter().enumerate() {
                let mut p = *p0;
                // ---- Gesicht (Hair ausgenommen, außer Kopfbreite)
                if face_active {
                    let head_w = v("head-width");
                    if nz(head_w) && p[1] > lm.neck[1] { let w = ((p[1] - lm.neck[1]) / (u * 0.9)).clamp(0.0, 1.0); p[0] = lm.head[0] + (p[0] - lm.head[0]) * (1.0 + 0.22 * head_w * w); }
                    if kind != Kind::Hair {
                        // Augen je Seite
                        for (side, c, sfx) in [(1.0f32, lm.eye_l, "l"), (-1.0, lm.eye_r, "r")] {
                            let r = u * 0.62;
                            let w = fall(dist(p, c), r);
                            if w > 0.0 {
                                let (sz, wd, ht, sp, tl) = (v(&format!("eye-size-{sfx}")), v(&format!("eye-width-{sfx}")), v(&format!("eye-height-{sfx}")), v(&format!("eye-spacing-{sfx}")), v(&format!("eye-tilt-{sfx}")));
                                let s = 1.0 + 0.8 * sz * w;
                                let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
                                let sxw = 1.0 + 0.5 * wd * w;
                                let ang = tl * 0.6 * w * side;
                                let (ca, sa) = (ang.cos(), ang.sin());
                                let (rx, ry) = ((dx * sxw) * s, dy * s);
                                p[0] = c[0] + rx * ca - ry * sa; p[1] = c[1] + rx * sa + ry * ca;
                                p[0] += side * sp * u * 0.4 * w; p[1] += ht * u * 0.4 * w;
                                p[2] += 0.0;
                            }
                        }
                        let bh = v("brow-height");
                        if nz(bh) { for c in [lm.eye_l, lm.eye_r] { let cc = [c[0], c[1] + u * 0.42, c[2]]; if p[1] > c[1] + u * 0.05 { p[1] += bh * u * 0.16 * fall(dist(p, cc), u * 0.55); } } }
                        let cw = v("cheek-width");
                        if nz(cw) { let cc = [0.0, (lm.eye_l[1] + lm.mouth[1]) * 0.5, lm.head[2]]; let w = fall(((p[1] - cc[1]).abs()) * 1.3, u * 1.1) * ((p[0].abs() / (lm.ear_l[0].abs().max(u))).clamp(0.0, 1.0)); p[0] += p[0].signum() * cw * u * 0.14 * w; }
                        // Nase
                        let (ns, nh, nw, nd) = (v("nose-size"), v("nose-height"), v("nose-width"), v("nose-depth"));
                        let wn = fall(dist(p, lm.nose), u * 0.55);
                        if wn > 0.0 && (nz(ns) || nz(nh) || nz(nw) || nz(nd)) { let s = 1.0 + 0.6 * ns * wn; p[0] = lm.nose[0] + (p[0] - lm.nose[0]) * s * (1.0 + 0.4 * nw * wn); p[1] = lm.nose[1] + (p[1] - lm.nose[1]) * s + nh * u * 0.14 * wn; p[2] = lm.nose[2] + (p[2] - lm.nose[2]) * (1.0 + 0.3 * ns * wn) + nd * u * 0.12 * wn; }
                        // Mund
                        let (mw, mh, ms) = (v("mouth-width"), v("mouth-height"), v("mouth-size"));
                        let wm = fall(dist(p, lm.mouth), u * 0.95);
                        if wm > 0.0 && (nz(mw) || nz(mh) || nz(ms)) { let s = 1.0 + 0.6 * ms * wm; p[0] = lm.mouth[0] + (p[0] - lm.mouth[0]) * s * (1.0 + 0.7 * mw * wm); p[1] = lm.mouth[1] + (p[1] - lm.mouth[1]) * s + mh * u * 0.14 * wm; }
                        // Kinn / Kiefer
                        let (cl, cwd, jw) = (v("chin-length"), v("chin-width"), v("jaw-width"));
                        if p[1] < lm.mouth[1] + u * 0.2 && p[1] > lm.chin[1] - u * 0.6 && p[2] > lm.head[2] - u * 0.5 {
                            let t = ((lm.mouth[1] - p[1]) / (lm.mouth[1] - lm.chin[1]).abs().max(1e-3)).clamp(0.0, 1.0);
                            p[1] += -cl * u * 0.26 * t * t;
                            p[0] *= 1.0 + 0.3 * (cwd * t * t * t + jw * t);
                        }
                        // Gesichtslänge: unteres Gesicht verschieben
                        let fl = v("face-length");
                        if nz(fl) && p[1] < lm.eye_l[1] && p[1] > lm.chin[1] - u * 0.8 && p[2] > lm.head[2] - u * 0.6 { let t = ((lm.eye_l[1] - p[1]) / (lm.eye_l[1] - lm.chin[1]).abs().max(1e-3)).clamp(0.0, 1.3); p[1] -= fl * u * 0.32 * t; }
                        // Ohren
                        let (es, el_) = (v("ear-size"), v("ear-length"));
                        for ear in [lm.ear_l, lm.ear_r] {
                            let w = fall(dist(p, ear), u * 0.75);
                            if w > 0.0 && (nz(es) || nz(el_)) { let up = if p[1] > ear[1] { 1.0 } else { 0.0 }; p[0] = ear[0] + (p[0] - ear[0]) * (1.0 + 0.5 * es * w); p[1] = ear[1] + (p[1] - ear[1]) * (1.0 + 0.5 * es * w + 1.4 * el_ * w * up); p[2] = ear[2] + (p[2] - ear[2]) * (1.0 + 0.5 * es * w); }
                        }
                    }
                }
                // ---- Körper (nur Teile ohne Gesicht/Haar-Kopfbereich)
                if body_active && kind != Kind::Face && kind != Kind::Eye {
                    let (bs, bh, bsp) = (v("breast-size"), v("breast-height"), v("breast-spacing"));
                    if nz(bs) || nz(bh) || nz(bsp) {
                        for c0 in [lm.breast_l, lm.breast_r] {
                            let c = [c0[0] + c0[0].signum() * bsp * u * 0.2, c0[1] + bh * u * 0.3, c0[2]];
                            let w = fall(dist([p[0], p[1], p[2] + 0.02], c), u * 1.35) * if p[2] > lm.belly[2] - 0.12 { 1.0 } else { 0.0 };
                            if w > 0.0 { let out = [p[0] - c[0], p[1] - c[1], 0.8 * u]; let l = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2]).sqrt().max(1e-6); for q in 0..3 { p[q] += out[q] / l * bs * u * 0.5 * w; } p[0] += c0[0].signum() * bsp * u * 0.2 * w; p[1] += bh * u * 0.3 * w; }
                        }
                    }
                    let (bl, wa) = (v("belly"), v("waist"));
                    if nz(bl) { let w = fall(dist(p, lm.belly), u * 1.9); if w > 0.0 && p[2] > lm.belly[2] - 0.1 { p[2] += bl * u * 0.55 * w; } }
                    if nz(wa) { let ty = lm.belly[1] + u * 0.6; let w = fall((p[1] - ty).abs(), u * 2.2); p[0] *= 1.0 + 0.22 * wa * w; p[2] = lm.belly[2] - (lm.belly[2] - p[2]) * (1.0 + 0.0) + 0.0; }
                    let gs = v("glute-size");
                    if nz(gs) { for c in [lm.glute_l, lm.glute_r] { let w = fall(dist(p, c), u * 1.6) * if p[2] < lm.belly[2] - 0.05 { 1.0 } else { 0.0 }; if w > 0.0 { p[2] -= gs * u * 0.55 * w; p[0] += c[0].signum() * gs * u * 0.12 * w; } } }
                    let td = v("torso-depth");
                    if nz(td) { let w = fall((p[1] - (lm.belly[1] + (lm.neck[1] - lm.belly[1]) * 0.4)).abs(), (lm.neck[1] - lm.belly[1]) * 0.55); let cz = lm.belly[2] - 0.08; p[2] = cz + (p[2] - cz) * (1.0 + 0.25 * td * w); }
                }
                dst[k] = p;
            }
        }
    }
    skeleton(m, &out, ob, vals)
}

fn norm(a: [f32; 3]) -> [f32; 3] { let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-9); [a[0] / l, a[1] / l, a[2] / l] }
type M3 = [[f32; 3]; 3];
fn mv(m: &M3, v: [f32; 3]) -> [f32; 3] { [m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2], m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2], m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2]] }
fn ident() -> M3 { [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] }
/// Streckung `f` entlang `a`, quer dazu `t`.
fn stretch(a: [f32; 3], f: f32, t: f32) -> M3 {
    let mut m = ident();
    for i in 0..3 { for j in 0..3 { m[i][j] = t * (if i == j { 1.0 } else { 0.0 } - a[i] * a[j]) + f * a[i] * a[j]; } }
    m
}

/// Knochen-Skinning: pro Bone eine lineare Abbildung um sein Gelenk; Gelenkpositionen werden von der Wurzel aus neu aufgebaut.
fn skeleton(m: &VrmModel, regional: &[Vec<[f32; 3]>], ob: &[[f32; 3]], vals: &[f32]) -> Deformed {
    let v = |id: &str| index_of(id).map(|i| vals[i]).unwrap_or(0.0);
    let n = m.bones.len();
    let hname: Vec<Option<&str>> = (0..n).map(|b| m.vrm_names[b].as_deref()).collect();
    // Operation je Bone (nächster humanoider Vorfahre bestimmt, wenn das Bone selbst keinen Namen hat)
    let mut group: Vec<Option<&str>> = hname.clone();
    for b in 0..n { if group[b].is_none() { let mut p = m.bones[b].parent; while let Some(x) = p { if hname[x].is_some() { group[b] = hname[x]; break; } p = m.bones[x].parent; } } }
    let children = |b: usize| -> Vec<usize> { (0..n).filter(|&c| m.bones[c].parent == Some(b)).collect() };
    let main_child = |b: usize| -> Option<usize> {
        let kids = children(b);
        // bevorzugt humanoides Kind entlang der Kette
        kids.iter().copied().find(|&c| hname[c].is_some()).or_else(|| kids.first().copied())
    };
    let axis_of = |b: usize| -> [f32; 3] { match main_child(b) { Some(c) => norm(sub(ob[c], ob[b])), None => [0.0, 1.0, 0.0] } };
    let hl = |name: &str| name.starts_with("left") || name.starts_with("right");
    let mut a_mat: Vec<M3> = vec![ident(); n];
    let mut extra: Vec<[f32; 3]> = vec![[0.0; 3]; n];
    let u = 0.18f32;
    for b in 0..n {
        let Some(g) = group[b] else { continue };
        let ax = axis_of(b);
        let mut mat = ident();
        let side = if g.starts_with("left") { 1.0 } else if g.starts_with("right") { -1.0 } else { 0.0 };
        let _ = hl;
        match g {
            "hips" => { let hw = v("hip-width"); mat = [[1.0 + 0.30 * hw, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0 + 0.12 * hw]]; }
            "spine" | "chest" | "upperChest" => { mat = stretch(ax, 1.0 + 0.4 * v("torso-length"), 1.0); }
            "neck" => { mat = stretch(ax, 1.0 + 0.6 * v("neck-length"), 1.0 + 0.3 * v("neck-thickness")); }
            "head" | "leftEye" | "rightEye" | "jaw" => { let s = 1.0 + 0.28 * v("head-size"); mat = [[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]]; }
            "leftShoulder" | "rightShoulder" => { extra[b] = [side * 0.0, 0.0, 0.0]; }
            "leftUpperArm" | "rightUpperArm" | "leftLowerArm" | "rightLowerArm" => { mat = stretch(ax, 1.0 + 0.4 * v("arm-length"), 1.0 + 0.35 * v("arm-thickness")); if g.ends_with("UpperArm") { extra[b] = [side * 0.07 * v("shoulder-width"), 0.0, 0.0]; } }
            "leftUpperLeg" | "rightUpperLeg" | "leftLowerLeg" | "rightLowerLeg" => { mat = stretch(ax, 1.0 + 0.4 * v("leg-length"), 1.0 + 0.35 * v("leg-thickness")); if g.ends_with("UpperLeg") { extra[b] = [side * 0.04 * v("hip-width"), 0.0, 0.0]; } }
            _ if g.ends_with("Hand") || g.contains("Thumb") || g.contains("Index") || g.contains("Middle") || g.contains("Ring") || g.contains("Little") => { let s = 1.0 + 0.4 * v("hand-size"); mat = [[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]]; }
            _ if g.ends_with("Foot") || g.ends_with("Toes") => { let s = 1.0 + 0.4 * v("foot-size"); mat = [[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]]; }
            _ => {}
        }
        a_mat[b] = mat;
    }
    let _ = u;
    // neue Gelenkpositionen (Eltern vor Kindern)
    let mut order: Vec<usize> = Vec::new(); let mut done = vec![false; n];
    while order.len() < n { let before = order.len(); for b in 0..n { if !done[b] && m.bones[b].parent.map(|p| done[p]).unwrap_or(true) { done[b] = true; order.push(b); } } if order.len() == before { break; } }
    let mut np: Vec<[f32; 3]> = ob.to_vec();
    for &b in &order {
        match m.bones[b].parent {
            Some(p) => { let d = mv(&a_mat[p], sub(ob[b], ob[p])); np[b] = [np[p][0] + d[0] + extra[b][0], np[p][1] + d[1] + extra[b][1], np[p][2] + d[2] + extra[b][2]]; }
            None => { np[b] = ob[b]; }
        }
    }
    // Skinning der Vertices
    let mut pos: Vec<Vec<[f32; 3]>> = Vec::with_capacity(regional.len());
    for (pi, part) in m.parts.iter().enumerate() {
        let src = &regional[pi];
        let mut dst = Vec::with_capacity(src.len());
        for (k, p) in src.iter().enumerate() {
            let (mut o, mut tot) = ([0f32; 3], 0f32);
            for c in 0..4 {
                let w = part.weights[k][c]; if w <= 0.0 { continue; }
                let b = part.joints[k][c] as usize; if b >= n { continue; }
                let d = mv(&a_mat[b], sub(*p, ob[b]));
                for q in 0..3 { o[q] += w * (np[b][q] + d[q]); }
                tot += w;
            }
            dst.push(if tot > 0.0 { [o[0] / tot, o[1] / tot, o[2] / tot] } else { *p });
        }
        pos.push(dst);
    }
    // Gesamthöhe (um den Boden y = 0)
    let h = 1.0 + 0.25 * v("height");
    if (h - 1.0).abs() > 1e-4 { for part in pos.iter_mut() { for q in part.iter_mut() { for c in 0..3 { q[c] *= h; } } } for q in np.iter_mut() { for c in 0..3 { q[c] *= h; } } }
    Deformed { pos, bones: np }
}

/// Werte als Karte (id → Wert) für Speichern.
pub fn to_map(vals: &[f32]) -> BTreeMap<String, f32> { DEFS.iter().zip(vals).filter(|(d, v)| (**v - d.def).abs() > 1e-6).map(|(d, v)| (d.id.to_string(), *v)).collect() }
pub fn from_map(map: &BTreeMap<String, f32>) -> Vec<f32> { let mut v = defaults(); for (k, x) in map { if let Some(i) = index_of(k) { v[i] = *x; } } v }
