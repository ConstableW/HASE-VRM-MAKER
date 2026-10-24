//! Morph-Evaluator: Sparse-Scatter mit inkrementeller Aktualisierung (nur geänderte Targets).
use crate::body::Body;
use std::time::Instant;

pub struct Evaluator {
    pub pos: Vec<[f32; 3]>,
    applied: Vec<f32>,
    drift: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ApplyStats { pub changed_targets: u32, pub active_targets: u32, pub touched: u32, pub full: bool, pub ms: f64 }

impl Evaluator {
    pub fn new(body: &Body) -> Self { Self { pos: body.base_pos.clone(), applied: vec![0.0; body.morphs.names.len()], drift: 0 } }

    /// Wendet Zielgewichte an. Gibt zurück, ob sich Positionen geändert haben.
    pub fn apply(&mut self, body: &Body, desired: &[f32]) -> Option<ApplyStats> {
        let t0 = Instant::now();
        let m = &body.morphs;
        let changed: Vec<usize> = (0..desired.len()).filter(|&t| (desired[t] - self.applied[t]).abs() > 1e-6).collect();
        if changed.is_empty() { return None; }
        let active = desired.iter().filter(|w| w.abs() > 1e-6).count() as u32;
        let full = changed.len() > 120 || self.drift > 20_000;
        let mut touched = 0u32;
        if full {
            self.pos.copy_from_slice(&body.base_pos);
            for (t, &w) in desired.iter().enumerate() {
                if w.abs() > 1e-6 { touched += scatter(&mut self.pos, m.t_off[t] as usize, m.t_off[t + 1] as usize, &m.t_idx, &m.t_delta, w); }
            }
            self.applied.copy_from_slice(desired);
            self.drift = 0;
        } else {
            for &t in &changed {
                let dw = desired[t] - self.applied[t];
                touched += scatter(&mut self.pos, m.t_off[t] as usize, m.t_off[t + 1] as usize, &m.t_idx, &m.t_delta, dw);
                self.applied[t] = desired[t];
            }
            self.drift += changed.len() as u32;
        }
        Some(ApplyStats { changed_targets: changed.len() as u32, active_targets: active, touched, full, ms: t0.elapsed().as_secs_f64() * 1e3 })
    }
}

#[inline]
fn scatter(pos: &mut [[f32; 3]], a: usize, b: usize, idx: &[u32], delta: &[[f32; 3]], w: f32) -> u32 {
    for e in a..b { let p = &mut pos[idx[e] as usize]; let d = delta[e]; p[0] += w * d[0]; p[1] += w * d[1]; p[2] += w * d[2]; }
    (b - a) as u32
}

/// Glatte Vertex-Normalen aus Dreiecken auf OBJ-Vertices.
pub fn normals(pos: &[[f32; 3]], tris: &[[u32; 3]], out: &mut Vec<[f32; 3]>) {
    out.clear();
    out.resize(pos.len(), [0.0; 3]);
    for t in tris {
        let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
        let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        for &i in t { let o = &mut out[i as usize]; o[0] += n[0]; o[1] += n[1]; o[2] += n[2]; }
    }
    for n in out.iter_mut() { let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12); *n = [n[0] / l, n[1] / l, n[2] / l]; }
}
