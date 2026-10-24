//! Körperbasis- und Speziesstudio: Bibliothek, Prüfstufen, Werte/Grenzen, Presets, Kompatibilität, Paket speichern.
use crate::app::App;
use crate::regions;
use crate::theme;
use eframe::egui;
use hase_core::packages::{bump_version, slug, Preset, Species};
use hase_core::validate::{self, Stage};

#[derive(Default)]
pub struct StudioState {
    pub sel: usize,
    pub tab: usize,
    pub new_name: String,
    pub preset_name: String,
    pub limit_width: f32,
    pub msg: String,
    pub import: Option<(String, Vec<Stage>)>,
    pub dirty: bool,
    pub body_name: String,
    pub import_path: Option<std::path::PathBuf>,
}

const TABS: [&str; 6] = ["Übersicht", "Körperbasis", "Werte & Grenzen", "Presets", "Kompatibilität", "Paket"];

fn stage_row(ui: &mut egui::Ui, s: &Stage) {
    let (sym, col) = match s.ok { Some(true) => ("OK", egui::Color32::from_rgb(0x6C, 0xC4, 0x7F)), Some(false) => ("FEHLT", theme::ERR), None => ("–", theme::TEXT2) };
    ui.horizontal_top(|ui| { ui.colored_label(col, egui::RichText::new(sym).strong().small()); ui.vertical(|ui| { ui.strong(s.name); ui.colored_label(theme::TEXT2, &s.detail); }); });
}

/// Kennzahl der Parameter, die keiner anatomischen Region zugeordnet sind.
fn unassigned(app: &App) -> usize { app.session.as_ref().map(|s| s.params.defs.iter().filter(|d| d.group != "Makro" && regions::region_of(d).is_none()).count()).unwrap_or(0) }

impl App {
    pub(crate) fn studio_library(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_mut() else { return };
        ui.strong("Speziesbibliothek"); ui.separator();
        let n = s.packages.species.len();
        self.studio.sel = self.studio.sel.min(n.saturating_sub(1));
        egui::ScrollArea::vertical().max_height(ui.available_height() - 190.0).show(ui, |ui| {
            for i in 0..n {
                let sp = &s.packages.species[i];
                let label = format!("{}{}{}", sp.label, if sp.user { "  · eigene" } else { "" }, if sp.stub { "  · Stub" } else { "" });
                if ui.selectable_label(self.studio.sel == i, label).clicked() { self.studio.sel = i; }
            }
        });
        ui.separator();
        ui.weak("Neue Spezies aus dem aktuellen Charakter (bewusste Umwandlung):");
        ui.text_edit_singleline(&mut self.studio.new_name);
        if ui.add_enabled(!self.studio.new_name.trim().is_empty(), egui::Button::new("Als neue Spezies speichern")).clicked() {
            let name = self.studio.new_name.trim().to_string();
            let id = format!("user.species.{}", slug(&name));
            if s.packages.species.iter().any(|x| x.id == id) { self.studio.msg = format!("ID {id} existiert bereits"); }
            else {
                let cur = s.packages.species[s.species_index()].clone();
                let defaults = s.params.defs.iter().enumerate().filter(|(i, d)| (s.state.values[*i] - d.default).abs() > 1e-4).map(|(i, d)| (d.id.clone(), s.state.values[i])).collect();
                let sp = Species { id, label: name.clone(), version: "1.0.0".into(), defaults, limits: Default::default(), traits: vec![], presets: vec![], components: cur.components.clone(), stub: false, user: true, description: format!("Abgeleitet vom Charakter (Basis-Spezies: {}).", cur.label), body_id: s.packages.body.id.clone(), dir: None, digest: String::new() };
                match sp.write(&s.repo, &s.packages.body) {
                    Ok(dir) => { let mut sp = sp; sp.digest = hase_core::packages::file_digest(&dir.join("hase-package.json")); sp.dir = Some(dir.clone()); s.packages.species.push(sp); self.studio.sel = s.packages.species.len() - 1; self.studio.msg = format!("Gespeichert: {}", dir.display()); self.studio.new_name.clear(); }
                    Err(e) => self.studio.msg = e,
                }
            }
        }
        if ui.button("Ausgewählte duplizieren").on_hover_text("Erzeugt eine bearbeitbare eigene Kopie").clicked() {
            let mut sp = s.packages.species[self.studio.sel].clone();
            let base = slug(&sp.label);
            let mut k = 1; let mut id = format!("user.species.{base}-kopie");
            while s.packages.species.iter().any(|x| x.id == id) { k += 1; id = format!("user.species.{base}-kopie-{k}"); }
            sp.id = id; sp.label = format!("{} (Kopie)", sp.label); sp.version = "1.0.0".into(); sp.user = true; sp.stub = false; sp.dir = None;
            sp.description = format!("Kopie von {}. {}", s.packages.species[self.studio.sel].label, sp.description);
            match sp.write(&s.repo, &s.packages.body) { Ok(dir) => { sp.digest = hase_core::packages::file_digest(&dir.join("hase-package.json")); sp.dir = Some(dir); s.packages.species.push(sp); self.studio.sel = s.packages.species.len() - 1; self.studio.msg = "Dupliziert".into(); } Err(e) => self.studio.msg = e }
        }
        if !self.studio.msg.is_empty() { ui.colored_label(theme::TEXT2, &self.studio.msg); }
    }

