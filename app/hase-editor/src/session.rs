//! Session: Projektzustand + Evaluator + Proxy-Laufzeit; erzeugt Render-Updates. Kein egui-/wgpu-Gerätebezug (nur Datenübergabe).
use crate::render::{ImageRgba, MatParams, MeshDesc, MeshUpdate};
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

pub enum LoadPlan { Ready(CharacterState, Vec<String>), NeedsBody(hase_core::packages::BodyPackage), MissingBody(String), Invalid }

/// Geöffnetes statisches VRM-/glTF-Modell (Ansehen, Posen, Ausdrücke).
pub struct VrmRuntime { pub model: hase_core::vrm_model::VrmModel, rig: hase_core::pose::PoseRig, tris: Vec<Vec<[u32; 3]>>, floor: f32, pub expr: std::collections::BTreeMap<String, f32>, pub raw: std::collections::BTreeMap<(usize, usize), f32>, shown: Vec<bool>, dirty: bool, pub path: PathBuf, pub hidden: std::collections::BTreeSet<usize>, pub skin_parts: std::collections::BTreeSet<usize>, dress: Option<VrmDress>, pub dress_info: String }

pub struct VrmDress { skin_pos: Vec<[f32; 3]>, skin_nrm: Vec<[f32; 3]>, skin_joints: Vec<[u16; 4]>, skin_weights: Vec<[f32; 4]>, skin_ids: Vec<u32>, grid: hase_core::fit::BodyGrid, base: Vec<[f32; 3]> }

#[derive(Default)]
pub struct Playback { pub clip: Option<hase_core::pose::Clip>, pub t: f32, pub playing: bool, pub looping: bool, pub speed: f32 }

pub struct LoadedProxy { pub proxy: Proxy, pub texture: Option<Arc<ImageRgba>> }

struct ProxyRt { proxy: Proxy, texture: Option<Arc<ImageRgba>>, fit: Vec<[f32; 3]>, nrm: Vec<[f32; 3]>, created: bool, yrange: (f32, f32) }

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
    style_dirty: bool,
    skin_dirty: bool,
    pub play: Playback,
    rig: Option<hase_core::export::RigData>,
    pose_rig: Option<hase_core::pose::PoseRig>,
    proxy_skin: HashMap<String, Vec<([u16; 4], [f32; 4])>>,
    pose_active: bool,
    resend_body: bool,
    push_grid: Option<hase_core::fit::BodyGrid>,
    body_uniq: Vec<u32>,
    pub vrm: Option<VrmRuntime>,
    masks: Option<crate::skin::Masks>,
    pub generic: bool,
    eyes_loaded: String,
    pub hide_body: bool,
    pub reference_body: hase_core::packages::BodyPackage,
    pub markers: Vec<String>,
}

fn dist2(a: [f32; 3], b: [f32; 3]) -> f32 { (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) }

pub const EYES_ID: &str = "eyes/high-poly";

pub fn load_image(path: &Path, max_dim: u32) -> Result<Arc<ImageRgba>, String> {
    let img = image::open(path).map_err(|e| format!("{}: {e}", path.display()))?.to_rgba8();
    let (w, h) = img.dimensions();
    Ok(image_from_rgba(w, h, img.into_raw(), max_dim))
}

