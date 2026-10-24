//! Anatomisches Morph-Modell (HASE): zusätzliche Targets, die auf dem Basismesh prozedural entstehen und an gemessenen Landmarken
//! (Brustwarzen-Positionen aus den MakeHuman-Targets) verankert sind. Sie ergänzen die MakeHuman-Targets um getrennte, anatomisch
//! motivierte Parameter (statt globaler Skalierung): glatte Falloffs, Verformung entlang der Hautnormalen, Übergang zum Brustkorb.
use crate::body::Morphs;

/// Beschreibung eines generierten Parameters: (Id-Stamm, Anzeigename, signiert, min).
pub struct AnatomyParam { pub stem: &'static str, pub label: &'static str, pub signed: bool }

pub const BREAST_PARAMS: &[AnatomyParam] = &[
    AnatomyParam { stem: "breast-volume", label: "Brust Volumen", signed: true },
    AnatomyParam { stem: "breast-projection", label: "Brust Projektion", signed: true },
    AnatomyParam { stem: "breast-upper-fullness", label: "Brust Obere Fülle", signed: true },
    AnatomyParam { stem: "breast-lower-fullness", label: "Brust Untere Fülle", signed: true },
    AnatomyParam { stem: "breast-width", label: "Brust Breite", signed: true },
    AnatomyParam { stem: "breast-mount-height", label: "Brust Ansatzhöhe", signed: true },
    AnatomyParam { stem: "breast-sag", label: "Brust Hängen (Schwerkraft)", signed: true },
];

