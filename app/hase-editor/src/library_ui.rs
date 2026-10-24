//! Bibliotheks-Oberfläche: Charakterbibliothek-Fenster (Karten mit Thumbnails, Vorlagen, Favoriten, Varianten, Snapshots, Vergleich)
//! und die untere Preset-/Charakterleiste.
use crate::app::App;
use crate::library::{self, LibEntry};
use crate::theme;
use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct LibState {
    pub open: bool,
    pub entries: Vec<LibEntry>,
    pub loaded: bool,
    pub tab: usize,
    pub filter: String,
    pub sel: Option<String>,
    pub current: Option<String>,
    pub new_name: String,
    pub rename_buf: String,
    pub snap_label: String,
    pub confirm_delete: bool,
    pub compare: bool,
    pub msg: String,
    pub tex: HashMap<PathBuf, egui::TextureHandle>,
    pub pending_thumbs: Vec<(String, PathBuf)>,
    pub bottom_tab: usize,
    pub bottom_filter: String,
}

impl App {
    pub(crate) fn lib_refresh(&mut self) {
        if let Some(s) = self.session.as_ref() { self.lib.entries = library::list(&s.repo); self.lib.loaded = true; }
    }

    /// Thumbnail-Datei in den Bibliotheksordner kopieren, sobald der Worker sie gerendert hat.
    pub(crate) fn lib_pending(&mut self) {
        let Some(s) = self.session.as_ref() else { return };
        let dir = s.repo.join(".hase-cache/thumbs");
        let mut done = Vec::new();
        for (i, (key, target)) in self.lib.pending_thumbs.iter().enumerate() {
            let f = dir.join(format!("{key}.png"));
            if f.exists() && std::fs::copy(&f, target.join("thumb.png")).is_ok() { done.push(i); }
        }
        for i in done.into_iter().rev() { let (_, t) = self.lib.pending_thumbs.remove(i); self.lib.tex.remove(&t.join("thumb.png")); }
    }

