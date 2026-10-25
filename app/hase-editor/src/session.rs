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
pub struct VrmRuntime { pub model: hase_core::vrm_model::VrmModel, rig: hase_core::pose::PoseRig, tris: Vec<Vec<[u32; 3]>>, floor: f32, pub expr: std::collections::BTreeMap<String, f32>, pub raw: std::collections::BTreeMap<(usize, usize), f32>, shown: Vec<bool>, dirty: bool, pub path: PathBuf, pub hidden: std::collections::BTreeSet<usize>, pub skin_parts: std::collections::BTreeSet<usize>, dress: Option<VrmDress>, pub dress_info: String, pub avatar: AvatarState, pub tints: std::collections::BTreeMap<usize, [f32; 3]>, pub base_id: Option<String>, pub base_sha: String }

/// Zustand, der für beide Basisarten gilt (Merkmale, Pinsel).
#[derive(Default)]
pub struct Shared { pub features: std::collections::BTreeMap<String, hase_core::features::FeatureState>, pub generated: hase_core::features::Generated, pub gen_rev: u64, pub feat_rev_seen: u64, pub feat_shown: Vec<String>, pub feat_dirty: bool, pub brush: hase_core::sculpt::Brush }

pub struct MhStroke { center: [f32; 3], verts: Vec<(u32, f32, f32)>, before: Vec<(u32, [f32; 3])> }

pub struct AvatarState { pub prep: hase_core::avatar::Prep, pub vals: Vec<f32>, orig: Vec<Vec<[f32; 3]>>, orig_bones: Vec<[f32; 3]>, changed: bool, pub eye_linked: bool, dress_due: Option<Instant>, pub sculpt: hase_core::sculpt::Layer, deformed: Vec<Vec<[f32; 3]>>, stroke: Option<Stroke>, sculpt_touched: Vec<(usize, u32)>, sculpt_changed: bool, hist: Vec<AvEdit>, redo: Vec<AvEdit>, last_edit: Option<(usize, Instant)>, adj: Vec<Option<Vec<Vec<u32>>>> }

pub enum AvEdit { Params { before: Vec<f32>, after: Vec<f32> }, Stroke { changes: Vec<(usize, u32, [f32; 3], [f32; 3])> } }

pub struct Stroke { center: [f32; 3], verts: Vec<(usize, u32, f32, f32)>, before: Vec<(usize, u32, [f32; 3])> }

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
    pub sh: Shared,
    pub mh_sculpt: std::collections::BTreeMap<u32, [f32; 3]>,
    /// Spezieskörper-Ebene (eingebackene Grundform + gewichtete Variationen), liegt unter der individuellen Formung.
    pub species_layer: std::collections::BTreeMap<u32, [f32; 3]>,
    /// Werte der Spezies-Variationen des aktuellen Charakters (Variation-ID → Wert).
    pub var_vals: std::collections::BTreeMap<String, f32>,
    /// Stil-/Proportionsregler (Landmarken-Verformung) auf der MakeHuman-Basis
    pub style_vals: Vec<f32>,
    style_layer: std::collections::BTreeMap<u32, [f32; 3]>,
    sty_dirty: bool,
    mh_stroke: Option<MhStroke>,
    mh_hist: Vec<(usize, Vec<(u32, [f32; 3], [f32; 3])>)>,
    mh_redo: Vec<(usize, Vec<(u32, [f32; 3], [f32; 3])>)>,
    mh_adj: Option<Vec<Vec<u32>>>,
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

