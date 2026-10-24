//! Augenstudio, Haarstudio, Haut-/Stilbereiche und gemeinsame Kachel-Komponenten (mit Thumbnails).
use crate::app::App;
use crate::camera::Camera;
use crate::theme;
use crate::thumbs::{make_key, ThumbJob};
use eframe::egui;
use hase_core::assets::Category;
use hase_core::state::{CharacterState, EyeSide};
use std::collections::BTreeMap;

/// Augenform-Presets: Wertesets auf den vorhandenen Augen-Morphs (links/rechts gleich).
pub const EYE_PRESETS: &[(&str, &[(&str, f32)])] = &[
    ("Natürlich", &[]),
    ("Mandelförmig", &[("eye-corner2-down-up", 0.55), ("eye-height1-decr-incr", 0.25), ("eye-scale-decr-incr", 0.1)]),
    ("Schmal", &[("eye-height1-decr-incr", -0.6), ("eye-height2-decr-incr", -0.5), ("eye-height3-decr-incr", -0.3)]),
    ("Rund", &[("eye-height1-decr-incr", 0.7), ("eye-height2-decr-incr", 0.5), ("eye-height3-decr-incr", 0.4), ("eye-scale-decr-incr", 0.2)]),
    ("Katzenartig", &[("eye-corner2-down-up", 0.95), ("eye-corner1-down-up", -0.25), ("eye-height1-decr-incr", -0.25), ("eye-epicanthus-in-out", 0.2)]),
    ("Hängend", &[("eye-corner2-down-up", -0.8), ("eye-corner1-down-up", 0.15), ("eye-bag-decr-incr", 0.2)]),
    ("Groß", &[("eye-scale-decr-incr", 0.9), ("eye-height1-decr-incr", 0.3)]),
    ("Tief liegend", &[("eye-push1-in-out", -0.7), ("eye-eyefold-down-up", 0.4), ("eye-bag-decr-incr", 0.3)]),
];

/// Haarfarb-Presets (Ansatz, Spitze).
pub const HAIR_COLORS: &[(&str, [f32; 3], [f32; 3])] = &[
    ("Schwarz", [0.03, 0.03, 0.04], [0.08, 0.08, 0.10]), ("Dunkelbraun", [0.12, 0.07, 0.04], [0.22, 0.13, 0.07]), ("Braun", [0.30, 0.19, 0.11], [0.55, 0.36, 0.22]),
    ("Honigblond", [0.45, 0.30, 0.14], [0.85, 0.65, 0.35]), ("Blond", [0.62, 0.48, 0.22], [0.95, 0.85, 0.55]), ("Rot", [0.38, 0.08, 0.04], [0.72, 0.22, 0.08]),
    ("Silber", [0.45, 0.46, 0.50], [0.85, 0.86, 0.90]), ("Weiß", [0.80, 0.80, 0.82], [0.97, 0.97, 0.98]), ("Rosa", [0.55, 0.20, 0.35], [0.95, 0.55, 0.70]),
    ("Blau", [0.05, 0.12, 0.35], [0.20, 0.45, 0.85]), ("Ombré braun→blond", [0.14, 0.08, 0.05], [0.85, 0.65, 0.35]), ("Ombré schwarz→rot", [0.04, 0.03, 0.03], [0.75, 0.15, 0.08]),
];

impl App {
    /// Kamera für Vorschaubilder; Höhenskalierung relativ zur Körpergröße.
    fn thumb_cam(&self, kind: &str) -> Camera {
        let hh = self.session.as_ref().map(|s| s.body_height()).unwrap_or(1.75);
        let k = (hh / 1.75).clamp(0.3, 1.5);
        match kind {
            "eye" => Camera { yaw: 0.0, pitch: 0.0, dist: 0.26 * k.max(0.6), target: [0.0, 0.935 * hh, 0.0], fov: 0.62 },
            "hair" => Camera { yaw: 0.45, pitch: 0.05, dist: 0.75 * k.max(0.6), target: [0.0, 0.915 * hh, 0.0], fov: 0.62 },
            "head" => Camera { yaw: 0.0, pitch: 0.02, dist: 0.62 * k.max(0.6), target: [0.0, 0.91 * hh, 0.0], fov: 0.62 },
            _ => Camera { yaw: 0.3, pitch: 0.05, dist: 3.2, target: [0.0, 0.9 * k, 0.0], fov: 0.62 },
        }
    }