pub fn image_from_rgba(mut w: u32, mut h: u32, mut cur: Vec<u8>, max_dim: u32) -> Arc<ImageRgba> {
    while w > max_dim || h > max_dim { let (nw, nh) = (w / 2, h / 2); cur = downsample(&cur, w, h); w = nw; h = nh; }
    let mut levels = vec![cur.clone()];
    let (mut lw, mut lh) = (w, h);
    while lw > 1 || lh > 1 { let (nw, nh) = ((lw / 2).max(1), (lh / 2).max(1)); let next = if lw > 1 && lh > 1 { downsample(levels.last().unwrap(), lw, lh) } else { levels.last().unwrap()[..(nw * nh * 4) as usize].to_vec() }; levels.push(next); lw = nw; lh = nh; }
    Arc::new(ImageRgba { w, h, levels })
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

/// Haarform: Länge (Tipps verschieben sich relativ zur Kopfhaut) und Volumen je Region entlang der Haarnormalen.
fn shape_hair(fit: &mut [[f32; 3]], nrm: &[[f32; 3]], vmap_used: &[u32], h: &hase_core::state::HairStyle) {
    let (mut ymin, mut ymax, mut zmin, mut zmax) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for &v in vmap_used { let p = fit[v as usize]; ymin = ymin.min(p[1]); ymax = ymax.max(p[1]); zmin = zmin.min(p[2]); zmax = zmax.max(p[2]); }
    let span = (ymax - ymin).max(1e-3); let zc = (zmin + zmax) * 0.5;
    let sstep = |a: f32, b: f32, x: f32| { let t = ((x - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
    let mut seen = std::collections::HashSet::new();
    for &v in vmap_used {
        if !seen.insert(v) { continue; }
        let i = v as usize; let p = fit[i]; let n = nrm[i];
        let depth = (ymax - p[1]) / span;                 // 0 Scheitel … 1 Spitze
        let front = sstep(zc, zc + 0.07, p[2]); let back = sstep(zc, zc - 0.07, p[2]);
        let side = sstep(0.035, 0.09, p[0].abs()) * (1.0 - front) * (1.0 - back);
        let top = sstep(0.55, 0.0, depth);
        let vol = h.vol_top * top + h.vol_front * front * sstep(0.7, 0.1, depth) + h.vol_side * side + h.vol_back * back;
        let mut q = [p[0] + n[0] * vol * 0.02, p[1] + n[1] * vol * 0.02, p[2] + n[2] * vol * 0.02];
        q[1] -= (ymax - q[1]) * h.length * sstep(0.12, 0.55, depth) * 0.9;
        fit[i] = q;
    }
}

fn eye_mat(st: &hase_core::state::EyeStyle) -> MatParams {
    let r = if st.linked { &st.left } else { &st.right };
    let l = &st.left;
    MatParams { kind: 1, gloss: 0.0, p: [
        [l.inner[0], l.inner[1], l.inner[2], l.pupil], [l.outer[0], l.outer[1], l.outer[2], l.custom as u32 as f32],
        [r.inner[0], r.inner[1], r.inner[2], r.pupil], [r.outer[0], r.outer[1], r.outer[2], r.custom as u32 as f32],
        [0.118, 0.040, st.reflection * 1.6, 0.0], [st.limbal, 0.0, 0.0, 0.0] ] }
}
fn hair_mat(h: &hase_core::state::HairStyle, yr: (f32, f32)) -> MatParams {
    MatParams { kind: 2, gloss: h.gloss, p: [[h.root[0], h.root[1], h.root[2], yr.1], [h.tip[0], h.tip[1], h.tip[2], yr.0], [h.highlight[0], h.highlight[1], h.highlight[2], h.highlight_amt], [0.0; 4], [0.0; 4], [0.0; 4]] }
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
        let mut state = CharacterState { style: Default::default(), body_ref: PkgRef::default(), species_ref: PkgRef::default(), species, values: params.defaults(), assets: vec![], skin, eyes };
        if let Some(sp) = sp0.as_ref() { for (k, v) in &sp.defaults { if let Some(i) = params.canonical(k) { state.values[i] = *v; } } }
        let n = body.morphs.names.len();
        let reference = packages.body.clone();
        let (br, sr) = (PkgRef { id: packages.body.id.clone(), version: packages.body.version.clone(), digest: packages.body.digest.clone() }, sp0.as_ref().map(|x| PkgRef { id: x.id.clone(), version: x.version.clone(), digest: x.digest.clone() }).unwrap_or_default());
        state.body_ref = br; state.species_ref = sr;
        let mut s = Session { repo: repo.to_path_buf(), packages, body, params, assets, eval, state, history: History::new(), perf: Perf::default(), symmetric: true, proxies: HashMap::new(), textures: HashMap::new(), pending: vec![], desired: vec![0.0; n], body_nrm: vec![], body_created: false, floor_off: 0.0, skin_tex: None, dirty: true, assets_dirty: false, style_dirty: true, skin_dirty: true, play: Playback::default(), rig: None, pose_rig: None, proxy_skin: HashMap::new(), pose_active: false, resend_body: false, push_grid: None, body_uniq: vec![], vrm: None, masks: None, generic: false, eyes_loaded: String::new(), hide_body: false, reference_body: reference, markers: vec![] };
        s.floor_off = -s.body.topo.vmap.iter().map(|&v| s.body.base_pos[v as usize][1]).fold(f32::MAX, f32::min);
        s.skin_tex = s.skin_path().and_then(|p| load_image(&p, 2048).ok());
        Ok(s)
    }

    fn skin_path(&self) -> Option<PathBuf> { self.assets.skins.iter().find(|(n, _)| *n == self.state.skin).map(|x| x.1.clone()) }
    fn eyes_path(&self) -> Option<PathBuf> { self.assets.eye_colors.iter().find(|(n, _)| *n == self.state.eyes).map(|x| x.1.clone()) }
    pub fn asset_entry(&self, id: &str) -> Option<&AssetEntry> { self.assets.assets.iter().find(|a| a.id == id) }
    pub fn is_selected(&self, id: &str) -> bool { self.state.assets.iter().any(|a| a.id == id) }
    pub fn species_index(&self) -> usize { self.packages.species.iter().position(|s| s.id == self.state.species).unwrap_or(0) }
    pub fn species_opt(&self) -> Option<usize> { self.packages.species.iter().position(|s| s.id == self.state.species) }
    /// Spezies, die zur aktiven Körperbasis passen.
    pub fn species_indices(&self) -> Vec<usize> { self.packages.species.iter().enumerate().filter(|(_, s)| s.body_id.is_empty() || s.body_id == self.packages.body.id).map(|(i, _)| i).collect() }

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
        self.proxy_skin.remove(id);
    }

    pub fn toggle_asset_off(&mut self, id: &str) { let before = self.state.clone(); self.remove_asset(id); self.history.record(&before, None); }

    pub fn insert_proxy(&mut self, id: String, lp: LoadedProxy) {
        if self.generic { return; }
        let n = lp.proxy.obj_pos_len;
        self.proxies.insert(id, ProxyRt { proxy: lp.proxy, texture: lp.texture, fit: Vec::with_capacity(n), nrm: vec![], created: false, yrange: (1.0, 0.0) });
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
        if !b.version.is_empty() && (b.version != self.packages.body.version || (!b.digest.is_empty() && b.digest != self.packages.body.digest)) { v.push(if b.version == self.packages.body.version { format!("Inhalt der Körperbasis v{} hat sich geändert (Geometrie, Morphs, Rig oder Konfiguration): dieselben Parameterwerte können eine andere Körperform ergeben – Original-Ressource oder geprüfte Migration nötig", b.version) } else { format!("Körperbasis geändert (gespeichert v{}, installiert v{}): Parameterwerte können eine andere Form ergeben", b.version, self.packages.body.version) }); }
        if st.species.is_empty() { return v; }
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

    /// Kopie des Zustands mit angewendeten Parameterwerten (für Preset-Vorschauen), unter Beachtung der Speziesgrenzen.
    pub fn state_with(&self, m: &std::collections::BTreeMap<String, f32>) -> CharacterState {
        let mut st = self.state.clone();
        for (k, v) in m { if let Some(i) = self.params.canonical(k) { st.values[i] = self.clamp_to_species(i, *v); if let Some(mi) = self.params.defs[i].mirror { st.values[mi] = st.values[i]; } } }
        st
    }

    pub fn clamp_all_to_species(&mut self) {
        let before = self.state.clone();
        for i in 0..self.params.defs.len() { self.state.values[i] = self.clamp_to_species(i, self.state.values[i]); }
        self.history.record(&before, None); self.dirty = true;
    }

    /// Neuer Charakter: Standardwerte der aktuellen Spezies, keine Assets, Standard-Stil.
    pub fn new_character(&mut self) {
        let before = self.state.clone();
        let ids: Vec<String> = self.state.assets.iter().filter(|a| a.id.starts_with("clothes/")).map(|a| a.id.clone()).collect();
        for id in ids { self.remove_asset(&id); }
        self.state.style = Default::default();
        self.state.values = self.params.defaults();
        if let Some(i) = self.species_opt() { let sp = self.packages.species[i].clone(); for (k, v) in &sp.defaults { if let Some(j) = self.params.canonical(k) { self.state.values[j] = self.clamp_to_species(j, *v); } } }
        self.history.record(&before, None);
        self.style_dirty = true; self.dirty = true;
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

    /// Ändert Stil-/Materialwerte (Augen, Haare, Haut) mit Undo (Slider-Ziehen wird über `key` zusammengefasst).
    pub fn edit_style(&mut self, key: usize, f: impl FnOnce(&mut hase_core::state::Style)) {
        let before = self.state.clone();
        f(&mut self.state.style);
        if self.state.style == before.style { return; }
        if before.style.skin != self.state.style.skin { self.skin_dirty = true; }
        let (a, b) = (&before.style.hair, &self.state.style.hair);
        if (a.length, a.vol_top, a.vol_front, a.vol_side, a.vol_back) != (b.length, b.vol_top, b.vol_front, b.vol_side, b.vol_back) { self.assets_dirty = true; }
        self.history.record(&before, Some(100_000 + key));
        self.style_dirty = true;
    }

    pub fn set_tint(&mut self, id: &str, tint: [f32; 3]) {
        if let Some(a) = self.state.assets.iter_mut().find(|a| a.id == id) { a.tint = tint; }
        self.pending.push(MeshUpdate::Tint { id: format!("proxy:{id}"), tint });
    }

    pub fn set_skin(&mut self, name: &str) {
        let before = self.state.clone();
        self.state.skin = name.into();
        self.skin_tex = self.skin_path().and_then(|p| load_image(&p, 2048).ok());
        self.skin_dirty = true;
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
        self.style_dirty = true; self.assets_dirty = true; self.skin_dirty = true;
        self.state = s;
        for a in &self.state.assets { if self.proxies.contains_key(&a.id) { self.pending.push(MeshUpdate::Tint { id: format!("proxy:{}", a.id), tint: a.tint }); } }
        if skin_changed { self.skin_dirty = true; self.skin_tex = self.skin_path().and_then(|p| load_image(&p, 2048).ok()); self.pending.push(MeshUpdate::Remove("body".into())); self.body_created = false; }
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
        if self.vrm.is_some() { return self.refresh_vrm(); }
        let need_body = !self.body_created && !self.hide_body;
        if self.hide_body && self.body_created { self.pending.push(MeshUpdate::Remove("body".into())); self.body_created = false; }
        if self.dirty || self.assets_dirty || need_body {
            let t0 = Instant::now();
            let mut morph_changed = need_body || std::mem::take(&mut self.resend_body);
            if self.dirty {
                self.params.target_weights(&self.state.values, self.body.morphs.names.len(), &mut self.desired);
                if let Some(st) = self.eval.apply(&self.body, &self.desired) { morph_changed = true; self.perf = Perf { morph_ms: st.ms, changed: st.changed_targets, active: st.active_targets, full: st.full, ..self.perf }; }
            }
            if morph_changed || self.assets_dirty {
                if morph_changed { self.pose_rig = None; self.push_grid = None; }
                if morph_changed {
                    self.floor_off = -self.body.topo.vmap.iter().map(|&v| self.eval.pos[v as usize][1]).fold(f32::MAX, f32::min);
                    normals(&self.eval.pos, &self.body.topo.base_tris, &mut self.body_nrm);
                    let data = Self::interleave(&self.eval.pos, &self.body_nrm, &self.body.topo.vmap, self.floor_off);
                    if self.hide_body { } else if !self.body_created {
                        self.pending.push(MeshUpdate::Create(MeshDesc { id: "body".into(), indices: self.body.topo.indices.clone(), uv: self.body.topo.uv.clone(), data, texture: self.skin_tex.clone(), tint: [1.0, 0.97, 0.95], mask: false, cull_back: true, mat: MatParams { kind: 3, gloss: self.state.style.skin.gloss, ..Default::default() } }));
                        self.body_created = true; self.skin_dirty = true;
                    } else { self.pending.push(MeshUpdate::Positions { id: "body".into(), data }); }
                }
                let t1 = Instant::now();
                let ids: Vec<String> = self.proxies.keys().cloned().collect();
                for id in ids {
                    let tint = self.state.assets.iter().find(|a| a.id == id).map(|a| a.tint).unwrap_or(if id == EYES_ID { [1.0; 3] } else { [0.9; 3] });
                    let rt = self.proxies.get_mut(&id).unwrap();
                    rt.proxy.fit(&self.eval.pos, &mut rt.fit);
                    if id.starts_with("clothes/") {
                        if self.push_grid.is_none() { if self.body_uniq.is_empty() { let mut uniq: Vec<u32> = self.body.topo.vmap.clone(); uniq.sort_unstable(); uniq.dedup(); self.body_uniq = uniq; } self.push_grid = Some(hase_core::fit::BodyGrid::build(&self.eval.pos, &self.body_uniq, 0.03)); }
                        if self.body_nrm.len() == self.eval.pos.len() { { let uniq = &self.body_uniq; hase_core::fit::push_out(&mut rt.fit, &rt.proxy.adj, &rt.proxy.topo.base_tris, self.push_grid.as_ref().unwrap(), &self.eval.pos, &self.body_nrm, uniq, 0.003, true); } }
                    }
                    if id.starts_with("hair/") && !self.state.style.hair.shape_zero() {
                        let tris2: Vec<[u32; 3]> = rt.proxy.topo.base_tris.clone();
                        normals(&rt.fit, &tris2, &mut rt.nrm);
                        shape_hair(&mut rt.fit, &rt.nrm, &rt.proxy.topo.vmap, &self.state.style.hair);
                    }
                    rt.yrange = rt.fit.iter().fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p[1]), a.1.max(p[1]))); rt.yrange = (rt.yrange.0 + self.floor_off, rt.yrange.1 + self.floor_off);
                    let tris: Vec<[u32; 3]> = rt.proxy.topo.base_tris.clone();
                    normals(&rt.fit, &tris, &mut rt.nrm);
                    let data = Self::interleave(&rt.fit, &rt.nrm, &rt.proxy.topo.vmap, self.floor_off);
                    if !rt.created {
                        let cull_back = false;
                        let mat = if id == EYES_ID { eye_mat(&self.state.style.eyes) } else if id.starts_with("hair/") { hair_mat(&self.state.style.hair, rt.yrange) } else { MatParams::default() };
                        self.pending.push(MeshUpdate::Create(MeshDesc { id: format!("proxy:{id}"), indices: rt.proxy.topo.indices.clone(), uv: rt.proxy.topo.uv.clone(), data, texture: rt.texture.clone(), tint, mask: rt.proxy.alpha_mask, cull_back, mat }));
                        rt.created = true;
                    } else { self.pending.push(MeshUpdate::Positions { id: format!("proxy:{id}"), data }); }
                }
                self.perf.fit_ms = t1.elapsed().as_secs_f64() * 1e3;
            }
            self.perf.total_ms = t0.elapsed().as_secs_f64() * 1e3;
            self.dirty = false; self.assets_dirty = false;
        }
        if self.skin_dirty && self.body_created {
            self.skin_dirty = false;
            if let Some(base) = self.skin_tex.clone() {
                if self.state.style.skin.has_details() {
                    if self.masks.is_none() { self.masks = Some(if self.generic { crate::skin::Masks::empty() } else { crate::skin::build_masks(&self.body) }); }
                    let img = crate::skin::compose(&base, self.masks.as_ref().unwrap(), &self.state.style.skin);
                    self.pending.push(MeshUpdate::Texture { id: "body".into(), img });
                } else { self.pending.push(MeshUpdate::Texture { id: "body".into(), img: base }); }
            }
        }
        if self.style_dirty {
            self.style_dirty = false;
            self.pending.push(MeshUpdate::Material { id: "body".into(), tint: [1.0, 0.97, 0.95], mat: MatParams { kind: 3, gloss: self.state.style.skin.gloss, ..Default::default() } });
            for (id, rt) in &self.proxies {
                if !rt.created { continue; }
                if *id == EYES_ID { self.pending.push(MeshUpdate::Material { id: format!("proxy:{id}"), tint: [1.0; 3], mat: eye_mat(&self.state.style.eyes) }); }
                else if id.starts_with("hair/") { self.pending.push(MeshUpdate::Material { id: format!("proxy:{id}"), tint: [1.0; 3], mat: hair_mat(&self.state.style.hair, rt.yrange) }); }
            }
        }
        if self.play.clip.is_some() { self.emit_pose(); }
        std::mem::take(&mut self.pending)
    }

    /// Körperhöhe (Scheitel über Boden) in m – für höhenrelative Kamerafokus-Punkte.
    pub fn body_height(&self) -> f32 { if let Some(v) = &self.vrm { return v.model.height + v.floor; } self.floor_off + self.body.topo.vmap.iter().map(|&v| self.eval.pos[v as usize][1]).fold(f32::MIN, f32::max) }

    /// Ersetzt den Zustand ohne Undo-Eintrag (Thumbnail-Worker).
    pub fn set_state_silent(&mut self, st: CharacterState) { self.restore(st); }

    /// Lädt ausgewählte Assets und das Augenmaterial synchron nach (nur Worker-/Screenshot-Kontexte).
    pub fn ensure_assets_blocking(&mut self) {
        if self.generic { return; }
        let ids: Vec<String> = self.state.assets.iter().map(|a| a.id.clone()).collect();
        for id in ids { if !self.proxies.contains_key(&id) { let _ = self.select_asset_blocking_noselect(&id); } }
        if self.eyes_loaded != self.state.eyes || !self.proxies.contains_key(EYES_ID) {
            let mhclo = self.repo.join("content/testcandidate/downloads/eyes/high-poly/high-poly.mhclo");
            if let Ok(lp) = load_proxy_blocking(&mhclo, self.eyes_path().as_deref()) { self.proxies.remove(EYES_ID); self.pending.push(MeshUpdate::Remove(format!("proxy:{EYES_ID}"))); self.insert_proxy(EYES_ID.into(), lp); self.eyes_loaded = self.state.eyes.clone(); }
        }
    }
    fn select_asset_blocking_noselect(&mut self, id: &str) -> Result<(), String> {
        let e = self.asset_entry(id).cloned().ok_or_else(|| format!("Asset {id} unbekannt"))?;
        let lp = load_proxy_blocking(&e.mhclo, None)?;
        self.insert_proxy(id.into(), lp);
        Ok(())
    }

    /// Wechselt die aktive Körperbasis (Assets/Proxys der alten Topologie werden entfernt, Parameterkatalog neu aufgebaut).
    pub fn switch_body(&mut self, body: Body, params: Catalog, pkg: hase_core::packages::BodyPackage, tex: Option<Arc<ImageRgba>>, generic: bool) {
        let ids: Vec<String> = self.proxies.keys().cloned().collect();
        for id in ids { self.proxies.remove(&id); self.pending.push(MeshUpdate::Remove(format!("proxy:{id}"))); }
        self.pending.push(MeshUpdate::Remove("body".into()));
        self.state.assets.clear();
        self.eval = Evaluator::new(&body);
        self.desired = vec![0.0; body.morphs.names.len()];
        self.state.values = params.defaults();
        self.state.body_ref = PkgRef { id: pkg.id.clone(), version: pkg.version.clone(), digest: pkg.digest.clone() };
        self.body = body; self.params = params; self.packages.body = pkg;
        self.skin_tex = if generic { tex } else { self.skin_path().and_then(|p| load_image(&p, 2048).ok()) };
        self.masks = None; self.skin_dirty = true; self.body_uniq.clear(); self.push_grid = None;
        self.generic = generic; self.body_created = false; self.hide_body = false; self.dirty = true; self.assets_dirty = true;
        self.history = History::new();
        let compat = self.species_indices();
        match compat.iter().find(|&&i| self.packages.species[i].id.ends_with(".human")).or(compat.first()).copied() {
            Some(i) => { let sp = self.packages.species[i].clone(); self.state.species = sp.id.clone(); for (k, v) in &sp.defaults { if let Some(j) = self.params.canonical(k) { self.state.values[j] = *v; } } self.state.species_ref = PkgRef { id: sp.id, version: sp.version, digest: sp.digest }; }
            None => { self.state.species = String::new(); self.state.species_ref = PkgRef::default(); }
        }
        self.floor_off = -self.body.topo.vmap.iter().map(|&v| self.body.base_pos[v as usize][1]).fold(f32::MAX, f32::min);
    }

    /// Aktiviert eine Körperbasis aus einem Package (importierte, aufbereitete glTF-Basis oder die MakeHuman-Referenz).
    pub fn activate_body(&mut self, pkg: &hase_core::packages::BodyPackage) -> Result<(), String> {
        if let Some(cfgv) = &pkg.config {
            let cfg: hase_core::import::ImportConfig = serde_json::from_value(cfgv.clone()).map_err(|e| e.to_string())?;
            self.preview_import(&pkg.data_root.join(&cfg.source), &cfg, pkg.clone())
        } else {
            let body = Body::load(&pkg.data_root, &self.repo.join(".hase-cache"))?;
            let params = Catalog::build(&body.morphs, &pkg.data_root.join("targets/target.json"));
            self.switch_body(body, params, pkg.clone(), None, false);
            self.clear_markers();
            Ok(())
        }
    }

    /// Plant das Laden einer Charakterdatei: passende Körperbasis aktiv? Sonst ist ein (bestätigter) Wechsel nötig.
    pub fn plan_load(&self, v: &serde_json::Value) -> LoadPlan {
        let bid = v["body"]["id"].as_str().unwrap_or("");
        if !bid.is_empty() && bid != self.packages.body.id {
            return match self.packages.bodies.iter().find(|b| b.id == bid && b.status != "draft") { Some(b) => LoadPlan::NeedsBody(b.clone()), None => LoadPlan::MissingBody(bid.to_string()) };
        }
        match CharacterState::from_json(v, &self.params) {
            Some(st) => { let mut issues = self.ref_issues_of(&st); let unknown = CharacterState::unknown_params(v, &self.params); if !unknown.is_empty() { issues.push(format!("{} gespeicherte Parameter sind im installierten Katalog unbekannt und gehen verloren: {}", unknown.len(), unknown.iter().take(4).cloned().collect::<Vec<_>>().join(", "))); } LoadPlan::Ready(st, issues) }
            None => LoadPlan::Invalid,
        }
    }

    /// Lädt ein importiertes Modell gemäß Konfiguration als aktive Körperbasis (auch für die Vorschau während der Aufbereitung).
    pub fn preview_import(&mut self, path: &Path, cfg: &hase_core::import::ImportConfig, pkg: hase_core::packages::BodyPackage) -> Result<(), String> {
        let (body, img) = hase_core::import::build_body(path, cfg)?;
        let mirror = hase_core::import::detect_mirror(&body.morphs.names);
        let mir: Vec<(usize, usize)> = if cfg.mirror.is_empty() { mirror } else { cfg.mirror.iter().filter_map(|[a, b]| Some((body.morphs.index_of(a)?, body.morphs.index_of(b)?))).collect() };
        let params = Catalog::from_morphs(&body.morphs.names, &cfg.regions, &mir);
        let tex = img.map(|i| image_from_rgba(i.w, i.h, i.rgba, 2048));
        self.switch_body(body, params, pkg, tex, true);
        Ok(())
    }

    /// Zielt per Strahl auf den Körper: Trefferpunkt (Modellkoordinaten ohne Bodenversatz) und nächster Vertex.
    pub fn pick(&self, o: [f32; 3], d: [f32; 3]) -> Option<([f32; 3], usize)> {
        let (t, ti) = hase_core::import::raycast(&self.eval.pos, self.floor_off, &self.body.topo.base_tris, o, d)?;
        let hit = [o[0] + d[0] * t, o[1] + d[1] * t - self.floor_off, o[2] + d[2] * t];
        let tri = self.body.topo.base_tris[ti];
        let v = tri.iter().copied().min_by(|&a, &b| { let da = dist2(self.eval.pos[a as usize], hit); let db = dist2(self.eval.pos[b as usize], hit); da.partial_cmp(&db).unwrap() })? as usize;
        Some((hit, v))
    }
    /// Morph-Targets mit der größten Wirkung auf einen Vertex (Index, Betrag).
    pub fn morphs_at_vertex(&self, v: usize, n: usize) -> Vec<(usize, f32)> {
        let m = &self.body.morphs;
        let mut out: Vec<(usize, f32)> = Vec::new();
        for t in 0..m.names.len() {
            if m.names[t].starts_with("macrodetails") { continue; }
            let (a, b) = (m.t_off[t] as usize, m.t_off[t + 1] as usize);
            if let Ok(k) = m.t_idx[a..b].binary_search(&(v as u32)) { let d = m.t_delta[a + k]; out.push((t, (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())); }
            else if let Some(k) = m.t_idx[a..b].iter().position(|&x| x as usize == v) { let d = m.t_delta[a + k]; out.push((t, (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())); }
        }
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        out.truncate(n);
        out
    }

    /// Kleine Oktaeder-Marker (Landmarken, Bones, Auswahl) in Weltkoordinaten (Modell + Bodenversatz).
    pub fn set_marker(&mut self, id: &str, model_pos: [f32; 3], size: f32, tint: [f32; 3]) {
        let c = [model_pos[0], model_pos[1] + self.floor_off, model_pos[2]];
        let dirs: [[f32; 3]; 6] = [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]];
        let mut data = Vec::new();
        for d in dirs { data.extend_from_slice(&[c[0] + d[0] * size, c[1] + d[1] * size, c[2] + d[2] * size, d[0], d[1], d[2]]); }
        let indices = vec![0, 2, 4, 4, 2, 1, 1, 2, 5, 5, 2, 0, 0, 4, 3, 4, 1, 3, 1, 5, 3, 5, 0, 3];
        let mid = format!("marker:{id}");
        self.pending.push(MeshUpdate::Remove(mid.clone()));
        self.pending.push(MeshUpdate::Create(MeshDesc { id: mid, indices, uv: vec![[0.0, 0.0]; 6], data, texture: None, tint, mask: false, cull_back: false, mat: Default::default() }));
        if !self.markers.iter().any(|m| m == id) { self.markers.push(id.into()); }
    }
    pub fn remove_marker(&mut self, id: &str) { self.pending.push(MeshUpdate::Remove(format!("marker:{id}"))); self.markers.retain(|m| m != id); }
    pub fn clear_markers(&mut self) { for m in self.markers.clone() { self.remove_marker(&m); } }

    /// UV-Koordinate (Texturraum, v nach unten) unter dem Strahl – zum Platzieren von Hautdetails.
    pub fn pick_uv(&self, o: [f32; 3], d: [f32; 3]) -> Option<[f32; 2]> {
        let (t, ti) = hase_core::import::raycast(&self.eval.pos, self.floor_off, &self.body.topo.base_tris, o, d)?;
        let tri = self.body.topo.base_tris[ti];
        let hit = [o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t];
        let g = |i: u32| { let p = self.eval.pos[i as usize]; [p[0], p[1] + self.floor_off, p[2]] };
        let (a, b, c) = (g(tri[0]), g(tri[1]), g(tri[2]));
        let sub = |x: [f32; 3], y: [f32; 3]| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
        let dot = |x: [f32; 3], y: [f32; 3]| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
        let (v0, v1, v2) = (sub(b, a), sub(c, a), sub(hit, a));
        let (d00, d01, d11, d20, d21) = (dot(v0, v0), dot(v0, v1), dot(v1, v1), dot(v2, v0), dot(v2, v1));
        let den = d00 * d11 - d01 * d01; if den.abs() < 1e-12 { return None; }
        let (bv, bw) = ((d11 * d20 - d01 * d21) / den, (d00 * d21 - d01 * d20) / den); let bu = 1.0 - bv - bw;
        let idx = &self.body.topo.indices[ti * 3..ti * 3 + 3];
        let uv = |i: u32| self.body.topo.uv[i as usize];
        let (ua, ub, uc) = (uv(idx[0]), uv(idx[1]), uv(idx[2]));
        Some([bu * ua[0] + bv * ub[0] + bw * uc[0], bu * ua[1] + bv * ub[1] + bw * uc[1]])
    }

    pub fn floor_offset(&self) -> f32 { self.floor_off }

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

#[cfg(test)]
mod body_content_tests {
    use super::*;
    #[test]
    fn body_content_change_is_flagged() {
        let mut s = Session::load(&hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap()).unwrap();
        assert!(s.ref_issues().is_empty());
        s.packages.body.digest = "geaendert".into();
        let i = s.ref_issues();
        assert_eq!(i.len(), 1);
        assert!(i[0].contains("Inhalt der Körperbasis"), "{i:?}");
    }
}

#[cfg(test)]
mod skin_tests {
    use super::*;
    #[test]
    fn decal_pick_and_compose() {
        let mut s = Session::load(&hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap()).unwrap();
        s.refresh();
        let h = s.body_height();
        let uv = s.pick_uv([0.0, h * 0.905, 3.0], [0.0, 0.0, -1.0]).expect("Treffer an der Nase");
        eprintln!("NASE_UV {} {}", uv[0], uv[1]);
        assert!(uv[0] > 0.0 && uv[0] < 1.0 && uv[1] > 0.0 && uv[1] < 1.0);
        s.edit_style(1, |st| st.skin.decals.push(hase_core::state::Decal { shape: 0, u: uv[0], v: uv[1], size: 0.01, rot: 0.0, color: [1.0, 0.0, 0.0], opacity: 1.0 }));
        let up = s.refresh();
        assert!(up.iter().any(|u| matches!(u, MeshUpdate::Texture { id, .. } if id == "body")), "Textur-Update erwartet");
        // ohne Details → Basistextur zurück
        s.edit_style(2, |st| st.skin.decals.clear());
        let up = s.refresh();
        assert!(up.iter().any(|u| matches!(u, MeshUpdate::Texture { .. })));
        // Persistenz
        s.edit_style(3, |st| { st.skin.lips.amount = 0.5; st.skin.freckles.amount = 0.3; });
        let j = s.state.style.to_json();
        assert_eq!(hase_core::state::Style::from_json(&j), s.state.style);
    }
}


// ---------------------------------------------------------------------------------------------------------
// VRM-Export: Zusammenstellung des Exportdokuments aus dem aktuellen Zustand (schnell, im UI-Thread); Schreiben im Hintergrund.
// ---------------------------------------------------------------------------------------------------------
pub struct ExportOptions { pub with_outfit: bool, pub expressions: bool, pub name: String, pub author: String, pub license_url: String, pub commercial: String, pub redistribution: bool }

const EXPR_STATES: &[(&str, &[(&str, f32)])] = &[
    ("BlinkL", &[("eyes/l-eye-height1-decr", 2.0), ("eyes/l-eye-height2-decr", 2.0), ("eyes/l-eye-height3-decr", 2.0)]),
    ("BlinkR", &[("eyes/r-eye-height1-decr", 2.0), ("eyes/r-eye-height2-decr", 2.0), ("eyes/r-eye-height3-decr", 2.0)]),
    ("JawOpen", &[("chin/chin-jaw-drop-incr", 2.0), ("mouth/mouth-lowerlip-middle-down", 2.0), ("mouth/mouth-upperlip-middle-up", 1.0)]),
    ("MouthWide", &[("mouth/mouth-scale-horiz-incr", 1.0)]),
    ("MouthNarrow", &[("mouth/mouth-scale-horiz-decr", 1.0)]),
    ("SmileUp", &[("mouth/mouth-angles-up", 1.0)]),
    ("FrownDown", &[("mouth/mouth-angles-down", 1.0)]),
];
const EXPRESSIONS: &[(&str, &[(&str, f32)])] = &[
    ("blink", &[("BlinkL", 1.0), ("BlinkR", 1.0)]), ("blinkLeft", &[("BlinkL", 1.0)]), ("blinkRight", &[("BlinkR", 1.0)]),
    ("aa", &[("JawOpen", 1.0)]), ("oh", &[("JawOpen", 0.6), ("MouthNarrow", 0.8)]), ("ou", &[("MouthNarrow", 1.0), ("JawOpen", 0.2)]),
    ("ee", &[("MouthWide", 0.8), ("JawOpen", 0.2)]), ("ih", &[("MouthWide", 0.5), ("JawOpen", 0.3)]),
    ("happy", &[("SmileUp", 1.0)]), ("relaxed", &[("SmileUp", 0.3)]), ("sad", &[("FrownDown", 1.0)]), ("surprised", &[("JawOpen", 0.5)]),
];

impl Session {
    pub fn build_export(&mut self, o: &ExportOptions) -> Result<(hase_core::export::ExDoc, Vec<String>), String> {
        use hase_core::export::*;
        if self.vrm.is_some() { return self.build_export_vrm(o); }
        if self.generic { return Err("Export für importierte Körperbasen ist noch nicht unterstützt (Rig-/Gewichtsübernahme aus dem Import fehlt).".into()); }
        let floor = self.floor_off;
        let mut report: Vec<String> = Vec::new();
        let (bones, vert_w) = load_rig(&self.packages.body.data_root, &self.body.obj, &self.eval.pos, floor)?;
        let pelvis = bones.iter().position(|b| b.name == "pelvis").unwrap_or(0) as u16;
        let mut nrm_body = Vec::new();
        normals(&self.eval.pos, &self.body.topo.base_tris, &mut nrm_body);
        let (mut meshes, mut materials, mut images): (Vec<ExMesh>, Vec<ExMaterial>, Vec<ExImage>) = (vec![], vec![], vec![]);

        // Von Kleidung verdeckte Körper-Vertices (nur mit Outfit)
        let topo = &self.body.topo;
        let mut covered = vec![false; self.eval.pos.len()];
        if o.with_outfit {
            for (id, rt) in &self.proxies {
                if !id.starts_with("clothes/") || !rt.created { continue; }
                let cn = &rt.nrm;
                let uniq: Vec<u32> = { let mut v: Vec<u32> = topo.vmap.clone(); v.sort_unstable(); v.dedup(); v };
                for &bv in &uniq {
                    let p = self.eval.pos[bv as usize];
                    let (mut best, mut bj) = (f32::MAX, 0usize);
                    for (j, c) in rt.fit.iter().enumerate() { let d = (c[0] - p[0]).powi(2) + (c[1] - p[1]).powi(2) + (c[2] - p[2]).powi(2); if d < best { best = d; bj = j; } }
                    let c = rt.fit[bj]; let n = cn.get(bj).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let signed = (p[0] - c[0]) * n[0] + (p[1] - c[1]) * n[1] + (p[2] - c[2]) * n[2];
                    if best.sqrt() < 0.025 && signed < 0.003 { covered[bv as usize] = true; }
                }
            }
        }
        // Körper-Mesh (Hilfsgeometrie ist nicht Teil von topo; verdeckte Flächen entfernen, Vertices kompaktieren)
        let keep_tri: Vec<bool> = topo.indices.chunks_exact(3).map(|t| !t.iter().all(|&r| covered[topo.vmap[r as usize] as usize])).collect();
        let mut remap = vec![u32::MAX; topo.vmap.len()]; let mut kept: Vec<u32> = Vec::new();
        let mut idx = Vec::new();
        for (ti, t) in topo.indices.chunks_exact(3).enumerate() { if !keep_tri[ti] { continue; } for &r in t { if remap[r as usize] == u32::MAX { remap[r as usize] = kept.len() as u32; kept.push(r); } idx.push(remap[r as usize]); } }
        let removed = keep_tri.iter().filter(|k| !**k).count();
        let mk_pos = |pos: &[[f32; 3]]| -> Vec<[f32; 3]> { kept.iter().map(|&r| { let p = pos[topo.vmap[r as usize] as usize]; [p[0], p[1] + floor, p[2]] }).collect() };
        let body_pos = mk_pos(&self.eval.pos);
        let (mut jo, mut we) = (Vec::new(), Vec::new());
        for &r in &kept { let (j, w) = pack_weights(&vert_w[topo.vmap[r as usize] as usize], pelvis); jo.push(j); we.push(w); }
        // Hauttextur
        let skin_img: Option<ImageRgba2> = self.skin_tex.clone().map(|base| {
            if self.state.style.skin.has_details() {
                if self.masks.is_none() { self.masks = Some(crate::skin::build_masks(&self.body)); }
                let c = crate::skin::compose(&base, self.masks.as_ref().unwrap(), &self.state.style.skin);
                ImageRgba2 { w: c.w, h: c.h, rgba: c.levels[0].clone() }
            } else { ImageRgba2 { w: base.w, h: base.h, rgba: base.levels[0].clone() } }
        });
        let skin_mat = { let im = skin_img.map(|i| { images.push(ExImage { name: "Skin".into(), w: i.w, h: i.h, rgba: i.rgba }); images.len() - 1 }); materials.push(ExMaterial { name: "Skin".into(), color: [1.0, 0.97, 0.95, 1.0], image: im, mask: false }); materials.len() - 1 };
        let body_i = meshes.len();
        meshes.push(ExMesh { name: "Body".into(), nrm: kept.iter().map(|&r| nrm_body[topo.vmap[r as usize] as usize]).collect(), uv: kept.iter().map(|&r| topo.uv[r as usize]).collect(), pos: body_pos, idx, joints: jo, weights: we, material: skin_mat, targets: vec![] });
        report.push(format!("Körper: {} Vertices, {} Dreiecke{}", meshes[0].pos.len(), meshes[0].idx.len() / 3, if removed > 0 { format!(" ({removed} von Kleidung verdeckte Flächen entfernt)") } else { String::new() }));

        // Proxys
        let mut ids: Vec<&String> = self.proxies.keys().collect(); ids.sort();
        let mut proxy_mesh: Vec<(usize, String)> = Vec::new(); // (Mesh-Index, Proxy-ID) für Expressions
        for id in ids {
            let rt = &self.proxies[id];
            if !rt.created { continue; }
            let clothes = id.starts_with("clothes/");
            if clothes && !o.with_outfit { continue; }
            let name = if id == EYES_ID { "Eyes".to_string() } else { let (cat, n) = id.split_once('/').unwrap_or(("", id)); format!("{}{}", match cat { "hair" => "Hair_", "eyebrows" => "Eyebrows_", "eyelashes" => "Eyelashes_", "clothes" => "Clothes_", _ => "" }, n) };
            let t = &rt.proxy.topo;
            let mut obj_w: HashMap<u32, Vec<(u16, f32)>> = HashMap::new();
            let mut wv = |k: usize| -> Vec<(u16, f32)> {
                let mut acc: HashMap<u16, f32> = HashMap::new();
                for (vi, w) in [(rt.proxy.ia[k], rt.proxy.w[k][0]), (rt.proxy.ib[k], rt.proxy.w[k][1]), (rt.proxy.ic[k], rt.proxy.w[k][2])] { for (b, bw) in &vert_w[vi as usize] { *acc.entry(*b).or_default() += w * bw; } }
                let mut v: Vec<(u16, f32)> = acc.into_iter().collect(); v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); v.truncate(4);
                let tot: f32 = v.iter().map(|x| x.1).sum(); if tot > 0.0 { for x in v.iter_mut() { x.1 /= tot; } } v
            };
            let (mut jo, mut we) = (Vec::new(), Vec::new());
            for &ov in &t.vmap { let w = obj_w.entry(ov).or_insert_with(|| wv(ov as usize)).clone(); let (j, x) = pack_weights(&w, pelvis); jo.push(j); we.push(x); }
            let pos: Vec<[f32; 3]> = t.vmap.iter().map(|&v| { let p = rt.fit[v as usize]; [p[0], p[1] + floor, p[2]] }).collect();
            let nrm: Vec<[f32; 3]> = t.vmap.iter().map(|&v| rt.nrm.get(v as usize).copied().unwrap_or([0.0, 1.0, 0.0])).collect();
            // Material/Textur
            let tint = self.state.assets.iter().find(|a| a.id == *id).map(|a| a.tint).unwrap_or([1.0; 3]);
            let mut color = [tint[0], tint[1], tint[2], 1.0];
            let tex = rt.texture.clone();
            let img_idx = if id == EYES_ID {
                tex.map(|im| {
                    // linkes Auge = Welt-x > 0: Mittelwert der UVs der Vertices mit x > 0 bestimmt die Textur-Insel
                    let (mut su, mut sv, mut n) = (0f32, 0f32, 0f32);
                    for (r, &ov) in t.vmap.iter().enumerate() { if rt.fit[ov as usize][0] > 0.0 { su += t.uv[r][0]; sv += t.uv[r][1]; n += 1.0; } }
                    let (u, v) = (su / n.max(1.0), sv / n.max(1.0));
                    let first = ((u - 0.705).powi(2) + (v - 0.296).powi(2)) < ((u - 0.290).powi(2) + (v - 0.710).powi(2));
                    let baked = hase_core::export::bake_eye(&im.levels[0], im.w, im.h, &self.state.style.eyes, first);
                    images.push(ExImage { name: "Eyes".into(), w: im.w, h: im.h, rgba: baked }); images.len() - 1
                })
            } else if id.starts_with("hair/") {
                let h = &self.state.style.hair;
                color = [(h.root[0] + h.tip[0]) * 0.5, (h.root[1] + h.tip[1]) * 0.5, (h.root[2] + h.tip[2]) * 0.5, 1.0];
                tex.map(|im| { images.push(ExImage { name: name.clone(), w: im.w, h: im.h, rgba: im.levels[0].clone() }); images.len() - 1 })
            } else { tex.map(|im| { images.push(ExImage { name: name.clone(), w: im.w, h: im.h, rgba: im.levels[0].clone() }); images.len() - 1 }) };
            materials.push(ExMaterial { name: name.clone(), color, image: img_idx, mask: rt.proxy.alpha_mask });
            meshes.push(ExMesh { name: name.clone(), pos, nrm, uv: t.uv.clone(), idx: t.indices.clone(), joints: jo, weights: we, material: materials.len() - 1, targets: vec![] });
            if id.starts_with("eyebrows/") || id.starts_with("eyelashes/") { proxy_mesh.push((meshes.len() - 1, id.clone())); }
            report.push(format!("{name}: {} Vertices", meshes.last().unwrap().pos.len()));
        }

        // Expressions (MakeHuman-Targets als Morph-Targets auf Körper, Brauen, Wimpern)
        let mut expressions = Vec::new();
        if o.expressions {
            let m = &self.body.morphs;
            let mut rest_fit: HashMap<usize, Vec<[f32; 3]>> = HashMap::new();
            let mut state_names = Vec::new();
            for (st, parts) in EXPR_STATES {
                let mut p = self.eval.pos.clone();
                let mut ok = true;
                for (tn, w) in *parts { match m.index_of(tn) { Some(t) => { let (a, b) = (m.t_off[t] as usize, m.t_off[t + 1] as usize); for k in a..b { let v = m.t_idx[k] as usize; let d = m.t_delta[k]; for c in 0..3 { p[v][c] += w * d[c] * hase_core::body::SCALE; } } } None => ok = false } }
                if !ok { continue; }
                state_names.push(st.to_string());
                // Körper
                let dpos: Vec<[f32; 3]> = kept.iter().map(|&r| { let ov = topo.vmap[r as usize] as usize; [p[ov][0] - self.eval.pos[ov][0], p[ov][1] - self.eval.pos[ov][1], p[ov][2] - self.eval.pos[ov][2]] }).collect();
                meshes[body_i].targets.push((st.to_string(), dpos));
                for (mi, id) in &proxy_mesh {
                    let rt = &self.proxies[id];
                    let mut f1 = Vec::new(); rt.proxy.fit(&p, &mut f1);
                    let r0 = rest_fit.entry(*mi).or_insert_with(|| rt.fit.clone()).clone();
                    let d: Vec<[f32; 3]> = rt.proxy.topo.vmap.iter().map(|&v| { let (a, b) = (f1[v as usize], r0[v as usize]); [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }).collect();
                    meshes[*mi].targets.push((st.to_string(), d));
                }
            }
            for (name, parts) in EXPRESSIONS {
                let mut binds = Vec::new();
                for (st, w) in *parts { if let Some(ti) = state_names.iter().position(|s| s == st) { for (mi, mesh) in meshes.iter().enumerate() { if mesh.targets.get(ti).map(|t| t.0 == *st).unwrap_or(false) { binds.push((mi, ti, *w)); } } } }
                if !binds.is_empty() { expressions.push((name.to_string(), binds)); }
            }
            report.push(format!("{} Expressions (aus MakeHuman-Targets; Mundöffnung begrenzt, da das Basismesh keinen Mundraum hat)", expressions.len()));
        }
        // Humanoid
        let mut humanoid = std::collections::BTreeMap::new();
        for (vrm, ue) in humanoid_map() { if let Some(i) = bones.iter().position(|b| b.name.eq_ignore_ascii_case(&ue)) { humanoid.insert(vrm, i); } }
        for req in ["hips", "spine", "head", "leftUpperLeg", "rightUpperLeg", "leftLowerLeg", "rightLowerLeg", "leftFoot", "rightFoot", "leftUpperArm", "rightUpperArm", "leftLowerArm", "rightLowerArm", "leftHand", "rightHand"] { if !humanoid.contains_key(req) { return Err(format!("Pflicht-Bone {req} fehlt im Rig")); } }
        report.push(format!("Skelett: {} Bones, {} Humanoid-Zuordnungen (alle 15 Pflicht-Bones)", bones.len(), humanoid.len()));
        if self.state.style.hair.highlight_amt > 0.0 || self.state.style.hair.root != self.state.style.hair.tip { report.push("Verlust: Haar-Farbverlauf und Highlights werden als Mittelfarbe exportiert (MToon kennt keinen Höhenverlauf).".into()); }
        if self.state.style.skin.has_details() { report.push("Hinweis: Make-up/Sommersprossen/Details sind in die Hauttextur eingerechnet (nicht mehr einzeln editierbar im VRM).".into()); }
        report.push("Hinweis: Material-Glanz/Reflexion der Augen wird nicht exportiert; Pupillen-/Irisfarben sind in die Augentextur eingebacken.".into());
        let meta = ExMeta { name: o.name.clone(), author: o.author.clone(), copyright: "Basismesh, Targets, Rig, Haare, Augen, Kleidung: MakeHuman/MPFB2-Community-Assets (CC0 1.0); Zusammenstellung mit HASE VRM MAKER".into(), license_url: o.license_url.clone(), commercial: o.commercial.clone(), redistribution: o.redistribution };
        Ok((ExDoc { meshes, materials, images, bones, humanoid, expressions, meta }, report))
    }
}

pub struct ImageRgba2 { pub w: u32, pub h: u32, pub rgba: Vec<u8> }


// ---------------------------------------------------------------------------------------------------------
// Pose/Animation-Vorschau (CPU-Skinning, nur Referenz-Körperbasis)
// ---------------------------------------------------------------------------------------------------------
impl Session {
    pub fn load_clip(&mut self, path: &Path) -> Result<(), String> {
        if self.vrm.is_some() && !self.vrm.as_ref().unwrap().model.humanoid.contains_key("hips") { return Err("Das Modell hat keine humanoide Zuordnung (hips) – Posen nicht möglich.".into()); }
        if self.generic && self.vrm.is_none() { return Err("Animationsvorschau ist für importierte Körperbasen noch nicht unterstützt (kein Rig/Gewichte aus dem Import).".into()); }
        let clip = hase_core::pose::Clip::load(path)?;
        self.play = Playback { clip: Some(clip), t: 0.0, playing: true, looping: true, speed: 1.0 };
        self.pose_active = true;
        Ok(())
    }
    pub fn stop_pose(&mut self) {
        self.play.clip = None; self.play.playing = false;
        if self.pose_active { self.pose_active = false; self.dirty = true; self.assets_dirty = true; self.resend_body = true; }
    }
    pub fn tick(&mut self, dt: f32) {
        if !self.play.playing { return; }
        let Some(c) = self.play.clip.as_ref() else { return };
        self.play.t += dt * self.play.speed;
        if self.play.t > c.duration { if self.play.looping { self.play.t %= c.duration.max(1e-3); } else { self.play.t = c.duration; self.play.playing = false; } }
    }

    fn emit_pose(&mut self) {
        use hase_core::pose::{skin_point, PoseRig};
        let Some(clip) = self.play.clip.as_ref() else { return };
        if self.rig.is_none() { match hase_core::export::RigData::load(&self.packages.body.data_root, &self.body.obj) { Ok(r) => self.rig = Some(r), Err(_) => return } }
        let floor = self.floor_off;
        if self.pose_rig.is_none() { let bones = self.rig.as_ref().unwrap().bones(&self.eval.pos, floor); self.pose_rig = Some(PoseRig::new(bones)); }
        let pr = self.pose_rig.as_ref().unwrap();
        let hip_scale = pr.hips_rest_y() / clip_hip_height(clip);
        let m = pr.skin_matrices(Some((clip, self.play.t)), hip_scale);
        let rig = self.rig.as_ref().unwrap();
        let pack = |w: &[(u16, f32)]| hase_core::export::pack_weights(w, 0);
        // Körper
        let topo = &self.body.topo;
        let mut out = vec![[0f32; 3]; self.eval.pos.len()];
        let mut done = vec![false; self.eval.pos.len()];
        for &v in &topo.vmap { let v = v as usize; if done[v] { continue; } done[v] = true; let (j, w) = pack(&rig.weights[v]); let p = self.eval.pos[v]; out[v] = skin_point(&m, j, w, [p[0], p[1] + floor, p[2]]); }
        let mut nrm = Vec::new(); normals(&out, &topo.base_tris, &mut nrm);
        if !self.hide_body { self.pending.push(MeshUpdate::Positions { id: "body".into(), data: Self::interleave(&out, &nrm, &topo.vmap, 0.0) }); }
        // Proxys
        let ids: Vec<String> = self.proxies.keys().cloned().collect();
        for id in ids {
            let rt = &self.proxies[&id];
            if !rt.created { continue; }
            if !self.proxy_skin.contains_key(&id) {
                let mut v = Vec::with_capacity(rt.proxy.ia.len());
                for k in 0..rt.proxy.ia.len() {
                    let mut acc: HashMap<u16, f32> = HashMap::new();
                    for (vi, w) in [(rt.proxy.ia[k], rt.proxy.w[k][0]), (rt.proxy.ib[k], rt.proxy.w[k][1]), (rt.proxy.ic[k], rt.proxy.w[k][2])] { for (b, bw) in &rig.weights[vi as usize] { *acc.entry(*b).or_default() += w * bw; } }
                    let mut l: Vec<(u16, f32)> = acc.into_iter().collect(); l.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); l.truncate(4);
                    v.push(pack(&l));
                }
                self.proxy_skin.insert(id.clone(), v);
            }
            let sk = &self.proxy_skin[&id];
            let rt = &self.proxies[&id];
            let pos: Vec<[f32; 3]> = rt.fit.iter().enumerate().map(|(k, p)| skin_point(&m, sk[k].0, sk[k].1, [p[0], p[1] + floor, p[2]])).collect();
            let tris: Vec<[u32; 3]> = rt.proxy.topo.base_tris.clone();
            let mut pn = Vec::new(); normals(&pos, &tris, &mut pn);
            self.pending.push(MeshUpdate::Positions { id: format!("proxy:{id}"), data: Self::interleave(&pos, &pn, &rt.proxy.topo.vmap, 0.0) });
        }
    }
}

fn clip_hip_height(c: &hase_core::pose::Clip) -> f32 { c.hips_rest_height() }

#[cfg(test)]
mod export_pose_tests {
    use super::*;

    fn session_with_assets() -> Session {
        let mut s = Session::load(&hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap()).unwrap();
        s.default_assets_blocking(&["eyebrows/mindfront_eyebrows_01", "eyelashes/mindfront_eyelashes_01", "hair/short02", "clothes/toigo_basic_tucked_t-shirt", "clothes/toigo_wool_pants"]);
        s.refresh();
        s
    }

    #[test]
    fn export_is_valid_glb_with_rig_and_expressions() {
        let mut s = session_with_assets();
        let o = ExportOptions { with_outfit: true, expressions: true, name: "T".into(), author: "A".into(), license_url: "https://vrm.dev/licenses/1.0/".into(), commercial: "personalNonProfit".into(), redistribution: true };
        let (doc, report) = s.build_export(&o).unwrap();
        assert!(report.iter().any(|r| r.contains("Pflicht-Bones")));
        assert!(doc.expressions.iter().any(|e| e.0 == "blink") && doc.humanoid.len() >= 15 && doc.bones.len() == 53);
        for m in &doc.meshes {
            for (w, j) in m.weights.iter().zip(&m.joints) {
                let sum: f32 = w.iter().sum(); assert!((sum - 1.0).abs() < 1e-4 && w.iter().all(|x| *x >= 0.0), "{}: Gewichte", m.name);
                for k in 0..4 { assert!(w[k] > 0.0 || j[k] == 0); }
            }
        }
        let glb = hase_core::export::write_glb(&doc).unwrap();
        let (meshes, mats, imgs, morphs) = hase_core::export::verify_glb(&glb).unwrap();
        assert!(meshes >= 6 && mats == meshes && imgs >= 5 && morphs > 20, "{meshes} {mats} {imgs} {morphs}");
        // Basisprofil: keine Kleidung
        let o2 = ExportOptions { with_outfit: false, ..o };
        let (d2, _) = s.build_export(&o2).unwrap();
        assert!(!d2.meshes.iter().any(|m| m.name.starts_with("Clothes_")) && d2.meshes.iter().any(|m| m.name == "Body"));
        assert!(d2.meshes[0].idx.len() > doc.meshes[0].idx.len(), "mit Outfit entfallen verdeckte Körperflächen");
    }

    #[test]
    fn pose_playback_keeps_character_upright_and_moves_limbs() {
        let mut s = session_with_assets();
        let repo = s.repo.clone();
        let rest_head_y = { let h = s.body_height(); h };
        s.load_clip(&repo.join("content/testcandidate/animations/UAL1/Idle_Loop.vrma")).unwrap();
        s.play.playing = false; s.play.t = 0.0;
        let up = s.refresh();
        let body = up.iter().rev().find_map(|u| match u { MeshUpdate::Positions { id, data } if id == "body" => Some(data.clone()), _ => None }).expect("Pose-Update für den Körper");
        let max_y = body.chunks_exact(6).map(|v| v[1]).fold(f32::MIN, f32::max);
        assert!((max_y - rest_head_y).abs() < 0.35, "Körper steht aufrecht: max y {max_y}, Ruhehöhe {rest_head_y}");
        // anderer Zeitpunkt → andere Positionen
        s.play.t = 0.5; let up2 = s.refresh();
        let body2 = up2.iter().rev().find_map(|u| match u { MeshUpdate::Positions { id, data } if id == "body" => Some(data.clone()), _ => None }).unwrap();
        assert!(body.iter().zip(&body2).any(|(a, b)| (a - b).abs() > 1e-4), "Animation verändert Positionen");
        s.stop_pose();
        let up3 = s.refresh();
        assert!(up3.iter().any(|u| matches!(u, MeshUpdate::Positions { id, .. } if id == "body")), "Ruhepose wird wieder gesendet");
    }
}

#[cfg(test)]
mod pose_perf {
    use super::*;
    #[test]
    fn pose_frame_cost() {
        let mut s = Session::load(&hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap()).unwrap();
        s.default_assets_blocking(&["eyebrows/mindfront_eyebrows_01", "eyelashes/mindfront_eyelashes_01", "hair/long01", "clothes/toigo_basic_tucked_t-shirt", "clothes/toigo_wool_pants"]);
        s.refresh();
        let repo = s.repo.clone();
        s.load_clip(&repo.join("content/testcandidate/animations/UAL1/Dance_Loop.vrma")).unwrap();
        s.play.playing = false;
        s.refresh();
        let t0 = std::time::Instant::now();
        for i in 0..100 { s.play.t = i as f32 * 0.01; let _ = s.refresh(); }
        let (v, t) = s.counts();
        eprintln!("POSE_FRAME {:.2} ms pro Frame ({} Vertices, {} Dreiecke)", t0.elapsed().as_secs_f64() * 10.0, v, t);
    }
}


// ---------------------------------------------------------------------------------------------------------
// Statisches VRM-/glTF-Modell öffnen
// ---------------------------------------------------------------------------------------------------------
impl Session {
    pub fn open_vrm(&mut self, path: &Path) -> Result<String, String> {
        use hase_core::pose::PoseRig;
        let model = hase_core::vrm_model::load(path)?;
        if model.bones.is_empty() { return Err("Das Modell hat kein Skelett.".into()); }
        let min_y = model.parts.iter().flat_map(|p| p.pos.iter().map(|v| v[1])).fold(f32::MAX, f32::min);
        let floor = -min_y;
        let bones: Vec<hase_core::export::ExBone> = model.bones.iter().map(|b| hase_core::export::ExBone { name: b.name.clone(), parent: b.parent, pos: [b.pos[0], b.pos[1] + floor, b.pos[2]] }).collect();
        let rig = PoseRig::generic(bones, model.rest_rot.clone(), model.vrm_names.clone());
        let tris: Vec<Vec<[u32; 3]>> = model.parts.iter().map(|p| p.idx.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect()).collect();
        let info = format!("{}: {} Teile, {} Vertices, {} Bones, {} Ausdrücke{}", model.name, model.parts.len(), model.parts.iter().map(|p| p.pos.len()).sum::<usize>(), model.bones.len(), model.expressions.len(), if model.spring_bones { ", Spring Bones vorhanden (in der Vorschau nicht simuliert)" } else { "" });
        // MakeHuman-Szene ausblenden
        self.stop_pose();
        self.pending.push(MeshUpdate::Remove("body".into()));
        for (id, rt) in self.proxies.iter_mut() { self.pending.push(MeshUpdate::Remove(format!("proxy:{id}"))); rt.created = false; }
        self.body_created = false;
        self.clear_markers();
        let skin_parts: std::collections::BTreeSet<usize> = model.parts.iter().enumerate().filter(|(_, p)| { let (lo, hi) = p.pos.iter().fold((f32::MAX, f32::MIN), |a, q| (a.0.min(q[1]), a.1.max(q[1]))); hi - lo >= 0.5 * model.height }).map(|(i, _)| i).collect();
        for rt in self.proxies.values_mut() { rt.created = false; }
        self.proxy_skin.clear(); self.assets_dirty = true;
        self.vrm = Some(VrmRuntime { model, rig, tris, floor, expr: Default::default(), raw: Default::default(), shown: vec![], dirty: true, path: path.to_path_buf(), hidden: Default::default(), skin_parts, dress: None, dress_info: String::new() });
        Ok(info)
    }

    pub fn close_vrm(&mut self) {
        if let Some(v) = self.vrm.take() { for i in 0..v.model.parts.len() { self.pending.push(MeshUpdate::Remove(format!("vrm:{i}"))); } }
        for (id, rt) in self.proxies.iter_mut() { self.pending.push(MeshUpdate::Remove(format!("proxy:{id}"))); rt.created = false; }
        self.proxy_skin.clear();
        self.stop_pose();
        self.dirty = true; self.assets_dirty = true; self.resend_body = true; self.style_dirty = true; self.skin_dirty = true; self.body_created = false;
    }

    pub fn set_expression(&mut self, name: &str, v: f32) { if let Some(r) = self.vrm.as_mut() { if v.abs() < 1e-4 { r.expr.remove(name); } else { r.expr.insert(name.to_string(), v); } r.dirty = true; } }
    pub fn set_raw_target(&mut self, part: usize, target: usize, v: f32) { if let Some(r) = self.vrm.as_mut() { if v.abs() < 1e-4 { r.raw.remove(&(part, target)); } else { r.raw.insert((part, target), v); } r.dirty = true; } }

    /// Bildet die MakeHuman-Basis auf die Haut des geöffneten Modells ab (einmal je Modell/Hautteil-Auswahl).
    fn ensure_dress(&mut self) -> Result<(), String> {
        use hase_core::dress::{register, Target};
        if self.vrm.as_ref().map(|v| v.dress.is_some()).unwrap_or(true) { return Ok(()); }
        if self.dirty { self.params.target_weights(&self.state.values, self.body.morphs.names.len(), &mut self.desired); self.eval.apply(&self.body, &self.desired); self.dirty = false; }
        if self.rig.is_none() { self.rig = Some(hase_core::export::RigData::load(&self.packages.body.data_root, &self.body.obj)?); }
        if self.body_uniq.is_empty() { let mut uniq: Vec<u32> = self.body.topo.vmap.clone(); uniq.sort_unstable(); uniq.dedup(); self.body_uniq = uniq; }
        let floor_off = -self.body.topo.vmap.iter().map(|&v| self.eval.pos[v as usize][1]).fold(f32::MAX, f32::min);
        let rig = self.rig.as_ref().unwrap();
        let mh_bones = rig.bones(&self.eval.pos, floor_off);
        let mh_pos: Vec<[f32; 3]> = self.eval.pos.iter().map(|p| [p[0], p[1] + floor_off, p[2]]).collect();
        let v = self.vrm.as_ref().unwrap();
        let (mut sp, mut sn, mut sj, mut sw) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for &pi in &v.skin_parts { let p = &v.model.parts[pi]; for k in 0..p.pos.len() { sp.push([p.pos[k][0], p.pos[k][1] + v.floor, p.pos[k][2]]); sn.push(p.nrm[k]); sj.push(p.joints[k]); sw.push(p.weights[k]); } }
        if sp.is_empty() { return Err("Keine Hautteile gewählt – Körperteile in der Liste als „Haut“ markieren.".into()); }
        let tb: Vec<hase_core::export::ExBone> = v.model.bones.iter().map(|b| hase_core::export::ExBone { name: b.name.clone(), parent: b.parent, pos: [b.pos[0], b.pos[1] + v.floor, b.pos[2]] }).collect();
        let reg = register(&mh_pos, &self.body_uniq, &self.body.topo.base_tris, &mh_bones, &rig.weights, &Target { bones: &tb, humanoid: &v.model.humanoid, skin_pos: &sp, skin_nrm: &sn })?;
        let ids: Vec<u32> = (0..sp.len() as u32).collect();
        let grid = hase_core::fit::BodyGrid::build(&sp, &ids, 0.03);
        let info = format!("Anprobe: {} Bones zugeordnet, Abweichung zur Haut {:.1} → {:.1} mm, Registrierung {:.0} ms", reg.paired, reg.icp_rms_before * 1000.0, reg.icp_rms_after * 1000.0, reg.ms);
        let v = self.vrm.as_mut().unwrap();
        v.dress = Some(VrmDress { skin_pos: sp, skin_nrm: sn, skin_joints: sj, skin_weights: sw, skin_ids: ids, grid, base: reg.base });
        v.dress_info = info;
        self.proxy_skin.clear();
        Ok(())
    }

    pub fn set_skin_part(&mut self, part: usize, on: bool) { if let Some(v) = self.vrm.as_mut() { if on { v.skin_parts.insert(part); } else { v.skin_parts.remove(&part); } v.dress = None; self.assets_dirty = true; } }
    pub fn set_part_hidden(&mut self, part: usize, hide: bool) { if let Some(v) = self.vrm.as_mut() { if hide { v.hidden.insert(part); } else { v.hidden.remove(&part); } v.dirty = true; } }

    fn refresh_vrm(&mut self) -> Vec<MeshUpdate> {
        use hase_core::pose::skin_point;
        if self.assets_dirty && self.proxies.keys().any(|k| k.starts_with("clothes/")) { if let Err(e) = self.ensure_dress() { if let Some(v) = self.vrm.as_mut() { v.dress_info = e; } } }
        let mut out = std::mem::take(&mut self.pending);
        let posing = self.play.clip.is_some();
        let clip_t = self.play.t;
        let assets_dirty = std::mem::take(&mut self.assets_dirty);
        let Some(v) = self.vrm.as_mut() else { return out };
        let floor = v.floor;
        if v.shown.len() != v.model.parts.len() { v.shown = vec![false; v.model.parts.len()]; }
        for (i, p) in v.model.parts.iter().enumerate() {
            let hide = v.hidden.contains(&i);
            if hide && v.shown[i] { out.push(MeshUpdate::Remove(format!("vrm:{i}"))); v.shown[i] = false; }
            else if !hide && !v.shown[i] {
                let mut data = Vec::with_capacity(p.pos.len() * 6);
                for (q, n) in p.pos.iter().zip(&p.nrm) { data.extend_from_slice(&[q[0], q[1] + floor, q[2], n[0], n[1], n[2]]); }
                let tex = p.image.as_ref().map(|im| image_from_rgba(im.w, im.h, im.rgba.clone(), 2048));
                out.push(MeshUpdate::Create(MeshDesc { id: format!("vrm:{i}"), indices: p.idx.clone(), uv: p.uv.clone(), data, texture: tex, tint: [p.color[0], p.color[1], p.color[2]], mask: p.mask, cull_back: !p.double_sided, mat: Default::default() }));
                v.shown[i] = true; v.dirty = true;
            }
        }
        let mats = if let Some(clip) = self.play.clip.as_ref() { Some(v.rig.skin_matrices(Some((clip, clip_t)), v.rig.hips_rest_y() / clip.hips_rest_height().max(1e-3))) } else { None };
        if v.dirty || posing {
            // Gewichte je (Teil, Target) aus Ausdrücken + rohen Reglern
            let mut tw: std::collections::BTreeMap<(usize, usize), f32> = v.raw.clone();
            for (name, val) in &v.expr { if let Some((_, binds)) = v.model.expressions.iter().find(|e| &e.0 == name) { for (pi, ti, w) in binds { *tw.entry((*pi, *ti)).or_default() += val * w; } } }
            for (i, p) in v.model.parts.iter().enumerate() {
                if !v.shown[i] { continue; }
                let mut pos: Vec<[f32; 3]> = p.pos.clone();
                for ((pi, ti), w) in &tw { if *pi == i && w.abs() > 1e-5 { for (k, d) in p.targets[*ti].1.iter().enumerate() { pos[k][0] += w * d[0]; pos[k][1] += w * d[1]; pos[k][2] += w * d[2]; } } }
                for q in pos.iter_mut() { q[1] += floor; }
                if let Some(m) = &mats { for (k, q) in pos.iter_mut().enumerate() { *q = skin_point(m, p.joints[k], p.weights[k], *q); } }
                let mut nrm = Vec::new(); normals(&pos, &v.tris[i], &mut nrm);
                let mut data = Vec::with_capacity(pos.len() * 6);
                for (q, n) in pos.iter().zip(&nrm) { data.extend_from_slice(&[q[0], q[1], q[2], n[0], n[1], n[2]]); }
                out.push(MeshUpdate::Positions { id: format!("vrm:{i}"), data });
            }
            v.dirty = false;
        }
        // Anprobe: Kleidung an die abgebildete Basis fitten, gegen die Haut kollisionskorrigieren, mit den Gewichten der Haut skinnen
        if let Some(d) = v.dress.as_ref() {
            let ids: Vec<String> = self.proxies.keys().filter(|k| k.starts_with("clothes/")).cloned().collect();
            for id in ids {
                let tint = self.state.assets.iter().find(|a| a.id == id).map(|a| a.tint).unwrap_or([0.9; 3]);
                let rt = self.proxies.get_mut(&id).unwrap();
                let refit = !rt.created || assets_dirty || !self.proxy_skin.contains_key(&id);
                if !refit && !posing { continue; }
                if refit {
                    rt.proxy.fit(&d.base, &mut rt.fit);
                    hase_core::fit::push_out(&mut rt.fit, &rt.proxy.adj, &rt.proxy.topo.base_tris, &d.grid, &d.skin_pos, &d.skin_nrm, &d.skin_ids, 0.004, false);
                    normals(&rt.fit, &rt.proxy.topo.base_tris, &mut rt.nrm);
                    self.proxy_skin.insert(id.clone(), hase_core::dress::transfer_weights(&rt.fit, &d.skin_pos, &d.skin_joints, &d.skin_weights, &d.grid));
                }
                let (pos, nrm) = if let Some(m) = &mats { let sk = &self.proxy_skin[&id]; let pp: Vec<[f32; 3]> = rt.fit.iter().enumerate().map(|(k, q)| skin_point(m, sk[k].0, sk[k].1, *q)).collect(); let mut nn = Vec::new(); normals(&pp, &rt.proxy.topo.base_tris, &mut nn); (pp, nn) } else { (rt.fit.clone(), rt.nrm.clone()) };
                let data = Self::interleave(&pos, &nrm, &rt.proxy.topo.vmap, 0.0);
                if !rt.created { out.push(MeshUpdate::Create(MeshDesc { id: format!("proxy:{id}"), indices: rt.proxy.topo.indices.clone(), uv: rt.proxy.topo.uv.clone(), data, texture: rt.texture.clone(), tint, mask: rt.proxy.alpha_mask, cull_back: false, mat: Default::default() })); rt.created = true; }
                else { out.push(MeshUpdate::Positions { id: format!("proxy:{id}"), data }); }
            }
        }
        out
    }
}


impl Session {
    /// Export eines geöffneten VRM-Modells (ggf. mit anprobierter Kleidung): Originalteile, Skelett, Humanoid und Ausdrücke bleiben erhalten.
    fn build_export_vrm(&mut self, o: &ExportOptions) -> Result<(hase_core::export::ExDoc, Vec<String>), String> {
        use hase_core::export::*;
        let v = self.vrm.as_ref().unwrap();
        let m = &v.model;
        let mut report = Vec::new();
        let (mut meshes, mut materials, mut images) = (Vec::<ExMesh>::new(), Vec::<ExMaterial>::new(), Vec::<ExImage>::new());
        let mut part_mesh: Vec<Option<usize>> = vec![None; m.parts.len()];
        for (i, p) in m.parts.iter().enumerate() {
            if v.hidden.contains(&i) { continue; }
            let img = p.image.as_ref().map(|im| { images.push(ExImage { name: format!("{}_{i}", p.name), w: im.w, h: im.h, rgba: im.rgba.clone() }); images.len() - 1 });
            materials.push(ExMaterial { name: if p.mat_name.is_empty() { format!("{}_{i}", p.name) } else { p.mat_name.clone() }, color: p.color, image: img, mask: p.mask });
            part_mesh[i] = Some(meshes.len());
            meshes.push(ExMesh { name: format!("{}_{i}", p.name), pos: p.pos.clone(), nrm: p.nrm.clone(), uv: p.uv.clone(), idx: p.idx.clone(), joints: p.joints.clone(), weights: p.weights.clone(), material: materials.len() - 1, targets: p.targets.clone() });
        }
        report.push(format!("Modell „{}“: {} Teile, {} Bones", m.name, meshes.len(), m.bones.len()));
        let floor = v.floor;
        let mut worn = 0;
        if o.with_outfit && v.dress.is_some() {
            let mut ids: Vec<&String> = self.proxies.keys().filter(|k| k.starts_with("clothes/")).collect(); ids.sort();
            for id in ids {
                let rt = &self.proxies[id];
                let Some(sk) = self.proxy_skin.get(id) else { continue };
                if !rt.created { continue; }
                let t = &rt.proxy.topo;
                let name = format!("Clothes_{}", id.split_once('/').map(|x| x.1).unwrap_or(id));
                let pos: Vec<[f32; 3]> = t.vmap.iter().map(|&vi| { let p = rt.fit[vi as usize]; [p[0], p[1] - floor, p[2]] }).collect();
                let nrm: Vec<[f32; 3]> = t.vmap.iter().map(|&vi| rt.nrm.get(vi as usize).copied().unwrap_or([0.0, 1.0, 0.0])).collect();
                let (jo, we): (Vec<[u16; 4]>, Vec<[f32; 4]>) = t.vmap.iter().map(|&vi| sk[vi as usize]).unzip();
                let tint = self.state.assets.iter().find(|a| a.id == *id).map(|a| a.tint).unwrap_or([1.0; 3]);
                let img = rt.texture.as_ref().map(|im| { images.push(ExImage { name: name.clone(), w: im.w, h: im.h, rgba: im.levels[0].clone() }); images.len() - 1 });
                materials.push(ExMaterial { name: name.clone(), color: [tint[0], tint[1], tint[2], 1.0], image: img, mask: rt.proxy.alpha_mask });
                meshes.push(ExMesh { name, pos, nrm, uv: t.uv.clone(), idx: t.indices.clone(), joints: jo, weights: we, material: materials.len() - 1, targets: vec![] });
                worn += 1;
            }
            report.push(format!("Kleidung: {worn} Teile, mit den Hautgewichten des Modells gekoppelt"));
        }
        let expressions: Vec<(String, Vec<(usize, usize, f32)>)> = m.expressions.iter().map(|(n, b)| (n.clone(), b.iter().filter_map(|(pi, ti, w)| part_mesh[*pi].map(|mi| (mi, *ti, *w))).collect())).filter(|(_, b): &(String, Vec<_>)| !b.is_empty()).collect();
        report.push(format!("{} Ausdrücke übernommen", expressions.len()));
        if m.spring_bones { report.push("Verlust: Spring Bones (Haar-/Zubehörphysik) und die MToon-Materialeinstellungen des Originals werden nicht übernommen; Materialien erscheinen vereinfacht.".into()); }
        else { report.push("Verlust: MToon-Materialeinstellungen des Originals werden vereinfacht.".into()); }
        let meta = ExMeta { name: o.name.clone(), author: o.author.clone(), copyright: format!("Basismodell „{}“ (Rechte beim Urheber des Originals), Kleidung: MakeHuman/MPFB2-Community-Assets (CC0 1.0); Zusammenstellung mit HASE VRM MAKER", m.name), license_url: o.license_url.clone(), commercial: o.commercial.clone(), redistribution: o.redistribution };
        let bones = m.bones.iter().map(|b| ExBone { name: b.name.clone(), parent: b.parent, pos: b.pos }).collect();
        Ok((ExDoc { meshes, materials, images, bones, humanoid: m.humanoid.clone(), expressions, meta }, report))
    }
}

#[cfg(test)]
mod vrm_open_tests {
    use super::*;

    #[test]
    fn open_vrm_express_pose_close() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        s.refresh();
        let info = s.open_vrm(&repo.join("content/testcandidate/avatar_b_makehuman.vrm")).unwrap();
        assert!(info.contains("Teile"));
        let up = s.refresh();
        assert!(up.iter().filter(|u| matches!(u, MeshUpdate::Create(d) if d.id.starts_with("vrm:"))).count() >= 2);
        assert!(up.iter().any(|u| matches!(u, MeshUpdate::Remove(id) if id == "body")), "MakeHuman-Körper wird ausgeblendet");
        // Ausdruck wirkt
        let before = s.refresh();
        assert!(before.is_empty(), "Ruhezustand erzeugt keine Updates");
        s.set_expression("blink", 1.0);
        assert!(s.refresh().iter().any(|u| matches!(u, MeshUpdate::Positions { id, .. } if id.starts_with("vrm:"))));
        // Pose wirkt und Modell bleibt aufrecht
        s.load_clip(&repo.join("content/testcandidate/animations/UAL1/Idle_Loop.vrma")).unwrap();
        s.play.playing = false; s.play.t = 0.3;
        let up = s.refresh();
        let top = up.iter().filter_map(|u| match u { MeshUpdate::Positions { data, .. } => Some(data.chunks_exact(6).map(|v| v[1]).fold(f32::MIN, f32::max)), _ => None }).fold(f32::MIN, f32::max);
        let h = s.body_height();
        assert!((top - h).abs() < 0.3 * h, "aufrecht: {top} vs {h}");
        s.close_vrm();
        let up = s.refresh();
        assert!(up.iter().any(|u| matches!(u, MeshUpdate::Remove(id) if id.starts_with("vrm:"))));
        assert!(up.iter().any(|u| matches!(u, MeshUpdate::Create(d) if d.id == "body")), "MakeHuman-Körper kehrt zurück");
    }
}

#[cfg(test)]
mod dress_tests {
    use super::*;

    #[test]
    fn dress_imported_vrm_pose_export() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        s.refresh();
        s.default_assets_blocking(&["clothes/toigo_basic_tucked_t-shirt"]);
        s.open_vrm(&repo.join("content/testcandidate/avatar_b_makehuman.vrm")).unwrap();
        let up = s.refresh();
        assert!(up.iter().any(|u| matches!(u, MeshUpdate::Create(d) if d.id.starts_with("proxy:clothes/"))), "Kleidung wird auf dem geöffneten Modell angezogen");
        assert!(s.vrm.as_ref().unwrap().dress_info.starts_with("Anprobe"));
        s.load_clip(&repo.join("content/testcandidate/animations/UAL1/Idle_Loop.vrma")).unwrap();
        s.play.playing = false; s.play.t = 0.3;
        assert!(s.refresh().iter().any(|u| matches!(u, MeshUpdate::Positions { id, .. } if id.starts_with("proxy:clothes/"))), "Kleidung folgt der Pose");
        let o = ExportOptions { with_outfit: true, expressions: true, name: "T".into(), author: "A".into(), license_url: "https://vrm.dev/licenses/1.0/".into(), commercial: "personalNonProfit".into(), redistribution: true };
        let (doc, _) = s.build_export(&o).unwrap();
        assert!(doc.meshes.iter().any(|m| m.name.starts_with("Clothes_")));
        let glb = hase_core::export::write_glb(&doc).unwrap();
        hase_core::export::verify_glb(&glb).unwrap();
    }
}