pub(crate) fn default_tint(cat: &Category, name: &str) -> [f32; 3] {
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
        let mut s = Session { repo: repo.to_path_buf(), packages, body, params, assets, eval, state, history: History::new(), perf: Perf::default(), symmetric: true, proxies: HashMap::new(), textures: HashMap::new(), pending: vec![], desired: vec![0.0; n], body_nrm: vec![], body_created: false, floor_off: 0.0, skin_tex: None, dirty: true, assets_dirty: false, style_dirty: true, skin_dirty: true, play: Playback::default(), rig: None, pose_rig: None, proxy_skin: HashMap::new(), pose_active: false, resend_body: false, push_grid: None, body_uniq: vec![], vrm: None, sh: Default::default(), mh_sculpt: Default::default(), species_layer: Default::default(), var_vals: Default::default(), style_vals: hase_core::avatar::defaults(), style_layer: Default::default(), sty_dirty: false, mh_stroke: None, mh_hist: vec![], mh_redo: vec![], mh_adj: None, masks: None, generic: false, eyes_loaded: String::new(), hide_body: false, reference_body: reference, markers: vec![] };
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
    /// Alle Spezies, deren Körperbasis aktiv oder aktivierbar (nicht Entwurf) ist.
    pub fn species_selectable(&self) -> Vec<usize> { self.packages.species.iter().enumerate().filter(|(_, s)| s.body_id.is_empty() || s.body_id == self.packages.body.id || self.packages.bodies.iter().any(|b| b.id == s.body_id && b.status != "draft")).map(|(i, _)| i).collect() }
    /// Wählt eine Spezies und wechselt dabei bei Bedarf auf ihre Körperbasis (Spezies ↔ Körperbasis).
    pub fn select_species(&mut self, idx: usize) -> Result<(), String> {
        let sp = self.packages.species.get(idx).cloned().ok_or("keine Spezies")?;
        if !sp.body_id.is_empty() && sp.body_id != self.packages.body.id {
            let b = self.packages.bodies.iter().find(|b| b.id == sp.body_id && b.status != "draft").cloned().ok_or_else(|| format!("Körperbasis {} nicht verfügbar", sp.body_id))?;
            self.activate_body(&b)?;
        }
        let i = self.packages.species.iter().position(|x| x.id == sp.id).unwrap_or(idx);
        self.apply_species(i);
        Ok(())
    }
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

    /// Spiegelt eine Änderung auf die Gegenseite? Augenparameter folgen der Verknüpfung im Augenstudio, alle anderen der Symmetrie-Option.
    pub fn mirrors(&self, i: usize) -> bool { if self.params.defs[i].id.contains("-eye-") { self.state.style.eyes.linked } else { self.symmetric } }

    pub fn set_param(&mut self, idx: usize, v: f32) {
        let before = self.state.clone();
        let v = self.clamp_to_species(idx, v);
        if (self.state.values[idx] - v).abs() < 1e-7 { return; }
        self.state.values[idx] = v;
        if self.mirrors(idx) { if let Some(m) = self.params.defs[idx].mirror { self.state.values[m] = v; } }
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
        if self.vrm.as_ref().map(|v| v.base_id.is_some()).unwrap_or(false) { self.reset_avatar(); return; }
        let before = self.state.clone();
        self.sh.features.clear(); self.sh.feat_dirty = true; self.mh_reset_sculpt();
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
        self.var_vals.clear();
        self.refresh_species_layer();
        self.apply_species_features(&sp);
        let (tc, ta) = sp.shape.skin_tone.unwrap_or(([1.0; 3], 0.0));
        if self.state.style.skin.tone != tc || self.state.style.skin.tone_amt != ta { self.state.style.skin.tone = tc; self.state.style.skin.tone_amt = ta; self.dirty = true; self.skin_dirty = true; self.style_dirty = true; }
    }
    /// Standardmerkmale der Spezies einschalten, nicht erlaubte ausschalten.
    pub fn apply_species_features(&mut self, sp: &hase_core::packages::Species) {
        for (kind, pi) in sp.default_features() { if sp.shape.allows(kind) && !self.sh.features.get(kind).map(|f| f.enabled).unwrap_or(false) { let f = self.feature_mut(kind).unwrap(); f.enabled = true; f.vals = hase_core::features::presets(kind)[pi.min(hase_core::features::presets(kind).len() - 1)].1.to_vec(); } }
        for (k, _) in hase_core::features::KINDS.iter() { if !sp.shape.allows(k) && self.sh.features.get(*k).map(|f| f.enabled).unwrap_or(false) { self.feature_mut(k).unwrap().enabled = false; } }
    }
    /// Ist das Merkmal für die gewählte Spezies zulässig?
    /// Nur für Vorschaubilder: Spezieskörper-Ebene und Standardmerkmale der Spezies im aktuellen Zustand anwenden.
    pub fn species_visuals(&mut self) {
        self.var_vals.clear();
        self.refresh_species_layer();
        if let Some(sp) = self.species_opt().map(|i| self.packages.species[i].clone()) { self.apply_species_features(&sp); }
    }
    pub fn feature_allowed(&self, kind: &str) -> bool { self.species_opt().map(|i| self.packages.species[i].shape.allows(kind)).unwrap_or(true) }

    /// Spezieskörper-Ebene neu berechnen (eingebackene Grundform + Variationen · Wert) und Differenz auf die Körperpositionen anwenden.
    pub fn refresh_species_layer(&mut self) {
        let mut new: std::collections::BTreeMap<u32, [f32; 3]> = Default::default();
        if self.vrm.is_none() {
            if let Some(sp) = self.species_opt().map(|i| &self.packages.species[i]) {
                for (v, d) in &sp.shape.baked { new.insert(*v, *d); }
                for var in &sp.shape.variations {
                    let w = self.var_vals.get(&var.id).copied().unwrap_or(0.0).clamp(var.min, var.max);
                    if w.abs() < 1e-6 { continue; }
                    for (v, d) in &var.delta { let e = new.entry(*v).or_insert([0.0; 3]); for c in 0..3 { e[c] += d[c] * w; } }
                }
            }
        }
        let n = self.eval.pos.len();
        let keys: std::collections::BTreeSet<u32> = self.species_layer.keys().chain(new.keys()).copied().filter(|v| (*v as usize) < n).collect();
        let mut any = false;
        for v in keys {
            let (o, nw) = (self.species_layer.get(&v).copied().unwrap_or([0.0; 3]), new.get(&v).copied().unwrap_or([0.0; 3]));
            if (0..3).all(|c| (o[c] - nw[c]).abs() < 1e-9) { continue; }
            let q = &mut self.eval.pos[v as usize]; for c in 0..3 { q[c] += nw[c] - o[c]; }
            any = true;
        }
        self.species_layer = new;
        if any { self.assets_dirty = true; self.sh.feat_dirty = true; self.resend_body = true; self.dirty = true; }
    }

    pub fn set_variation(&mut self, id: &str, v: f32) { self.var_vals.insert(id.into(), v); self.refresh_species_layer(); }

    /// Aktuelle Formung (Pinsel) fest in den Spezieskörper einbacken: Grundgeometrie der Spezies, kein Regler.
    pub fn bake_into_species(&mut self, idx: usize) -> Result<usize, String> {
        if self.vrm.is_some() { return Err("Nur für die parametrische Körperbasis".into()); }
        let sp = self.packages.species.get_mut(idx).ok_or("keine Spezies")?;
        if !sp.user { return Err("Mitgelieferte Spezies sind schreibgeschützt – zuerst duplizieren".into()); }
        let n = self.mh_sculpt.len(); if n == 0 { return Err("Keine Formung vorhanden".into()); }
        for (v, d) in &self.mh_sculpt { let e = sp.shape.baked.entry(*v).or_insert([0.0; 3]); for c in 0..3 { e[c] += d[c]; } }
        self.finish_shape_move(idx);
        Ok(n)
    }

    /// Aktuelle Formung als benannte Variation (Regler mit Bereich) der Spezies definieren.
    pub fn define_variation(&mut self, idx: usize, label: &str, min: f32, max: f32, split_lr: bool) -> Result<usize, String> {
        if self.vrm.is_some() { return Err("Nur für die parametrische Körperbasis".into()); }
        let sp = self.packages.species.get_mut(idx).ok_or("keine Spezies")?;
        if !sp.user { return Err("Mitgelieferte Spezies sind schreibgeschützt – zuerst duplizieren".into()); }
        let n = self.mh_sculpt.len(); if n == 0 { return Err("Keine Formung vorhanden".into()); }
        let id = format!("var-{}", hase_core::packages::slug(label)); if id == "var-" { return Err("Name fehlt".into()); }
        if sp.shape.variations.iter().any(|x| x.id == id) { return Err(format!("Variation „{label}“ existiert bereits")); }
        if split_lr {
            // getrennte Regler links/rechts: Aufteilung nach Seite der Vertexposition (+x = linke Körperseite)
            let (mut l, mut r) = (std::collections::BTreeMap::new(), std::collections::BTreeMap::new());
            for (v, d) in &self.mh_sculpt { if self.eval.pos[*v as usize][0] >= 0.0 { l.insert(*v, *d); } else { r.insert(*v, *d); } }
            sp.shape.variations.push(hase_core::packages::Variation { id: format!("{id}-l"), label: format!("{label} links"), min, max, delta: l });
            sp.shape.variations.push(hase_core::packages::Variation { id: format!("{id}-r"), label: format!("{label} rechts"), min, max, delta: r });
        } else {
            sp.shape.variations.push(hase_core::packages::Variation { id, label: label.into(), min, max, delta: self.mh_sculpt.clone() });
        }
        self.finish_shape_move(idx);
        Ok(n)
    }

    #[cfg(test)]
    fn finish_shape_move_for_test(&mut self, idx: usize) { self.finish_shape_move(idx); }
    /// Spezieskörper für eine aus dem Charakter abgeleitete Spezies: Basis-Spezies-Geometrie + individuelle Formung (eingebacken), Variationen und Merkmalsfreigaben der Basis-Spezies.
    pub fn derived_shape(&self) -> hase_core::packages::SpeciesShape {
        let mut sh = self.species_opt().map(|i| self.packages.species[i].shape.clone()).unwrap_or_default();
        if self.vrm.is_none() { for (v, d) in &self.mh_sculpt { let e = sh.baked.entry(*v).or_insert([0.0; 3]); for c in 0..3 { e[c] += d[c]; } } }
        if self.state.style.skin.tone_amt > 0.0 { sh.skin_tone = Some((self.state.style.skin.tone, self.state.style.skin.tone_amt)); }
        sh
    }
    /// Nach dem Anlegen einer abgeleiteten Spezies: individuelle Formung steckt nun im Spezieskörper.
    pub fn adopt_new_species(&mut self) {
        let layer = std::mem::take(&mut self.mh_sculpt);
        for (v, d) in &layer { let q = &mut self.eval.pos[*v as usize]; for c in 0..3 { q[c] -= d[c]; } }
        self.mh_hist.clear(); self.mh_redo.clear(); self.mh_stroke = None;
        let old = std::mem::take(&mut self.species_layer);
        for (v, d) in &old { let q = &mut self.eval.pos[*v as usize]; for c in 0..3 { q[c] -= d[c]; } }
        self.refresh_species_layer();
        self.resend_body = true; self.dirty = true; self.assets_dirty = true; self.sh.feat_dirty = true;
    }
    fn finish_shape_move(&mut self, idx: usize) {
        // Formung wandert in die Spezies; der sichtbare Körper bleibt (Baked) bzw. wird bei Wert 0 der Variation neutral
        let layer = std::mem::take(&mut self.mh_sculpt);
        for (v, d) in &layer { let q = &mut self.eval.pos[*v as usize]; for c in 0..3 { q[c] -= d[c]; } }
        self.mh_hist.clear(); self.mh_redo.clear(); self.mh_stroke = None;
        let old = std::mem::take(&mut self.species_layer);
        for (v, d) in &old { let q = &mut self.eval.pos[*v as usize]; for c in 0..3 { q[c] -= d[c]; } }
        self.refresh_species_layer();
        if let Some(sp) = self.packages.species.get(idx) { if let Some(d) = sp.write(&self.repo, &self.packages.body).ok() { let dg = hase_core::packages::package_digest(&d); self.packages.species[idx].digest = dg.clone(); self.state.species_ref.digest = dg; } }
        self.resend_body = true; self.dirty = true; self.assets_dirty = true; self.sh.feat_dirty = true;
    }

    pub fn apply_map(&mut self, m: &std::collections::BTreeMap<String, f32>) {
        let before = self.state.clone();
        for (k, v) in m { if let Some(i) = self.params.canonical(k) { let v = self.clamp_to_species(i, *v); self.state.values[i] = v; if self.mirrors(i) { if let Some(mi) = self.params.defs[i].mirror { self.state.values[mi] = v; } } } }
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
    pub fn undo(&mut self) {
        if self.vrm.is_some() { self.avatar_undo(); return; }
        if self.mh_hist.last().map(|e| e.0 == self.history.undo_len()).unwrap_or(false) { let e = self.mh_hist.pop().unwrap(); self.mh_apply(&e.1, true); self.mh_redo.push(e); return; }
        if let Some(s) = self.history.undo(&self.state) { self.restore(s); }
    }
    pub fn redo(&mut self) {
        if self.vrm.is_some() { self.avatar_redo(); return; }
        if self.mh_redo.last().map(|e| e.0 == self.history.undo_len()).unwrap_or(false) { let e = self.mh_redo.pop().unwrap(); self.mh_apply(&e.1, false); self.mh_hist.push(e); return; }
        if let Some(s) = self.history.redo(&self.state) { self.restore(s); }
    }
    /// Sofortige Auswertung der Parameter (nur für Arbeits-/Worker-Kontexte).
    fn apply_eval_now(&mut self) {
        self.params.target_weights(&self.state.values, self.body.morphs.names.len(), &mut self.desired);
        if let Some(st) = self.eval.apply(&self.body, &self.desired) { if st.full { for (v, d) in self.species_layer.iter().chain(self.mh_sculpt.iter()) { let q = &mut self.eval.pos[*v as usize]; q[0] += d[0]; q[1] += d[1]; q[2] += d[2]; } } }
    }
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
        let mut body_moved = false;
        if self.dirty || self.assets_dirty || need_body {
            let t0 = Instant::now();
            let mut morph_changed = need_body || std::mem::take(&mut self.resend_body);
            if self.dirty {
                self.params.target_weights(&self.state.values, self.body.morphs.names.len(), &mut self.desired);
                if let Some(st) = self.eval.apply(&self.body, &self.desired) { morph_changed = true; if st.full { for (v, d) in self.species_layer.iter().chain(self.mh_sculpt.iter()) { let q = &mut self.eval.pos[*v as usize]; q[0] += d[0]; q[1] += d[1]; q[2] += d[2]; } } self.perf = Perf { morph_ms: st.ms, changed: st.changed_targets, active: st.active_targets, full: st.full, ..self.perf }; }
            }
            if self.sty_dirty || (morph_changed && !self.style_layer.is_empty()) { self.recompute_style(); morph_changed = true; }
            body_moved = morph_changed;
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
                let style_vals = self.style_vals.clone();
                for id in ids {
                    let tint = self.state.assets.iter().find(|a| a.id == id).map(|a| a.tint).unwrap_or(if id == EYES_ID { [1.0; 3] } else { [0.9; 3] });
                    let rt = self.proxies.get_mut(&id).unwrap();
                    rt.proxy.fit(&self.eval.pos, &mut rt.fit);
                    if id == EYES_ID { style_eyes(&style_vals, &mut rt.fit); }
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
        if body_moved || self.sh.feat_dirty { self.mh_regen_features(); }
        self.mh_emit_features();
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
        if let Some(vr) = &self.vrm {
            let cl: Vec<&ProxyRt> = self.proxies.iter().filter(|(k, p)| k.starts_with("clothes/") && p.created).map(|(_, p)| p).collect();
            let v = vr.model.parts.iter().enumerate().filter(|(i, _)| !vr.hidden.contains(i)).map(|(_, p)| p.pos.len()).sum::<usize>() + cl.iter().map(|p| p.proxy.topo.vmap.len()).sum::<usize>();
            let t = vr.model.parts.iter().enumerate().filter(|(i, _)| !vr.hidden.contains(i)).map(|(_, p)| p.idx.len() / 3).sum::<usize>() + cl.iter().map(|p| p.proxy.topo.indices.len() / 3).sum::<usize>();
            return (v, t);
        }
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
        let (mut bones, vert_w) = load_rig(&self.packages.body.data_root, &self.body.obj, &self.eval.pos, floor)?;
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
        // Merkmale
        for b in &self.sh.generated.bones { bones.push(ExBone { name: b.name.clone(), parent: b.parent, pos: b.pos }); }
        for g in &self.sh.generated.parts {
            materials.push(ExMaterial { name: format!("Feature_{}", g.name), color: [g.color[0], g.color[1], g.color[2], 1.0], image: None, mask: false });
            meshes.push(ExMesh { name: g.name.clone(), pos: g.pos.clone(), nrm: g.nrm.clone(), uv: g.uv.clone(), idx: g.idx.clone(), joints: g.joints.clone(), weights: g.weights.clone(), material: materials.len() - 1, targets: vec![] });
        }
        if !self.sh.generated.parts.is_empty() { report.push(format!("Merkmale: {} Teile, {} Zusatz-Bones, {} Spring-Bone-Ketten", self.sh.generated.parts.len(), self.sh.generated.bones.len(), self.sh.generated.springs.len())); }
        let springs = self.sh.generated.springs.clone();
        Ok((ExDoc { meshes, materials, images, bones, humanoid, expressions, meta, springs }, report))
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
        if self.pose_rig.is_none() {
            let mut bones = self.rig.as_ref().unwrap().bones(&self.eval.pos, floor);
            let map = hase_core::export::humanoid_map();
            let mut names: Vec<Option<String>> = bones.iter().map(|b| map.iter().find(|(_, ue)| ue.eq_ignore_ascii_case(&b.name)).map(|(v, _)| v.clone())).collect();
            for g in &self.sh.generated.bones { bones.push(hase_core::export::ExBone { name: g.name.clone(), parent: g.parent, pos: g.pos }); names.push(None); }
            let n = bones.len();
            self.pose_rig = Some(PoseRig::generic(bones, vec![[0.0, 0.0, 0.0, 1.0]; n], names));
        }
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
        // Merkmale (Ohren/Schweif/Hörner) folgen der Pose
        for (k, g) in self.sh.generated.parts.iter().enumerate() {
            let pos: Vec<[f32; 3]> = g.pos.iter().enumerate().map(|(j, q)| skin_point(&m, g.joints[j], g.weights[j], *q)).collect();
            let tris: Vec<[u32; 3]> = g.idx.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect();
            let mut nn = Vec::new(); normals(&pos, &tris, &mut nn);
            let mut data = Vec::with_capacity(pos.len() * 6);
            for (q, n) in pos.iter().zip(&nn) { data.extend_from_slice(&[q[0], q[1], q[2], n[0], n[1], n[2]]); }
            self.pending.push(MeshUpdate::Positions { id: format!("feat:{k}"), data });
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
        let mut model = hase_core::vrm_model::load(path)?;
        let mut rigless_info = String::new();
        if model.rigless { rigless_info = self.rig_rigless(&mut model)?; }
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
        let avatar = { let model_ref = &model; let prep = hase_core::avatar::prepare(model_ref); AvatarState { prep, vals: hase_core::avatar::defaults(), orig: model_ref.parts.iter().map(|p| p.pos.clone()).collect(), orig_bones: model_ref.bones.iter().map(|b| b.pos).collect(), changed: false, eye_linked: true, dress_due: None, sculpt: vec![Default::default(); model_ref.parts.len()], deformed: model_ref.parts.iter().map(|p| p.pos.clone()).collect(), stroke: None, sculpt_touched: vec![], sculpt_changed: false, hist: vec![], redo: vec![], last_edit: None, adj: vec![None; model_ref.parts.len()] } };
        self.vrm = Some(VrmRuntime { model, rig, tris, floor, expr: Default::default(), raw: Default::default(), shown: vec![], dirty: true, path: path.to_path_buf(), hidden: Default::default(), skin_parts, dress: None, dress_info: String::new(), avatar, tints: Default::default(), base_id: None, base_sha: String::new() });
        let info = if rigless_info.is_empty() { info } else { format!("{info} · {rigless_info}") };
        Ok(info)
    }

    /// Körper ohne Skelett: MakeHuman-Basis auf das Mesh abbilden, Skelett und Gewichte aus der abgebildeten Basis übernehmen.
    fn rig_rigless(&mut self, model: &mut hase_core::vrm_model::VrmModel) -> Result<String, String> {
        use hase_core::export::{humanoid_map, pack_weights, ExBone};
        if self.dirty { self.apply_eval_now(); self.dirty = false; }
        if self.rig.is_none() { self.rig = Some(hase_core::export::RigData::load(&self.packages.body.data_root, &self.body.obj)?); }
        if self.body_uniq.is_empty() { let mut uniq: Vec<u32> = self.body.topo.vmap.clone(); uniq.sort_unstable(); uniq.dedup(); self.body_uniq = uniq; }
        let floor_off = -self.body.topo.vmap.iter().map(|&v| self.eval.pos[v as usize][1]).fold(f32::MAX, f32::min);
        let min_y = model.parts.iter().flat_map(|p| p.pos.iter().map(|v| v[1])).fold(f32::MAX, f32::min);
        let floor = -min_y;
        let (mut sp, mut sn) = (Vec::new(), Vec::new());
        for p in model.parts.iter() { let (lo, hi) = p.pos.iter().fold((f32::MAX, f32::MIN), |a, q| (a.0.min(q[1]), a.1.max(q[1]))); if hi - lo >= 0.5 * model.height { for k in 0..p.pos.len() { sp.push([p.pos[k][0], p.pos[k][1] + floor, p.pos[k][2]]); sn.push(p.nrm[k]); } } }
        if sp.is_empty() { return Err("Kein Körperteil erkennbar (Teil mit mindestens halber Modellhöhe).".into()); }
        let rig = self.rig.as_ref().unwrap();
        let mh_bones = rig.bones(&self.eval.pos, floor_off);
        let mh_pos: Vec<[f32; 3]> = self.eval.pos.iter().map(|p| [p[0], p[1] + floor_off, p[2]]).collect();
        let reg = hase_core::dress::register_rigless(&mh_pos, &self.body_uniq, &self.body.topo.base_tris, &mh_bones, &rig.weights, &sp, &sn)?;
        let fb = rig.bones(&reg.base, 0.0);
        let bones: Vec<ExBone> = fb.iter().map(|b| ExBone { name: b.name.clone(), parent: b.parent, pos: [b.pos[0], b.pos[1] - floor, b.pos[2]] }).collect();
        let mut humanoid = std::collections::BTreeMap::new();
        for (vrm, ue) in humanoid_map() { if let Some(i) = bones.iter().position(|b| b.name.eq_ignore_ascii_case(&ue)) { humanoid.insert(vrm, i); } }
        let pelvis = bones.iter().position(|b| b.name == "pelvis").unwrap_or(0) as u16;
        let grid = hase_core::fit::BodyGrid::build(&reg.base, &self.body_uniq, 0.03);
        for p in model.parts.iter_mut() {
            for k in 0..p.pos.len() {
                let q = [p.pos[k][0], p.pos[k][1] + floor, p.pos[k][2]];
                let w = grid.nearest_where(&reg.base, q, 6, |_| true).map(|(v, _)| rig.weights[v as usize].clone()).unwrap_or_default();
                let (j, x) = pack_weights(&w, pelvis); p.joints[k] = j; p.weights[k] = x;
            }
        }
        let n = bones.len();
        model.vrm_names = vec![None; n]; for (k, &b) in &humanoid { model.vrm_names[b] = Some(k.clone()); }
        model.rest_rot = vec![[0.0, 0.0, 0.0, 1.0]; n];
        model.bones = bones; model.humanoid = humanoid; model.identity_rest = true; model.rigless = false;
        Ok(format!("Skelett aus abgebildeter MakeHuman-Basis abgeleitet (Abweichung {:.1} → {:.1} mm, {:.0} ms)", reg.icp_rms_before * 1000.0, reg.icp_rms_after * 1000.0, reg.ms))
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
        if self.dirty { self.apply_eval_now(); self.dirty = false; }
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

    // ---- Avatar-Erstellung (parametrische Verformung der geöffneten Basis)
    pub fn avatar_set(&mut self, i: usize, val: f32) {
        if self.vrm.is_none() { return self.style_set(i, val); }
        let Some(v) = self.vrm.as_mut() else { return };
        let d = &hase_core::avatar::DEFS[i];
        let val = val.clamp(d.min, d.max);
        let before = v.avatar.vals.clone();
        v.avatar.vals[i] = val;
        if v.avatar.eye_linked { if let Some(m) = hase_core::avatar::mirror(i) { v.avatar.vals[m] = val; } }
        // Undo: Änderungen desselben Reglers innerhalb von 0,7 s zu einem Schritt zusammenfassen
        let now = Instant::now();
        let merge = matches!(v.avatar.last_edit, Some((li, t)) if li == i && now.duration_since(t).as_millis() < 700) && matches!(v.avatar.hist.last(), Some(AvEdit::Params { .. }));
        if merge { if let Some(AvEdit::Params { after, .. }) = v.avatar.hist.last_mut() { *after = v.avatar.vals.clone(); } }
        else if before != v.avatar.vals { v.avatar.hist.push(AvEdit::Params { before, after: v.avatar.vals.clone() }); if v.avatar.hist.len() > 200 { v.avatar.hist.remove(0); } }
        v.avatar.last_edit = Some((i, now)); v.avatar.redo.clear();
        v.avatar.changed = true;
    }
    pub fn avatar_can_undo(&self) -> bool { self.vrm.as_ref().map(|v| !v.avatar.hist.is_empty()).unwrap_or(false) }
    pub fn avatar_can_redo(&self) -> bool { self.vrm.as_ref().map(|v| !v.avatar.redo.is_empty()).unwrap_or(false) }
    pub fn avatar_undo(&mut self) {
        let Some(v) = self.vrm.as_mut() else { return };
        let Some(e) = v.avatar.hist.pop() else { return };
        match &e { AvEdit::Params { before, .. } => v.avatar.vals = before.clone(), AvEdit::Stroke { changes } => { for (pi, vi, old, _) in changes { if old.iter().all(|c| c.abs() < 1e-9) { v.avatar.sculpt[*pi].remove(vi); } else { v.avatar.sculpt[*pi].insert(*vi, *old); } } } }
        v.avatar.redo.push(e); v.avatar.changed = true; v.avatar.last_edit = None;
    }
    pub fn avatar_redo(&mut self) {
        let Some(v) = self.vrm.as_mut() else { return };
        let Some(e) = v.avatar.redo.pop() else { return };
        match &e { AvEdit::Params { after, .. } => v.avatar.vals = after.clone(), AvEdit::Stroke { changes } => { for (pi, vi, _, new) in changes { if new.iter().all(|c| c.abs() < 1e-9) { v.avatar.sculpt[*pi].remove(vi); } else { v.avatar.sculpt[*pi].insert(*vi, *new); } } } }
        v.avatar.hist.push(e); v.avatar.changed = true; v.avatar.last_edit = None;
    }
    pub fn can_undo(&self) -> bool { if self.vrm.is_some() { self.avatar_can_undo() } else { self.history.can_undo() || !self.mh_hist.is_empty() } }
    pub fn can_redo(&self) -> bool { if self.vrm.is_some() { self.avatar_can_redo() } else { self.history.can_redo() || !self.mh_redo.is_empty() } }

    // ---- Direkte Formung
    /// Strahl (Weltraum inkl. Boden) gegen die sichtbaren Teile; liefert den Treffer in Modellkoordinaten.
    pub fn avatar_pick(&self, o: [f32; 3], d: [f32; 3]) -> Option<[f32; 3]> {
        if self.vrm.is_none() { return self.mh_pick(o, d); }
        let v = self.vrm.as_ref()?;
        let o = [o[0], o[1] - v.floor, o[2]];
        let mut best: Option<f32> = None;
        for (i, p) in v.model.parts.iter().enumerate() {
            if v.hidden.contains(&i) { continue; }
            if let Some((t, _)) = hase_core::sculpt::ray_mesh(o, d, &p.pos, &p.idx) { if best.map(|b| t < b).unwrap_or(true) { best = Some(t); } }
        }
        best.map(|t| [o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t])
    }
    pub fn sculpt_floor(&self) -> f32 {
        if self.vrm.is_none() { return self.floor_off; } self.vrm.as_ref().map(|v| v.floor).unwrap_or(0.0) }

    pub fn sculpt_begin(&mut self, hit: [f32; 3]) {
        if self.vrm.is_none() { return self.mh_begin(hit); }
        use hase_core::sculpt::*;
        let Some(v) = self.vrm.as_mut() else { return };
        let b = self.sh.brush;
        let mut verts = Vec::new(); let mut before = Vec::new();
        for (pi, p) in v.model.parts.iter().enumerate() {
            if v.hidden.contains(&pi) { continue; }
            for (vi, q) in p.pos.iter().enumerate() {
                let w1 = fall(len(sub(*q, hit)), b.radius);
                let w2 = if b.symmetric { fall(len(sub(*q, mirror(hit))), b.radius) } else { 0.0 };
                if w1 > 0.0 || w2 > 0.0 { verts.push((pi, vi as u32, w1, w2)); before.push((pi, vi as u32, v.avatar.sculpt[pi].get(&(vi as u32)).copied().unwrap_or([0.0; 3]))); }
            }
        }
        v.avatar.stroke = Some(Stroke { center: hit, verts, before });
    }

    /// Ziehen: Verschiebung `delta` (Modellkoordinaten) gegenüber dem Greifpunkt.
    pub fn sculpt_grab(&mut self, delta: [f32; 3]) {
        if self.vrm.is_none() { return self.mh_grab(delta); }
        use hase_core::sculpt::*;
        let Some(v) = self.vrm.as_mut() else { return };
        let Some(st) = v.avatar.stroke.as_ref() else { return };
        let k = self.sh.brush.strength;
        let total = mul(delta, k);
        let mut touched = Vec::with_capacity(st.verts.len());
        for (idx, (pi, vi, w1, w2)) in st.verts.iter().enumerate() {
            let old = st.before[idx].2;
            let d = if w1 >= w2 { mul(total, *w1) } else { mul(mirror(total), *w2) };
            v.avatar.sculpt[*pi].insert(*vi, add(old, d));
            touched.push((*pi, *vi));
        }
        v.avatar.sculpt_touched = touched; v.avatar.sculpt_changed = true;
    }

    /// Inkrementelle Pinsel (Aufblasen, Skalieren, Glätten); `amount` in Metern (Vorzeichen = Richtung).
    pub fn sculpt_step(&mut self, amount: f32) {
        if self.vrm.is_none() { return self.mh_step(amount); }
        use hase_core::sculpt::*;
        let Some(v) = self.vrm.as_mut() else { return };
        let Some(st) = v.avatar.stroke.as_ref() else { return };
        let b = self.sh.brush;
        let amount = amount * b.strength;
        let mut touched = Vec::with_capacity(st.verts.len());
        if b.kind == BrushKind::Smooth {
            for pi in 0..v.model.parts.len() { if v.avatar.adj[pi].is_none() { v.avatar.adj[pi] = Some(adjacency(v.model.parts[pi].pos.len(), &v.model.parts[pi].idx)); } }
        }
        let mut updates: Vec<(usize, u32, [f32; 3])> = Vec::with_capacity(st.verts.len());
        for (pi, vi, w1, w2) in &st.verts {
            let w = w1.max(*w2);
            let part = &v.model.parts[*pi];
            let p = part.pos[*vi as usize];
            let center = if w1 >= w2 { st.center } else { mirror(st.center) };
            let d = match b.kind {
                BrushKind::Inflate => mul(part.nrm[*vi as usize], amount * w),
                BrushKind::Scale => mul(sub(p, center), amount * 8.0 * w),
                BrushKind::Smooth => { let adj = v.avatar.adj[*pi].as_ref().unwrap(); let nb = &adj[*vi as usize]; if nb.is_empty() { [0.0; 3] } else { let mut m = [0f32; 3]; for &j in nb { m = add(m, part.pos[j as usize]); } m = mul(m, 1.0 / nb.len() as f32); mul(sub(m, p), 0.5 * b.strength.min(1.0) * w) } }
                BrushKind::Grab => [0.0; 3],
            };
            updates.push((*pi, *vi, d));
        }
        for (pi, vi, d) in updates { let e = v.avatar.sculpt[pi].entry(vi).or_insert([0.0; 3]); *e = add(*e, d); touched.push((pi, vi)); }
        v.avatar.sculpt_touched = touched; v.avatar.sculpt_changed = true;
    }

    pub fn sculpt_end(&mut self) {
        if self.vrm.is_none() { return self.mh_end(); }
        let Some(v) = self.vrm.as_mut() else { return };
        let Some(st) = v.avatar.stroke.take() else { return };
        let mut changes = Vec::new();
        for (pi, vi, old) in st.before { let new = v.avatar.sculpt[pi].get(&vi).copied().unwrap_or([0.0; 3]); if (0..3).any(|c| (new[c] - old[c]).abs() > 1e-7) { changes.push((pi, vi, old, new)); } }
        if !changes.is_empty() { v.avatar.hist.push(AvEdit::Stroke { changes }); v.avatar.redo.clear(); if v.avatar.hist.len() > 200 { v.avatar.hist.remove(0); } }
        self.sh.feat_dirty = true; // Merkmale/Anprobe an die neue Form anpassen
        if v.dress.is_some() { v.avatar.dress_due = Some(Instant::now() + std::time::Duration::from_millis(500)); }
    }
    pub fn sculpt_clear(&mut self) {
        if self.vrm.is_none() { return self.mh_clear(); }
        let Some(v) = self.vrm.as_mut() else { return };
        let mut changes = Vec::new();
        for (pi, m) in v.avatar.sculpt.iter().enumerate() { for (vi, d) in m { changes.push((pi, *vi, *d, [0.0; 3])); } }
        if changes.is_empty() { return; }
        for m in v.avatar.sculpt.iter_mut() { m.clear(); }
        v.avatar.hist.push(AvEdit::Stroke { changes }); v.avatar.redo.clear(); v.avatar.changed = true;
    }
    pub fn sculpt_count(&self) -> usize {
        if self.vrm.is_none() { return self.mh_sculpt.len(); } self.vrm.as_ref().map(|v| v.avatar.sculpt.iter().map(|m| m.len()).sum()).unwrap_or(0) }

    pub fn avatar_reset(&mut self, idx: &[usize]) {
        if self.vrm.is_none() { for &i in idx { self.style_vals[i] = hase_core::avatar::DEFS[i].def; } self.sty_dirty = true; return; } if let Some(v) = self.vrm.as_mut() { for &i in idx { v.avatar.vals[i] = hase_core::avatar::DEFS[i].def; } v.avatar.changed = true; } }
    pub fn avatar_pending(&self) -> bool { self.vrm.as_ref().map(|v| v.avatar.changed || v.avatar.dress_due.is_some()).unwrap_or(false) }
    pub fn set_part_tint(&mut self, part: usize, rgb: [f32; 3]) {
        let Some(v) = self.vrm.as_mut() else { return };
        v.tints.insert(part, rgb);
        let p = &v.model.parts[part];
        let shown = v.shown.get(part).copied().unwrap_or(false);
        let m = p.mat_name.to_lowercase();
        let _ = &m;
        let recolor = recolorable(&p.mat_name) && p.image.is_some();
        if recolor {
            let im = p.image.as_ref().unwrap();
            let data = recolor_rgba(&im.rgba, rgb);
            if shown { let img = image_from_rgba(im.w, im.h, data, 2048); self.pending.push(MeshUpdate::Texture { id: format!("vrm:{part}"), img }); self.pending.push(MeshUpdate::Material { id: format!("vrm:{part}"), tint: [p.color[0], p.color[1], p.color[2]], mat: Default::default() }); }
        } else if shown {
            let c = p.color;
            self.pending.push(MeshUpdate::Material { id: format!("vrm:{part}"), tint: [c[0] * rgb[0], c[1] * rgb[1], c[2] * rgb[2]], mat: Default::default() });
        }
    }

    fn avatar_update(&mut self) {
        let Some(v) = self.vrm.as_mut() else { return };
        if let Some(t) = v.avatar.dress_due { if Instant::now() >= t { v.avatar.dress_due = None; v.dress = None; self.assets_dirty = true; } }
        // Schnellpfad: nur Formung (Pinselstrich) → betroffene Vertices neu setzen
        if !v.avatar.changed && !self.sh.feat_dirty && v.avatar.sculpt_changed && !v.avatar.deformed.is_empty() {
            v.avatar.sculpt_changed = false;
            let touched = std::mem::take(&mut v.avatar.sculpt_touched);
            let mut parts_t: std::collections::BTreeSet<usize> = Default::default();
            for (pi, vi) in touched { let d = v.avatar.sculpt[pi].get(&vi).copied().unwrap_or([0.0; 3]); let b = v.avatar.deformed[pi][vi as usize]; v.model.parts[pi].pos[vi as usize] = [b[0] + d[0], b[1] + d[1], b[2] + d[2]]; parts_t.insert(pi); }
            for pi in parts_t { let p = &mut v.model.parts[pi]; normals(&p.pos, &v.tris[pi], &mut p.nrm); }
            v.dirty = true;
            return;
        }
        let deform = v.avatar.changed;
        if !deform && !self.sh.feat_dirty { return; }
        v.avatar.changed = false; self.sh.feat_dirty = false;
        if deform {
            let mut d = hase_core::avatar::deform(&v.model, &v.avatar.prep, &v.avatar.orig, &v.avatar.orig_bones, &v.avatar.vals);
            v.avatar.deformed = d.pos.clone();
            for (pi, m) in v.avatar.sculpt.iter().enumerate() { for (vi, dl) in m { if let Some(q) = d.pos[pi].get_mut(*vi as usize) { *q = [q[0] + dl[0], q[1] + dl[1], q[2] + dl[2]]; } } }
            v.avatar.sculpt_changed = false; v.avatar.sculpt_touched.clear();
            for (i, p) in v.model.parts.iter_mut().enumerate() { p.pos = d.pos[i].clone(); normals(&p.pos, &v.tris[i], &mut p.nrm); }
            for (b, bn) in v.model.bones.iter_mut().enumerate() { bn.pos = d.bones[b]; }
            let (lo, hi) = v.model.parts.iter().flat_map(|p| p.pos.iter().map(|q| q[1])).fold((f32::MAX, f32::MIN), |a, y| (a.0.min(y), a.1.max(y)));
            v.model.height = hi; v.floor = -lo;
        }
        // Merkmale (Ohren, Schweif, Hörner) auf der verformten Basis neu erzeugen
        let any = self.sh.features.values().any(|f| f.enabled);
        self.sh.generated = if any { let lm = hase_core::avatar::prepare(&v.model).lm; hase_core::features::build(&v.model, &lm, &self.sh.features) } else { Default::default() };
        self.sh.gen_rev += 1;
        let mut bones: Vec<hase_core::export::ExBone> = v.model.bones.iter().map(|b| hase_core::export::ExBone { name: b.name.clone(), parent: b.parent, pos: [b.pos[0], b.pos[1] + v.floor, b.pos[2]] }).collect();
        let mut rest = v.model.rest_rot.clone(); let mut names = v.model.vrm_names.clone();
        for b in &self.sh.generated.bones { bones.push(hase_core::export::ExBone { name: b.name.clone(), parent: b.parent, pos: [b.pos[0], b.pos[1] + v.floor, b.pos[2]] }); rest.push([0.0, 0.0, 0.0, 1.0]); names.push(None); }
        v.rig = hase_core::pose::PoseRig::generic(bones, rest, names);
        v.dirty = true;
        if deform && v.dress.is_some() { v.avatar.dress_due = Some(Instant::now() + std::time::Duration::from_millis(400)); }
    }

    // ---- Merkmale
    pub fn feature_mut(&mut self, kind: &str) -> Option<&mut hase_core::features::FeatureState> {
        self.sh.feat_dirty = true;
        Some(self.sh.features.entry(kind.to_string()).or_insert_with(|| hase_core::features::FeatureState::new(kind)))
    }
    pub fn feature(&self, kind: &str) -> hase_core::features::FeatureState { self.sh.features.get(kind).cloned().unwrap_or_else(|| hase_core::features::FeatureState::new(kind)) }

    pub fn set_skin_part(&mut self, part: usize, on: bool) { if let Some(v) = self.vrm.as_mut() { if on { v.skin_parts.insert(part); } else { v.skin_parts.remove(&part); } v.dress = None; self.assets_dirty = true; } }
    pub fn set_part_hidden(&mut self, part: usize, hide: bool) { if let Some(v) = self.vrm.as_mut() { if hide { v.hidden.insert(part); } else { v.hidden.remove(&part); } v.dirty = true; } }

    fn refresh_vrm(&mut self) -> Vec<MeshUpdate> {
        use hase_core::pose::skin_point;
        self.avatar_update();
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
                let tint_i = v.tints.get(&i).copied().unwrap_or([1.0; 3]);
                let rec = recolorable(&p.mat_name);
                let tex = p.image.as_ref().map(|im| image_from_rgba(im.w, im.h, if rec { recolor_rgba(&im.rgba, tint_i) } else { im.rgba.clone() }, 2048));
                out.push(MeshUpdate::Create(MeshDesc { id: format!("vrm:{i}"), indices: p.idx.clone(), uv: p.uv.clone(), data, texture: tex, tint: { let t = if rec { [1.0; 3] } else { tint_i }; [p.color[0] * t[0], p.color[1] * t[1], p.color[2] * t[2]] }, mask: p.mask, cull_back: !p.double_sided, mat: Default::default() }));
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
        // Merkmale: Meshes neu anlegen, wenn neu erzeugt; bei Posen mit Bone-Matrizen skinnen
        let recreated = self.sh.gen_rev != self.sh.feat_rev_seen;
        if recreated {
            for id in self.sh.feat_shown.drain(..) { out.push(MeshUpdate::Remove(id)); }
            self.sh.feat_rev_seen = self.sh.gen_rev;
        }
        if recreated || posing {
            for (k, g) in self.sh.generated.parts.iter().enumerate() {
                let id = format!("feat:{k}");
                let mut pos: Vec<[f32; 3]> = g.pos.iter().map(|q| [q[0], q[1] + floor, q[2]]).collect();
                if let Some(m) = &mats { for (j, q) in pos.iter_mut().enumerate() { *q = skin_point(m, g.joints[j], g.weights[j], *q); } }
                let tris: Vec<[u32; 3]> = g.idx.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect();
                let mut nrm = Vec::new(); normals(&pos, &tris, &mut nrm);
                let mut data = Vec::with_capacity(pos.len() * 6);
                for (q, n) in pos.iter().zip(&nrm) { data.extend_from_slice(&[q[0], q[1], q[2], n[0], n[1], n[2]]); }
                if recreated { out.push(MeshUpdate::Create(MeshDesc { id: id.clone(), indices: g.idx.clone(), uv: g.uv.clone(), data, texture: None, tint: g.color, mask: false, cull_back: false, mat: Default::default() })); self.sh.feat_shown.push(id); }
                else { out.push(MeshUpdate::Positions { id, data }); }
            }
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
                    if std::env::var("HASE_DBG").is_ok() { let (mut inside, mut tot, mut worst) = (0, 0, 0f32); for p in &rt.fit { if let Some((q, d2)) = d.grid.nearest(&d.skin_pos, *p) { if d2 < 0.05 * 0.05 { tot += 1; let n = d.skin_nrm[q as usize]; let sg = (p[0] - d.skin_pos[q as usize][0]) * n[0] + (p[1] - d.skin_pos[q as usize][1]) * n[1] + (p[2] - d.skin_pos[q as usize][2]) * n[2]; if sg < 0.0 { inside += 1; worst = worst.min(sg); } } } } eprintln!("DBGCLOTH {id}: {inside}/{tot} Vertices unter Hautnähe innen, schlimmster {:.1} mm", worst * 1000.0); }
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
        let mut bones: Vec<ExBone> = m.bones.iter().map(|b| ExBone { name: b.name.clone(), parent: b.parent, pos: b.pos }).collect();
        for b in &self.sh.generated.bones { bones.push(ExBone { name: b.name.clone(), parent: b.parent, pos: b.pos }); }
        for g in &self.sh.generated.parts {
            materials.push(ExMaterial { name: format!("Feature_{}", g.name), color: [g.color[0], g.color[1], g.color[2], 1.0], image: None, mask: false });
            meshes.push(ExMesh { name: g.name.clone(), pos: g.pos.clone(), nrm: g.nrm.clone(), uv: g.uv.clone(), idx: g.idx.clone(), joints: g.joints.clone(), weights: g.weights.clone(), material: materials.len() - 1, targets: vec![] });
        }
        if !self.sh.generated.parts.is_empty() { report.push(format!("Merkmale: {} Teile, {} Zusatz-Bones, {} Spring-Bone-Ketten", self.sh.generated.parts.len(), self.sh.generated.bones.len(), self.sh.generated.springs.len())); }
        let springs = self.sh.generated.springs.clone();
        Ok((ExDoc { meshes, materials, images, bones, humanoid: m.humanoid.clone(), expressions, meta, springs }, report))
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

#[cfg(test)]
mod dump_params {
    use super::*;
    #[test]
    fn dump_param_ids() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let s = Session::load(&repo).unwrap();
        let mut out = String::new();
        for d in &s.params.defs { out += &format!("{}|{}|{}|{}\n", d.id, d.group, d.min, d.max); }
        std::fs::write("/tmp/param_ids.txt", out).unwrap();
    }
}

/// Umfärben über die Helligkeitszeichnung der Textur: neue Farbe = Zielfarbe × (Luma / mittlere Luma). Weiß = Original.
pub fn recolor_rgba(rgba: &[u8], rgb: [f32; 3]) -> Vec<u8> {
    if rgb.iter().all(|c| (*c - 1.0).abs() < 1e-3) { return rgba.to_vec(); }
    let luma = |px: &[u8]| 0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32;
    let (mut sum, mut n) = (0f64, 0f64);
    for px in rgba.chunks_exact(4) { if px[3] > 128 { sum += luma(px) as f64; n += 1.0; } }
    let mean = (sum / n.max(1.0)).max(8.0) as f32;
    let mut out = rgba.to_vec();
    for px in out.chunks_exact_mut(4) { let k = luma(px) / mean; for c in 0..3 { px[c] = (rgb[c] * 255.0 * k).clamp(0.0, 255.0) as u8; } }
    out
}
pub fn recolorable(mat: &str) -> bool { let m = mat.to_lowercase(); m.contains("hair") || m.contains("eyeiris") }

impl Session {
    pub fn avatar_json(&self) -> Option<serde_json::Value> {
        let v = self.vrm.as_ref()?; let id = v.base_id.as_ref()?;
        let tints: serde_json::Map<String, serde_json::Value> = v.tints.iter().map(|(k, c)| (k.to_string(), serde_json::json!(c))).collect();
        let assets: Vec<serde_json::Value> = self.state.assets.iter().filter(|a| a.id.starts_with("clothes/")).map(|a| serde_json::json!({"id": a.id, "tint": a.tint})).collect();
        Some(serde_json::json!({"formatVersion": "0.1-avatar", "kind": "avatar", "species": {"id": ""}, "body": {"id": format!("avatar:{id}")}, "base": {"id": id, "sha": v.base_sha},
            "params": hase_core::avatar::to_map(&v.avatar.vals), "sculpt": hase_core::sculpt::to_json(&v.avatar.sculpt), "eyeLinked": v.avatar.eye_linked, "tints": tints, "hidden": v.hidden.iter().collect::<Vec<_>>(), "clothes": assets,
            "features": self.sh.features.iter().map(|(k, f)| (k.clone(), serde_json::json!({"enabled": f.enabled, "vals": f.vals, "color": f.color, "color2": f.color2}))).collect::<serde_json::Map<String, serde_json::Value>>()}))
    }

    /// Öffnet eine registrierte Basis als neuen Avatar (nackt, Standardwerte).
    pub fn open_avatar_base(&mut self, base: &crate::anime_bases::AnimeBase) -> Result<String, String> {
        let ids: Vec<String> = self.state.assets.iter().filter(|a| a.id.starts_with("clothes/")).map(|a| a.id.clone()).collect();
        for id in ids { self.remove_asset(&id); }
        let info = self.open_vrm(&base.vrm())?;
        if let Some(v) = self.vrm.as_mut() { v.base_id = Some(base.id.clone()); v.base_sha = base.sha.clone(); }
        Ok(info)
    }

    pub fn apply_avatar_json(&mut self, j: &serde_json::Value) {
        let Some(v) = self.vrm.as_mut() else { return };
        let params: std::collections::BTreeMap<String, f32> = j["params"].as_object().map(|o| o.iter().filter_map(|(k, x)| Some((k.clone(), x.as_f64()? as f32))).collect()).unwrap_or_default();
        v.avatar.vals = hase_core::avatar::from_map(&params);
        v.avatar.eye_linked = j["eyeLinked"].as_bool().unwrap_or(true);
        v.avatar.changed = true;
        v.tints.clear();
        for (k, c) in j["tints"].as_object().into_iter().flatten() { if let (Ok(i), Some(a)) = (k.parse::<usize>(), c.as_array()) { if i < v.model.parts.len() && a.len() == 3 { v.tints.insert(i, [a[0].as_f64().unwrap_or(1.0) as f32, a[1].as_f64().unwrap_or(1.0) as f32, a[2].as_f64().unwrap_or(1.0) as f32]); } } }
        v.avatar.sculpt = hase_core::sculpt::from_json(&j["sculpt"], v.model.parts.len());
        v.avatar.hist.clear(); v.avatar.redo.clear();
        self.sh.features.clear();
        for (k, f) in j["features"].as_object().into_iter().flatten() {
            let mut st = hase_core::features::FeatureState::new(k);
            st.enabled = f["enabled"].as_bool().unwrap_or(false);
            if let Some(a) = f["vals"].as_array() { let vals: Vec<f32> = a.iter().filter_map(|x| x.as_f64().map(|y| y as f32)).collect(); if vals.len() == st.vals.len() { st.vals = vals; } }
            for (key, dst) in [("color", &mut st.color), ("color2", &mut st.color2)] { if let Some(a) = f[key].as_array() { if a.len() == 3 { *dst = [a[0].as_f64().unwrap_or(0.5) as f32, a[1].as_f64().unwrap_or(0.5) as f32, a[2].as_f64().unwrap_or(0.5) as f32]; } } }
            self.sh.features.insert(k.clone(), st);
        }
        self.sh.feat_dirty = true;
        v.hidden = j["hidden"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|i| i as usize)).collect()).unwrap_or_default();
        let clothes: Vec<(String, [f32; 3])> = j["clothes"].as_array().map(|a| a.iter().filter_map(|c| { let t = c["tint"].as_array()?; Some((c["id"].as_str()?.to_string(), [t.first()?.as_f64()? as f32, t.get(1)?.as_f64()? as f32, t.get(2)?.as_f64()? as f32])) }).collect()).unwrap_or_default();
        let old: Vec<String> = self.state.assets.iter().filter(|a| a.id.starts_with("clothes/")).map(|a| a.id.clone()).collect();
        for id in old { self.remove_asset(&id); }
        for (id, tint) in clothes { self.state.assets.push(hase_core::state::AssetSel { id, tint }); }
        self.assets_dirty = true;
    }

    /// Zurück auf Standardwerte (nackt, ohne Farben).
    pub fn reset_avatar(&mut self) {
        let ids: Vec<String> = self.state.assets.iter().filter(|a| a.id.starts_with("clothes/")).map(|a| a.id.clone()).collect();
        for id in ids { self.remove_asset(&id); }
        if let Some(v) = self.vrm.as_mut() { v.avatar.vals = hase_core::avatar::defaults(); v.avatar.changed = true; for m in v.avatar.sculpt.iter_mut() { m.clear(); } v.avatar.hist.clear(); v.avatar.redo.clear(); self.sh.features.clear(); self.sh.feat_dirty = true; v.tints.clear(); v.hidden.clear(); v.shown.iter_mut().for_each(|x| *x = false); for i in 0..v.model.parts.len() { self.pending.push(MeshUpdate::Remove(format!("vrm:{i}"))); } v.dirty = true; }
    }
}

#[cfg(test)]
mod avatar_tests {
    use super::*;

    #[test]
    fn avatar_json_roundtrip_and_deform() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        s.refresh();
        s.open_vrm(&repo.join("content/testcandidate/avatar_b_makehuman.vrm")).unwrap();
        if let Some(v) = s.vrm.as_mut() { v.base_id = Some("testbase".into()); v.base_sha = "x".into(); }
        let (ih, ie) = (hase_core::avatar::index_of("height").unwrap(), hase_core::avatar::index_of("eye-size-l").unwrap());
        s.avatar_set(ih, 0.5); s.avatar_set(ie, 0.8);
        s.set_part_tint(0, [0.2, 0.4, 0.9]);
        let h0 = s.body_height();
        s.refresh();
        let h1 = s.body_height();
        assert!(h1 > h0 * 1.05, "Körpergröße wirkt: {h0} -> {h1}");
        let j = s.avatar_json().unwrap();
        assert_eq!(j["kind"], "avatar");
        let mut s2 = Session::load(&repo).unwrap();
        s2.refresh();
        s2.open_vrm(&repo.join("content/testcandidate/avatar_b_makehuman.vrm")).unwrap();
        s2.apply_avatar_json(&j);
        s2.refresh();
        let v2 = s2.vrm.as_ref().unwrap();
        assert!((v2.avatar.vals[ih] - 0.5).abs() < 1e-6 && (v2.avatar.vals[ie] - 0.8).abs() < 1e-6);
        assert!((s2.body_height() - h1).abs() < 1e-4);
        assert_eq!(v2.tints.get(&0), Some(&[0.2, 0.4, 0.9]));
        // Verknüpfte Augen spiegeln; getrennte nicht
        let ir = hase_core::avatar::index_of("eye-size-r").unwrap();
        assert!((v2.avatar.vals[ir] - 0.8).abs() < 1e-6, "verknüpft: rechtes Auge folgt");
        s2.vrm.as_mut().unwrap().avatar.eye_linked = false;
        s2.avatar_set(ie, -0.3);
        assert!((s2.vrm.as_ref().unwrap().avatar.vals[ir] - 0.8).abs() < 1e-6, "getrennt: rechtes Auge bleibt");
    }
}

#[cfg(test)]
mod sculpt_tests {
    use super::*;

    #[test]
    fn sculpt_grab_undo_json() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        s.refresh();
        s.open_vrm(&repo.join("content/testcandidate/avatar_b_makehuman.vrm")).unwrap();
        if let Some(v) = s.vrm.as_mut() { v.base_id = Some("t".into()); v.base_sha = "x".into(); }
        s.refresh();
        // Treffer in der Körpermitte von vorn
        let h = s.body_height();
        let hit = s.avatar_pick([0.0, h * 0.55, 3.0], [0.0, 0.0, -1.0]).expect("Treffer");
        let before = s.vrm.as_ref().unwrap().model.parts.iter().map(|p| p.pos.clone()).collect::<Vec<_>>();
        s.sh.brush.radius = 0.1;
        s.sculpt_begin(hit);
        s.sculpt_grab([0.0, 0.0, 0.05]);
        s.refresh();
        let moved = s.vrm.as_ref().unwrap().model.parts.iter().enumerate().flat_map(|(i, p)| p.pos.iter().enumerate().map(move |(k, q)| (i, k, *q))).filter(|(i, k, q)| (q[2] - before[*i][*k][2]).abs() > 1e-4).count();
        assert!(moved > 10, "Pinsel verschiebt Vertices: {moved}");
        s.sculpt_end();
        assert!(s.sculpt_count() >= moved);
        let j = s.avatar_json().unwrap();
        assert!(!j["sculpt"].as_array().unwrap().iter().all(|a| a.as_array().unwrap().is_empty()));
        // Undo stellt den Ausgangszustand her
        s.undo(); s.refresh();
        let after = s.vrm.as_ref().unwrap().model.parts.iter().map(|p| p.pos.clone()).collect::<Vec<_>>();
        let diff = after.iter().zip(&before).flat_map(|(a, b)| a.iter().zip(b)).filter(|(a, b)| (a[2] - b[2]).abs() > 1e-5).count();
        assert_eq!(diff, 0, "Undo");
        s.redo(); s.refresh();
        assert!(s.sculpt_count() >= moved);
        // Laden in neuer Sitzung
        let mut s2 = Session::load(&repo).unwrap();
        s2.refresh();
        s2.open_vrm(&repo.join("content/testcandidate/avatar_b_makehuman.vrm")).unwrap();
        s2.apply_avatar_json(&j); s2.refresh();
        assert_eq!(s2.sculpt_count(), s.sculpt_count());
    }
}

// ---------------------------------------------------------------------------------------------------------
// MakeHuman-Basis: direkte Formung und Merkmale (gleiche Werkzeuge wie auf der Avatar-Basis)
// ---------------------------------------------------------------------------------------------------------
impl Session {
    fn mh_unique(&mut self) {
        if self.body_uniq.is_empty() { let mut u: Vec<u32> = self.body.topo.vmap.clone(); u.sort_unstable(); u.dedup(); self.body_uniq = u; }
    }
    pub(crate) fn mh_pick(&self, o: [f32; 3], d: [f32; 3]) -> Option<[f32; 3]> {
        let idx: Vec<u32> = self.body.topo.base_tris.iter().flat_map(|t| [t[0], t[1], t[2]]).collect();
        let o = [o[0], o[1] - self.floor_off, o[2]];
        let (t, _) = hase_core::sculpt::ray_mesh(o, d, &self.eval.pos, &idx)?;
        Some([o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t])
    }
    fn mh_set(&mut self, v: u32, new: [f32; 3]) {
        let old = self.mh_sculpt.get(&v).copied().unwrap_or([0.0; 3]);
        let q = &mut self.eval.pos[v as usize];
        for c in 0..3 { q[c] += new[c] - old[c]; }
        if new.iter().all(|c| c.abs() < 1e-9) { self.mh_sculpt.remove(&v); } else { self.mh_sculpt.insert(v, new); }
    }
    pub(crate) fn mh_begin(&mut self, hit: [f32; 3]) {
        use hase_core::sculpt::*;
        self.mh_unique();
        if self.body_nrm.len() != self.eval.pos.len() { normals(&self.eval.pos, &self.body.topo.base_tris, &mut self.body_nrm); }
        let b = self.sh.brush;
        let (mut verts, mut before) = (Vec::new(), Vec::new());
        for &v in &self.body_uniq {
            let q = self.eval.pos[v as usize];
            let w1 = fall(len(sub(q, hit)), b.radius);
            let w2 = if b.symmetric { fall(len(sub(q, mirror(hit))), b.radius) } else { 0.0 };
            if w1 > 0.0 || w2 > 0.0 { verts.push((v, w1, w2)); before.push((v, self.mh_sculpt.get(&v).copied().unwrap_or([0.0; 3]))); }
        }
        self.mh_stroke = Some(MhStroke { center: hit, verts, before });
    }
    pub(crate) fn mh_grab(&mut self, delta: [f32; 3]) {
        use hase_core::sculpt::*;
        let Some(st) = self.mh_stroke.as_ref() else { return };
        let total = mul(delta, self.sh.brush.strength);
        let ups: Vec<(u32, [f32; 3])> = st.verts.iter().zip(&st.before).map(|((v, w1, w2), (_, old))| { let d = if w1 >= w2 { mul(total, *w1) } else { mul(mirror(total), *w2) }; (*v, add(*old, d)) }).collect();
        for (v, n) in ups { self.mh_set(v, n); }
        self.resend_body = true; self.dirty = true;
    }
    pub(crate) fn mh_step(&mut self, amount: f32) {
        use hase_core::sculpt::*;
        let Some(st) = self.mh_stroke.as_ref() else { return };
        let b = self.sh.brush;
        let amount = amount * b.strength;
        if b.kind == BrushKind::Smooth && self.mh_adj.is_none() { self.mh_adj = Some(adjacency(self.eval.pos.len(), &self.body.topo.base_tris.iter().flat_map(|t| [t[0], t[1], t[2]]).collect::<Vec<u32>>())); }
        let mut ups: Vec<(u32, [f32; 3])> = Vec::with_capacity(st.verts.len());
        for (v, w1, w2) in &st.verts {
            let w = w1.max(*w2);
            let p = self.eval.pos[*v as usize];
            let center = if w1 >= w2 { st.center } else { mirror(st.center) };
            let d = match b.kind {
                BrushKind::Inflate => mul(self.body_nrm[*v as usize], amount * w),
                BrushKind::Scale => mul(sub(p, center), amount * 8.0 * w),
                BrushKind::Smooth => { let nb = &self.mh_adj.as_ref().unwrap()[*v as usize]; if nb.is_empty() { [0.0; 3] } else { let mut m = [0f32; 3]; for &j in nb { m = add(m, self.eval.pos[j as usize]); } m = mul(m, 1.0 / nb.len() as f32); mul(sub(m, p), 0.5 * b.strength.min(1.0) * w) } }
                BrushKind::Grab => [0.0; 3],
            };
            ups.push((*v, add(self.mh_sculpt.get(v).copied().unwrap_or([0.0; 3]), d)));
        }
        for (v, n) in ups { self.mh_set(v, n); }
        self.resend_body = true; self.dirty = true;
    }
    pub(crate) fn mh_end(&mut self) {
        let Some(st) = self.mh_stroke.take() else { return };
        let mut changes = Vec::new();
        for (v, old) in st.before { let new = self.mh_sculpt.get(&v).copied().unwrap_or([0.0; 3]); if (0..3).any(|c| (new[c] - old[c]).abs() > 1e-7) { changes.push((v, old, new)); } }
        if !changes.is_empty() { self.mh_hist.push((self.history.undo_len(), changes)); self.mh_redo.clear(); if self.mh_hist.len() > 200 { self.mh_hist.remove(0); } }
        self.assets_dirty = true; self.sh.feat_dirty = true; self.resend_body = true; self.dirty = true;
    }
    pub(crate) fn mh_clear(&mut self) {
        let changes: Vec<(u32, [f32; 3], [f32; 3])> = self.mh_sculpt.iter().map(|(v, d)| (*v, *d, [0.0; 3])).collect();
        if changes.is_empty() { return; }
        for (v, _, _) in &changes { self.mh_set(*v, [0.0; 3]); }
        self.mh_hist.push((self.history.undo_len(), changes)); self.mh_redo.clear();
        self.assets_dirty = true; self.sh.feat_dirty = true; self.resend_body = true; self.dirty = true;
    }
    fn mh_apply(&mut self, changes: &[(u32, [f32; 3], [f32; 3])], undo: bool) {
        for (v, old, new) in changes { self.mh_set(*v, if undo { *old } else { *new }); }
        self.assets_dirty = true; self.sh.feat_dirty = true; self.resend_body = true; self.dirty = true;
    }
    /// Entfernt die Formung vollständig (neuer Charakter, Basiswechsel).
    pub(crate) fn mh_reset_sculpt(&mut self) {
        let keys: Vec<u32> = self.mh_sculpt.keys().copied().collect();
        for v in keys { self.mh_set(v, [0.0; 3]); }
        self.mh_hist.clear(); self.mh_redo.clear(); self.mh_stroke = None;
        self.resend_body = true; self.dirty = true; self.assets_dirty = true; self.sh.feat_dirty = true;
    }

    // ---- Merkmale
    /// Momentaufnahme der MakeHuman-Basis als Basismodell (Boden bei y = 0), damit dieselben Merkmals-Generatoren wie beim Avatar laufen.
    fn mh_host_model(&mut self) -> Option<(hase_core::vrm_model::VrmModel, Option<([f32; 3], [f32; 3])>)> {
        use hase_core::vrm_model::{VrmModel, VrmPart};
        if self.rig.is_none() { self.rig = hase_core::export::RigData::load(&self.packages.body.data_root, &self.body.obj).ok(); }
        let rig = self.rig.as_ref()?;
        let fo = -self.body.topo.vmap.iter().map(|&v| self.eval.pos[v as usize][1]).fold(f32::MAX, f32::min);
        let bones = rig.bones(&self.eval.pos, fo);
        let map = hase_core::export::humanoid_map();
        let mut humanoid = std::collections::BTreeMap::new();
        let mut vrm_names: Vec<Option<String>> = vec![None; bones.len()];
        for (vn, ue) in &map { if let Some(i) = bones.iter().position(|b| b.name.eq_ignore_ascii_case(ue)) { humanoid.insert(vn.clone(), i); vrm_names[i] = Some(vn.clone()); } }
        let mut pos: Vec<[f32; 3]> = self.eval.pos.iter().map(|p| [p[0], p[1] + fo, p[2]]).collect();
        let used: std::collections::BTreeSet<u32> = self.body.topo.vmap.iter().copied().collect();
        // nur Hautvertices behalten (Hilfsgeometrie weit wegschieben würde Landmarken stören)
        let keep: Vec<[f32; 3]> = used.iter().map(|&v| pos[v as usize]).collect();
        pos = keep;
        let height = pos.iter().map(|p| p[1]).fold(0.0, f32::max);
        let n = pos.len();
        let part = VrmPart { name: "Body".into(), pos, nrm: vec![[0.0, 1.0, 0.0]; n], uv: vec![[0.0, 0.0]; n], idx: vec![], joints: vec![[0; 4]; n], weights: vec![[1.0, 0.0, 0.0, 0.0]; n], image: None, color: [1.0; 4], mask: false, double_sided: false, mat_name: "Body".into(), targets: vec![] };
        let nb = bones.len();
        let eyes = self.proxies.get(EYES_ID).filter(|rt| !rt.fit.is_empty()).and_then(|rt| {
            let (mut l, mut r) = (([0f32; 3], 0f32), ([0f32; 3], 0f32));
            for p in &rt.fit { let t = if p[0] > 0.0 { &mut l } else { &mut r }; for c in 0..3 { t.0[c] += p[c]; } t.1 += 1.0; }
            if l.1 > 0.0 && r.1 > 0.0 { Some(([l.0[0] / l.1, l.0[1] / l.1 + fo, l.0[2] / l.1], [r.0[0] / r.1, r.0[1] / r.1 + fo, r.0[2] / r.1])) } else { None }
        });
        Some((VrmModel { name: "mh".into(), parts: vec![part], bones, humanoid, expressions: vec![], spring_bones: false, rigless: false, identity_rest: true, height, rest_rot: vec![[0.0, 0.0, 0.0, 1.0]; nb], vrm_names }, eyes))
    }

    fn mh_regen_features(&mut self) {
        self.sh.feat_dirty = false;
        let any = self.sh.features.values().any(|f| f.enabled);
        if !any { if !self.sh.generated.parts.is_empty() { self.sh.generated = Default::default(); self.sh.gen_rev += 1; self.pose_rig = None; } return; }
        let Some((m, eyes)) = self.mh_host_model() else { return };
        let prep = hase_core::avatar::prepare_with(&m, eyes);
        self.sh.generated = hase_core::features::build(&m, &prep.lm, &self.sh.features);
        self.sh.gen_rev += 1;
        self.pose_rig = None;
    }

    fn mh_emit_features(&mut self) {
        if self.sh.gen_rev == self.sh.feat_rev_seen { return; }
        for id in self.sh.feat_shown.drain(..) { self.pending.push(MeshUpdate::Remove(id)); }
        self.sh.feat_rev_seen = self.sh.gen_rev;
        for (k, g) in self.sh.generated.parts.iter().enumerate() {
            let id = format!("feat:{k}");
            let mut data = Vec::with_capacity(g.pos.len() * 6);
            for (q, n) in g.pos.iter().zip(&g.nrm) { data.extend_from_slice(&[q[0], q[1], q[2], n[0], n[1], n[2]]); }
            self.pending.push(MeshUpdate::Create(MeshDesc { id: id.clone(), indices: g.idx.clone(), uv: g.uv.clone(), data, texture: None, tint: g.color, mask: false, cull_back: false, mat: Default::default() }));
            self.sh.feat_shown.push(id);
        }
    }

    /// Gemeinsames JSON (Merkmale; bei MakeHuman auch die Formung) – wird an das Charakter-JSON angehängt.
    pub fn shared_json(&self) -> serde_json::Value {
        let feats: serde_json::Map<String, serde_json::Value> = self.sh.features.iter().map(|(k, f)| (k.clone(), serde_json::json!({"enabled": f.enabled, "vals": f.vals, "color": f.color, "color2": f.color2}))).collect();
        let sculpt: Vec<serde_json::Value> = self.mh_sculpt.iter().flat_map(|(v, d)| [serde_json::json!(v), serde_json::json!((d[0] * 1e5).round() / 1e5), serde_json::json!((d[1] * 1e5).round() / 1e5), serde_json::json!((d[2] * 1e5).round() / 1e5)]).collect();
        serde_json::json!({"features": feats, "mhSculpt": sculpt, "variations": self.var_vals, "style": hase_core::avatar::to_map(&self.style_vals)})
    }
    pub fn apply_shared_json(&mut self, j: &serde_json::Value) {
        self.sh.features.clear();
        for (k, f) in j["features"].as_object().into_iter().flatten() {
            let mut st = hase_core::features::FeatureState::new(k);
            st.enabled = f["enabled"].as_bool().unwrap_or(false);
            if let Some(a) = f["vals"].as_array() { let vals: Vec<f32> = a.iter().filter_map(|x| x.as_f64().map(|y| y as f32)).collect(); if vals.len() == st.vals.len() { st.vals = vals; } }
            for (key, dst) in [("color", &mut st.color), ("color2", &mut st.color2)] { if let Some(a) = f[key].as_array() { if a.len() == 3 { *dst = [a[0].as_f64().unwrap_or(0.5) as f32, a[1].as_f64().unwrap_or(0.5) as f32, a[2].as_f64().unwrap_or(0.5) as f32]; } } }
            self.sh.features.insert(k.clone(), st);
        }
        self.sh.feat_dirty = true;
        if self.vrm.is_none() {
            let sm: std::collections::BTreeMap<String, f32> = j["style"].as_object().map(|o| o.iter().filter_map(|(k, x)| Some((k.clone(), x.as_f64()? as f32))).collect()).unwrap_or_default();
            self.style_vals = hase_core::avatar::from_map(&sm); self.sty_dirty = true;
            self.mh_reset_sculpt();
            self.var_vals = j["variations"].as_object().map(|o| o.iter().filter_map(|(k, x)| Some((k.clone(), x.as_f64()? as f32))).collect()).unwrap_or_default();
            self.refresh_species_layer();
            let nums: Vec<f64> = j["mhSculpt"].as_array().into_iter().flatten().filter_map(|x| x.as_f64()).collect();
            for c in nums.chunks_exact(4) { let v = c[0] as u32; if (v as usize) < self.eval.pos.len() { self.mh_set(v, [c[1] as f32, c[2] as f32, c[3] as f32]); } }
            self.resend_body = true; self.dirty = true; self.assets_dirty = true;
        }
    }
}

#[cfg(test)]
mod mh_tool_tests {
    use super::*;

    /// Erzeugt die prozeduralen Spezieskörper (Schnauze) der Beispiel-Spezies. Aufruf: `cargo test --release generate_species_shapes -- --ignored`.
    #[test]
    #[ignore]
    fn generate_species_shapes() {
        use hase_core::packages::{SpeciesShape, Variation};
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        s.refresh();
        let h = s.body_height();
        let top = s.body.topo.vmap.iter().map(|&v| s.eval.pos[v as usize][1]).fold(f32::MIN, f32::max);
        let head: Vec<u32> = s.body.topo.vmap.iter().copied().filter(|&v| s.eval.pos[v as usize][1] > top - h * 0.135).collect();
        let nose = head.iter().map(|&v| s.eval.pos[v as usize]).filter(|p| p[0].abs() < 0.012).fold([0.0f32, 0.0, -9.0], |a, p| if p[2] > a[2] { p } else { a });
        let muzzle = |amount: f32, sigma: f32, drop: f32| -> std::collections::BTreeMap<u32, [f32; 3]> {
            let c = [0.0, nose[1] - drop, nose[2] - 0.012];
            head.iter().filter_map(|&v| { let p = s.eval.pos[v as usize]; if p[2] < c[2] - 0.05 || p[1] < nose[1] - 0.04 { return None; } let d2 = (p[0] - c[0]).powi(2) + 1.6 * (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2); let w = (-d2 / (2.0 * sigma * sigma)).exp(); if w < 0.02 { None } else { Some((v, [0.0, -0.1 * amount * w, amount * w])) } }).collect()
        };
        // Elf: schmaleres, spitzeres Kinn (Unterkiefer verjüngt, leicht verlängert)
        let jaw = |amount: f32| -> std::collections::BTreeMap<u32, [f32; 3]> {
            head.iter().filter_map(|&v| { let p = s.eval.pos[v as usize]; let t = ((nose[1] - 0.025 - p[1]) / 0.03).clamp(0.0, 1.0) * ((nose[1] - 0.13 - p[1]) / -0.03).clamp(0.0, 1.0); if t < 0.01 || p[2] < nose[2] - 0.16 { None } else { Some((v, [-p[0] * 0.14 * amount * t, -0.008 * amount * t, 0.0])) } }).collect()
        };
        let write = |id: &str, shape: SpeciesShape| { let d = repo.join("examples/species/packages").join(id); std::fs::write(d.join("shape.json"), shape.to_json().to_string()).unwrap(); };
        write("org.hase.species.dog-demi", SpeciesShape { baked: muzzle(0.022, 0.035, 0.014), variations: vec![Variation { id: "var-schnauzenlaenge".into(), label: "Schnauzenlänge".into(), min: -0.6, max: 1.6, delta: muzzle(0.018, 0.035, 0.014) }], features: None, skin_tone: None });
        write("org.hase.species.elf", SpeciesShape { baked: jaw(1.0), variations: vec![Variation { id: "var-kinnspitze".into(), label: "Kinnspitze".into(), min: -1.0, max: 1.5, delta: jaw(1.0) }], features: None, skin_tone: Some(([0.9, 0.9, 1.0], 0.35)) });
        // Orkin: breiter, schwerer Unterkiefer und kräftige Brauenwülste; Zwerg: große, breite Nase und kantigeres Gesicht
        let heavy = |amount: f32| -> std::collections::BTreeMap<u32, [f32; 3]> {
            let mut m = std::collections::BTreeMap::new();
            for &v in &head { let p = s.eval.pos[v as usize];
                let t = ((nose[1] - 0.02 - p[1]) / 0.03).clamp(0.0, 1.0) * ((nose[1] - 0.14 - p[1]) / -0.03).clamp(0.0, 1.0);
                let b = (-((p[1] - (nose[1] + 0.045)).powi(2)) / (2.0 * 0.012f32.powi(2))).exp() * if p[2] > nose[2] - 0.07 { 1.0 } else { 0.0 };
                let d = [p[0] * 0.16 * amount * t, 0.0, 0.006 * amount * t + 0.012 * amount * b];
                if d.iter().any(|c| c.abs() > 1e-5) { m.insert(v, d); } }
            m
        };
        let nosey = |amount: f32| -> std::collections::BTreeMap<u32, [f32; 3]> {
            let c = [0.0, nose[1] - 0.005, nose[2] - 0.01];
            head.iter().filter_map(|&v| { let p = s.eval.pos[v as usize]; if p[2] < c[2] - 0.05 { return None; } let d2 = (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2); let w = (-d2 / (2.0 * 0.022f32.powi(2))).exp(); if w < 0.02 { None } else { Some((v, [p[0] * 0.3 * amount * w, -0.002 * amount * w, 0.018 * amount * w])) } }).collect()
        };
        let mkspecies = |id: &str, name: &str, desc: &str, defaults: serde_json::Value, shape: SpeciesShape| {
            let src: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(repo.join("examples/species/packages/org.hase.species.elf/hase-package.json")).unwrap()).unwrap();
            let mut m = src.clone();
            m["id"] = id.into(); m["name"] = serde_json::json!({"de": name}); m["description"] = desc.into(); m["dev"] = serde_json::json!({"status": "specimen"});
            m["requires"] = serde_json::json!([src["requires"][0].clone()]);
            m["species"]["defaults"] = defaults; m["species"]["limits"] = serde_json::json!({});
            m["species"]["presets"] = serde_json::json!([]); m["species"]["traits"] = serde_json::json!({}); m["species"]["components"] = serde_json::json!([]);
            let d = repo.join("examples/species/packages").join(id); std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("hase-package.json"), serde_json::to_string_pretty(&m).unwrap()).unwrap();
            std::fs::write(d.join("shape.json"), shape.to_json().to_string()).unwrap();
        };
        mkspecies("org.hase.species.orc", "Orkin", "Humanoide mit breitem, schwerem Unterkiefer und kräftigen Brauenwülsten (eingebackene Grundform), breite Schultern, kräftige Arme; Hörner erlaubt.", serde_json::json!({"mh:macro-age": 0.5, "mh:macro-gender": 0.8, "mh:torso-scale-horiz-decr-incr": 0.45, "mh:measure-shoulder-dist-decr-incr": 0.5, "mh:upperarm-fat-decr-incr": 0.4, "mh:measure-neck-circ-decr-incr": 0.5}),
            SpeciesShape { baked: heavy(1.0), variations: vec![Variation { id: "var-kiefer".into(), label: "Kieferbreite".into(), min: -0.8, max: 1.5, delta: heavy(0.6) }], features: Some(vec!["horns".into(), "ears".into()]), skin_tone: Some(([0.5, 0.7, 0.55], 0.8)) });
        mkspecies("org.hase.species.dwarf", "Zwerg", "Humanoide mit großer, breiter Nase und kantigerem Gesicht (eingebackene Grundform), kurze Beine, breiter Rumpf.", serde_json::json!({"mh:macro-age": 0.55, "mh:macro-gender": 0.7, "mh:measure-lowerleg-height-decr-incr": -0.75, "mh:measure-upperleg-height-decr-incr": -0.5, "mh:torso-scale-horiz-decr-incr": 0.4, "mh:torso-scale-depth-decr-incr": 0.3, "mh:upperleg-fat-decr-incr": 0.2}),
            SpeciesShape { baked: nosey(1.0), variations: vec![Variation { id: "var-nasengroesse".into(), label: "Nasengröße".into(), min: -0.8, max: 1.6, delta: nosey(0.8) }], features: None, skin_tone: None });
        write("org.hase.species.cat-demi", SpeciesShape { baked: muzzle(0.007, 0.03, 0.012), variations: vec![Variation { id: "var-schnauzenlaenge".into(), label: "Schnauzenlänge".into(), min: -1.0, max: 2.0, delta: muzzle(0.01, 0.03, 0.012) }], features: None, skin_tone: None });
    }

    #[test]
    fn generated_hair() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        s.refresh();
        let f = s.feature_mut("hair").unwrap(); f.enabled = true; f.vals = hase_core::features::presets("hair")[2].1.to_vec();
        let up = s.refresh();
        assert!(up.iter().any(|u| matches!(u, MeshUpdate::Create(d) if d.id.starts_with("feat:"))), "Frisur erscheint");
        let n = s.sh.generated.parts.iter().find(|p| p.name == "Hair").map(|p| p.pos.len()).unwrap();
        assert!(n > 5000 && n < 400_000, "Strähnenzahl plausibel: {n}");
        assert!(s.sh.generated.springs.iter().filter(|(k, _)| k == "hair").count() >= 6, "Haar-Spring-Ketten");
        let hp = s.sh.generated.parts.iter().find(|p| p.name == "Hair").unwrap();
        let nb = (s.body.topo.vmap.len() > 0) as usize; let _ = nb;
        assert!(hp.joints.iter().zip(&hp.weights).all(|(j, w)| (w[0] + w[1] + w[2] + w[3] - 1.0).abs() < 1e-4 && j.iter().all(|&b| (b as usize) < 1000)), "Gewichte normiert");
    }

    #[test]
    fn species_default_features() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        let i = s.packages.species.iter().position(|x| x.id.ends_with("cat-demi")).unwrap();
        s.select_species(i).unwrap();
        assert!(s.sh.features["ears"].enabled && s.sh.features["tail"].enabled && !s.feature_allowed("horns"));
        let h = s.packages.species.iter().position(|x| x.id.ends_with(".human")).unwrap();
        s.select_species(h).unwrap();
        assert!(s.feature_allowed("horns"));
    }

    #[test]
    fn species_bake_and_variation() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        let mut sp = s.packages.species[s.species_opt().unwrap_or(0)].clone();
        sp.id = "user.species.test-shape".into(); sp.label = "TestShape".into(); sp.user = true; sp.dir = None; sp.shape = Default::default();
        s.packages.species.push(sp);
        let idx = s.packages.species.len() - 1;
        s.apply_species(idx); s.refresh();
        let h = s.body_height();
        let hit = s.avatar_pick([0.0, h * 0.62, 3.0], [0.0, 0.0, -1.0]).expect("Treffer");
        s.sh.brush.radius = 0.08;
        let p0 = s.eval.pos.clone();
        s.sculpt_begin(hit); s.sculpt_grab([0.0, 0.0, 0.04]); s.refresh(); s.sculpt_end(); s.refresh();
        let p1 = s.eval.pos.clone();
        // Variation: Formung wandert in die Spezies, Körper ist bei Wert 0 neutral, bei 1 geformt
        s.define_variation(idx, "Bauch", 0.0, 1.5, false).unwrap();
        assert_eq!(s.mh_sculpt.len(), 0);
        assert_eq!(s.eval.pos.iter().zip(&p0).filter(|(a, b)| (a[2] - b[2]).abs() > 1e-5).count(), 0, "Wert 0 = neutral");
        s.set_variation("var-bauch", 1.0);
        assert_eq!(s.eval.pos.iter().zip(&p1).filter(|(a, b)| (a[2] - b[2]).abs() > 1e-5).count(), 0, "Wert 1 = Formung");
        // Links/rechts getrennt
        s.sculpt_begin(hit); s.sculpt_grab([0.0, 0.0, 0.04]); s.refresh(); s.sculpt_end(); s.refresh();
        s.define_variation(idx, "Brust", 0.0, 1.0, true).unwrap();
        let sh = &s.packages.species[idx].shape;
        let (l, r) = (sh.variations.iter().find(|x| x.id == "var-brust-l").unwrap(), sh.variations.iter().find(|x| x.id == "var-brust-r").unwrap());
        assert!(l.delta.len() + r.delta.len() > 10 && l.delta.keys().all(|k| !r.delta.contains_key(k)));
        // Einbacken: fester Teil des Körpers
        s.set_variation("var-bauch", 0.0);
        s.sculpt_begin(hit); s.sculpt_grab([0.0, 0.0, 0.04]); s.refresh(); s.sculpt_end(); s.refresh();
        let p2 = s.eval.pos.clone();
        s.bake_into_species(idx).unwrap();
        assert_eq!(s.eval.pos.iter().zip(&p2).filter(|(a, b)| (a[2] - b[2]).abs() > 1e-5).count(), 0, "Einbacken ändert die Form nicht");
        // Abgeleitete Spezies: individuelle Formung wandert in den Spezieskörper, Körper bleibt gleich
        s.sculpt_begin(hit); s.sculpt_grab([0.0, 0.0, -0.03]); s.refresh(); s.sculpt_end(); s.refresh();
        let p3 = s.eval.pos.clone();
        let mut sp2 = s.packages.species[idx].clone(); sp2.id = "user.species.test-shape2".into(); sp2.dir = None; sp2.shape = s.derived_shape();
        s.packages.species.push(sp2); let i2 = s.packages.species.len() - 1;
        s.state.species = "user.species.test-shape2".into(); s.adopt_new_species();
        assert_eq!(s.mh_sculpt.len(), 0);
        assert_eq!(s.eval.pos.iter().zip(&p3).filter(|(a, b)| (a[2] - b[2]).abs() > 1e-5).count(), 0, "abgeleitete Spezies behält die Form");
        let _ = i2;
        // Erlaubte Merkmale
        s.packages.species[idx].shape.features = Some(vec!["ears".into()]);
        s.feature_mut("tail").unwrap().enabled = true;
        s.apply_species(idx);
        assert!(s.feature_allowed("ears") && !s.feature_allowed("tail") && !s.sh.features["tail"].enabled);
        s.finish_shape_move_for_test(idx);
        // Neu laden: Spezies samt Form kommt aus dem Paket
        let pk = hase_core::packages::Packages::load(&repo).unwrap();
        let back = pk.species.iter().find(|x| x.id == "user.species.test-shape").unwrap();
        let _ = std::fs::remove_dir_all(hase_core::packages::user_dir(&repo).join("user.species.test-shape"));
        assert!(!back.shape.baked.is_empty() && back.shape.variations.len() == 3 && back.shape.features.as_deref() == Some(&["ears".to_string()][..]));
    }

    #[test]
    fn makehuman_features_sculpt_export() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let mut s = Session::load(&repo).unwrap();
        s.default_assets_blocking(&["hair/short02"]);
        s.refresh();
        // Merkmale
        for k in ["ears", "tail", "horns"] { s.feature_mut(k).unwrap().enabled = true; }
        let up = s.refresh();
        assert!(up.iter().filter(|u| matches!(u, MeshUpdate::Create(d) if d.id.starts_with("feat:"))).count() >= 5, "Ohren, Schweif, Hörner erscheinen");
        assert!(!s.sh.generated.bones.is_empty());
        // Formung am Körper + Undo
        let h = s.body_height();
        let hit = s.avatar_pick([0.0, h * 0.62, 3.0], [0.0, 0.0, -1.0]).expect("Treffer");
        s.sh.brush.radius = 0.08;
        let p0 = s.eval.pos.clone();
        s.sculpt_begin(hit); s.sculpt_grab([0.0, 0.0, 0.04]); s.refresh(); s.sculpt_end(); s.refresh();
        let moved = s.eval.pos.iter().zip(&p0).filter(|(a, b)| (a[2] - b[2]).abs() > 1e-4).count();
        assert!(moved > 10 && s.sculpt_count() > 10, "Formung wirkt am MakeHuman-Körper: {moved}");
        // Formung überlebt eine vollständige Neuberechnung der Morphs
        s.state.values[0] = s.state.values[0] + 0.0;
        s.undo(); s.refresh();
        let back = s.eval.pos.iter().zip(&p0).filter(|(a, b)| (a[2] - b[2]).abs() > 1e-5).count();
        assert_eq!(back, 0, "Undo");
        s.redo(); s.refresh();
        // Export enthält die Merkmale
        let o = ExportOptions { with_outfit: false, expressions: false, name: "T".into(), author: "A".into(), license_url: "https://vrm.dev/licenses/1.0/".into(), commercial: "personalNonProfit".into(), redistribution: true };
        let (doc, _) = s.build_export(&o).unwrap();
        assert!(doc.meshes.iter().any(|m| m.name == "Tail") && !doc.springs.is_empty());
        // Weitere Formate: schlichtes glTF ohne VRMC, OBJ mit MTL
        let plain = hase_core::export::write_glb_opt(&doc, false).unwrap();
        assert!(!String::from_utf8_lossy(&plain).contains("VRMC_") && hase_core::export::verify_glb(&plain).is_ok());
        let td = std::env::temp_dir().join("hase-obj-test"); let _ = std::fs::create_dir_all(&td);
        let (tris, files) = hase_core::export::write_obj(&doc, &td.join("t.obj")).unwrap();
        assert!(tris > 1000 && files.iter().any(|f| f.ends_with(".mtl")) && std::fs::read_to_string(td.join("t.obj")).unwrap().contains("\nf "));
        let _ = std::fs::remove_dir_all(&td);
        let glb = hase_core::export::write_glb(&doc).unwrap();
        hase_core::export::verify_glb(&glb).unwrap();
        // Speichern/Laden
        let j = s.shared_json();
        let mut s2 = Session::load(&repo).unwrap();
        s2.refresh();
        s2.apply_shared_json(&j); s2.refresh();
        assert_eq!(s2.sculpt_count(), s.sculpt_count());
        assert!(s2.feature("tail").enabled);
    }
}

