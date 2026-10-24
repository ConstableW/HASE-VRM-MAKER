//! egui-Anwendung: Parameterpanel, Asset-Bibliotheken, 3D-Viewport mit Orbit-Kamera, Profiling-Overlay.
use crate::camera::{Camera, LIGHTS, PRESETS};
use crate::theme;
use crate::regions::{self, Main, CLOTHES_KINDS, REGIONS, TIER_LABELS};
use crate::render::{MeshUpdate, SceneRenderer};
use crate::session::{load_proxy_blocking, LoadedProxy, Session, EYES_ID};
use eframe::egui;
use eframe::egui_wgpu;
use hase_core::assets::Category;
use hase_core::params::Kind;
use hase_core::state::CharacterState;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tool { Orbit, Pan, Zoom, Focus, Region, Decal }

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Workspace { Charakter, Outfit, Finalisieren, Studio }

pub(crate) enum Msg { Session(Result<Box<Session>, String>), Asset { id: String, res: Result<LoadedProxy, String> } }

enum Row { Header(String, usize), Param(usize) }

struct SceneCallback { updates: Mutex<Option<Vec<MeshUpdate>>>, globals: crate::render::Globals }

impl egui_wgpu::CallbackTrait for SceneCallback {
    fn prepare(&self, device: &wgpu::Device, queue: &wgpu::Queue, _s: &egui_wgpu::ScreenDescriptor, _e: &mut wgpu::CommandEncoder, res: &mut egui_wgpu::CallbackResources) -> Vec<wgpu::CommandBuffer> {
        if let Some(r) = res.get_mut::<SceneRenderer>() {
            if let Some(u) = self.updates.lock().unwrap().take() { r.apply_updates(device, queue, u); }
            r.write_globals(queue, self.globals);
        }
        Vec::new()
    }
    fn paint(&self, _i: egui::PaintCallbackInfo, pass: &mut wgpu::RenderPass<'static>, res: &egui_wgpu::CallbackResources) {
        if let Some(r) = res.get::<SceneRenderer>() { r.draw(pass); }
    }
}

pub struct App {
    pub(crate) rx: Receiver<Msg>, pub(crate) tx: Sender<Msg>,
    pub(crate) session: Option<Session>,
    pub(crate) error: Option<String>,
    pub(crate) camera: Camera,
    pub(crate) ws: Workspace,
    pub(crate) main: Main,
    pub(crate) region: usize,
    pub(crate) sub: usize,
    pub(crate) expert: bool,
    pub(crate) saved_cam: Option<Camera>,
    pub(crate) reg_idx: Vec<usize>, pub(crate) reg_key: (usize, usize),
    pub(crate) filter: String, pub(crate) asset_filter: String,
    pub(crate) open_groups: HashSet<String>,
    pub(crate) rows: Vec<Row>, pub(crate) rows_key: (String, usize),
    pub(crate) loading: HashSet<String>,
    pub(crate) show_stats: bool,
    pub(crate) frame_ms: f32, pub(crate) frame_avg: f32,
    pub(crate) started: Instant,
    pub(crate) status: String,
    pub(crate) gpu_name: String,
    pub(crate) studio: crate::studio::StudioState,
    pub(crate) thumbs: crate::thumbs::ThumbService,
    pub(crate) eye_tab: usize,
    pub(crate) eye_side: usize,
    pub(crate) hair_tab: usize,
    pub(crate) lib: crate::library_ui::LibState,
    pub(crate) export: crate::export_ui::ExportState,
    pub(crate) ctx: egui::Context,
    pub(crate) tool: Tool,
    pub(crate) open_on_start: Option<std::path::PathBuf>,
    pub(crate) decal_shape: u8,
    pub(crate) decal_sel: Option<usize>,
    pub(crate) region_energy: Vec<f32>,
    pub(crate) light: usize,
    pub(crate) grid: bool,
    pub(crate) pending_load: Option<(CharacterState, Vec<String>, String)>,
    pub(crate) pending_body: Option<(serde_json::Value, hase_core::packages::BodyPackage, String)>,
}