fn sstep(a: f32, b: f32, x: f32) -> f32 { let t = ((x - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
fn add(a: [f32; 3], b: [f32; 3], k: f32) -> [f32; 3] { [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn norm(a: [f32; 3]) -> [f32; 3] { let l = dot(a, a).sqrt().max(1e-9); [a[0] / l, a[1] / l, a[2] / l] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

pub const GLUTE_PARAMS: &[AnatomyParam] = &[
    AnatomyParam { stem: "glute-volume", label: "Gesäß Volumen", signed: true },
    AnatomyParam { stem: "glute-projection", label: "Gesäß Projektion", signed: true },
    AnatomyParam { stem: "glute-lift", label: "Gesäß Ansatzhöhe (Straffung)", signed: true },
    AnatomyParam { stem: "glute-width", label: "Gesäß Breite", signed: true },
];

/// Brustwarzen-Position je Seite (+x = links des Charakters) aus dem Schwerpunkt des `nipple-size-incr`-Targets.
fn nipples(m: &Morphs, base: &[[f32; 3]]) -> Option<[[f32; 3]; 2]> {
    let t = m.index_of("breast/nipple-size-incr")?;
    let (a, b) = (m.t_off[t] as usize, m.t_off[t + 1] as usize);
    let (mut c, mut w) = ([[0f32; 3]; 2], [0f32; 2]);
    for k in a..b {
        let p = base[m.t_idx[k] as usize]; let d = m.t_delta[k]; let wt = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let s = if p[0] >= 0.0 { 0 } else { 1 };
        for i in 0..3 { c[s][i] += p[i] * wt; } w[s] += wt;
    }
    if w[0] <= 0.0 || w[1] <= 0.0 { return None; }
    Some([[c[0][0] / w[0], c[0][1] / w[0], c[0][2] / w[0]], [c[1][0] / w[1], c[1][1] / w[1], c[1][2] / w[1]]])
}

/// Erzeugt die Brust-Targets (je Parameter positive und negative Richtung, links/rechts) und hängt sie an `m` an.
/// Rückgabe: Anzahl erzeugter Targets (0, wenn die Landmarken fehlen, z. B. bei fremden Körperbasen).
pub fn append_breast(m: &mut Morphs, base: &[[f32; 3]], normals: &[[f32; 3]]) -> usize {
    let Some(nip) = nipples(m, base) else { return 0 };
    let mut made = 0;
    for (si, side) in ["l", "r"].iter().enumerate() {
        let n0 = nip[si];
        // lokales Koordinatensystem: f = Hautnormale an der Brustwarze (Mittel der Nachbarn), u = oben, w = lateral (nach außen)
        let mut f = [0f32; 3]; let mut cnt = 0;
        for (i, p) in base.iter().enumerate() { let d = [p[0] - n0[0], p[1] - n0[1], p[2] - n0[2]]; if dot(d, d) < 0.012f32.powi(2) { f = add(f, normals[i], 1.0); cnt += 1; } }
        if cnt == 0 { continue; }
        let f = norm(f);
        let u = norm(add([0.0, 1.0, 0.0], f, -f[1]));
        let sign = if si == 0 { 1.0 } else { -1.0 };
        let w = norm(cross(u, f)); let w = if dot(w, [sign, 0.0, 0.0]) < 0.0 { [-w[0], -w[1], -w[2]] } else { w };
        let (rw, ru) = (0.115f32, 0.125f32); // Fußabdruck-Radien (m)
        let mut fields: Vec<(&'static str, Vec<(u32, [f32; 3])>)> = BREAST_PARAMS.iter().map(|p| (p.stem, Vec::new())).collect();
        for (vi, p) in base.iter().enumerate() {
            let d = [p[0] - n0[0], p[1] - n0[1], p[2] - n0[2]];
            let (a, b, c) = (dot(d, w), dot(d, u), dot(d, f));
            // nur Vorderseite des Brustkorbs (nicht die Rückseite/Arme): vor der Ebene durch die Brustwarze und nahe an ihr
            if c < -0.07 { continue; }
            let rho2 = (a / rw).powi(2) + (b / ru).powi(2);
            if rho2 >= 1.0 { continue; }
            let phi = (1.0 - rho2).powi(2); // glatter Falloff, 0 am Rand (C1-stetig)
            let rho = rho2.sqrt();
            let nrm = normals[vi];
            // Verformungsrichtung: Mischung aus Hautnormale und Brust-Vorwärtsrichtung, damit der Übergang zum Brustkorb glatt bleibt
            let dir = norm(add(add([0.0; 3], nrm, 0.55), f, 0.45));
            let up_w = sstep(-0.35, 0.85, b / ru);      // 0 unten … 1 oben
            let lo_w = 1.0 - sstep(-0.85, 0.35, b / ru);
            let ring = (rho * (1.0 - rho)).max(0.0) * 4.0; // Maximum bei halbem Radius
            let tang = norm([a * w[0] + b * u[0], a * w[1] + b * u[1], a * w[2] + b * u[2]]);
            let mut put = |stem: &str, dv: [f32; 3]| { if let Some(e) = fields.iter_mut().find(|e| e.0 == stem) { e.1.push((vi as u32, dv)); } };
            put("breast-volume", [dir[0] * 0.032 * phi + w[0] * 0.008 * phi, dir[1] * 0.032 * phi + w[1] * 0.008 * phi, dir[2] * 0.032 * phi + w[2] * 0.008 * phi]);
            let nar = (1.0 - rho2 / 0.36).max(0.0).powi(2);
            put("breast-projection", [f[0] * 0.030 * nar, f[1] * 0.030 * nar, f[2] * 0.030 * nar]);
            put("breast-upper-fullness", [dir[0] * 0.026 * phi * up_w, dir[1] * 0.026 * phi * up_w, dir[2] * 0.026 * phi * up_w]);
            put("breast-lower-fullness", [dir[0] * 0.026 * phi * lo_w, dir[1] * 0.026 * phi * lo_w, dir[2] * 0.026 * phi * lo_w]);
            put("breast-width", [tang[0] * 0.020 * ring, tang[1] * 0.020 * ring * 0.6, tang[2] * 0.020 * ring]);
            let wide = (1.0 - rho2 / 1.44).max(0.0).powi(2);
            put("breast-mount-height", [u[0] * 0.025 * wide, u[1] * 0.025 * wide, u[2] * 0.025 * wide]);
            // Hängen: untere Masse sinkt, Brustwarze folgt, oberer Bereich flacht leicht ab
            put("breast-sag", [-u[0] * 0.030 * phi * (0.35 + 0.65 * lo_w) + f[0] * -0.006 * phi * up_w, -u[1] * 0.030 * phi * (0.35 + 0.65 * lo_w) + f[1] * -0.006 * phi * up_w, -u[2] * 0.030 * phi * (0.35 + 0.65 * lo_w) + f[2] * -0.006 * phi * up_w]);
        }
        for (stem, v) in fields {
            if v.is_empty() { continue; }
            let neg_k = match stem { "breast-volume" => -0.55, "breast-projection" => -0.6, "breast-upper-fullness" | "breast-lower-fullness" => -0.7, "breast-width" => -0.6, "breast-mount-height" => -1.0, _ => -0.7 };
            for (suffix, k) in [("pos", 1.0f32), ("neg", neg_k)] {
                let name = format!("hase/{stem}-{side}-{suffix}");
                for (vi, d) in &v { m.t_idx.push(*vi); m.t_delta.push([d[0] * k, d[1] * k, d[2] * k]); }
                m.t_off.push(m.t_idx.len() as u32); m.names.push(name);
                made += 1;
            }
        }
    }
    made
}

/// Gesäß-Targets, verankert am Schwerpunkt des MakeHuman-`buttocks-volume-incr`-Targets (je Seite).
pub fn append_glutes(m: &mut Morphs, base: &[[f32; 3]], normals: &[[f32; 3]]) -> usize {
    let Some(t) = m.index_of("buttocks/buttocks-volume-incr") else { return 0 };
    let (a, b) = (m.t_off[t] as usize, m.t_off[t + 1] as usize);
    let (mut c, mut w) = ([[0f32; 3]; 2], [0f32; 2]);
    for k in a..b { let p = base[m.t_idx[k] as usize]; let d = m.t_delta[k]; let wt = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(); let s = if p[0] >= 0.0 { 0 } else { 1 }; for i in 0..3 { c[s][i] += p[i] * wt; } w[s] += wt; }
    if w[0] <= 0.0 || w[1] <= 0.0 { return 0; }
    let mut made = 0;
    for (si, side) in ["l", "r"].iter().enumerate() {
        let cen = [c[si][0] / w[si], c[si][1] / w[si], c[si][2] / w[si]];
        // Schwerpunkt liegt wegen der Krümmung evtl. unter der Oberfläche → nächster Rückseiten-Vertex als Anker
        let n0 = base.iter().enumerate().filter(|(i, p)| normals[*i][2] < -0.3 && (p[0] >= 0.0) == (si == 0)).min_by(|a, b| { let da = (a.1[0] - cen[0]).powi(2) + (a.1[1] - cen[1]).powi(2) + (a.1[2] - cen[2]).powi(2); let db = (b.1[0] - cen[0]).powi(2) + (b.1[1] - cen[1]).powi(2) + (b.1[2] - cen[2]).powi(2); da.partial_cmp(&db).unwrap() }).map(|(_, p)| *p).unwrap_or(cen);
        let mut f = [0f32; 3]; let mut cnt = 0;
        for (i, p) in base.iter().enumerate() { let d = [p[0] - n0[0], p[1] - n0[1], p[2] - n0[2]]; if dot(d, d) < 0.03f32.powi(2) && normals[i][2] < 0.0 { f = add(f, normals[i], 1.0); cnt += 1; } }
        if cnt == 0 { continue; }
        let f = norm(f);
        let u = norm(add([0.0, 1.0, 0.0], f, -f[1]));
        let sign = if si == 0 { 1.0 } else { -1.0 };
        let w = norm(cross(u, f)); let w = if dot(w, [sign, 0.0, 0.0]) < 0.0 { [-w[0], -w[1], -w[2]] } else { w };
        let (rw, ru) = (0.12f32, 0.13f32);
        let mut fields: Vec<(&'static str, Vec<(u32, [f32; 3])>)> = GLUTE_PARAMS.iter().map(|p| (p.stem, Vec::new())).collect();
        for (vi, p) in base.iter().enumerate() {
            let d = [p[0] - n0[0], p[1] - n0[1], p[2] - n0[2]];
            let (aa, bb, cc) = (dot(d, w), dot(d, u), dot(d, f));
            if cc < -0.06 { continue; }
            let rho2 = (aa / rw).powi(2) + (bb / ru).powi(2);
            if rho2 >= 1.0 { continue; }
            let phi = (1.0 - rho2).powi(2); let rho = rho2.sqrt();
            let nrm = normals[vi];
            let dir = norm(add(add([0.0; 3], nrm, 0.5), f, 0.5));
            let nar = (1.0 - rho2 / 0.6).max(0.0).powi(2);
            let ring = (rho * (1.0 - rho)).max(0.0) * 4.0;
            let tang = norm([aa * w[0] + bb * u[0], aa * w[1] + bb * u[1], aa * w[2] + bb * u[2]]);
            let up_w = sstep(-0.4, 0.8, bb / ru);
            let mut put = |stem: &str, dv: [f32; 3]| { if let Some(e) = fields.iter_mut().find(|e| e.0 == stem) { e.1.push((vi as u32, dv)); } };
            put("glute-volume", [dir[0] * 0.030 * phi, dir[1] * 0.030 * phi, dir[2] * 0.030 * phi]);
            put("glute-projection", [f[0] * 0.024 * nar, f[1] * 0.024 * nar, f[2] * 0.024 * nar]);
            put("glute-lift", [u[0] * 0.028 * phi * (0.4 + 0.6 * (1.0 - up_w)), u[1] * 0.028 * phi * (0.4 + 0.6 * (1.0 - up_w)), u[2] * 0.028 * phi * (0.4 + 0.6 * (1.0 - up_w))]);
            put("glute-width", [tang[0] * 0.022 * ring, tang[1] * 0.022 * ring * 0.5, tang[2] * 0.022 * ring]);
        }
        for (stem, v) in fields {
            if v.is_empty() { continue; }
            for (suffix, k) in [("pos", 1.0f32), ("neg", -0.6)] {
                let name = format!("hase/{stem}-{side}-{suffix}");
                for (vi, d) in &v { m.t_idx.push(*vi); m.t_delta.push([d[0] * k, d[1] * k, d[2] * k]); }
                m.t_off.push(m.t_idx.len() as u32); m.names.push(name); made += 1;
            }
        }
    }
    made
}
