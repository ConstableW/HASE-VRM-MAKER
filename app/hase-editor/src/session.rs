//! Session: Projektzustand + Evaluator + Proxy-Laufzeit; erzeugt Render-Updates. Kein egui-/wgpu-Gerätebezug (nur Datenübergabe).
use crate::render::{ImageRgba, MeshDesc, MeshUpdate};
use hase_core::assets::{self, AssetEntry, Category};
use hase_core::body::Body;
use hase_core::eval::{normals, Evaluator};
use hase_core::fit::Proxy;
use hase_core::packages::Packages;
use hase_core::params::Catalog;
use hase_core::state::{AssetSel, CharacterState, History, PkgRef};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

pub struct LoadedProxy { pub proxy: Proxy, pub texture: Option<Arc<ImageRgba>> }

struct ProxyRt { proxy: Proxy, texture: Option<Arc<ImageRgba>>, fit: Vec<[f32; 3]>, nrm: Vec<[f32; 3]>, created: bool }

#[derive(Default, Clone, Copy)]
pub struct Perf { pub morph_ms: f64, pub fit_ms: f64, pub total_ms: f64, pub changed: u32, pub active: u32, pub full: bool }

pub struct Session {
    pub repo: PathBuf,
    pub packages: Packages,
    pub body: Body,
    pub params: Catalog,
    pub assets: assets::Catalog,
    pub eval: Evaluator,
    pub state: CharacterState,
    pub history: History,
    pub perf: Perf,
    pub symmetric: bool,
    proxies: HashMap<String, ProxyRt>,
    textures: HashMap<PathBuf, Arc<ImageRgba>>,
    pending: Vec<MeshUpdate>,
    desired: Vec<f32>,
    body_nrm: Vec<[f32; 3]>,
    body_created: bool,
    floor_off: f32,
    skin_tex: Option<Arc<ImageRgba>>,
    dirty: bool,
    assets_dirty: bool,
}

pub const EYES_ID: &str = "eyes/high-poly";

pub fn load_image(path: &Path, max_dim: u32) -> Result<Arc<ImageRgba>, String> {
    let img = image::open(path).map_err(|e| format!("{}: {e}", path.display()))?.to_rgba8();
    let (mut w, mut h) = img.dimensions();
    let mut cur = img.into_raw();
    while w > max_dim || h > max_dim { let (nw, nh) = (w / 2, h / 2); cur = downsample(&cur, w, h); w = nw; h = nh; }
    let mut levels = vec![cur.clone()];
    let (mut lw, mut lh) = (w, h);
    while lw > 1 || lh > 1 { let (nw, nh) = ((lw / 2).max(1), (lh / 2).max(1)); let next = if lw > 1 && lh > 1 { downsample(levels.last().unwrap(), lw, lh) } else { levels.last().unwrap()[..(nw * nh * 4) as usize].to_vec() }; levels.push(next); lw = nw; lh = nh; }
    Ok(Arc::new(ImageRgba { w, h, levels }))
}
fn downsample(src: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (nw, nh) = (w / 2, h / 2);
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh { for x in 0..nw { for c in 0..4 {
        let i = |xx: u32, yy: u32| src[((yy * w + xx) * 4 + c) as usize] as u32;
        out[((y * nw + x) * 4 + c) as usize] = ((i(2 * x, 2 * y) + i(2 * x + 1, 2 * y) + i(2 * x, 2 * y + 1) + i(2 * x + 1, 2 * y + 1) + 2) / 4) as u8;
    } } }
    out
}

pub fn load_proxy_blocking(mhclo: &Path, texture_override: Option<&Path>) -> Result<LoadedProxy, String> {
    let proxy = Proxy::load(mhclo)?;
    let tex_path = texture_override.map(|p| p.to_path_buf()).or_else(|| proxy.texture.clone());
    let texture = match tex_path { Some(p) => Some(load_image(&p, 2048)?), None => None };
    Ok(LoadedProxy { proxy, texture })
}

