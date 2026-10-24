//! Proxy-Fitting (.mhclo): Barycentric-Bindung an Basis-Vertices + Offset, skaliert mit aktueller Körpergröße.
use crate::body::SCALE;
use crate::obj::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Proxy {
    pub name: String,
    pub obj_pos_len: usize,
    pub ia: Vec<u32>, pub ib: Vec<u32>, pub ic: Vec<u32>,
    pub w: Vec<[f32; 3]>,
    pub off: Vec<[f32; 3]>,
    pub scale: [(u32, u32, f32); 3],
    pub topo: RenderTopo,
    pub texture: Option<PathBuf>,
    pub alpha_mask: bool,
    pub diffuse: [f32; 3],
}

fn parse_mhmat(p: &Path) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(s) = std::fs::read_to_string(p) {
        for l in s.lines() { let t: Vec<&str> = l.split_whitespace().collect(); if t.len() >= 2 && !t[0].starts_with('#') && !t[0].starts_with("//") { m.insert(t[0].to_string(), t[1..].join(" ")); } }
    }
    m
}

impl Proxy {
    pub fn load(mhclo: &Path) -> Result<Proxy, String> {
        let dir = mhclo.parent().ok_or("kein Verzeichnis")?;
        let txt = std::fs::read_to_string(mhclo).map_err(|e| format!("{}: {e}", mhclo.display()))?;
        let (mut scale, mut obj_file, mut material, mut in_verts) = ([(0u32, 0u32, 1.0f32); 3], String::new(), String::new(), false);
        let (mut ia, mut ib, mut ic, mut w, mut off) = (vec![], vec![], vec![], vec![], vec![]);
        for l in txt.lines() {
            let t: Vec<&str> = l.split_whitespace().collect();
            if t.is_empty() || t[0].starts_with('#') { continue; }
            match t[0] {
                "verts" => in_verts = true,
                "delete_verts" | "weights" => in_verts = false,
                "x_scale" | "y_scale" | "z_scale" if t.len() >= 4 => { let i = match t[0] { "x_scale" => 0, "y_scale" => 1, _ => 2 }; scale[i] = (t[1].parse().unwrap_or(0), t[2].parse().unwrap_or(0), t[3].parse().unwrap_or(1.0)); in_verts = false; }
                "obj_file" => obj_file = t[1..].join(" "),
                "material" => material = t[1..].join(" "),
                _ if in_verts && t.len() == 9 => {
                    let f: Vec<f32> = t.iter().map(|x| x.parse().unwrap_or(0.0)).collect();
                    ia.push(f[0] as u32); ib.push(f[1] as u32); ic.push(f[2] as u32); w.push([f[3], f[4], f[5]]); off.push([f[6], f[7], f[8]]);
                }
                _ => {}
            }
        }
        let obj = parse_obj(&dir.join(&obj_file)).map_err(|e| format!("{obj_file}: {e}"))?;
        if obj.pos.len() != ia.len() { return Err(format!("{}: Vertexanzahl {} ≠ Bindungen {}", mhclo.display(), obj.pos.len(), ia.len())); }
        let topo = build_render_topo(&obj, |_| true);
        let mat = if material.is_empty() { Default::default() } else { parse_mhmat(&dir.join(&material)) };
        let texture = mat.get("diffuseTexture").map(|t| dir.join(t)).filter(|p| p.exists());
        let alpha_mask = mat.get("transparent").map(|v| v.eq_ignore_ascii_case("true")).unwrap_or(false);
        let diffuse = mat.get("diffuseColor").map(|v| { let f: Vec<f32> = v.split_whitespace().filter_map(|x| x.parse().ok()).collect(); if f.len() >= 3 { [f[0], f[1], f[2]] } else { [1.0; 3] } }).unwrap_or([1.0; 3]);
        Ok(Proxy { name: mhclo.file_stem().unwrap().to_string_lossy().into(), obj_pos_len: obj.pos.len(), ia, ib, ic, w, off, scale, topo, texture, alpha_mask, diffuse })
    }

    /// Passt das Proxy an die aktuelle (gemorphte) Basis an. `out` erhält die Positionen aller Proxy-OBJ-Vertices.
    pub fn fit(&self, base: &[[f32; 3]], out: &mut Vec<[f32; 3]>) {
        let mut sc = [1.0f32; 3];
        for i in 0..3 { let (a, b, den) = self.scale[i]; if den > 0.0 { sc[i] = (base[a as usize][i] - base[b as usize][i]).abs() / (den * SCALE); } }
        out.clear();
        out.reserve(self.ia.len());
        for k in 0..self.ia.len() {
            let (a, b, c, w, o) = (base[self.ia[k] as usize], base[self.ib[k] as usize], base[self.ic[k] as usize], self.w[k], self.off[k]);
            out.push([
                w[0] * a[0] + w[1] * b[0] + w[2] * c[0] + o[0] * sc[0] * SCALE,
                w[0] * a[1] + w[1] * b[1] + w[2] * c[1] + o[1] * sc[1] * SCALE,
                w[0] * a[2] + w[1] * b[2] + w[2] * c[2] + o[2] * sc[2] * SCALE,
            ]);
        }
    }
}