    /// Vorschaukachel: zeigt ein (ggf. noch gerendertes) Thumbnail des Zustands `st`; Klick liefert `true`.
    pub(crate) fn tile(&mut self, ui: &mut egui::Ui, size: egui::Vec2, st: CharacterState, cam_kind: &str, label: &str, selected: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        let Some(s) = self.session.as_ref() else { return false };
        let cam = self.thumb_cam(cam_kind);
        let (w, h) = (192u32, (192.0 * size.y / size.x) as u32);
        let key = make_key(&st, &s.packages.body.digest, &cam, w, h, &s.params);
        let body = s.packages.body.clone();
        let ctx = ui.ctx().clone();
        let tex = self.thumbs.get(&ctx, &key, || ThumbJob { key: key.clone(), state: st, cam, w, h, body });
        let p = ui.painter();
        let fill = if selected { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::PANEL2 };
        p.rect_filled(rect, 6.0, fill);
        match tex {
            Some(t) => { p.image(t.id(), rect.shrink(2.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE); }
            None => { p.text(rect.center(), egui::Align2::CENTER_CENTER, "…", egui::FontId::proportional(18.0), theme::TEXT2); }
        }
        if !label.is_empty() {
            let lr = egui::Rect::from_min_max(egui::pos2(rect.left() + 2.0, rect.bottom() - 20.0), rect.right_bottom() - egui::vec2(2.0, 2.0));
            p.rect_filled(lr, 4.0, egui::Color32::from_black_alpha(150));
            p.text(lr.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(12.0), theme::TEXT);
        }
        p.rect_stroke(rect, 6.0, egui::Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { theme::ACCENT } else { theme::LINE }), egui::StrokeKind::Inside);
        resp.clicked()
    }

