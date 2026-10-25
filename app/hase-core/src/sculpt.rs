//! Direkte Geometrieformung: Pinsel mit weichem Abfall, Strahl-Pick auf Dreiecksnetzen. Die Formung wird als dünn besetzte Verschiebungsebene
//! (Teil, Vertex → Delta) neben den Parametern gespeichert und nach der parametrischen Verformung addiert – reproduzierbar, speicher- und wiederverwendbar.
use std::collections::BTreeMap;

pub type V3 = [f32; 3];
pub type Layer = Vec<BTreeMap<u32, V3>>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrushKind { Grab, Inflate, Smooth, Scale }
impl BrushKind {
    pub const ALL: [BrushKind; 4] = [BrushKind::Grab, BrushKind::Inflate, BrushKind::Smooth, BrushKind::Scale];
    pub fn label(self) -> &'static str { match self { BrushKind::Grab => "Ziehen", BrushKind::Inflate => "Aufblasen", BrushKind::Smooth => "Glätten", BrushKind::Scale => "Skalieren" } }
    pub fn tip(self) -> &'static str { match self { BrushKind::Grab => "Bereich am Modell greifen und ziehen", BrushKind::Inflate => "Ziehen nach oben = ausbeulen, nach unten = eindellen", BrushKind::Smooth => "Unebenheiten glätten", BrushKind::Scale => "Ziehen nach oben = Bereich vergrößern, nach unten = verkleinern" } }
}

#[derive(Clone, Copy, Debug)]
pub struct Brush { pub kind: BrushKind, pub radius: f32, pub strength: f32, pub symmetric: bool }
impl Default for Brush { fn default() -> Self { Brush { kind: BrushKind::Grab, radius: 0.06, strength: 1.0, symmetric: true } } }

pub fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
pub fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
pub fn mul(a: V3, k: f32) -> V3 { [a[0] * k, a[1] * k, a[2] * k] }
pub fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
pub fn len(a: V3) -> f32 { dot(a, a).sqrt() }
pub fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
pub fn mirror(p: V3) -> V3 { [-p[0], p[1], p[2]] }

/// Weicher Abfall: 1 im Zentrum, 0 ab Radius.
pub fn fall(d: f32, r: f32) -> f32 { if d >= r { 0.0 } else { let t = 1.0 - (d / r) * (d / r); t * t } }

/// Strahl gegen ein Dreiecksnetz (Möller–Trumbore); liefert (t, Dreieck).
pub fn ray_mesh(o: V3, d: V3, pos: &[V3], idx: &[u32]) -> Option<(f32, usize)> {
    let mut best: Option<(f32, usize)> = None;
    for (ti, t) in idx.chunks_exact(3).enumerate() {
        let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
        let (e1, e2) = (sub(b, a), sub(c, a));
        let p = cross(d, e2); let det = dot(e1, p);
        if det.abs() < 1e-12 { continue; }
        let inv = 1.0 / det; let s = sub(o, a);
        let u = dot(s, p) * inv; if !(0.0..=1.0).contains(&u) { continue; }
        let q = cross(s, e1); let v = dot(d, q) * inv; if v < 0.0 || u + v > 1.0 { continue; }
        let tt = dot(e2, q) * inv;
        if tt > 1e-4 && best.map(|b| tt < b.0).unwrap_or(true) { best = Some((tt, ti)); }
    }
    best
}

/// Vertex-Nachbarschaft eines Netzes (für Glätten).
pub fn adjacency(n: usize, idx: &[u32]) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for t in idx.chunks_exact(3) { for k in 0..3 { let (a, b) = (t[k], t[(k + 1) % 3]); if !adj[a as usize].contains(&b) { adj[a as usize].push(b); } if !adj[b as usize].contains(&a) { adj[b as usize].push(a); } } }
    adj
}

/// Dünn besetzte Ebene → Textform für das Speichern (flache Zahlenlisten je Teil).
pub fn to_json(l: &Layer) -> serde_json::Value {
    serde_json::Value::Array(l.iter().map(|m| serde_json::Value::Array(m.iter().flat_map(|(i, d)| [serde_json::json!(i), serde_json::json!((d[0] * 1e5).round() / 1e5), serde_json::json!((d[1] * 1e5).round() / 1e5), serde_json::json!((d[2] * 1e5).round() / 1e5)]).collect())).collect())
}
pub fn from_json(v: &serde_json::Value, parts: usize) -> Layer {
    let mut l: Layer = vec![BTreeMap::new(); parts];
    for (pi, a) in v.as_array().into_iter().flatten().enumerate().take(parts) {
        let nums: Vec<f64> = a.as_array().into_iter().flatten().filter_map(|x| x.as_f64()).collect();
        for c in nums.chunks_exact(4) { l[pi].insert(c[0] as u32, [c[1] as f32, c[2] as f32, c[3] as f32]); }
    }
    l
}
