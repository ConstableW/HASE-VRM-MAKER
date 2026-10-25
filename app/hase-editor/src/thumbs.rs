//! Thumbnail-Dienst: rendert Vorschaubilder (Presets, Frisuren, Charaktere) asynchron in einem Worker-Thread mit eigenem Offscreen-Renderer
//! und eigener Session. Ergebnisse werden auf Platte (`.hase-cache/thumbs`) und als egui-Texturen gecacht. Der UI-Thread wird nie blockiert.
use crate::camera::{Camera, LIGHTS};
use crate::offscreen::Offscreen;
use crate::session::{LoadPlan, Session};
use eframe::egui;
use hase_core::packages::{sha256_hex, BodyPackage};
use hase_core::state::CharacterState;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

const RENDER_VERSION: &str = "t6";

pub struct ThumbJob { pub key: String, pub state: CharacterState, pub cam_kind: String, pub w: u32, pub h: u32, pub body: BodyPackage, pub avatar: Option<(serde_json::Value, PathBuf, String)>, pub extra: Option<serde_json::Value> }
pub struct ThumbDone { pub key: String, pub w: u32, pub h: u32, pub rgba: Vec<u8> }

pub struct ThumbService {
    tx: Sender<ThumbJob>,
    rx: Receiver<ThumbDone>,
    pending: HashSet<String>,
    cache: HashMap<String, egui::TextureHandle>,
    dir: PathBuf,
    pub rendered: usize,
}

/// Kamera je Ansichtsart relativ zur Körpergröße `hh` des dargestellten Zustands.
pub fn cam_for(kind: &str, hh: f32) -> Camera {
    let k = (hh / 1.75).clamp(0.3, 1.5);
    match kind {
        "eye" => Camera { yaw: 0.0, pitch: 0.0, dist: 0.26 * k.max(0.6), target: [0.0, 0.935 * hh, 0.0], fov: 0.62 },
        "hair" => Camera { yaw: 0.45, pitch: 0.05, dist: 0.75 * k.max(0.6), target: [0.0, 0.915 * hh, 0.0], fov: 0.62 },
        "upper" => Camera { yaw: 0.25, pitch: 0.05, dist: 1.55 * k.max(0.6), target: [0.0, 0.72 * hh, 0.0], fov: 0.62 },
        "lower" => Camera { yaw: 0.25, pitch: 0.05, dist: 1.9 * k.max(0.6), target: [0.0, 0.40 * hh, 0.0], fov: 0.62 },
        "feet" => Camera { yaw: 0.5, pitch: 0.15, dist: 1.0 * k.max(0.6), target: [0.0, 0.07 * hh, 0.0], fov: 0.62 },
        "head" => Camera { yaw: 0.0, pitch: 0.02, dist: 0.62 * k.max(0.6), target: [0.0, 0.91 * hh, 0.0], fov: 0.62 },
        _ => Camera { yaw: 0.3, pitch: 0.05, dist: 3.2 * k.max(0.6), target: [0.0, 0.52 * hh, 0.0], fov: 0.62 },
    }
}

pub fn make_key_avatar(j: &serde_json::Value, base_sha: &str, cam_kind: &str, w: u32, h: u32) -> String { sha256_hex(format!("{RENDER_VERSION}|avatar|{base_sha}|{j}|{cam_kind}|{w}x{h}").as_bytes())[..32].to_string() }

pub fn make_key(state: &CharacterState, body_digest: &str, cam_kind: &str, w: u32, h: u32, params: &hase_core::params::Catalog) -> String {
    let mut j = state.to_json(params, &state.body_ref.id);
    j["style"] = state.style.to_json();
    let s = format!("{RENDER_VERSION}|{body_digest}|{j}|{cam_kind}|{w}x{h}");
    sha256_hex(s.as_bytes())[..32].to_string()
}

impl ThumbService {
    pub fn new(repo: PathBuf, ctx: egui::Context) -> Self {
        let (tx, jrx) = channel::<ThumbJob>();
        let (dtx, rx) = channel::<ThumbDone>();
        let dir = repo.join(".hase-cache/thumbs");
        let _ = std::fs::create_dir_all(&dir);
        let (r2, d2) = (repo.clone(), dir.clone());
        std::thread::spawn(move || worker(r2, d2, jrx, dtx, ctx));
        Self { tx, rx, pending: HashSet::new(), cache: HashMap::new(), dir, rendered: 0 }
    }

    /// Eingegangene Ergebnisse in Texturen umwandeln.
    pub fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(d) = self.rx.try_recv() {
            self.pending.remove(&d.key);
            let img = egui::ColorImage::from_rgba_unmultiplied([d.w as usize, d.h as usize], &d.rgba);
            self.cache.insert(d.key.clone(), ctx.load_texture(&d.key, img, egui::TextureOptions::LINEAR));
            self.rendered += 1;
        }
    }

    /// Fertige Textur oder `None` (dann ist der Auftrag eingereiht bzw. läuft). `job` wird nur bei Bedarf erzeugt.
    pub fn get(&mut self, ctx: &egui::Context, key: &str, job: impl FnOnce() -> ThumbJob) -> Option<egui::TextureHandle> {
        if let Some(t) = self.cache.get(key) { return Some(t.clone()); }
        let f = self.dir.join(format!("{key}.png"));
        if let Ok(img) = image::open(&f) {
            let img = img.to_rgba8();
            let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
            let t = ctx.load_texture(key, ci, egui::TextureOptions::LINEAR);
            self.cache.insert(key.to_string(), t.clone());
            return Some(t);
        }
        if !self.pending.contains(key) && self.pending.len() < 24 { let j = job(); self.pending.insert(key.to_string()); let _ = self.tx.send(j); }
        None
    }
    pub fn busy(&self) -> bool { !self.pending.is_empty() }
}

fn worker(repo: PathBuf, dir: PathBuf, rx: Receiver<ThumbJob>, tx: Sender<ThumbDone>, ctx: egui::Context) {
    let Ok(mut sess) = Session::load(&repo) else { return };
    let Ok(mut off) = Offscreen::new() else { return };
    while let Ok(job) = rx.recv() {
        if let Some((j, path, base_id)) = &job.avatar {
            let same = sess.vrm.as_ref().map(|v| v.base_id.as_deref() == Some(base_id.as_str())).unwrap_or(false);
            if !same { if sess.open_vrm(path).is_err() { continue; } if let Some(v) = sess.vrm.as_mut() { v.base_id = Some(base_id.clone()); } }
            sess.apply_avatar_json(j);
            sess.ensure_assets_blocking();
        } else {
            if sess.vrm.is_some() { sess.close_vrm(); }
            // Körperbasis passend halten
            if sess.packages.body.id != job.body.id { if sess.activate_body(&job.body).is_err() { continue; } }
            sess.set_state_silent(job.state.clone());
            if let Some(x) = &job.extra { sess.apply_shared_json(x); } else { sess.apply_shared_json(&serde_json::json!({})); }
            sess.ensure_assets_blocking();
        }
        let updates = sess.refresh();
        let aspect = job.w as f32 / job.h as f32;
        let cam = cam_for(&job.cam_kind, sess.body_height());
        if let Ok(rgba) = off.render(updates, cam.globals_lit(aspect, &LIGHTS[0], false), job.w, job.h) {
            if let Some(img) = image::RgbaImage::from_raw(job.w, job.h, rgba.clone()) { let _ = img.save(dir.join(format!("{}.png", job.key))); }
            let _ = tx.send(ThumbDone { key: job.key, w: job.w, h: job.h, rgba });
            ctx.request_repaint();
        }
    }
    let _ = LoadPlan::Invalid;
}