// ---------------------------------------------------------------------------------------------------------
// Stil-/Proportionsregler auf der MakeHuman-Basis (Landmarken-Verformung, gleiche Regler wie `avatar.rs`)
// ---------------------------------------------------------------------------------------------------------
impl Session {
    pub fn style_value(&self, i: usize) -> f32 { self.style_vals[i] }
    pub fn style_active(&self) -> bool { self.style_vals.iter().zip(hase_core::avatar::DEFS).any(|(v, d)| (v - d.def).abs() > 1e-5) }
    pub fn style_set(&mut self, i: usize, val: f32) {
        let d = &hase_core::avatar::DEFS[i];
        let val = val.clamp(d.min, d.max);
        self.style_vals[i] = val;
        if self.state.style.eyes.linked { if let Some(m) = hase_core::avatar::mirror(i) { self.style_vals[m] = val; } }
        self.sty_dirty = true;
    }

    /// Berechnet die Verschiebungsebene der Stilregler neu (Reihenfolge: Morphs → Stilregler → Formung).
    fn recompute_style(&mut self) {
        self.sty_dirty = false;
        let n = self.eval.pos.len();
        // Morph-Zustand ohne Stil- und Formungsebene
        let mut pos_m = self.eval.pos.clone();
        for (v, d) in &self.style_layer { let q = &mut pos_m[*v as usize]; for c in 0..3 { q[c] -= d[c]; } }
        for (v, d) in self.species_layer.iter().chain(self.mh_sculpt.iter()) { let q = &mut pos_m[*v as usize]; for c in 0..3 { q[c] -= d[c]; } }
        let mut new_layer: std::collections::BTreeMap<u32, [f32; 3]> = Default::default();
        if self.style_active() {
            if self.rig.is_none() { self.rig = hase_core::export::RigData::load(&self.packages.body.data_root, &self.body.obj).ok(); }
            if let Some(rig) = self.rig.as_ref() {
                let fo = -self.body.topo.vmap.iter().map(|&v| pos_m[v as usize][1]).fold(f32::MAX, f32::min);
                let lifted: Vec<[f32; 3]> = pos_m.iter().map(|p| [p[0], p[1] + fo, p[2]]).collect();
                let bones = rig.bones(&pos_m, fo);
                let map = hase_core::export::humanoid_map();
                let mut humanoid = std::collections::BTreeMap::new();
                let mut vrm_names: Vec<Option<String>> = vec![None; bones.len()];
                for (vn, ue) in &map { if let Some(i) = bones.iter().position(|b| b.name.eq_ignore_ascii_case(ue)) { humanoid.insert(vn.clone(), i); vrm_names[i] = Some(vn.clone()); } }
                let pelvis = bones.iter().position(|b| b.name == "pelvis").unwrap_or(0) as u16;
                let (mut jo, mut we) = (Vec::with_capacity(n), Vec::with_capacity(n));
                for w in rig.weights.iter().take(n) { let (j, x) = hase_core::export::pack_weights(w, pelvis); jo.push(j); we.push(x); }
                let nb = bones.len();
                let height = lifted.iter().map(|p| p[1]).fold(0.0, f32::max);
                let part = hase_core::vrm_model::VrmPart { name: "Body".into(), pos: lifted.clone(), nrm: vec![[0.0, 1.0, 0.0]; n], uv: vec![[0.0, 0.0]; n], idx: vec![], joints: jo, weights: we, image: None, color: [1.0; 4], mask: false, double_sided: false, mat_name: "Body".into(), targets: vec![] };
                let model = hase_core::vrm_model::VrmModel { name: "mh".into(), parts: vec![part], bones: bones.clone(), humanoid, expressions: vec![], spring_bones: false, rigless: false, identity_rest: true, height, rest_rot: vec![[0.0, 0.0, 0.0, 1.0]; nb], vrm_names };
                // Landmarken auf der Hautgeometrie (ohne Hilfsvertices)
                let used: std::collections::BTreeSet<u32> = self.body.topo.vmap.iter().copied().collect();
                let skin: Vec<[f32; 3]> = used.iter().map(|&v| lifted[v as usize]).collect();
                let sn = skin.len();
                let host = hase_core::vrm_model::VrmPart { name: "Body".into(), pos: skin, nrm: vec![[0.0, 1.0, 0.0]; sn], uv: vec![[0.0, 0.0]; sn], idx: vec![], joints: vec![[0; 4]; sn], weights: vec![[1.0, 0.0, 0.0, 0.0]; sn], image: None, color: [1.0; 4], mask: false, double_sided: false, mat_name: "Body".into(), targets: vec![] };
                let mini = hase_core::vrm_model::VrmModel { name: "mh".into(), parts: vec![host], bones: bones.clone(), humanoid: model.humanoid.clone(), expressions: vec![], spring_bones: false, rigless: false, identity_rest: true, height, rest_rot: model.rest_rot.clone(), vrm_names: model.vrm_names.clone() };
                let eyes = self.proxies.get(EYES_ID).filter(|rt| !rt.fit.is_empty()).and_then(|rt| {
                    let (mut l, mut r) = (([0f32; 3], 0f32), ([0f32; 3], 0f32));
                    for p in &rt.fit { let t = if p[0] > 0.0 { &mut l } else { &mut r }; for c in 0..3 { t.0[c] += p[c]; } t.1 += 1.0; }
                    if l.1 > 0.0 && r.1 > 0.0 { Some(([l.0[0] / l.1, l.0[1] / l.1 + fo, l.0[2] / l.1], [r.0[0] / r.1, r.0[1] / r.1 + fo, r.0[2] / r.1])) } else { None }
                });
                let prep = hase_core::avatar::prepare_with(&mini, eyes);
                let ob: Vec<[f32; 3]> = bones.iter().map(|b| b.pos).collect();
                let d = hase_core::avatar::deform(&model, &prep, &[lifted.clone()], &ob, &self.style_vals);
                for (v, q) in d.pos[0].iter().enumerate() {
                    let delta = [q[0] - lifted[v][0], q[1] - lifted[v][1], q[2] - lifted[v][2]];
                    if delta.iter().any(|c| c.abs() > 1e-6) { new_layer.insert(v as u32, delta); }
                }
            }
        }
        // Ergebnis: eval.pos = Morphs + Stilebene + Formung
        let mut out = pos_m;
        for (v, d) in &new_layer { let q = &mut out[*v as usize]; for c in 0..3 { q[c] += d[c]; } }
        for (v, d) in self.species_layer.iter().chain(self.mh_sculpt.iter()) { let q = &mut out[*v as usize]; for c in 0..3 { q[c] += d[c]; } }
        self.eval.pos = out;
        self.style_layer = new_layer;
        self.pose_rig = None; self.push_grid = None;
    }
}