fn default_tint(cat: &Category, name: &str) -> [f32; 3] {
    match cat {
        Category::Hair => [0.55, 0.36, 0.22],
        Category::Eyebrows => [0.30, 0.20, 0.14],
        Category::Eyelashes => [0.12, 0.09, 0.08],
        Category::Clothes => if name.contains("pants") || name.contains("jeans") || name.contains("shorts") { [0.48, 0.60, 0.88] } else { [0.95, 0.66, 0.72] },
    }
}

impl Session {
    pub fn load(repo: &Path) -> Result<Session, String> {
        let packages = Packages::load(repo)?;
        let body = Body::load(&packages.body.data_root, &repo.join(".hase-cache"))?;
        let params = Catalog::build(&body.morphs, &packages.body.data_root.join("targets/target.json"));
        let assets = assets::Catalog::scan(repo);
        let eval = Evaluator::new(&body);
        let skin = assets.skins.iter().find(|(n, _)| n == "young_caucasian_female").or(assets.skins.first()).map(|x| x.0.clone()).unwrap_or_default();
        let eyes = assets.eye_colors.iter().find(|(n, _)| n == "brown").or(assets.eye_colors.first()).map(|x| x.0.clone()).unwrap_or_default();
        let sp0 = packages.species.iter().find(|s| s.id.ends_with(".human")).or(packages.species.first()).cloned();
        let species = sp0.as_ref().map(|s| s.id.clone()).unwrap_or_default();
        let mut state = CharacterState { body_ref: PkgRef::default(), species_ref: PkgRef::default(), species, values: params.defaults(), assets: vec![], skin, eyes };
        if let Some(sp) = sp0.as_ref() { for (k, v) in &sp.defaults { if let Some(i) = params.canonical(k) { state.values[i] = *v; } } }
        let n = body.morphs.names.len();
        let (br, sr) = (PkgRef { id: packages.body.id.clone(), version: packages.body.version.clone(), digest: packages.body.digest.clone() }, sp0.as_ref().map(|x| PkgRef { id: x.id.clone(), version: x.version.clone(), digest: x.digest.clone() }).unwrap_or_default());
        state.body_ref = br; state.species_ref = sr;
        let mut s = Session { repo: repo.to_path_buf(), packages, body, params, assets, eval, state, history: History::new(), perf: Perf::default(), symmetric: true, proxies: HashMap::new(), textures: HashMap::new(), pending: vec![], desired: vec![0.0; n], body_nrm: vec![], body_created: false, floor_off: 0.0, skin_tex: None, dirty: true, assets_dirty: false };
        s.floor_off = -s.body.topo.vmap.iter().map(|&v| s.body.base_pos[v as usize][1]).fold(f32::MAX, f32::min);
        s.skin_tex = s.skin_path().and_then(|p| load_image(&p, 2048).ok());
        Ok(s)
    }

    fn skin_path(&self) -> Option<PathBuf> { self.assets.skins.iter().find(|(n, _)| *n == self.state.skin).map(|x| x.1.clone()) }
    fn eyes_path(&self) -> Option<PathBuf> { self.assets.eye_colors.iter().find(|(n, _)| *n == self.state.eyes).map(|x| x.1.clone()) }
    pub fn asset_entry(&self, id: &str) -> Option<&AssetEntry> { self.assets.assets.iter().find(|a| a.id == id) }
    pub fn is_selected(&self, id: &str) -> bool { self.state.assets.iter().any(|a| a.id == id) }
    pub fn species_index(&self) -> usize { self.packages.species.iter().position(|s| s.id == self.state.species).unwrap_or(0) }