pub fn spawn_session_load(tx: Sender<Msg>, ctx: egui::Context, repo: std::path::PathBuf) {
    std::thread::spawn(move || {
        let r = Session::load(&repo).map(|mut s| {
            s.default_assets_blocking(&["eyebrows/mindfront_eyebrows_01", "eyelashes/mindfront_eyelashes_01", "hair/short02"]);
            Box::new(s)
        });
        let _ = tx.send(Msg::Session(r));
        ctx.request_repaint();
    });
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, repo: std::path::PathBuf) -> Self {
        let (tx, rx) = channel();
        let mut gpu_name = String::from("?");
        if let Some(rs) = cc.wgpu_render_state.as_ref() {
            let info = rs.adapter.get_info();
            gpu_name = format!("{} ({:?})", info.name, info.backend);
            let r = SceneRenderer::new(&rs.device, &rs.queue, rs.target_format, wgpu::TextureFormat::Depth32Float, 4);
            rs.renderer.write().callback_resources.insert(r);
        }
        crate::theme::apply(&cc.egui_ctx);
        spawn_session_load(tx.clone(), cc.egui_ctx.clone(), repo.clone());
        Self { rx, tx, session: None, error: None, camera: Camera::default(), ws: Workspace::Charakter, main: Main::Charakter, region: usize::MAX, sub: 0, expert: false, saved_cam: None, reg_idx: vec![], reg_key: (usize::MAX, 0), filter: String::new(), asset_filter: String::new(), open_groups: ["Makro".to_string()].into_iter().collect(), rows: vec![], rows_key: (String::new(), usize::MAX), loading: HashSet::new(), show_stats: false, frame_ms: 0.0, frame_avg: 0.0, started: Instant::now(), status: "Lade Daten …".into(), gpu_name, studio: Default::default(), thumbs: crate::thumbs::ThumbService::new(repo.clone(), cc.egui_ctx.clone()), eye_tab: 0, eye_side: 0, hair_tab: 0, lib: Default::default(), export: Default::default(), ctx: cc.egui_ctx.clone(), tool: Tool::Orbit, open_on_start: None, decal_shape: 0, decal_sel: None, region_energy: vec![], light: 0, grid: true, pending_load: None, pending_body: None }
    }

    pub(crate) fn start_asset_load(&mut self, ctx: &egui::Context, id: &str, mhclo: std::path::PathBuf, tex: Option<std::path::PathBuf>) {
        if self.loading.contains(id) { return; }
        self.loading.insert(id.to_string());
        let (tx, ctx2, id2) = (self.tx.clone(), ctx.clone(), id.to_string());
        std::thread::spawn(move || { let res = load_proxy_blocking(&mhclo, tex.as_deref()); let _ = tx.send(Msg::Asset { id: id2, res }); ctx2.request_repaint(); });
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Session(Ok(s)) => {
                    if let Some(p) = self.open_on_start.take() { let mut s2 = *s; let r = s2.open_vrm(&p); self.status = match r { Ok(i) => format!("Modell geöffnet – {i}"), Err(e) => format!("Modell: {e}") }; self.session = Some(s2); continue; } self.status = format!("Bereit – Daten in {:.0} ms geladen{}", s.body.load_ms, if s.body.from_cache { " (Cache)" } else { " (erster Start, Cache wird angelegt)" }); self.session = Some(*s); }
                Msg::Session(Err(e)) => self.error = Some(e),
                Msg::Asset { id, res } => {
                    self.loading.remove(&id);
                    match res { Ok(lp) => { if let Some(s) = self.session.as_mut() { if s.is_selected(&id) || id == EYES_ID { s.insert_proxy(id, lp); } } } Err(e) => self.status = format!("Asset-Fehler: {e}") }
                }
            }
        }
        // Ausgewählte, aber nicht geladene Assets nachladen (z. B. nach Undo oder Laden einer Datei)
        let missing = self.session.as_ref().map(|s| s.missing_assets()).unwrap_or_default();
        for e in missing { self.start_asset_load(ctx, &e.id, e.mhclo.clone(), None); }
    }

    fn rebuild_rows(&mut self) {
        let Some(s) = self.session.as_ref() else { return };
        let key = (self.filter.to_lowercase(), self.open_groups.len() * 1000 + s.params.defs.len());
        if key == self.rows_key && !self.rows.is_empty() { return; }
        self.rows.clear();
        let mut groups: Vec<String> = Vec::new();
        for d in &s.params.defs { if !groups.contains(&d.group) { groups.push(d.group.clone()); } }
        groups.sort_by_key(|g| if g == "Makro" { 0 } else { 1 });
        let f = key.0.clone();
        for g in groups {
            let items: Vec<usize> = s.params.defs.iter().enumerate().filter(|(_, d)| d.group == g && (f.is_empty() || d.label.to_lowercase().contains(&f) || d.id.contains(&f))).map(|(i, _)| i).collect();
            if items.is_empty() { continue; }
            let open = !f.is_empty() || self.open_groups.contains(&g);
            self.rows.push(Row::Header(g.clone(), items.len()));
            if open { for i in items { self.rows.push(Row::Param(i)); } }
        }
        self.rows_key = key;
    }

    fn params_panel(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_mut() else { return };
        ui.horizontal(|ui| {
            ui.label("Spezies:");
            let compat = s.species_indices();
            let cur = s.species_opt().unwrap_or(usize::MAX);
            let mut sel = cur;
            egui::ComboBox::from_id_salt("species").selected_text(s.species_opt().map(|i| s.packages.species[i].label.clone()).unwrap_or_else(|| "(keine Spezies)".into())).show_ui(ui, |ui| {
                for &i in &compat { let sp = &s.packages.species[i]; ui.selectable_value(&mut sel, i, format!("{}{}", sp.label, if sp.stub { "  (Stub)" } else { "" })); }
            });
            if sel != cur && sel != usize::MAX { s.apply_species(sel); self.rows_key.1 = usize::MAX; }
        });
        let Some(spi) = s.species_opt() else { ui.colored_label(theme::WARN, "Für diese Körperbasis gibt es noch keine Spezies – im Werkzeug „Körperbasen & Spezies“ eine erstellen. Der Charakter kann auch ohne Spezies gestaltet werden."); return; };
        let sp = s.packages.species[spi].clone();
        if !sp.presets.is_empty() || !sp.traits.is_empty() {
            ui.horizontal_wrapped(|ui| {
                for p in &sp.presets { if ui.small_button(format!("Preset: {}", p.label)).clicked() { s.apply_map(&p.params); } }
                for t in &sp.traits { if ui.small_button(format!("Merkmal: {}", t.label)).on_hover_text("Setzt die zugehörigen Parameter").clicked() { s.apply_map(&t.set); } }
            });
        }
        if !sp.components.is_empty() {
            ui.horizontal_wrapped(|ui| { ui.weak("Komponenten:"); for c in &sp.components { ui.weak(format!("{}{}", c.asset.rsplit('.').next().unwrap_or(&c.asset), if c.stub { " (Stub: noch keine Geometrie)" } else { "" })); } });
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut s.symmetric, "Symmetrie");
            if ui.button("Alle zurücksetzen").clicked() { s.reset_params(); }
            if ui.add_enabled(s.history.can_undo(), egui::Button::new("Rückgängig")).on_hover_text("Rückgängig (Strg+Z)").clicked() { s.undo(); }
            if ui.add_enabled(s.history.can_redo(), egui::Button::new("Wiederholen")).on_hover_text("Wiederholen (Strg+Y)").clicked() { s.redo(); }
        });
        ui.horizontal(|ui| { ui.label("Suche:"); if ui.text_edit_singleline(&mut self.filter).changed() { self.rows_key.1 = usize::MAX; } if ui.small_button("✕").clicked() { self.filter.clear(); self.rows_key.1 = usize::MAX; } });
        ui.separator();
        self.rebuild_rows();
        let s = self.session.as_mut().unwrap();
        let (n, h) = (self.rows.len(), 22.0);
        let mut toggle: Option<String> = None;
        let mut set: Option<(usize, f32)> = None;
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show_rows(ui, h, n, |ui, range| {
            for r in range {
                match &self.rows[r] {
                    Row::Header(g, c) => { let open = !self.filter.is_empty() || self.open_groups.contains(g); if ui.selectable_label(false, format!("{} {}  ({})", if open { "▼" } else { "▶" }, g, c)).clicked() { toggle = Some(g.clone()); } }
                    Row::Param(i) => {
                        let d = &s.params.defs[*i];
                        let mut v = s.state.values[*i];
                        let (lo, hi) = s.range(*i);
                        let resp = ui.add(egui::Slider::new(&mut v, lo..=hi).text(d.label.as_str()).show_value(true).fixed_decimals(2));
                        if resp.changed() { set = Some((*i, v)); }
                        if resp.double_clicked() { set = Some((*i, d.default)); }
                    }
                }
            }
        });
        if let Some(g) = toggle { if !self.open_groups.remove(&g) { self.open_groups.insert(g); } self.rows_key.1 = usize::MAX; }
        if let Some((i, v)) = set { s.set_param(i, v); }
    }

    fn assets_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, cats: &[Category], kind: Option<usize>) {
        let Some(s) = self.session.as_mut() else { return };
        ui.horizontal(|ui| { ui.label("Suche:"); ui.text_edit_singleline(&mut self.asset_filter); });
        let f = self.asset_filter.to_lowercase();
        let list: Vec<usize> = s.assets.assets.iter().enumerate().filter(|(_, a)| cats.contains(&a.category) && kind.map(|k| regions::clothes_kind(&a.name) == k).unwrap_or(true) && (f.is_empty() || a.name.to_lowercase().contains(&f))).map(|(i, _)| i).collect();
        ui.weak(format!("{} Einträge (nur Assets mit geklärter Lizenz; {} gesperrt)", list.len(), s.assets.skipped_locked));
        ui.separator();
        let mut click: Option<usize> = None;
        let mut tint: Option<(String, [f32; 3])> = None;
        let sel_h = (ui.available_height() * 0.6).max(120.0);
        egui::ScrollArea::vertical().id_salt("assets").max_height(sel_h).auto_shrink([false, false]).show_rows(ui, 22.0, list.len(), |ui, range| {
            for r in range {
                let a = &s.assets.assets[list[r]];
                let on = s.is_selected(&a.id);
                let mut label = format!("{}{}", a.name, if a.needs_attribution { "  (CC-BY)" } else { "" });
                if self.loading.contains(&a.id) { label.push_str("  …lädt"); }
                if ui.selectable_label(on, label).clicked() { click = Some(list[r]); }
            }
        });
        ui.separator();
        ui.label("Ausgewählt:");
        for a in s.state.assets.clone() {
            if !cats.iter().any(|c| s.asset_entry(&a.id).map(|e| e.category == *c).unwrap_or(false)) { continue; }
            ui.horizontal(|ui| { let mut t = a.tint; if ui.color_edit_button_rgb(&mut t).changed() { tint = Some((a.id.clone(), t)); } ui.label(a.id.rsplit('/').next().unwrap_or(&a.id)); });
        }
        if let Some((id, t)) = tint { s.set_tint(&id, t); }
        if let Some(i) = click {
            let e = s.assets.assets[i].clone();
            if s.is_selected(&e.id) { s.toggle_asset_off(&e.id); }
            else { s.add_asset_selection(&e); self.start_asset_load(ctx, &e.id, e.mhclo.clone(), None); }
        }
    }

    pub(crate) fn skin_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, skin: bool) {
        let Some(s) = self.session.as_mut() else { return };
        if skin {
        ui.label("Hautton");
        let mut pick = None;
        egui::ScrollArea::vertical().id_salt("skins").max_height(300.0).show(ui, |ui| { for (n, _) in &s.assets.skins { if ui.selectable_label(s.state.skin == *n, n.replace('_', " ")).clicked() { pick = Some(n.clone()); } } });
        if let Some(n) = pick { s.set_skin(&n); }
        return; }
        ui.label("Augenfarbe");
        let mut eye = None;
        for (n, _) in &s.assets.eye_colors { if ui.selectable_label(s.state.eyes == *n, n.as_str()).clicked() { eye = Some(n.clone()); } }
        if let Some(n) = eye {
            s.state.eyes = n.clone();
            let p = s.assets.eye_colors.iter().find(|(x, _)| *x == n).map(|x| x.1.clone());
            let mh = s.repo.join("content/testcandidate/downloads/eyes/high-poly/high-poly.mhclo");
            self.start_asset_load(ctx, EYES_ID, mh, p);
        }
    }

    fn viewport(&mut self, ui: &mut egui::Ui) {
        let (rect, resp) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        let tool = self.tool;
        if resp.dragged_by(egui::PointerButton::Primary) { let d = resp.drag_delta(); match tool { Tool::Orbit | Tool::Region | Tool::Focus | Tool::Decal => self.camera.orbit(d.x, d.y), Tool::Pan => self.camera.pan(d.x, d.y), Tool::Zoom => self.camera.zoom(-d.y * 4.0) } }
        if resp.dragged_by(egui::PointerButton::Secondary) || resp.dragged_by(egui::PointerButton::Middle) { let d = resp.drag_delta(); self.camera.pan(d.x, d.y); }
        if resp.hovered() { let sc = ui.input(|i| i.smooth_scroll_delta.y); if sc != 0.0 { self.camera.zoom(sc); } }
        if resp.double_clicked() { self.camera = Camera::default(); self.saved_cam = None; }
        let aspect_pick = (rect.width() / rect.height().max(1.0)).max(0.1);
        let studio_pick = self.ws == Workspace::Studio && self.studio.prep.as_ref().map(|p| p.pick != crate::studio::Pick::None).unwrap_or(false);
        if resp.clicked() && (studio_pick || matches!(tool, Tool::Region | Tool::Focus | Tool::Decal)) {
            if let Some(pp) = resp.interact_pointer_pos() {
                let (nx, ny) = ((pp.x - rect.left()) / rect.width() * 2.0 - 1.0, 1.0 - (pp.y - rect.top()) / rect.height() * 2.0);
                let (o, d) = self.camera.ray(nx, ny, aspect_pick);
                if studio_pick { if let Some(s) = self.session.as_mut() { crate::studio::on_pick(s, &mut self.studio, o, d); } }
                else if tool == Tool::Decal { self.place_decal(o, d); }
                else { self.pick_tool(o, d); }
            }
        }
        let updates = self.session.as_mut().map(|s| s.refresh()).unwrap_or_default();
        let aspect = (rect.width() / rect.height().max(1.0)).max(0.1);
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(rect, SceneCallback { updates: Mutex::new(Some(updates)), globals: self.camera.globals_lit(aspect, &LIGHTS[self.light.min(2)], self.grid) }));
        // Kamera-Presets (rechts)
        let mut y = rect.top() + 8.0;
        for (i, (n, c)) in PRESETS.iter().enumerate() {
            let r = egui::Rect::from_min_size(egui::pos2(rect.right() - 120.0, y), egui::vec2(112.0, 22.0));
            if ui.put(r, egui::Button::new(format!("{} {}", i + 1, n)).small()).clicked() { self.camera = *c; }
            y += 24.0;
        }
        // Werkzeuge (links)
        let mut y = rect.top() + 8.0;
        for (t, n, tip) in [(Tool::Orbit, "Rotieren", "Ziehen dreht die Kamera"), (Tool::Pan, "Bewegen", "Ziehen verschiebt die Ansicht"), (Tool::Zoom, "Zoomen", "Ziehen zoomt"), (Tool::Focus, "Fokus", "Klick auf den Körper zentriert die Kamera dort"), (Tool::Region, "Bereich", "Klick auf eine Körperstelle öffnet den passenden Bereich zum Bearbeiten")] {
            let r = egui::Rect::from_min_size(egui::pos2(rect.left() + 8.0, y), egui::vec2(84.0, 26.0));
            if ui.put(r, egui::Button::new(n).selected(self.tool == t)).on_hover_text(tip).clicked() { self.tool = t; }
            y += 30.0;
        }
        // Licht & Hintergrund (oben Mitte)
        let w = 3.0 * 76.0 + 70.0;
        let mut x = rect.center().x - w / 2.0;
        for (i, l) in LIGHTS.iter().enumerate() {
            let r = egui::Rect::from_min_size(egui::pos2(x, rect.top() + 8.0), egui::vec2(72.0, 24.0));
            if ui.put(r, egui::Button::new(l.name).selected(self.light == i)).clicked() { self.light = i; }
            x += 76.0;
        }
        let r = egui::Rect::from_min_size(egui::pos2(x, rect.top() + 8.0), egui::vec2(66.0, 24.0));
        if ui.put(r, egui::Button::new("Raster").selected(self.grid)).clicked() { self.grid = !self.grid; }
        // Kompakte Leistungsanzeige (unten links)
        ui.painter().text(rect.left_bottom() + egui::vec2(10.0, -10.0), egui::Align2::LEFT_BOTTOM, format!("{:.0} FPS · {:.1} ms{}", 1000.0 / self.frame_avg.max(0.1), self.frame_avg, self.session.as_ref().map(|s| format!(" · Morph {:.2} ms", s.perf.morph_ms)).unwrap_or_default()), egui::FontId::monospace(11.0), egui::Color32::from_rgb(190, 205, 230));
        if self.session.is_none() { ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, if self.error.is_some() { "Fehler beim Laden" } else { "Lade Charakterdaten …" }, egui::FontId::proportional(18.0), egui::Color32::WHITE); }
        if self.show_stats {
            let mut t = format!("Frame {:.1} ms (Ø {:.1}) · {:.0} fps\nGPU: {}", self.frame_ms, self.frame_avg, 1000.0 / self.frame_avg.max(0.1), self.gpu_name);
            if let Some(s) = &self.session {
                let (v, tr) = s.counts();
                t += &format!("\nMorph: {:.3} ms · Fit: {:.3} ms · Gesamt: {:.3} ms{}\nAktive Targets: {} (geändert: {}) · Parameter: {}\nVertices {} · Dreiecke {} · Assets {}", s.perf.morph_ms, s.perf.fit_ms, s.perf.total_ms, if s.perf.full { " [voll]" } else { "" }, s.perf.active, s.perf.changed, s.params.defs.len(), v, tr, s.active_asset_count());
                t += &format!("\nTargets {} · Deltas {:.2} Mio · Daten {}", s.body.morphs.names.len(), s.body.morphs.nnz() as f64 / 1e6, if s.body.from_cache { "aus Cache" } else { "geparst" });
            }
            ui.painter().text(rect.left_top() + egui::vec2(8.0, 6.0), egui::Align2::LEFT_TOP, t, egui::FontId::monospace(11.0), egui::Color32::from_rgb(210, 225, 255));
        }
    }

    fn save_dialog(&mut self) {
        let Some(s) = self.session.as_ref() else { return };
        if let Some(p) = rfd::FileDialog::new().add_filter("HASE Charakter", &["json"]).set_file_name("charakter.hase.json").save_file() {
            let v = s.state.to_json(&s.params, &s.packages.body.id);
            self.status = match std::fs::write(&p, serde_json::to_string_pretty(&v).unwrap()) { Ok(_) => format!("Gespeichert: {}", p.display()), Err(e) => format!("Speichern fehlgeschlagen: {e}") };
        }
    }
    /// Werkzeug „Fokus“/„Bereich“: Klick auf den Körper. Bereich = Region mit der größten Morph-Wirkung an der Stelle.
    fn pick_tool(&mut self, o: [f32; 3], d: [f32; 3]) {
        let Some(s) = self.session.as_ref() else { return };
        let Some((hit, v)) = s.pick(o, d) else { self.status = "Kein Treffer am Körper".into(); return };
        if self.tool == Tool::Focus { self.focus_point([hit[0], hit[1] + s.floor_offset(), hit[2]]); return; }
        // Gesamt-„Verformungsenergie“ je Region (einmalig): Σ|Delta| aller zugehörigen Targets
        if self.region_energy.len() != REGIONS.len() {
            let mut e = vec![0f32; REGIONS.len()];
            for def in &s.params.defs { if let Some(r) = regions::region_of(def) { for t in [def.pos, def.neg].into_iter().flatten() { let (a, b) = (s.body.morphs.t_off[t] as usize, s.body.morphs.t_off[t + 1] as usize); e[r] += s.body.morphs.t_delta[a..b].iter().map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()).sum::<f32>(); } } }
            self.region_energy = e;
        }
        let mut score: std::collections::HashMap<usize, f32> = std::collections::HashMap::new();
        for (t, mag) in s.morphs_at_vertex(v, 300) {
            for def in s.params.defs.iter() { if def.pos == Some(t) || def.neg == Some(t) { if let Some(r) = regions::region_of(def) { *score.entry(r).or_default() += mag; } } }
        }
        // Spezifität: Anteil der Gesamtverformung der Region, der an dieser Stelle wirkt
        let score: Vec<(usize, f32)> = score.into_iter().map(|(r, m)| (r, m / self.region_energy[r].max(1e-6))).collect();
        match score.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
            Some((r, _)) => { let reg = &REGIONS[r]; self.ws = Workspace::Charakter; self.main = reg.main; self.region = r; self.sub = 0; self.focus(reg.cam); self.status = format!("Bereich: {}", reg.label); }
            None => self.status = "Für diese Stelle gibt es keinen zugeordneten Bearbeitungsbereich".into(),
        }
    }
    /// Werkzeug „Detail“: Klick platziert ein Hautdetail (Muttermal, Tattoo-Motiv, Narbe) an der UV-Position unter dem Cursor.
    fn place_decal(&mut self, o: [f32; 3], d: [f32; 3]) {
        let shape = self.decal_shape;
        let Some(s) = self.session.as_mut() else { return };
        let Some(uv) = s.pick_uv(o, d) else { self.status = "Kein Treffer am Körper".into(); return };
        let (size, color, opacity) = match shape { 0 => (0.004, [0.18, 0.10, 0.07], 0.9), 5 => (0.012, [0.75, 0.52, 0.48], 0.7), _ => (0.014, [0.10, 0.12, 0.25], 0.85) };
        let n = s.state.style.skin.decals.len();
        s.edit_style(1000 + n, |st| st.skin.decals.push(hase_core::state::Decal { shape, u: uv[0], v: uv[1], size, rot: 0.0, color, opacity }));
        self.decal_sel = Some(n);
        self.status = format!("{} platziert", hase_core::state::DECAL_SHAPES[shape as usize]);
    }

    fn focus_point(&mut self, p: [f32; 3]) {
        if self.saved_cam.is_none() { self.saved_cam = Some(self.camera); }
        self.camera.target = p; self.camera.dist = (self.camera.dist * 0.6).clamp(0.35, 3.0);
    }

    /// Speichern in die Bibliothek: vorhandenen Eintrag aktualisieren, sonst nach einem Namen fragen.
    pub(crate) fn quick_save(&mut self) {
        if self.session.as_ref().map(|s| s.vrm.is_some()).unwrap_or(false) { self.status = "Ein geöffnetes VRM-Modell lässt sich nicht in der Bibliothek speichern (nur gestaltete Charaktere).".into(); return; }
        let ctx = self.ctx.clone();
        if self.lib.current.is_some() { self.lib_save_current(&ctx); self.status = "In Bibliothek gespeichert".into(); }
        else { self.lib.save_prompt = true; }
    }
    pub(crate) fn open_vrm_dialog(&mut self) {
        let Some(p) = rfd::FileDialog::new().add_filter("VRM / GLB", &["vrm", "glb"]).pick_file() else { return };
        self.open_vrm_path(&p);
    }
    pub(crate) fn open_vrm_path(&mut self, p: &std::path::Path) {
        let Some(s) = self.session.as_mut() else { return };
        self.status = match s.open_vrm(p) { Ok(i) => { self.ws = Workspace::Charakter; self.main = Main::Charakter; self.region = usize::MAX; format!("Modell geöffnet – {i}") } Err(e) => format!("Modell konnte nicht geöffnet werden: {e}") };
    }
    pub(crate) fn goto_export(&mut self) { self.ws = Workspace::Finalisieren; self.main = Main::Animation; self.export.sub = 1; }

    pub(crate) fn continue_load(&mut self, v: serde_json::Value, path: String) {
        let Some(s) = self.session.as_mut() else { return };
        match s.plan_load(&v) {
            crate::session::LoadPlan::Ready(st, issues) => {
                if issues.is_empty() { s.apply_state(st); self.status = format!("Geladen: {path}"); }
                else { self.pending_load = Some((st, issues, path)); }
            }
            crate::session::LoadPlan::NeedsBody(b) => { self.pending_body = Some((v, b, path)); }
            crate::session::LoadPlan::MissingBody(id) => self.status = format!("Körperbasis {id} ist nicht installiert oder nicht fertig konfiguriert – Charakter nicht geladen"),
            crate::session::LoadPlan::Invalid => self.status = "Datei hat kein passendes Format".into(),
        }
    }

    fn load_dialog(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("HASE Charakter", &["json"]).pick_file() {
            let r = std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).map_err(|e| e.to_string()));
            match (r, self.session.as_mut()) {
                (Ok(v), Some(s)) => self.continue_load(v, p.display().to_string()),
                (Err(e), _) => self.status = format!("Laden fehlgeschlagen: {e}"),
                _ => {}
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let t0 = Instant::now();
        let ctx = ui.ctx().clone();
        let dt = ctx.input(|i| i.unstable_dt) * 1000.0;
        self.frame_ms = dt; self.frame_avg = if self.frame_avg == 0.0 { dt } else { self.frame_avg * 0.95 + dt * 0.05 };
        self.poll(&ctx);
        self.thumbs.poll(&ctx);
        self.export_poll();
        if let Some(s) = self.session.as_mut() { if s.play.playing { s.tick(ctx.input(|i| i.stable_dt).min(0.1)); ctx.request_repaint(); } }
        if self.session.is_none() { ctx.request_repaint_after(std::time::Duration::from_millis(100)); }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) { if let Some(s) = self.session.as_mut() { s.undo(); } }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) { if let Some(s) = self.session.as_mut() { s.redo(); } }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) { self.quick_save(); }
        if ctx.input(|i| i.key_pressed(egui::Key::F3)) { self.show_stats = !self.show_stats; }
        if !ctx.egui_wants_keyboard_input() { for (i, (_, c)) in PRESETS.iter().enumerate() { let k = [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4, egui::Key::Num5][i]; if ctx.input(|inp| inp.key_pressed(k)) { self.camera = *c; } } }

        egui::Panel::top("menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Datei", |ui| {
                    if ui.button("In Bibliothek speichern  Strg+S").clicked() { ui.close(); self.quick_save(); }
                    if ui.button("Als Datei speichern …").clicked() { ui.close(); self.save_dialog(); }
                    if ui.button("Aus Bibliothek öffnen …").clicked() { ui.close(); self.lib.open = true; }
                    if ui.button("Aus Datei laden …").clicked() { ui.close(); self.load_dialog(); }
                    ui.separator();
                    if ui.button("Modell öffnen (VRM/GLB) …").on_hover_text("Fertiges VRM-Modell ansehen, posen und Ausdrücke vorführen").clicked() { ui.close(); self.open_vrm_dialog(); }
                    if ui.button("VRM exportieren …").clicked() { ui.close(); self.goto_export(); }
                    ui.separator();
                    if ui.button("Beenden").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
                ui.menu_button("Bearbeiten", |ui| {
                    if ui.add_enabled(self.session.as_ref().map(|s| s.history.can_undo()).unwrap_or(false), egui::Button::new("Rückgängig  Strg+Z")).clicked() { if let Some(s) = self.session.as_mut() { s.undo(); } ui.close(); }
                    if ui.add_enabled(self.session.as_ref().map(|s| s.history.can_redo()).unwrap_or(false), egui::Button::new("Wiederholen  Strg+Y")).clicked() { if let Some(s) = self.session.as_mut() { s.redo(); } ui.close(); }
                });
                ui.menu_button("Ansicht", |ui| {
                    for (i, (n, c)) in PRESETS.iter().enumerate() { if ui.button(format!("{n}  ({})", i + 1)).clicked() { self.camera = *c; ui.close(); } }
                    ui.separator();
                    ui.checkbox(&mut self.show_stats, "Leistungsanzeige  (F3)");
                });
                ui.menu_button("Hilfe", |ui| { ui.label("Linke Maus: Kamera drehen · Rechte/Mittlere Maus: verschieben · Mausrad: Zoom"); ui.label("Doppelklick: Kamera zurücksetzen · 1–5: Ansichten"); ui.label("Slider: Doppelklick = zurücksetzen"); });
            });
        });
        egui::Panel::bottom("status").show(ui, |ui| { ui.horizontal(|ui| { ui.label(&self.status); if let Some(e) = &self.error { ui.colored_label(egui::Color32::LIGHT_RED, e); } ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.weak("HASE VRM MAKER – Entwicklungsstand (Testcandidate)"); }); }); });
        self.load_modal(&ctx);
        egui::Panel::top("nav").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("HASE VRM MAKER").strong().size(16.0)); ui.weak("Character Studio");
                ui.add_space(16.0);
                for (n, ws, t) in [("1  Charakter", Workspace::Charakter, "Anatomie & Merkmale"), ("2  Outfit", Workspace::Outfit, "Kleidung & Accessoires"), ("3  Finalisieren", Workspace::Finalisieren, "Presets, Export & DB")] { self.ws_button(ui, n, ws, t); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Exportieren").on_hover_text("VRM 1.0 exportieren (Finalisieren › Export)").clicked() { self.goto_export(); }
                    if ui.button("Speichern").on_hover_text("Charakter in der Bibliothek speichern (Strg+S)").clicked() { self.quick_save(); }
                    if ui.button("Bibliothek").on_hover_text("Gespeicherte Charaktere, Vorlagen, Varianten").clicked() { self.lib.open = true; }
                    ui.separator();
                    self.ws_button(ui, "Körperbasen & Spezies", Workspace::Studio, "Authoring-Werkzeug: Körperbasen und Spezies erstellen, importieren, prüfen, speichern");
                    ui.weak("Werkzeuge");
                });
            });
        });
        self.lib_pending();
        self.lib_window(&ctx);
        if self.ws == Workspace::Charakter && !self.expert && !self.session.as_ref().map(|s| s.vrm.is_some()).unwrap_or(false) { egui::Panel::bottom("presetbar").min_size(130.0).show(ui, |ui| self.preset_bar(ui)); }
        egui::Panel::left("regions").default_size(190.0).min_size(150.0).show(ui, |ui| self.region_list(ui));
        egui::Panel::right("detail").default_size(400.0).min_size(300.0).show(ui, |ui| self.detail_panel(ui, &ctx));
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| self.viewport(ui));
        if self.started.elapsed().as_secs() < 5 || self.session.as_ref().map(|s| s.perf.total_ms > 0.0).unwrap_or(false) { /* UI-Zeit nur für Overlay */ }
        let _ = t0;
    }
}