/// Stilregler für Augen wirken auch auf die Augäpfel (Größe, Breite, Winkel um den jeweiligen Augenmittelpunkt; die Lage folgt bereits der Basis).
fn style_eyes(vals: &[f32], fit: &mut [[f32; 3]]) {
    let v = |id: &str| hase_core::avatar::index_of(id).map(|i| vals[i]).unwrap_or(0.0);
    if ["eye-size-l", "eye-size-r", "eye-width-l", "eye-width-r", "eye-tilt-l", "eye-tilt-r"].iter().all(|k| v(k).abs() < 1e-4) { return; }
    let (mut l, mut r) = (([0f32; 3], 0f32), ([0f32; 3], 0f32));
    for p in fit.iter() { let t = if p[0] > 0.0 { &mut l } else { &mut r }; for c in 0..3 { t.0[c] += p[c]; } t.1 += 1.0; }
    if l.1 < 1.0 || r.1 < 1.0 { return; }
    let cl = [l.0[0] / l.1, l.0[1] / l.1, l.0[2] / l.1];
    let cr = [r.0[0] / r.1, r.0[1] / r.1, r.0[2] / r.1];
    for p in fit.iter_mut() {
        let (side, c, sfx) = if p[0] > 0.0 { (1.0f32, cl, "l") } else { (-1.0, cr, "r") };
        let (sz, wd, tl) = (v(&format!("eye-size-{sfx}")), v(&format!("eye-width-{sfx}")), v(&format!("eye-tilt-{sfx}")));
        let (s, sx) = (1.0 + 0.8 * sz, 1.0 + 0.5 * wd);
        let (dx, dy, dz) = ((p[0] - c[0]) * sx * s, (p[1] - c[1]) * s, (p[2] - c[2]) * (1.0 + 0.25 * sz));
        let a = tl * 0.6 * side;
        let (ca, sa) = (a.cos(), a.sin());
        *p = [c[0] + dx * ca - dy * sa, c[1] + dx * sa + dy * ca, c[2] + dz];
    }
}