    /// Standard-Ausstattung (synchron) – für Start und Screenshot-Modus.
    pub fn default_assets_blocking(&mut self, names: &[&str]) {
        let mhclo = self.repo.join("content/testcandidate/downloads/eyes/high-poly/high-poly.mhclo");
        if let Ok(lp) = load_proxy_blocking(&mhclo, self.eyes_path().as_deref()) { self.insert_proxy(EYES_ID.into(), lp); }
        for n in names { let _ = self.select_asset_blocking(n); }
    }

    pub fn select_asset_blocking(&mut self, id: &str) -> Result<(), String> {
        let e = self.asset_entry(id).cloned().ok_or_else(|| format!("Asset {id} unbekannt"))?;
        let lp = load_proxy_blocking(&e.mhclo, None)?;
        self.add_asset_selection(&e);
        self.insert_proxy(id.into(), lp);
        Ok(())
    }

    pub fn add_asset_selection(&mut self, e: &AssetEntry) {
        if self.is_selected(&e.id) { return; }
        let before = self.state.clone();
        if e.category == Category::Hair || e.category == Category::Eyebrows || e.category == Category::Eyelashes {
            let same: Vec<String> = self.state.assets.iter().filter(|a| a.id.starts_with(match e.category { Category::Hair => "hair/", Category::Eyebrows => "eyebrows/", _ => "eyelashes/" })).map(|a| a.id.clone()).collect();
            for id in same { self.remove_asset(&id); }
        }
        self.state.assets.push(AssetSel { id: e.id.clone(), tint: default_tint(&e.category, &e.name) });
        self.history.record(&before, None);
    }

    pub fn remove_asset(&mut self, id: &str) {
        if let Some(i) = self.state.assets.iter().position(|a| a.id == id) { self.state.assets.remove(i); }
        if self.proxies.remove(id).is_some() { self.pending.push(MeshUpdate::Remove(format!("proxy:{id}"))); }
    }

    pub fn toggle_asset_off(&mut self, id: &str) { let before = self.state.clone(); self.remove_asset(id); self.history.record(&before, None); }

    pub fn insert_proxy(&mut self, id: String, lp: LoadedProxy) {
        let n = lp.proxy.obj_pos_len;
        self.proxies.insert(id, ProxyRt { proxy: lp.proxy, texture: lp.texture, fit: Vec::with_capacity(n), nrm: vec![], created: false });
        self.assets_dirty = true;
    }

    pub fn set_param(&mut self, idx: usize, v: f32) {
        let before = self.state.clone();
        let v = self.clamp_to_species(idx, v);
        if (self.state.values[idx] - v).abs() < 1e-7 { return; }
        self.state.values[idx] = v;
        if self.symmetric { if let Some(m) = self.params.defs[idx].mirror { self.state.values[m] = v; } }
        self.history.record(&before, Some(idx));
        self.dirty = true;
    }

    /// Setzt nur die angegebenen Parameter auf ihren Standard (eine Undo-Stufe).
    pub fn reset_indices(&mut self, idx: &[usize]) {
        let before = self.state.clone();
        for &i in idx { self.state.values[i] = self.params.defs[i].default; }
        self.history.record(&before, None);
        self.dirty = true;
    }

    /// Wirksamer Wertebereich: Parameterbereich geschnitten mit den Grenzen der aktuellen Spezies.
    pub fn range(&self, idx: usize) -> (f32, f32) {
        let d = &self.params.defs[idx];
        let lim = self.packages.species.iter().find(|s| s.id == self.state.species).and_then(|sp| sp.limits.iter().find(|(k, _)| self.params.canonical(k) == Some(idx)).map(|(_, l)| *l));
        match lim { Some(l) => (l[0].max(d.min), l[1].min(d.max)), None => (d.min, d.max) }
    }
    pub fn clamp_to_species(&self, idx: usize, v: f32) -> f32 { let (lo, hi) = self.range(idx); v.clamp(lo.min(hi), hi.max(lo)) }
    /// Parameter, deren aktueller Wert außerhalb der Speziesgrenzen liegt (z. B. nach Spezieswechsel oder Laden).
    pub fn out_of_limits(&self) -> Vec<usize> { (0..self.params.defs.len()).filter(|&i| { let (lo, hi) = self.range(i); let v = self.state.values[i]; v < lo - 1e-4 || v > hi + 1e-4 }).collect() }

