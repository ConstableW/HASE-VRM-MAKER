//! Augenstudio, Haarstudio, Haut-/Stilbereiche und gemeinsame Kachel-Komponenten (mit Thumbnails).
use crate::app::App;
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
    /// Vorschaukachel: zeigt ein (ggf. noch gerendertes) Thumbnail des Zustands `st`; Klick liefert `true`.
    pub(crate) fn tile(&mut self, ui: &mut egui::Ui, size: egui::Vec2, st: CharacterState, cam_kind: &str, label: &str, selected: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        let Some(s) = self.session.as_ref() else { return false };
        let (w, h) = (192u32, (192.0 * size.y / size.x) as u32);
        let key = make_key(&st, &s.packages.body.digest, cam_kind, w, h, &s.params);
        let body = s.packages.body.clone();
        let ctx = ui.ctx().clone();
        let tex = self.thumbs.get(&ctx, &key, || ThumbJob { key: key.clone(), state: st, cam_kind: cam_kind.to_string(), w, h, body, avatar: None });
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

    /// Kachel mit Beschriftung unter dem Vorschaubild (Presetleiste, Spezies-Karte).
    pub(crate) fn tile_card(&mut self, ui: &mut egui::Ui, w: f32, h: f32, st: CharacterState, cam_kind: &str, label: &str, selected: bool) -> bool {
        let lab_h = if label.is_empty() { 0.0 } else { 20.0 };
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h + lab_h), egui::Sense::click());
        let Some(s) = self.session.as_ref() else { return false };
        let (tw, th) = (192u32, (192.0 * h / w) as u32);
        let key = make_key(&st, &s.packages.body.digest, cam_kind, tw, th, &s.params);
        let body = s.packages.body.clone();
        let ctx = ui.ctx().clone();
        let tex = self.thumbs.get(&ctx, &key, || ThumbJob { key: key.clone(), state: st, cam_kind: cam_kind.to_string(), w: tw, h: th, body, avatar: None });
        let p = ui.painter();
        let img = egui::Rect::from_min_size(rect.min, egui::vec2(w, h));
        p.rect_filled(img, 7.0, if selected { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::PANEL2 });
        match tex { Some(t) => { p.image(t.id(), img.shrink(2.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE); } None => { p.text(img.center(), egui::Align2::CENTER_CENTER, "…", egui::FontId::proportional(18.0), theme::TEXT2); } }
        p.rect_stroke(img, 7.0, egui::Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { theme::ACCENT } else if resp.hovered() { theme::TEXT2 } else { theme::LINE }), egui::StrokeKind::Inside);
        if !label.is_empty() { let mut job = egui::text::LayoutJob::simple_singleline(label.to_string(), egui::FontId::proportional(12.5), if selected { egui::Color32::WHITE } else { theme::TEXT }); job.wrap = egui::text::TextWrapping { max_width: w - 4.0, max_rows: 1, break_anywhere: true, overflow_character: Some('…') }; let g = p.layout_job(job); let pos = egui::pos2(rect.center().x - g.size().x / 2.0, rect.bottom() - 9.0 - g.size().y / 2.0); p.galley(pos, g, theme::TEXT); }
        resp.clicked()
    }

    pub(crate) fn eye_studio(&mut self, ui: &mut egui::Ui) {
        if self.session.is_none() { return; }
        ui.horizontal(|ui| { for (i, t) in ["Form", "Iris", "Pupille", "Material"].iter().enumerate() { if ui.selectable_label(self.eye_tab == i, *t).clicked() { self.eye_tab = i; } } });
        {
            let s = self.session.as_mut().unwrap();
            let mut linked = s.state.style.eyes.linked;
            if ui.checkbox(&mut linked, "Links und rechts gemeinsam bearbeiten").changed() { s.edit_style(1, |st| { st.eyes.linked = linked; if linked { st.eyes.right = st.eyes.left.clone(); } }); }
            if !linked {
                let mut side = self.eye_side;
                if crate::ui_kit::segmented(ui, &["Linkes Auge", "Rechtes Auge"], &mut side, 130.0) { self.eye_side = side; }
            }
        }
        ui.separator();
        match self.eye_tab {
            0 => {
                ui.weak("Presets setzen die Augenform-Parameter (danach frei anpassbar).");
                let base = self.session.as_ref().unwrap().state.clone();
                let (linked, side) = { let e = &self.session.as_ref().unwrap().state.style.eyes; (e.linked, self.eye_side) };
                let mut apply: Option<BTreeMap<String, f32>> = None;
                let tile_w = ((ui.available_width() - 16.0) / 3.0).floor().min(120.0);
                ui.horizontal_wrapped(|ui| {
                    for (name, vals) in EYE_PRESETS {
                        let map: BTreeMap<String, f32> = vals.iter().flat_map(|(k, v)| [(format!("mh:l-{k}"), *v), (format!("mh:r-{k}"), *v)]).filter(|(k, _)| linked || (side == 0 && k.starts_with("mh:l-")) || (side == 1 && k.starts_with("mh:r-"))).collect();
                        let mut st = self.session.as_ref().unwrap().state_with(&BTreeMap::new());
                        st.values = base.values.clone();
                        // Preset ersetzt die Augenwerte vollständig (Basis: Augenparameter auf 0)
                        let s = self.session.as_ref().unwrap();
                        for (i, d) in s.params.defs.iter().enumerate() { if d.id.contains("-eye-") && (linked || (side == 0 && d.id.starts_with("mh:l-")) || (side == 1 && d.id.starts_with("mh:r-"))) { st.values[i] = 0.0; } }
                        let st = { let mut x = st; let s = self.session.as_ref().unwrap(); for (k, v) in &map { if let Some(i) = s.params.canonical(k) { x.values[i] = *v; } } x };
                        if self.tile(ui, egui::vec2(tile_w, tile_w * 0.8), st, "eye", name, false) { apply = Some(map); }
                    }
                });
                if let Some(m) = apply {
                    let s = self.session.as_mut().unwrap();
                    let before = s.state.clone();
                    let idx: Vec<usize> = s.params.defs.iter().enumerate().filter(|(_, d)| d.id.contains("-eye-") && (linked || (side == 0 && d.id.starts_with("mh:l-")) || (side == 1 && d.id.starts_with("mh:r-")))).map(|(i, _)| i).collect();
                    for i in idx { s.state.values[i] = 0.0; }
                    s.apply_map(&m);
                    let _ = before;
                    self.reg_key = (usize::MAX, 0);
                }
                ui.add_space(6.0);
                self.side_filter = if linked { 1 } else { side as u8 + 1 };
                self.region_sliders(ui, Some(crate::regions::REGIONS.iter().position(|r| r.label == "Augen").unwrap()), false, false);
                self.side_filter = 0;
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
                crate::ui_kit::slider(ui, "Pupillengröße (Faktor)", &mut v, 0.4..=2.2);
                if (v - old).abs() > 1e-6 { let linked = s.state.style.eyes.linked; s.edit_style(6 + side_idx, |st| { if side_idx == 0 { st.eyes.left.pupil = v; if linked { st.eyes.right.pupil = v; } } else { st.eyes.right.pupil = v; } }); }
                ui.weak("Verändert die Pupille in der Iris-Textur (Dilatation). Pupillenform (Schlitz/Herz …) ist noch nicht unterstützt.");
            }
            _ => {
                let s = self.session.as_mut().unwrap();
                let (mut refl, mut limb) = (s.state.style.eyes.reflection, s.state.style.eyes.limbal);
                let (o1, o2) = (refl, limb);
                crate::ui_kit::slider(ui, "Hornhautglanz / Reflexion", &mut refl, 0.0..=1.0);
                crate::ui_kit::slider(ui, "Limbusring (Irisrand-Abdunklung)", &mut limb, 0.0..=1.0);
                if refl != o1 { s.edit_style(10, |st| st.eyes.reflection = refl); }
                if limb != o2 { s.edit_style(11, |st| st.eyes.limbal = limb); }
                ui.weak("Augenweiß-Farbe, Augen-Make-up und Irismuster: noch nicht unterstützt.");
            }
        }
    }

    /// Kleidungsauswahl als Kachelraster mit Vorschaubildern + Liste der getragenen Teile (Farbe, Ablegen).
    pub(crate) fn clothes_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, kind: usize) {
        ui.horizontal(|ui| { crate::icons::show(ui, crate::icons::Icon::Search, 16.0, theme::TEXT2); ui.add(egui::TextEdit::singleline(&mut self.asset_filter).hint_text("Suche Kleidung …").desired_width(f32::INFINITY)); });
        let f = self.asset_filter.to_lowercase();
        let list: Vec<usize> = { let s = self.session.as_ref().unwrap(); s.assets.assets.iter().enumerate().filter(|(_, a)| a.category == Category::Clothes && crate::regions::clothes_kind(&a.name) == kind && (f.is_empty() || a.name.to_lowercase().contains(&f))).map(|(i, _)| i).collect() };
        ui.colored_label(theme::TEXT2, format!("{} Teile (nur Assets mit geklärter Lizenz)", list.len()));
        ui.add_space(4.0);
        let cols = 3usize; let tile = ((ui.available_width() - 2.0 * 8.0) / cols as f32).floor();
        let cam = ["full", "upper", "lower", "feet", "upper"][kind.min(4)];
        let mut click: Option<usize> = None;
        let rows = list.len().div_ceil(cols);
        for r in 0..rows {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                for c in 0..cols {
                    let Some(&idx) = list.get(r * cols + c) else { break };
                    let (e, on, st) = {
                        let s = self.session.as_ref().unwrap();
                        let e = s.assets.assets[idx].clone();
                        let mut st = s.state.clone();
                        st.assets.retain(|a| !a.id.starts_with("clothes/"));
                        st.assets.push(hase_core::state::AssetSel { id: e.id.clone(), tint: crate::session::default_tint(&e.category, &e.name) });
                        (e.clone(), s.is_selected(&e.id), st)
                    };
                    let busy = self.loading.contains(&e.id);
                    if self.tile(ui, egui::vec2(tile, tile * 1.15), st, cam, &e.name.replace('_', " "), on) && !busy { click = Some(idx); }
                }
            });
            ui.add_space(4.0);
        }
        let worn: Vec<hase_core::state::AssetSel> = self.session.as_ref().unwrap().state.assets.iter().filter(|a| a.id.starts_with("clothes/")).cloned().collect();
        crate::ui_kit::hr(ui);
        crate::ui_kit::section(ui, "Getragen");
        if worn.is_empty() { ui.colored_label(theme::TEXT2, "Noch nichts ausgewählt – Kachel anklicken."); }
        let mut tint: Option<(String, [f32; 3])> = None; let mut off: Option<String> = None;
        for a in &worn {
            ui.horizontal(|ui| {
                let mut t = a.tint;
                if ui.color_edit_button_rgb(&mut t).changed() { tint = Some((a.id.clone(), t)); }
                ui.label(a.id.rsplit('/').next().unwrap_or(&a.id).replace('_', " "));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if crate::ui_kit::icon_tile(ui, crate::icons::Icon::Close, "", egui::vec2(24.0, 24.0), false, "Ablegen").clicked() { off = Some(a.id.clone()); } });
            });
        }
        let s = self.session.as_mut().unwrap();
        if let Some((id, t)) = tint { s.set_tint(&id, t); }
        if let Some(id) = off { s.toggle_asset_off(&id); }
        if let Some(i) = click {
            let e = self.session.as_ref().unwrap().assets.assets[i].clone();
            let s = self.session.as_mut().unwrap();
            if s.is_selected(&e.id) { s.toggle_asset_off(&e.id); } else { s.add_asset_selection(&e); self.start_asset_load(ctx, &e.id, e.mhclo.clone(), None); }
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

    pub(crate) fn hair_shape_panel(&mut self, ui: &mut egui::Ui) {
        let s = self.session.as_mut().unwrap();
        let h = s.state.style.hair.clone();
        let mut n = h.clone();
        crate::ui_kit::slider(ui, "Länge (kürzer ↔ länger)", &mut n.length, -0.6..=0.8);
        crate::ui_kit::slider(ui, "Volumen Oberkopf", &mut n.vol_top, -0.8..=1.5);
        crate::ui_kit::slider(ui, "Volumen Vorderseite / Pony", &mut n.vol_front, -0.8..=1.5);
        crate::ui_kit::slider(ui, "Volumen Seiten", &mut n.vol_side, -0.8..=1.5);
        crate::ui_kit::slider(ui, "Volumen Hinterkopf", &mut n.vol_back, -0.8..=1.5);
        if ui.button("Form zurücksetzen").clicked() { n.length = 0.0; n.vol_top = 0.0; n.vol_front = 0.0; n.vol_side = 0.0; n.vol_back = 0.0; }
        ui.weak("Verformt die angepasste Frisur (Länge relativ zur Kopfhaut, Volumen entlang der Haaroberfläche). Kollision mit Schultern/Körper wird nicht berechnet; Scheitel, Strähnen, Locken und Wellen: noch nicht unterstützt.");
        if n != h { s.edit_style(21, |st| st.hair = n.clone()); }
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
        crate::ui_kit::slider(ui, "Highlight-Anteil", &mut n.highlight_amt, 0.0..=1.0);
        crate::ui_kit::slider(ui, "Glanz", &mut n.gloss, 0.0..=1.0);
        ui.weak("Der Verlauf läuft von der Kopfhaut zur Haarspitze (über die Haarhöhe). Rauheit/Transparenz als eigene Regler: noch nicht unterstützt.");
        if n != h { s.edit_style(20, |st| st.hair = n.clone()); }
    }

    pub(crate) fn makeup_panel(&mut self, ui: &mut egui::Ui) {
        let s = self.session.as_mut().unwrap();
        if s.generic { ui.colored_label(theme::WARN, "Make-up-Bereiche (Lippen, Lider, Wangen) benötigen Masken aus Gesichts-Morphs; für importierte Körperbasen noch nicht verfügbar. Details (Muttermale/Tattoos/Narben) funktionieren."); }
        let cur = s.state.style.skin.clone();
        let mut n = cur.clone();
        for (label, l) in [("Lippenfarbe", &mut n.lips), ("Lidschatten", &mut n.shadow), ("Wangenrouge", &mut n.blush)] {
            ui.horizontal(|ui| { ui.color_edit_button_rgb(&mut l.color); crate::ui_kit::slider(ui, label, &mut l.amount, 0.0..=1.0); });
        }
        ui.weak("Die Bereiche folgen der Gesichtsform (Masken aus den Morph-Fußabdrücken von Lippen, Lidfalten und Wangen). Eyeliner, Glanz-/Matt-Lippen und Make-up-Presets: noch nicht unterstützt.");
        if n != cur { s.edit_style(40, |st| st.skin = n.clone()); }
    }

    pub(crate) fn details_panel(&mut self, ui: &mut egui::Ui) {
        let cur = self.session.as_ref().unwrap().state.style.skin.clone();
        let mut n = cur.clone();
        ui.strong("Sommersprossen");
        ui.horizontal(|ui| { ui.color_edit_button_rgb(&mut n.freckles.color); crate::ui_kit::slider(ui, "Dichte", &mut n.freckles.amount, 0.0..=1.0); });
        crate::ui_kit::slider(ui, "Größe", &mut n.freckle_size, 0.4..=2.5);
        if ui.button("Neu verteilen").clicked() { n.freckle_seed = n.freckle_seed.wrapping_mul(1664525).wrapping_add(1013904223) >> 8; }
        ui.separator();
        ui.strong("Muttermale, Tattoos, Narben");
        ui.horizontal_wrapped(|ui| { for (i, nm) in hase_core::state::DECAL_SHAPES.iter().enumerate() { if ui.selectable_label(self.decal_shape == i as u8, *nm).clicked() { self.decal_shape = i as u8; } } });
        let on = self.tool == crate::app::Tool::Decal;
        if ui.selectable_label(on, if on { "Platzieren aktiv – Klick auf den Körper" } else { "Am Modell platzieren …" }).clicked() { self.tool = if on { crate::app::Tool::Orbit } else { crate::app::Tool::Decal }; }
        ui.weak("Das Motiv wird an der angeklickten Hautstelle in die Hauttextur eingerechnet.");
        let mut del = None;
        let mut sel = self.decal_sel;
        for (i, d) in n.decals.iter_mut().enumerate() {
            let name = hase_core::state::DECAL_SHAPES[d.shape as usize];
            ui.horizontal(|ui| {
                if ui.selectable_label(sel == Some(i), format!("{} {}", name, i + 1)).clicked() { sel = Some(i); }
                if ui.small_button("Entfernen").clicked() { del = Some(i); }
            });
            if sel == Some(i) {
                ui.indent(("decal", i), |ui| {
                    ui.horizontal(|ui| { ui.color_edit_button_rgb(&mut d.color); crate::ui_kit::slider(ui, "Deckkraft", &mut d.opacity, 0.05..=1.0); });
                    ui.add(egui::Slider::new(&mut d.size, 0.002..=0.06).logarithmic(true).text("Größe"));
                    crate::ui_kit::slider(ui, "Drehung", &mut d.rot, -3.14..=3.14);
                });
            }
        }
        self.decal_sel = sel;
        if let Some(i) = del { n.decals.remove(i); self.decal_sel = None; }
        if n != cur { let s = self.session.as_mut().unwrap(); s.edit_style(41, |st| st.skin = n.clone()); }
    }

    pub(crate) fn skin_style_panel(&mut self, ui: &mut egui::Ui) {
        let s = self.session.as_mut().unwrap();
        let mut g = s.state.style.skin.gloss; let old = g;
        crate::ui_kit::slider(ui, "Hautglanz", &mut g, 0.0..=1.0);
        if g != old { s.edit_style(30, |st| st.skin.gloss = g); }
    }
}