impl App {
    fn ws_button(&mut self, ui: &mut egui::Ui, n: &str, ws: Workspace, tip: &str) {
        if ui.add(egui::Button::new(egui::RichText::new(n).size(15.0)).selected(self.ws == ws)).on_hover_text(tip).clicked() {
            self.ws = ws;
            self.main = match ws { Workspace::Charakter => Main::Charakter, Workspace::Outfit => Main::Kleidung, Workspace::Finalisieren => Main::Animation, Workspace::Studio => Main::Charakter };
            self.region = usize::MAX; self.sub = 0; self.asset_filter.clear();
        }
    }

    /// Rückfrage beim Laden eines Charakters, dessen Ressourcen sich geändert haben – niemals stilles Umschalten.
    fn load_modal(&mut self, ctx: &egui::Context) {
        if let Some((v, b, path)) = self.pending_body.clone() {
            let mut done = false;
            egui::Window::new("Charakter benötigt eine andere Körperbasis").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.label(&path);
                ui.label(format!("Der Charakter wurde mit der Körperbasis „{}“ (v{}) gestaltet. Aktive Körperbasis: „{}“.", b.label, b.version, self.session.as_ref().map(|s| s.packages.body.label.clone()).unwrap_or_default()));
                ui.weak("Ein Wechsel ersetzt die aktuelle Bearbeitung (nicht gespeicherte Änderungen gehen verloren).");
                ui.horizontal(|ui| {
                    if ui.button("Körperbasis wechseln und laden").clicked() {
                        let r = self.session.as_mut().map(|s| s.activate_body(&b));
                        done = true;
                        match r { Some(Ok(())) => self.continue_load(v.clone(), path.clone()), Some(Err(e)) => self.status = e, None => {} }
                    }
                    if ui.button("Abbrechen").clicked() { done = true; }
                });
            });
            if done { self.pending_body = None; }
            return;
        }
        let Some((st, issues, path)) = self.pending_load.clone() else { return };
        let mut done = false;
        egui::Window::new("Charakter laden – Ressourcen haben sich geändert").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(&path);
            for i in &issues { ui.colored_label(theme::WARN, format!("• {i}")); }
            ui.add_space(6.0);
            ui.label("Die gespeicherten Parameterwerte bleiben unverändert; der Charakter sieht so aus wie gespeichert. Die Bindung an die installierten Versionen wird nur auf Wunsch aktualisiert.");
            ui.horizontal(|ui| {
                if ui.button("Laden (Werte & alte Bindung behalten)").clicked() { if let Some(s) = self.session.as_mut() { s.apply_state(st.clone()); } self.status = format!("Geladen mit Hinweisen: {path}"); done = true; }
                if ui.button("Laden und an installierte Versionen binden").clicked() { if let Some(s) = self.session.as_mut() { s.apply_state(st.clone()); s.rebind_refs(); } self.status = format!("Geladen und gebunden: {path}"); done = true; }
                if ui.button("Abbrechen").clicked() { done = true; }
            });
        });
        if done { self.pending_load = None; }
    }

    fn nav_button(&mut self, ui: &mut egui::Ui, m: Main) {
        if ui.selectable_label(self.main == m, egui::RichText::new(m.label()).size(15.0)).clicked() { self.main = m; self.region = usize::MAX; self.sub = 0; self.asset_filter.clear(); }
    }

    fn focus(&mut self, mut cam: Camera) {
        if self.saved_cam.is_none() { self.saved_cam = Some(self.camera); }
        // Fokuspunkte sind für 1,75 m Körperhöhe definiert → auf aktuelle Höhe skalieren
        if let Some(s) = self.session.as_ref() { let k = (s.body_height() / 1.75).clamp(0.3, 1.5); cam.target[1] *= k; cam.dist *= k.max(0.6); }
        self.camera = cam;
    }

    fn region_list(&mut self, ui: &mut egui::Ui) {
        if self.session.as_ref().map(|s| s.vrm.is_some()).unwrap_or(false) && self.ws != Workspace::Finalisieren && self.ws != Workspace::Studio && self.ws != Workspace::Outfit { ui.strong("Geöffnetes Modell"); ui.separator(); ui.weak("Körper und Haare sind im Mesh fest enthalten und hier nicht veränderbar."); ui.add_space(6.0); ui.weak("Kleidung anprobieren: 2 Outfit"); ui.weak("Posen: 3 Finalisieren › Posen & Animation"); return; }
        match self.ws {
            Workspace::Charakter => {
                ui.strong("Bereiche");
                ui.separator();
                let mut go: Option<(Main, usize, usize, Camera)> = None;
                let sec = |ui: &mut egui::Ui, t: &str| { ui.add_space(4.0); ui.weak(t); };
                egui::ScrollArea::vertical().auto_shrink([false; 2]).max_height(ui.available_height() - 140.0).show(ui, |ui| {
                    sec(ui, "CHARAKTER");
                    for (i, l) in ["Spezies & Presets", "Alter, Geschlecht & Typ"].iter().enumerate() { if ui.selectable_label(self.main == Main::Charakter && self.sub == i, *l).clicked() { go = Some((Main::Charakter, usize::MAX, i, regions::CAM_BODY)); } }
                    for (m, t) in [(Main::Gesicht, "GESICHT"), (Main::Koerper, "KÖRPER")] {
                        sec(ui, t);
                        for (i, r) in regions::regions_of(m) { if ui.selectable_label(self.main == m && self.region == i, r.label).clicked() { go = Some((m, i, 0, r.cam)); } }
                    }
                    sec(ui, "HAARE & STIL");
                    if ui.selectable_label(self.main == Main::Haare, "Frisur").clicked() { go = Some((Main::Haare, usize::MAX, 0, regions::CAM_HEAD)); }
                    for (i, l) in ["Haut", "Make-up", "Details (Sommersprossen, Muttermale, Tattoos, Narben)", "Augen (→ Gesicht › Augen)", "Brauen & Wimpern"].iter().enumerate() { if ui.selectable_label(self.main == Main::Stil && self.sub == i, *l).clicked() { if i == 3 { go = Some((Main::Gesicht, crate::regions::REGIONS.iter().position(|r| r.label == "Augen").unwrap(), 0, crate::regions::REGIONS.iter().find(|r| r.label == "Augen").unwrap().cam)); } else { go = Some((Main::Stil, usize::MAX, i, if i == 0 { regions::CAM_BODY } else { regions::CAM_HEAD })); } } }
                });
                if let Some((m, r, sub, c)) = go { self.main = m; self.region = r; self.sub = sub; self.focus(c); }
            }
            Workspace::Outfit => {
                ui.strong("Outfit"); ui.separator();
                for (i, l) in CLOTHES_KINDS.iter().enumerate() { if ui.selectable_label(self.sub == i, *l).clicked() { self.sub = i; } }
            }
            Workspace::Studio => self.studio_library(ui),
            Workspace::Finalisieren => { ui.strong("Finalisieren"); ui.separator(); for (i, l) in ["Posen & Animation", "Export (VRM 1.0)"].iter().enumerate() { if ui.selectable_label(self.export.sub == i, *l).clicked() { self.export.sub = i; } } if ui.button("Vorlagen & Varianten …").on_hover_text("Öffnet die Charakterbibliothek").clicked() { self.lib.open = true; } }
        }
        ui.separator();
        if let Some(c) = self.saved_cam { if ui.button("Eigene Kamera").on_hover_text("Zur manuell eingestellten Ansicht zurück").clicked() { self.camera = c; self.saved_cam = None; } }
        if let Some(s) = self.session.as_mut() {
            ui.horizontal(|ui| {
                if ui.add_enabled(s.history.can_undo(), egui::Button::new("Rückgängig")).on_hover_text("Strg+Z").clicked() { s.undo(); }
                if ui.add_enabled(s.history.can_redo(), egui::Button::new("Wiederholen")).on_hover_text("Strg+Y").clicked() { s.redo(); }
            });
            ui.checkbox(&mut s.symmetric, "Symmetrie (links = rechts)");
        }
    }

    fn species_panel(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_mut() else { return };
        ui.horizontal(|ui| {
            ui.label("Spezies:");
            let compat = s.species_indices();
            let cur = s.species_opt().unwrap_or(usize::MAX);
            let mut sel = cur;
            egui::ComboBox::from_id_salt("species").selected_text(s.species_opt().map(|i| s.packages.species[i].label.clone()).unwrap_or_else(|| "(keine Spezies)".into())).show_ui(ui, |ui| {
                for &i in &compat { let sp = &s.packages.species[i]; ui.selectable_value(&mut sel, i, format!("{}{}", sp.label, if sp.stub { "  (Stub)" } else { "" })); }
            });
            if sel != cur && sel != usize::MAX { s.apply_species(sel); self.rows_key.1 = usize::MAX; }
        });
        let Some(spi) = s.species_opt() else { ui.colored_label(theme::WARN, "Für diese Körperbasis gibt es noch keine Spezies – im Werkzeug „Körperbasen & Spezies“ eine erstellen. Der Charakter kann auch ohne Spezies gestaltet werden."); return; };
        let sp = s.packages.species[spi].clone();
        if !sp.presets.is_empty() { ui.label("Presets"); ui.horizontal_wrapped(|ui| { for p in &sp.presets { if ui.button(&p.label).clicked() { s.apply_map(&p.params); } } }); }
        if !sp.traits.is_empty() { ui.label("Merkmale"); ui.horizontal_wrapped(|ui| { for t in &sp.traits { if ui.button(&t.label).on_hover_text("Setzt die zugehörigen Parameter").clicked() { s.apply_map(&t.set); } } }); }
        if !sp.components.is_empty() { ui.label("Komponenten"); for c in &sp.components { ui.weak(format!("{}{}", c.asset.rsplit('.').next().unwrap_or(&c.asset), if c.stub { " (Stub: noch keine Geometrie)" } else { "" })); } }
        if ui.button("Alle Parameter zurücksetzen").clicked() { s.reset_params(); }
        let issues = s.ref_issues();
        if !issues.is_empty() {
            ui.separator();
            for i in &issues { ui.colored_label(theme::WARN, format!("• {i}")); }
            ui.weak("Der Charakter bleibt unverändert, bis du ihn ausdrücklich an die installierte Version bindest.");
            if ui.button("An installierte Version binden").clicked() { s.rebind_refs(); }
        }
        let out = s.out_of_limits();
        if !out.is_empty() {
            ui.colored_label(theme::WARN, format!("• {} Werte liegen außerhalb der Grenzen dieser Spezies", out.len()));
            if ui.button("In Speziesgrenzen bringen").clicked() { s.clamp_all_to_species(); }
        }
    }

    /// Regler einer Region/Auswahl in Unterabschnitten (Form / Proportionen / Position).
    pub(crate) fn region_sliders(&mut self, ui: &mut egui::Ui, region: Option<usize>, macros: bool) {
        let Some(s) = self.session.as_mut() else { return };
        let key = (if macros { 9999 } else { region.unwrap_or(10_000) }, s.params.defs.len());
        if key != self.reg_key {
            self.reg_idx = (0..s.params.defs.len()).filter(|&i| { let d = &s.params.defs[i]; if macros { d.group == "Makro" } else { match region { Some(r) => regions::region_of(d) == Some(r), None => d.group != "Makro" && regions::region_of(d).is_none() } } }).collect();
            self.reg_idx.sort_by_key(|&i| (regions::tier(&s.params.defs[i]), s.params.defs[i].label.clone()));
            self.reg_key = key;
        }
        if self.reg_idx.is_empty() { ui.weak("Keine Parameter in diesem Bereich."); return; }
        let idx = self.reg_idx.clone();
        if ui.button("Bereich zurücksetzen").clicked() { s.reset_indices(&idx); }
        let mut set: Option<(usize, f32)> = None;
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let mut last = usize::MAX;
            for &i in &idx {
                let d = &s.params.defs[i];
                if !macros { let t = regions::tier(d); if t != last { ui.add_space(6.0); ui.strong(TIER_LABELS[t]); last = t; } }
                let mut v = s.state.values[i];
                let resp = ui.add(egui::Slider::new(&mut v, { let (lo, hi) = s.range(i); lo..=hi.max(lo + 1e-4) }).text(crate::theme::label_de(&d.label)).fixed_decimals(2));
                if resp.changed() { set = Some((i, v)); }
                if resp.double_clicked() { set = Some((i, d.default)); }
            }
        });
        if let Some((i, v)) = set { s.set_param(i, v); }
    }

    fn detail_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.session.is_none() { ui.label("Lade …"); return; }
        if self.session.as_ref().map(|s| s.vrm.is_some()).unwrap_or(false) && self.ws != Workspace::Finalisieren && self.ws != Workspace::Studio && self.ws != Workspace::Outfit { self.vrm_panel(ui); return; }
        if self.ws == Workspace::Charakter {
            ui.horizontal(|ui| { if ui.selectable_label(!self.expert, "Parameter").clicked() { self.expert = false; } if ui.selectable_label(self.expert, "Erweitert (alle Parameter)").clicked() { self.expert = true; } });
            ui.separator();
        }
        if self.ws == Workspace::Studio { self.studio_detail(ui); return; }
        if self.expert && self.ws == Workspace::Charakter { self.params_panel(ui); return; }
        match self.main {
            Main::Charakter => {
                if self.sub == 0 { ui.heading("Spezies & Presets"); ui.separator(); self.species_panel(ui); }
                else { ui.heading("Alter, Geschlecht & Typ"); ui.separator(); self.region_sliders(ui, None, true); }
            }
            Main::Gesicht | Main::Koerper => {
                if self.region == usize::MAX {
                    ui.heading(format!("{} · Übersicht", self.main.label())); ui.separator();
                    ui.label("Wähle links einen Bereich. Die Kamera fokussiert automatisch; „Eigene Kamera“ stellt deine Ansicht wieder her.");
                    if self.main == Main::Koerper {
                        ui.add_space(8.0);
                        ui.colored_label(theme::TEXT2, "Brust und Gesäß besitzen ein anatomisches Modell (Abschnitt „Anatomisches Modell“): Volumen, Projektion, obere/untere Fülle, Breite, Ansatzhöhe, Hängen – links/rechts getrennt, mit glatten Übergängen zum Körper. Muskulatur/Hüfte nutzen weiterhin die MakeHuman-Parameter.");
                    }
                } else {
                    ui.heading(REGIONS[self.region].label); ui.separator();
                    if REGIONS[self.region].label == "Augen" { self.eye_studio(ui); } else { self.region_sliders(ui, Some(self.region), false); }
                }
            }
            Main::Haare => {
                ui.heading("Haarstudio"); ui.separator();
                ui.horizontal(|ui| { for (i, t) in ["Auswahl", "Form", "Farbe & Material"].iter().enumerate() { if ui.selectable_label(self.hair_tab == i, *t).clicked() { self.hair_tab = i; } } });
                ui.weak("Haarphysik und Strähnen-/Scheitelbearbeitung: noch nicht unterstützt.");
                ui.separator();
                match self.hair_tab { 0 => self.hair_grid(ui, ctx), 1 => self.hair_shape_panel(ui), _ => self.hair_color_panel(ui) }
            }
            Main::Stil => match self.sub {
                0 => { ui.heading("Haut"); ui.separator(); self.skin_style_panel(ui); self.skin_panel(ui, ctx, true); }
                1 => { ui.heading("Make-up"); ui.separator(); self.makeup_panel(ui); }
                2 => { ui.heading("Details"); ui.separator(); self.details_panel(ui); }
                3 => { ui.heading("Augen"); ui.separator(); ui.label("Das Augenstudio befindet sich unter Gesicht › Augen."); }
                _ => { ui.heading("Brauen & Wimpern"); ui.separator(); self.assets_panel(ui, ctx, &[Category::Eyebrows, Category::Eyelashes], None); }
            },
            Main::Kleidung => {
                ui.heading(format!("Outfit · {}", CLOTHES_KINDS[self.sub.min(4)])); ui.separator();
                ui.weak("Kleidung ist vom Charakter getrennt: der Körper bleibt unverändert, Teile werden an die Körperform angepasst.");
                self.assets_panel(ui, ctx, &[Category::Clothes], Some(self.sub.min(4)));
            }
            Main::Animation => {
                if self.export.sub == 0 { self.anim_panel(ui); } else { self.export_panel(ui); }
            }
        }
    }
}