    /// Abweichungen zwischen den im Charakter gespeicherten Ressourcenversionen und den installierten.
    pub fn ref_issues(&self) -> Vec<String> { self.ref_issues_of(&self.state) }
    pub fn ref_issues_of(&self, st: &CharacterState) -> Vec<String> {
        let mut v = Vec::new();
        let (b, sr) = (&st.body_ref, &st.species_ref);
        if !b.version.is_empty() && (b.version != self.packages.body.version || (!b.digest.is_empty() && b.digest != self.packages.body.digest)) { v.push(format!("Körperbasis geändert (gespeichert v{}, installiert v{})", b.version, self.packages.body.version)); }
        match self.packages.species.iter().find(|x| x.id == st.species) {
            None => v.push(format!("Spezies {} ist nicht installiert", st.species)),
            Some(sp) => if !sr.version.is_empty() && (sr.version != sp.version || (!sr.digest.is_empty() && sr.digest != sp.digest)) { v.push(format!("Spezies „{}“ geändert (gespeichert v{}, installiert v{}{})", sp.label, sr.version, sp.version, if sr.version == sp.version { ", gleiche Version, anderer Inhalt" } else { "" })); },
        }
        v
    }
    /// Bindet den Charakter ausdrücklich an die installierten Versionen (Werte bleiben unverändert).
    pub fn rebind_refs(&mut self) {
        let before = self.state.clone();
        self.state.body_ref = PkgRef { id: self.packages.body.id.clone(), version: self.packages.body.version.clone(), digest: self.packages.body.digest.clone() };
        if let Some(sp) = self.packages.species.iter().find(|x| x.id == self.state.species) { self.state.species_ref = PkgRef { id: sp.id.clone(), version: sp.version.clone(), digest: sp.digest.clone() }; }
        self.history.record(&before, None);
    }

    pub fn clamp_all_to_species(&mut self) {
        let before = self.state.clone();
        for i in 0..self.params.defs.len() { self.state.values[i] = self.clamp_to_species(i, self.state.values[i]); }
        self.history.record(&before, None); self.dirty = true;
    }

    pub fn reset_params(&mut self) { let before = self.state.clone(); self.state.values = self.params.defaults(); self.history.record(&before, None); self.dirty = true; }

    pub fn apply_species(&mut self, idx: usize) {
        let Some(sp) = self.packages.species.get(idx).cloned() else { return };
        let before = self.state.clone();
        self.state.species = sp.id.clone();
        self.state.values = self.params.defaults();
        for (k, v) in &sp.defaults { if let Some(i) = self.params.canonical(k) { self.state.values[i] = self.clamp_to_species(i, *v); } }
        self.state.species_ref = PkgRef { id: sp.id.clone(), version: sp.version.clone(), digest: sp.digest.clone() };
        self.history.record(&before, None);
        self.dirty = true;
    }

    pub fn apply_map(&mut self, m: &std::collections::BTreeMap<String, f32>) {
        let before = self.state.clone();
        for (k, v) in m { if let Some(i) = self.params.canonical(k) { let v = self.clamp_to_species(i, *v); self.state.values[i] = v; if self.symmetric { if let Some(mi) = self.params.defs[i].mirror { self.state.values[mi] = v; } } } }
        self.history.record(&before, None);
        self.dirty = true;
    }

    pub fn set_tint(&mut self, id: &str, tint: [f32; 3]) {
        if let Some(a) = self.state.assets.iter_mut().find(|a| a.id == id) { a.tint = tint; }
        self.pending.push(MeshUpdate::Tint { id: format!("proxy:{id}"), tint });
    }