    pub(crate) fn eye_studio(&mut self, ui: &mut egui::Ui) {
        if self.session.is_none() { return; }
        ui.horizontal(|ui| { for (i, t) in ["Form", "Iris", "Pupille", "Material"].iter().enumerate() { if ui.selectable_label(self.eye_tab == i, *t).clicked() { self.eye_tab = i; } } });
        {
            let s = self.session.as_mut().unwrap();
            let mut linked = s.state.style.eyes.linked;
            ui.horizontal(|ui| {
                if ui.checkbox(&mut linked, "Links und rechts gemeinsam bearbeiten").changed() { s.edit_style(1, |st| { st.eyes.linked = linked; if linked { st.eyes.right = st.eyes.left.clone(); } }); }
            });
            if !linked && self.eye_tab != 0 {
                ui.horizontal(|ui| { for (i, t) in ["Linkes Auge", "Rechtes Auge"].iter().enumerate() { if ui.selectable_label(self.eye_side == i, *t).clicked() { self.eye_side = i; } } });
            }
        }
        ui.separator();
        match self.eye_tab {
            0 => {
                ui.weak("Presets setzen die Augenform-Parameter (danach frei anpassbar).");
                let base = self.session.as_ref().unwrap().state.clone();
                let mut apply: Option<BTreeMap<String, f32>> = None;
                let tile_w = ((ui.available_width() - 16.0) / 3.0).floor().min(120.0);
                ui.horizontal_wrapped(|ui| {
                    for (name, vals) in EYE_PRESETS {
                        let map: BTreeMap<String, f32> = vals.iter().flat_map(|(k, v)| [(format!("mh:l-{k}"), *v), (format!("mh:r-{k}"), *v)]).collect();
                        let mut st = self.session.as_ref().unwrap().state_with(&BTreeMap::new());
                        st.values = base.values.clone();
                        // Preset ersetzt die Augenwerte vollständig (Basis: Augenparameter auf 0)
                        let s = self.session.as_ref().unwrap();
                        for (i, d) in s.params.defs.iter().enumerate() { if d.id.contains("-eye-") { st.values[i] = 0.0; } }
                        let st = { let mut x = st; let s = self.session.as_ref().unwrap(); for (k, v) in &map { if let Some(i) = s.params.canonical(k) { x.values[i] = *v; } } x };
                        if self.tile(ui, egui::vec2(tile_w, tile_w * 0.8), st, "eye", name, false) { apply = Some(map); }
                    }
                });
                if let Some(m) = apply {
                    let s = self.session.as_mut().unwrap();
                    let before = s.state.clone();
                    let idx: Vec<usize> = s.params.defs.iter().enumerate().filter(|(_, d)| d.id.contains("-eye-")).map(|(i, _)| i).collect();
                    for i in idx { s.state.values[i] = 0.0; }
                    s.apply_map(&m);
                    let _ = before;
                    self.reg_key = (usize::MAX, 0);
                }
                ui.add_space(6.0);
                self.region_sliders(ui, Some(crate::regions::REGIONS.iter().position(|r| r.label == "Augen").unwrap()), false);
            }
            1 => {
                let s = self.session.as_mut().unwrap();
                let side_idx = if s.state.style.eyes.linked { 0 } else { self.eye_side };
                let cur = if side_idx == 0 { s.state.style.eyes.left.clone() } else { s.state.style.eyes.right.clone() };
                let mut e = cur.clone();
                ui.checkbox(&mut e.custom, "Eigene Irisfarben (statt Textur-Farbe)");
                if e.custom {
                    egui::Grid::new("iris").num_columns(2).show(ui, |ui| {
                        ui.label("Iris innen"); ui.color_edit_button_rgb(&mut e.inner); ui.end_row();
                        ui.label("Iris außen"); ui.color_edit_button_rgb(&mut e.outer); ui.end_row();
                    });
                    ui.weak("Farbverlauf von der Pupille (innen) zum Irisrand (außen); die Texturzeichnung bleibt als Struktur erhalten.");
                } else {
                    ui.label("Augenfarbe (Textur)");
                }
                if e != cur { let side = side_idx; let linked = s.state.style.eyes.linked; s.edit_style(2 + side, |st| { if side == 0 { st.eyes.left = e.clone(); if linked { st.eyes.right = e.clone(); } } else { st.eyes.right = e.clone(); } }); }
                if !e.custom {
                    let colors: Vec<String> = s.assets.eye_colors.iter().map(|(n, _)| n.clone()).collect();
                    let mut pick = None;
                    ui.horizontal_wrapped(|ui| { for n in &colors { if ui.selectable_label(s.state.eyes == *n, n.as_str()).clicked() { pick = Some(n.clone()); } } });
                    if let Some(n) = pick {
                        s.state.eyes = n.clone();
                        let p = s.assets.eye_colors.iter().find(|(x, _)| *x == n).map(|x| x.1.clone());
                        let mh = s.repo.join("content/testcandidate/downloads/eyes/high-poly/high-poly.mhclo");
                        let ctx = ui.ctx().clone();
                        self.start_asset_load(&ctx, crate::session::EYES_ID, mh, p);
                    }
                }
            }
            2 => {
                let s = self.session.as_mut().unwrap();
                let side_idx = if s.state.style.eyes.linked { 0 } else { self.eye_side };
                let mut v = if side_idx == 0 { s.state.style.eyes.left.pupil } else { s.state.style.eyes.right.pupil };
                let old = v;
                ui.add(egui::Slider::new(&mut v, 0.4..=2.2).text("Pupillengröße (Faktor)"));
                if (v - old).abs() > 1e-6 { let linked = s.state.style.eyes.linked; s.edit_style(6 + side_idx, |st| { if side_idx == 0 { st.eyes.left.pupil = v; if linked { st.eyes.right.pupil = v; } } else { st.eyes.right.pupil = v; } }); }
                ui.weak("Verändert die Pupille in der Iris-Textur (Dilatation). Pupillenform (Schlitz/Herz …) ist noch nicht unterstützt.");
            }
            _ => {
                let s = self.session.as_mut().unwrap();
                let (mut refl, mut limb) = (s.state.style.eyes.reflection, s.state.style.eyes.limbal);
                let (o1, o2) = (refl, limb);
                ui.add(egui::Slider::new(&mut refl, 0.0..=1.0).text("Hornhautglanz / Reflexion"));
                ui.add(egui::Slider::new(&mut limb, 0.0..=1.0).text("Limbusring (Irisrand-Abdunklung)"));
                if refl != o1 { s.edit_style(10, |st| st.eyes.reflection = refl); }
                if limb != o2 { s.edit_style(11, |st| st.eyes.limbal = limb); }
                ui.weak("Augenweiß-Farbe, Augen-Make-up und Irismuster: noch nicht unterstützt.");
            }
        }
    }