    fn lib_texture(&mut self, ctx: &egui::Context, dir: &std::path::Path) -> Option<egui::TextureHandle> {
        let f = dir.join("thumb.png");
        if let Some(t) = self.lib.tex.get(&f) { return Some(t.clone()); }
        let img = image::open(&f).ok()?.to_rgba8();
        let t = ctx.load_texture(f.display().to_string(), egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw()), egui::TextureOptions::LINEAR);
        self.lib.tex.insert(f, t.clone());
        Some(t)
    }

    fn request_thumb_for(&mut self, ctx: &egui::Context, dir: PathBuf) {
        let Some(s) = self.session.as_ref() else { return };
        let st = s.state.clone();
        let (w, h) = (192u32, 192u32);
        let key = crate::thumbs::make_key(&st, &s.packages.body.digest, "head", w, h, &s.params);
        let body = s.packages.body.clone();
        let _ = self.thumbs.get(ctx, &key, || crate::thumbs::ThumbJob { key: key.clone(), state: st, cam_kind: "head".into(), w, h, body });
        self.lib.pending_thumbs.push((key, dir));
    }

    fn character_json(&self) -> Option<serde_json::Value> { self.session.as_ref().map(|s| s.state.to_json(&s.params, &s.packages.body.id)) }

    pub(crate) fn lib_save_new(&mut self, ctx: &egui::Context, name: &str, template: bool) {
        let (Some(json), Some(repo)) = (self.character_json(), self.session.as_ref().map(|s| s.repo.clone())) else { return };
        match library::create(&repo, name, &json, template, None) {
            Ok(e) => { self.lib.current = Some(e.id.clone()); self.lib.sel = Some(e.id.clone()); self.request_thumb_for(ctx, e.dir.clone()); self.lib.msg = format!("„{name}“ gespeichert"); self.lib_refresh(); }
            Err(e) => self.lib.msg = e,
        }
    }

    pub(crate) fn lib_save_current(&mut self, ctx: &egui::Context) {
        let Some(id) = self.lib.current.clone() else { return };
        let Some(json) = self.character_json() else { return };
        if let Some(e) = self.lib.entries.iter_mut().find(|e| e.id == id) {
            match library::update(e, &json) { Ok(()) => { let d = e.dir.clone(); self.request_thumb_for(ctx, d); self.lib.msg = "Gespeichert".into(); } Err(x) => self.lib.msg = x }
        }
        self.lib_refresh();
    }

    pub(crate) fn lib_open(&mut self, e: &LibEntry) {
        match library::load(e) { Ok(v) => { self.lib.current = Some(e.id.clone()); self.continue_load(v, e.name.clone()); } Err(x) => self.lib.msg = x }
    }

    pub(crate) fn lib_window(&mut self, ctx: &egui::Context) {
        if !self.lib.open { return; }
        if !self.lib.loaded { self.lib_refresh(); }
        let mut open = true;
        egui::Window::new("Charakterbibliothek").open(&mut open).default_size([760.0, 560.0]).resizable(true).show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, t) in ["Meine Charaktere", "Vorlagen", "Favoriten"].iter().enumerate() { if ui.selectable_label(self.lib.tab == i, *t).clicked() { self.lib.tab = i; } }
                ui.separator(); ui.label("Suche:"); ui.text_edit_singleline(&mut self.lib.filter);
            });
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.lib.new_name);
                let ok = !self.lib.new_name.trim().is_empty();
                if ui.add_enabled(ok, egui::Button::new("Aktuellen Charakter speichern")).clicked() { let n = self.lib.new_name.trim().to_string(); self.lib_save_new(ctx, &n, false); self.lib.new_name.clear(); }
                if ui.add_enabled(ok, egui::Button::new("Als Vorlage speichern")).on_hover_text("Basischarakter ohne Outfit-Bezug als wiederverwendbare Vorlage").clicked() { let n = self.lib.new_name.trim().to_string(); self.lib_save_new(ctx, &n, true); self.lib.new_name.clear(); }
                if let Some(cur) = self.lib.current.clone() { if ui.button("Änderungen speichern").on_hover_text("Überschreibt den geöffneten Bibliothekseintrag").clicked() { self.lib_save_current(ctx); } ui.weak(format!("geöffnet: {}", self.lib.entries.iter().find(|e| e.id == cur).map(|e| e.name.clone()).unwrap_or_default())); }
            });
            if !self.lib.msg.is_empty() { ui.colored_label(theme::TEXT2, &self.lib.msg); }
            ui.separator();
            let f = self.lib.filter.to_lowercase();
            let list: Vec<LibEntry> = self.lib.entries.iter().filter(|e| (self.lib.tab == 0 || (self.lib.tab == 1 && e.template) || (self.lib.tab == 2 && e.favorite)) && (f.is_empty() || e.name.to_lowercase().contains(&f))).cloned().collect();
            ui.columns(2, |cols| {
                // Karten
                egui::ScrollArea::vertical().id_salt("libcards").auto_shrink([false; 2]).show(&mut cols[0], |ui| {
                    if list.is_empty() { ui.weak("Keine Einträge. Speichere den aktuellen Charakter, um die Bibliothek zu füllen."); }
                    ui.horizontal_wrapped(|ui| {
                        for e in &list {
                            let sel = self.lib.sel.as_deref() == Some(e.id.as_str());
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(108.0, 140.0), egui::Sense::click());
                            let tex = self.lib_texture(ui.ctx(), &e.dir);
                            let p = ui.painter();
                            p.rect_filled(rect, 6.0, if sel { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::PANEL2 });
                            let ir = egui::Rect::from_min_size(rect.min + egui::vec2(4.0, 4.0), egui::vec2(100.0, 100.0));
                            match tex { Some(t) => { p.image(t.id(), ir, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE); } None => { p.rect_filled(ir, 4.0, theme::BG); p.text(ir.center(), egui::Align2::CENTER_CENTER, "kein Bild", egui::FontId::proportional(11.0), theme::TEXT2); } }
                            p.text(egui::pos2(rect.center().x, rect.bottom() - 26.0), egui::Align2::CENTER_CENTER, &e.name, egui::FontId::proportional(13.0), theme::TEXT);
                            let tag = format!("{}{}{}", if e.template { "Vorlage " } else { "" }, if e.favorite { "Favorit " } else { "" }, if e.parent.is_some() { "Variante" } else { "" });
                            p.text(egui::pos2(rect.center().x, rect.bottom() - 10.0), egui::Align2::CENTER_CENTER, tag, egui::FontId::proportional(10.0), theme::TEXT2);
                            p.rect_stroke(rect, 6.0, egui::Stroke::new(if sel { 2.0 } else { 1.0 }, if sel { theme::ACCENT } else { theme::LINE }), egui::StrokeKind::Inside);
                            if resp.clicked() { self.lib.sel = Some(e.id.clone()); self.lib.rename_buf = e.name.clone(); self.lib.confirm_delete = false; }
                            if resp.double_clicked() { self.lib_open(e); }
                        }
                    });
                });
                // Details
                let ui = &mut cols[1];
                let Some(e) = self.lib.sel.as_ref().and_then(|id| self.lib.entries.iter().find(|e| e.id == *id)).cloned() else { ui.weak("Wähle einen Charakter."); return; };
                ui.heading(&e.name);
                ui.colored_label(theme::TEXT2, format!("Erstellt {} · geändert {}", e.created, e.modified));
                ui.colored_label(theme::TEXT2, format!("Spezies: {} · Körperbasis: {}", if e.species.is_empty() { "–" } else { &e.species }, e.body));
                if let Some(p) = &e.parent { ui.colored_label(theme::TEXT2, format!("Variante von: {}", self.lib.entries.iter().find(|x| x.id == *p).map(|x| x.name.clone()).unwrap_or_else(|| p.clone()))); }
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Öffnen").clicked() { self.lib_open(&e); }
                    if ui.button("Duplizieren (Variante)").clicked() { if let Some(repo) = self.session.as_ref().map(|s| s.repo.clone()) { match library::duplicate(&repo, &e) { Ok(n) => { self.lib.sel = Some(n.id); self.lib.msg = "Variante angelegt".into(); } Err(x) => self.lib.msg = x } self.lib_refresh(); } }
                    if ui.button("Vergleichen").on_hover_text("Mit dem aktuellen Charakter vergleichen").clicked() { self.lib.compare = true; }
                });
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.lib.rename_buf);
                    if ui.button("Umbenennen").clicked() { if let Some(m) = self.lib.entries.iter_mut().find(|x| x.id == e.id) { let n = self.lib.rename_buf.trim().to_string(); if !n.is_empty() { let _ = library::rename(m, &n); } } self.lib_refresh(); }
                });
                ui.horizontal(|ui| {
                    let (mut fav, mut tpl) = (e.favorite, e.template);
                    if ui.checkbox(&mut fav, "Favorit").changed() || ui.checkbox(&mut tpl, "Vorlage").changed() { if let Some(m) = self.lib.entries.iter_mut().find(|x| x.id == e.id) { m.favorite = fav; m.template = tpl; let _ = library::set_flags(m); } }
                });
                ui.separator();
                ui.strong("Snapshots");
                ui.horizontal(|ui| { ui.text_edit_singleline(&mut self.lib.snap_label); if ui.button("Snapshot anlegen").clicked() { let l = if self.lib.snap_label.trim().is_empty() { "snapshot".to_string() } else { self.lib.snap_label.trim().to_string() }; self.lib.msg = match library::snapshot(&e, &l) { Ok(()) => "Snapshot angelegt".into(), Err(x) => x }; self.lib.snap_label.clear(); } });
                for (n, path) in library::snapshots(&e).into_iter().take(8) {
                    ui.horizontal(|ui| { ui.label(&n); if ui.small_button("Wiederherstellen").on_hover_text("Lädt den Snapshot in den Editor (Bibliothekseintrag bleibt unverändert, bis du speicherst)").clicked() { if let Ok(v) = std::fs::read_to_string(&path).map_err(|x| x.to_string()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).map_err(|x| x.to_string())) { self.continue_load(v, format!("Snapshot {n}")); } } });
                }
                ui.separator();
                if !self.lib.confirm_delete { if ui.button("Löschen …").clicked() { self.lib.confirm_delete = true; } }
                else { ui.colored_label(theme::WARN, "Wirklich löschen (inkl. Snapshots)?"); ui.horizontal(|ui| { if ui.button("Ja, löschen").clicked() { let _ = library::delete(&e); self.lib.sel = None; if self.lib.current.as_deref() == Some(e.id.as_str()) { self.lib.current = None; } self.lib.confirm_delete = false; self.lib_refresh(); } if ui.button("Abbrechen").clicked() { self.lib.confirm_delete = false; } }); }
            });
        });
        if !open { self.lib.open = false; }
        self.lib_compare(ctx);
    }

    /// Vorher/Nachher: aktueller Zustand neben einem Bibliothekseintrag (gleiche Körperbasis) mit Parameter-Unterschieden.
    fn lib_compare(&mut self, ctx: &egui::Context) {
        if !self.lib.compare { return; }
        let Some(e) = self.lib.sel.as_ref().and_then(|id| self.lib.entries.iter().find(|e| e.id == *id)).cloned() else { self.lib.compare = false; return };
        let mut open = true;
        egui::Window::new(format!("Vergleich: aktuell ↔ {}", e.name)).open(&mut open).default_size([640.0, 520.0]).show(ctx, |ui| {
            let Some(s) = self.session.as_ref() else { return };
            let other = library::load(&e).ok().and_then(|v| hase_core::state::CharacterState::from_json(&v, &s.params));
            let same_body = e.body == s.packages.body.id;
            let Some(other) = other.filter(|_| same_body) else { ui.colored_label(theme::WARN, "Der Eintrag nutzt eine andere Körperbasis – Vergleich nicht möglich."); return };
            let cur = s.state.clone();
            let mut diffs: Vec<(String, f32, f32)> = s.params.defs.iter().enumerate().filter(|(i, _)| (cur.values[*i] - other.values[*i]).abs() > 1e-3).map(|(i, d)| (crate::theme::label_de(&d.label), cur.values[i], other.values[i])).collect();
            diffs.sort_by(|a, b| (b.1 - b.2).abs().partial_cmp(&(a.1 - a.2).abs()).unwrap());
            ui.horizontal(|ui| {
                ui.vertical(|ui| { ui.strong("Aktuell"); self.tile(ui, egui::vec2(280.0, 330.0), cur, "head", "", false); });
                ui.vertical(|ui| { ui.strong(&e.name); self.tile(ui, egui::vec2(280.0, 330.0), other, "head", "", false); });
            });
            ui.separator();
            ui.label(format!("{} Parameter unterscheiden sich", diffs.len()));
            egui::ScrollArea::vertical().max_height(110.0).show(ui, |ui| { for (n, a, b) in diffs.iter().take(30) { ui.label(format!("{n}: {a:.2} → {b:.2}")); } });
        });
        if !open { self.lib.compare = false; }
    }

    /// Untere Leiste: Presets der Spezies, eigene Charaktere, Favoriten – jeweils mit Vorschaubildern.
    pub(crate) fn preset_bar(&mut self, ui: &mut egui::Ui) {
        if self.session.is_none() { return; }
        if !self.lib.loaded { self.lib_refresh(); }
        ui.horizontal(|ui| {
            for (i, t) in ["Presets", "Meine Charaktere", "Favoriten"].iter().enumerate() { if ui.selectable_label(self.lib.bottom_tab == i, *t).clicked() { self.lib.bottom_tab = i; } }
            ui.separator(); ui.label("Suche:"); ui.add(egui::TextEdit::singleline(&mut self.lib.bottom_filter).desired_width(160.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.button("Bibliothek öffnen").clicked() { self.lib.open = true; } });
        });
        let f = self.lib.bottom_filter.to_lowercase();
        let ctx = ui.ctx().clone();
        egui::ScrollArea::horizontal().id_salt("bottombar").show(ui, |ui| {
            ui.horizontal(|ui| {
                // Neu
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(72.0, 96.0), egui::Sense::click());
                ui.painter().rect_filled(rect, 6.0, if resp.hovered() { theme::HOVER } else { theme::PANEL2 });
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "+\nNeu", egui::FontId::proportional(16.0), theme::TEXT);
                if resp.on_hover_text("Neuer Charakter mit den Standardwerten der Spezies").clicked() { if let Some(s) = self.session.as_mut() { s.new_character(); self.lib.current = None; } }
                match self.lib.bottom_tab {
                    0 => {
                        let Some(s) = self.session.as_ref() else { return };
                        let presets: Vec<(String, BTreeMapF)> = s.species_opt().map(|i| s.packages.species[i].presets.iter().filter(|p| f.is_empty() || p.label.to_lowercase().contains(&f)).map(|p| (p.label.clone(), p.params.clone())).collect()).unwrap_or_default();
                        let traits: Vec<(String, BTreeMapF)> = s.species_opt().map(|i| s.packages.species[i].traits.iter().filter(|t| f.is_empty() || t.label.to_lowercase().contains(&f)).map(|t| (t.label.clone(), t.set.clone())).collect()).unwrap_or_default();
                        if s.species_opt().is_none() { ui.weak("Diese Körperbasis hat noch keine Spezies/Presets."); }
                        for (label, map) in presets.into_iter().chain(traits.into_iter().map(|(l, m)| (format!("+ {l}"), m))) {
                            let st = self.session.as_ref().unwrap().state_with(&map);
                            if self.tile(ui, egui::vec2(96.0, 96.0), st, "head", &label, false) { if let Some(s) = self.session.as_mut() { s.apply_map(&map); } }
                        }
                    }
                    _ => {
                        let list: Vec<LibEntry> = self.lib.entries.iter().filter(|e| (self.lib.bottom_tab == 1 || e.favorite) && (f.is_empty() || e.name.to_lowercase().contains(&f))).cloned().collect();
                        if list.is_empty() { ui.weak("Noch keine Einträge – im Bibliotheksfenster speichern."); }
                        for e in list {
                            let tex = self.lib_texture(&ctx, &e.dir);
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(96.0, 96.0), egui::Sense::click());
                            let p = ui.painter();
                            p.rect_filled(rect, 6.0, if resp.hovered() { theme::HOVER } else { theme::PANEL2 });
                            if let Some(t) = tex { p.image(t.id(), rect.shrink(2.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE); }
                            let lr = egui::Rect::from_min_max(egui::pos2(rect.left() + 2.0, rect.bottom() - 18.0), rect.right_bottom() - egui::vec2(2.0, 2.0));
                            p.rect_filled(lr, 4.0, egui::Color32::from_black_alpha(150)); p.text(lr.center(), egui::Align2::CENTER_CENTER, &e.name, egui::FontId::proportional(11.0), theme::TEXT);
                            p.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, theme::LINE), egui::StrokeKind::Inside);
                            if resp.clicked() { self.lib_open(&e); }
                        }
                    }
                }
            });
        });
    }
}

type BTreeMapF = std::collections::BTreeMap<String, f32>;