    pub(crate) fn studio_detail(&mut self, ui: &mut egui::Ui) {
        if self.session.is_none() { return; }
        let un = unassigned(self);
        ui.horizontal_wrapped(|ui| { for (i, t) in TABS.iter().enumerate() { if ui.selectable_label(self.studio.tab == i, *t).clicked() { self.studio.tab = i; } } });
        ui.separator();
        let sel = self.studio.sel;
        let s = self.session.as_mut().unwrap();
        let sel = sel.min(s.packages.species.len() - 1);
        let editable = s.packages.species[sel].user;
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| match self.studio.tab {
            0 => {
                let sp = s.packages.species[sel].clone();
                ui.heading(&sp.label);
                ui.colored_label(theme::TEXT2, format!("{} · Version {}{}", sp.id, sp.version, if sp.user { " · eigene Ressource" } else { " · mitgelieferte Beispiel-Spezies (schreibgeschützt, zum Bearbeiten duplizieren)" }));
                ui.label(&sp.description);
                ui.add_space(8.0);
                ui.label(format!("Körperbasis: {}", s.packages.body.label));
                ui.label(format!("{} Standardwerte · {} Grenzen · {} Presets · {} Merkmale · {} Komponenten", sp.defaults.len(), sp.limits.len(), sp.presets.len(), sp.traits.len(), sp.components.len()));
                ui.add_space(8.0);
                if ui.button("Im Viewport als Charakter-Vorlage ansehen").on_hover_text("Wendet die Standardwerte dieser Spezies auf den aktuellen Charakter an").clicked() { s.apply_species(sel); }
                ui.weak("Ein individueller Charakter ist keine Spezies. Die Umwandlung geschieht bewusst über „Als neue Spezies speichern“ (links).");
            }
            1 => {
                ui.heading("Körperbasis");
                let pkg = s.packages.body.clone();
                ui.colored_label(theme::TEXT2, format!("{} · {} v{}", pkg.label, pkg.id, pkg.version));
                ui.add_space(6.0);
                for st in validate::body_stages(&s.body, &s.params, &pkg, un) { stage_row(ui, &st); }
                ui.separator();
                ui.strong("Körperbasen");
                for b in &s.packages.bodies { ui.label(format!("{}  v{}  · {}{}", b.label, b.version, if b.status.is_empty() { "fertig" } else { &b.status }, if b.user { " · eigene" } else { "" })); }
                ui.separator();
                ui.strong("Externes Modell importieren (glTF/GLB/VRM/OBJ)");
                ui.weak("Analyse gegen die aktive Körperbasis. Ein Modell ohne Rig oder Morphs gilt nicht als editierbar; fehlende Voraussetzungen werden benannt. Mit „Als Entwurf anlegen“ wird das Modell als Körperbasis-Paket (Status: Entwurf) gespeichert.");
                if ui.button("Modell wählen und analysieren …").clicked() {
                    if let Some(p) = rfd::FileDialog::new().add_filter("3D-Modell", &["glb", "gltf", "vrm", "obj"]).pick_file() {
                        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                        let r = if ext == "obj" { validate::analyze_obj(&p, &s.body) } else { validate::analyze_gltf(&p, &s.body) };
                        self.studio.import = Some(match r { Ok(r) => (p.display().to_string(), r), Err(e) => (p.display().to_string(), vec![Stage { name: "Import fehlgeschlagen", ok: Some(false), detail: e }]) });
                        self.studio.import_path = Some(p);
                    }
                }
                if let Some((path, rep)) = &self.studio.import {
                    ui.colored_label(theme::TEXT2, path);
                    for st in rep { stage_row(ui, st); }
                    let usable = rep.first().map(|r| r.ok == Some(true)).unwrap_or(false) && rep.get(1).map(|r| r.ok == Some(true)).unwrap_or(false);
                    ui.add_space(4.0);
                    ui.colored_label(theme::WARN, "Aufbereitung: fehlende Stufen (Rig mit Bone-Mapping auf VRM-Humanoid, Morph-Targets/Shape Keys, Regionen, Links/Rechts) müssen ergänzt werden, z. B. in Blender (glTF-Export mit Skin und Shape Keys). Das Zuordnen von Regionen/Morphs in der Oberfläche und die Nutzung einer importierten Körperbasis im Editor sind noch nicht implementiert.");
                    if usable {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.studio.body_name);
                            if ui.add_enabled(!self.studio.body_name.trim().is_empty(), egui::Button::new("Als Entwurf anlegen")).clicked() {
                                if let Some(src) = self.studio.import_path.clone() {
                                    let stages: Vec<(String, Option<bool>, String)> = rep.iter().map(|x| (x.name.to_string(), x.ok, x.detail.clone())).collect();
                                    match hase_core::packages::write_body_draft(&s.repo, self.studio.body_name.trim(), &src, &stages) {
                                        Ok(dir) => { self.studio.msg = format!("Entwurf gespeichert: {}", dir.display()); self.studio.body_name.clear(); if let Ok(pk) = hase_core::packages::Packages::load(&s.repo) { s.packages.bodies = pk.bodies; } }
                                        Err(e) => self.studio.msg = e,
                                    }
                                }
                            }
                        });
                    }
                    if !self.studio.msg.is_empty() { ui.colored_label(theme::TEXT2, &self.studio.msg); }
                }
            }
            2 => {
                ui.heading("Werte & Grenzen");
                if !editable { ui.colored_label(theme::WARN, "Schreibgeschützt – zum Bearbeiten duplizieren."); }
                let sp = &mut s.packages.species[sel];
                ui.label(format!("Standardwerte ({}):", sp.defaults.len()));
                let mut rm = None;
                for (k, v) in sp.defaults.iter() { ui.horizontal(|ui| { ui.label(format!("{} = {:.2}", theme::label_de(&k.trim_start_matches("mh:").replace('-', " ")), v)); if editable && ui.small_button("Entfernen").clicked() { rm = Some(k.clone()); } }); }
                if let Some(k) = rm { sp.defaults.remove(&k); self.studio.dirty = true; }
                if editable && ui.button("Standardwerte aus aktuellem Charakter übernehmen").clicked() {
                    sp.defaults = s.params.defs.iter().enumerate().filter(|(i, d)| (s.state.values[*i] - d.default).abs() > 1e-4).map(|(i, d)| (d.id.clone(), s.state.values[i])).collect();
                    self.studio.dirty = true;
                }
                ui.separator();
                ui.label(format!("Erlaubte Bereiche ({}):", sp.limits.len()));
                let mut rm = None;
                for (k, l) in sp.limits.iter() { ui.horizontal(|ui| { ui.label(format!("{}: {:.2} … {:.2}", theme::label_de(&k.trim_start_matches("mh:").replace('-', " ")), l[0], l[1])); if editable && ui.small_button("Entfernen").clicked() { rm = Some(k.clone()); } }); }
                if let Some(k) = rm { sp.limits.remove(&k); self.studio.dirty = true; }
                if editable {
                    if self.studio.limit_width == 0.0 { self.studio.limit_width = 0.3; }
                    ui.horizontal(|ui| { ui.label("± Breite"); ui.add(egui::Slider::new(&mut self.studio.limit_width, 0.05..=1.0)); });
                    if ui.button("Grenzen um geänderte Werte des aktuellen Charakters setzen").clicked() {
                        for (i, d) in s.params.defs.iter().enumerate() { let v = s.state.values[i]; if (v - d.default).abs() > 1e-4 { sp.limits.insert(d.id.clone(), [(v - self.studio.limit_width).max(d.min), (v + self.studio.limit_width).min(d.max)]); } }
                        self.studio.dirty = true;
                    }
                    ui.weak("Grenzen werden im Paket gespeichert; die Durchsetzung in den Reglern ist noch nicht implementiert.");
                }
            }
            3 => {
                ui.heading("Presets & Merkmale");
                let sp = &mut s.packages.species[sel];
                let mut rm = None; let mut apply = None;
                for (i, p) in sp.presets.iter().enumerate() { ui.horizontal(|ui| { if ui.button(&p.label).clicked() { apply = Some(i); } ui.weak(format!("{} Werte", p.params.len())); if editable && ui.small_button("Entfernen").clicked() { rm = Some(i); } }); }
                if let Some(i) = apply { let m = sp.presets[i].params.clone(); s.apply_map(&m); return; }
                if let Some(i) = rm { sp.presets.remove(i); self.studio.dirty = true; }
                if sp.presets.is_empty() { ui.weak("Keine Presets."); }
                ui.label(format!("Merkmale: {}", sp.traits.iter().map(|t| t.label.clone()).collect::<Vec<_>>().join(", ")));
                if editable {
                    ui.separator();
                    ui.text_edit_singleline(&mut self.studio.preset_name);
                    if ui.add_enabled(!self.studio.preset_name.trim().is_empty(), egui::Button::new("Aktuellen Charakter als Preset hinzufügen")).clicked() {
                        let params = s.params.defs.iter().enumerate().filter(|(i, d)| (s.state.values[*i] - d.default).abs() > 1e-4).map(|(i, d)| (d.id.clone(), s.state.values[i])).collect();
                        sp.presets.push(Preset { id: slug(&self.studio.preset_name), label: self.studio.preset_name.trim().to_string(), params });
                        self.studio.preset_name.clear(); self.studio.dirty = true;
                    }
                }
            }
            4 => {
                ui.heading("Kompatibilität");
                let sp = s.packages.species[sel].clone();
                ui.label(format!("Körperbasis-Interface: {}", s.packages.body.interface));
                ui.label(format!("Referenzierte Körperbasis: {}", if sp.body_id.is_empty() { "–" } else { &sp.body_id }));
                ui.add_space(6.0);
                let issues = validate::species_issues(&sp, &s.params);
                if issues.is_empty() { ui.colored_label(egui::Color32::from_rgb(0x6C, 0xC4, 0x7F), "OK – Als Speziespaket validiert (Parameter, Grenzen, Presets, Komponenten)"); }
                for i in issues { ui.colored_label(theme::WARN, format!("• {i}")); }
                ui.weak("Hinweis: Die Beispiel-Packages nutzen teils ältere Parameter-IDs; sie werden hier als unbekannt gemeldet, bis sie angeglichen sind.");
            }
            _ => {
                ui.heading("Paket & Version");
                let sp = &mut s.packages.species[sel];
                ui.label(format!("{}  v{}", sp.id, sp.version));
                if let Some(d) = &sp.dir { ui.colored_label(theme::TEXT2, d.display().to_string()); }
                if !editable { ui.colored_label(theme::WARN, "Mitgeliefert – nicht veränderbar. Duplizieren, um eine eigene Version zu pflegen."); return; }
                ui.weak("Versionsregel: Patch = Korrektur ohne Wertänderung; Minor = neue Presets/Merkmale/Werte (bestehende Charaktere bleiben gültig); Major = entfernte oder bedeutungsändernde Parameter (Charaktere benötigen Migration).");
                ui.horizontal(|ui| { for (l, lv) in [("Patch +1", 2usize), ("Minor +1", 1), ("Major +1", 0)] { if ui.button(l).clicked() { sp.version = bump_version(&sp.version, lv); self.studio.dirty = true; } } });
                ui.add(egui::TextEdit::multiline(&mut sp.description).desired_rows(3).hint_text("Beschreibung"));
                if self.studio.dirty { ui.colored_label(theme::WARN, "Ungespeicherte Änderungen"); }
                if ui.button("Paket speichern").clicked() {
                    let sp = s.packages.species[sel].clone();
                    self.studio.msg = match sp.write(&s.repo, &s.packages.body) { Ok(d) => { s.packages.species[sel].digest = hase_core::packages::file_digest(&d.join("hase-package.json")); self.studio.dirty = false; format!("Gespeichert: {}", d.display()) } Err(e) => e };
                }
                if !self.studio.msg.is_empty() { ui.colored_label(theme::TEXT2, &self.studio.msg); }
            }
        });
    }
}