    /// Frisuren-Auswahl als Kachelraster mit Vorschaubildern (virtualisiert).
    pub(crate) fn hair_grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| { ui.label("Suche:"); ui.text_edit_singleline(&mut self.asset_filter); });
        let f = self.asset_filter.to_lowercase();
        let list: Vec<usize> = { let s = self.session.as_ref().unwrap(); s.assets.assets.iter().enumerate().filter(|(_, a)| a.category == Category::Hair && (f.is_empty() || a.name.to_lowercase().contains(&f))).map(|(i, _)| i).collect() };
        ui.weak(format!("{} Frisuren", list.len()));
        let cols = 3usize; let tile = ((ui.available_width() - 24.0) / cols as f32).floor().min(150.0);
        let rows = list.len().div_ceil(cols);
        let mut click: Option<usize> = None;
        egui::ScrollArea::vertical().id_salt("hairgrid").auto_shrink([false, false]).show_rows(ui, tile + 6.0, rows, |ui, range| {
            for r in range {
                ui.horizontal(|ui| {
                    for c in 0..cols {
                        let Some(&idx) = list.get(r * cols + c) else { break };
                        let (e, on, st) = {
                            let s = self.session.as_ref().unwrap();
                            let e = s.assets.assets[idx].clone();
                            let mut st = s.state.clone();
                            st.assets.retain(|a| !a.id.starts_with("hair/"));
                            st.assets.push(hase_core::state::AssetSel { id: e.id.clone(), tint: [0.9; 3] });
                            (e.clone(), s.is_selected(&e.id), st)
                        };
                        if self.tile(ui, egui::vec2(tile, tile), st, "hair", &e.name.replace('_', " "), on) { click = Some(idx); }
                    }
                });
            }
        });
        if let Some(i) = click {
            let s = self.session.as_mut().unwrap();
            let e = s.assets.assets[i].clone();
            if s.is_selected(&e.id) { s.toggle_asset_off(&e.id); } else { s.add_asset_selection(&e); self.start_asset_load(ctx, &e.id, e.mhclo.clone(), None); }
        }
    }

    pub(crate) fn hair_color_panel(&mut self, ui: &mut egui::Ui) {
        let s = self.session.as_mut().unwrap();
        let h = s.state.style.hair.clone();
        let mut n = h.clone();
        ui.strong("Farbvorlagen");
        let mut pick: Option<usize> = None;
        ui.horizontal_wrapped(|ui| {
            for (i, (name, root, tip)) in HAIR_COLORS.iter().enumerate() {
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::click());
                let p = ui.painter();
                let c = |c: &[f32; 3]| egui::Color32::from_rgb((c[0].powf(1.0 / 2.2) * 255.0) as u8, (c[1].powf(1.0 / 2.2) * 255.0) as u8, (c[2].powf(1.0 / 2.2) * 255.0) as u8);
                p.rect_filled(rect, 6.0, c(root));
                p.rect_filled(egui::Rect::from_min_max(egui::pos2(rect.left(), rect.center().y), rect.right_bottom()), egui::CornerRadius { nw: 0, ne: 0, sw: 6, se: 6 }, c(tip));
                p.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, theme::LINE), egui::StrokeKind::Inside);
                if resp.on_hover_text(*name).clicked() { pick = Some(i); }
            }
        });
        if let Some(i) = pick { n.root = HAIR_COLORS[i].1; n.tip = HAIR_COLORS[i].2; }
        ui.add_space(6.0);
        egui::Grid::new("haircol").num_columns(2).show(ui, |ui| {
            ui.label("Ansatzfarbe (Kopfhaut)"); ui.color_edit_button_rgb(&mut n.root); ui.end_row();
            ui.label("Spitzenfarbe"); ui.color_edit_button_rgb(&mut n.tip); ui.end_row();
            ui.label("Strähnen-/Highlightfarbe"); ui.color_edit_button_rgb(&mut n.highlight); ui.end_row();
        });
        ui.add(egui::Slider::new(&mut n.highlight_amt, 0.0..=1.0).text("Highlight-Anteil"));
        ui.add(egui::Slider::new(&mut n.gloss, 0.0..=1.0).text("Glanz"));
        ui.weak("Der Verlauf läuft von der Kopfhaut zur Haarspitze (über die Haarhöhe). Rauheit/Transparenz als eigene Regler: noch nicht unterstützt.");
        if n != h { s.edit_style(20, |st| st.hair = n.clone()); }
    }

    pub(crate) fn skin_style_panel(&mut self, ui: &mut egui::Ui) {
        let s = self.session.as_mut().unwrap();
        let mut g = s.state.style.skin.gloss; let old = g;
        ui.add(egui::Slider::new(&mut g, 0.0..=1.0).text("Hautglanz"));
        if g != old { s.edit_style(30, |st| st.skin.gloss = g); }
    }
}