    pub fn set_skin(&mut self, name: &str) {
        let before = self.state.clone();
        self.state.skin = name.into();
        self.skin_tex = self.skin_path().and_then(|p| load_image(&p, 2048).ok());
        self.pending.push(MeshUpdate::Remove("body".into()));
        self.body_created = false; self.dirty = true;
        self.history.record(&before, None);
    }

    /// Ersetzt den gesamten Zustand (Datei laden).
    pub fn apply_state(&mut self, st: CharacterState) { let before = self.state.clone(); self.history.record(&before, None); self.restore(st); }
    pub fn undo(&mut self) { if let Some(s) = self.history.undo(&self.state) { self.restore(s); } }
    pub fn redo(&mut self) { if let Some(s) = self.history.redo(&self.state) { self.restore(s); } }
    fn restore(&mut self, s: CharacterState) {
        let removed: Vec<String> = self.state.assets.iter().filter(|a| !s.assets.iter().any(|b| b.id == a.id)).map(|a| a.id.clone()).collect();
        for id in removed { if self.proxies.remove(&id).is_some() { self.pending.push(MeshUpdate::Remove(format!("proxy:{id}"))); } }
        let skin_changed = s.skin != self.state.skin;
        self.state = s;
        if skin_changed { self.skin_tex = self.skin_path().and_then(|p| load_image(&p, 2048).ok()); self.pending.push(MeshUpdate::Remove("body".into())); self.body_created = false; }
        self.dirty = true;
    }
    /// Assets, die ausgewählt, aber (noch) nicht geladen sind.
    pub fn missing_assets(&self) -> Vec<AssetEntry> { self.state.assets.iter().filter(|a| !self.proxies.contains_key(&a.id)).filter_map(|a| self.asset_entry(&a.id).cloned()).collect() }
    pub fn loaded(&self, id: &str) -> bool { self.proxies.contains_key(id) }

    fn interleave(pos: &[[f32; 3]], nrm: &[[f32; 3]], vmap: &[u32], floor: f32) -> Vec<f32> {
        let mut d = Vec::with_capacity(vmap.len() * 6);
        for &v in vmap { let (p, n) = (pos[v as usize], nrm[v as usize]); d.extend_from_slice(&[p[0], p[1] + floor, p[2], n[0], n[1], n[2]]); }
        d
    }

