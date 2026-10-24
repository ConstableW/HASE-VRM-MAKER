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

const RENDER_VERSION: &str = "t5";

pub struct ThumbJob { pub key: String, pub state: CharacterState, pub cam: Camera, pub w: u32, pub h: u32, pub body: BodyPackage }
pub struct ThumbDone { pub key: String, pub w: u32, pub h: u32, pub rgba: Vec<u8> }

pub struct ThumbService {
    tx: Sender<ThumbJob>,
    rx: Receiver<ThumbDone>,
    pending: HashSet<String>,
    cache: HashMap<String, egui::TextureHandle>,
    dir: PathBuf,
    pub rendered: usize,
}

pub fn make_key(state: &CharacterState, body_digest: &str, cam: &Camera, w: u32, h: u32, params: &hase_core::params::Catalog) -> String {
    let mut j = state.to_json(params, &state.body_ref.id);
    j["style"] = state.style.to_json();
    let s = format!("{RENDER_VERSION}|{body_digest}|{j}|{:.3},{:.3},{:.3},{:?}|{w}x{h}", cam.yaw, cam.dist, cam.pitch, cam.target);
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
        // Körperbasis passend halten
        if sess.packages.body.id != job.body.id { if sess.activate_body(&job.body).is_err() { continue; } }
        sess.set_state_silent(job.state.clone());
        sess.ensure_assets_blocking();
        let updates = sess.refresh();
        let aspect = job.w as f32 / job.h as f32;
        if let Ok(rgba) = off.render(updates, job.cam.globals_lit(aspect, &LIGHTS[0], false), job.w, job.h) {
            if let Some(img) = image::RgbaImage::from_raw(job.w, job.h, rgba.clone()) { let _ = img.save(dir.join(format!("{}.png", job.key))); }
            let _ = tx.send(ThumbDone { key: job.key, w: job.w, h: job.h, rgba });
            ctx.request_repaint();
        }
    }
    let _ = LoadPlan::Invalid;
}