    /// Berechnet bei Bedarf Morphs/Fits neu und liefert die anstehenden Render-Updates.
    pub fn refresh(&mut self) -> Vec<MeshUpdate> {
        if self.dirty || self.assets_dirty || !self.body_created {
            let t0 = Instant::now();
            let mut morph_changed = !self.body_created;
            if self.dirty {
                self.params.target_weights(&self.state.values, self.body.morphs.names.len(), &mut self.desired);
                if let Some(st) = self.eval.apply(&self.body, &self.desired) { morph_changed = true; self.perf = Perf { morph_ms: st.ms, changed: st.changed_targets, active: st.active_targets, full: st.full, ..self.perf }; }
            }
            if morph_changed || self.assets_dirty {
                if morph_changed {
                    self.floor_off = -self.body.topo.vmap.iter().map(|&v| self.eval.pos[v as usize][1]).fold(f32::MAX, f32::min);
                    normals(&self.eval.pos, &self.body.topo.base_tris, &mut self.body_nrm);
                    let data = Self::interleave(&self.eval.pos, &self.body_nrm, &self.body.topo.vmap, self.floor_off);
                    if !self.body_created {
                        self.pending.push(MeshUpdate::Create(MeshDesc { id: "body".into(), indices: self.body.topo.indices.clone(), uv: self.body.topo.uv.clone(), data, texture: self.skin_tex.clone(), tint: [1.0, 0.97, 0.95], mask: false, cull_back: true }));
                        self.body_created = true;
                    } else { self.pending.push(MeshUpdate::Positions { id: "body".into(), data }); }
                }
                let t1 = Instant::now();
                let ids: Vec<String> = self.proxies.keys().cloned().collect();
                for id in ids {
                    let tint = self.state.assets.iter().find(|a| a.id == id).map(|a| a.tint).unwrap_or(if id == EYES_ID { [1.0; 3] } else { [0.9; 3] });
                    let rt = self.proxies.get_mut(&id).unwrap();
                    rt.proxy.fit(&self.eval.pos, &mut rt.fit);
                    let tris: Vec<[u32; 3]> = rt.proxy.topo.base_tris.clone();
                    normals(&rt.fit, &tris, &mut rt.nrm);
                    let data = Self::interleave(&rt.fit, &rt.nrm, &rt.proxy.topo.vmap, self.floor_off);
                    if !rt.created {
                        let cull_back = false;
                        self.pending.push(MeshUpdate::Create(MeshDesc { id: format!("proxy:{id}"), indices: rt.proxy.topo.indices.clone(), uv: rt.proxy.topo.uv.clone(), data, texture: rt.texture.clone(), tint, mask: rt.proxy.alpha_mask, cull_back }));
                        rt.created = true;
                    } else { self.pending.push(MeshUpdate::Positions { id: format!("proxy:{id}"), data }); }
                }
                self.perf.fit_ms = t1.elapsed().as_secs_f64() * 1e3;
            }
            self.perf.total_ms = t0.elapsed().as_secs_f64() * 1e3;
            self.dirty = false; self.assets_dirty = false;
        }
        std::mem::take(&mut self.pending)
    }

    /// Körperhöhe (Scheitel über Boden) in m – für höhenrelative Kamerafokus-Punkte.
    pub fn body_height(&self) -> f32 { self.floor_off + self.body.topo.vmap.iter().map(|&v| self.eval.pos[v as usize][1]).fold(f32::MIN, f32::max) }

    pub fn counts(&self) -> (usize, usize) {
        let v = self.body.topo.vmap.len() + self.proxies.values().map(|p| p.proxy.topo.vmap.len()).sum::<usize>();
        let t = self.body.topo.indices.len() / 3 + self.proxies.values().map(|p| p.proxy.topo.indices.len() / 3).sum::<usize>();
        (v, t)
    }
    pub fn active_asset_count(&self) -> usize { self.proxies.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session { Session::load(&hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap()).unwrap() }

    #[test]
    fn species_limits_are_enforced() {
        let mut s = session();
        let elf = s.packages.species.iter().position(|x| x.id.ends_with(".elf")).unwrap();
        s.apply_species(elf);
        let i = s.params.canonical("mh:head-scale-vert").unwrap();
        s.set_param(i, -1.0);
        assert!((s.state.values[i] - (-0.2)).abs() < 1e-5, "Wert {}", s.state.values[i]);
        s.set_param(i, 1.0);
        assert!((s.state.values[i] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn version_binding_detects_changes_without_changing_values() {
        let mut s = session();
        assert!(s.ref_issues().is_empty());
        let before = s.state.values.clone();
        let id = s.state.species.clone();
        s.packages.species.iter_mut().find(|x| x.id == id).unwrap().version = "2.0.0".into();
        let issues = s.ref_issues();
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert_eq!(s.state.values, before);
        s.rebind_refs();
        assert!(s.ref_issues().is_empty());
        // Digest-Änderung bei gleicher Version
        s.packages.species.iter_mut().find(|x| x.id == id).unwrap().digest = "abc".into();
        assert!(s.ref_issues()[0].contains("anderer Inhalt"));
    }

    #[test]
    fn old_param_ids_resolve() {
        let s = session();
        assert!(s.params.canonical("mh:head-age").is_some());
        assert!(s.params.canonical("mh:jaw-drop").is_some());
        assert!(s.params.canonical("mh:gibtsnicht").is_none());
    }
}
