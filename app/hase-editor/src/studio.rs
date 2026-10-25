//! Körperbasis- und Speziesstudio: Bibliothek, Prüfstufen, Werte/Grenzen, Presets, Kompatibilität, Paket speichern.
use crate::app::App;
use crate::regions::{self, Main};
use crate::theme;
use eframe::egui;
use hase_core::packages::{bump_version, slug, Preset, Species};
use hase_core::import::{self, ImportConfig, Model, LANDMARKS, REGION_NAMES, VRM_SLOTS};
use hase_core::validate::{self, Stage};
use crate::session::Session;
use std::path::PathBuf;

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
    pub prep: Option<Prep>,
    pub var_name: String,
    pub var_min: f32,
    pub var_max: f32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Pick { None, Landmark(usize), Morphs }
impl Default for Pick { fn default() -> Self { Pick::None } }

pub struct Hit { pub pos: [f32; 3], pub vertex: usize, pub morphs: Vec<(usize, f32)> }

pub struct Prep {
    pub pkg: hase_core::packages::BodyPackage,
    pub model: Model,
    pub cfg: ImportConfig,
    pub pick: Pick,
    pub hit: Option<Hit>,
    pub bone_sel: Option<usize>,
    pub show_bones: bool,
    pub msg: String,
}

const TABS: [&str; 7] = ["Übersicht", "Körperbasis", "Aufbereitung", "Werte & Grenzen", "Presets", "Kompatibilität", "Paket"];

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
                let label = format!("{}{}{}{}", sp.label, if sp.user { "  · eigene" } else { "" }, if sp.stub { "  · Stub" } else { "" }, if !sp.body_id.is_empty() && sp.body_id != s.packages.body.id { "  · andere Körperbasis" } else { "" });
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
                let cur = s.species_opt().map(|i| s.packages.species[i].clone());
                let defaults = s.params.defs.iter().enumerate().filter(|(i, d)| (s.state.values[*i] - d.default).abs() > 1e-4).map(|(i, d)| (d.id.clone(), s.state.values[i])).collect();
                let sp = Species { shape: Default::default(), id, label: name.clone(), version: "1.0.0".into(), defaults, limits: Default::default(), traits: vec![], presets: vec![], components: cur.as_ref().map(|c| c.components.clone()).unwrap_or_default(), stub: false, user: true, description: format!("Abgeleitet vom Charakter (Körperbasis: {}{}).", s.packages.body.label, cur.as_ref().map(|c| format!(", Basis-Spezies: {}", c.label)).unwrap_or_default()), body_id: s.packages.body.id.clone(), dir: None, digest: String::new() };
                match sp.write(&s.repo, &s.packages.body) {
                    Ok(dir) => { let mut sp = sp; sp.digest = hase_core::packages::file_digest(&dir.join("hase-package.json")); sp.dir = Some(dir.clone()); let (nid, nver, ndig) = (sp.id.clone(), sp.version.clone(), sp.digest.clone()); s.packages.species.push(sp); self.studio.sel = s.packages.species.len() - 1; s.state.species = nid.clone(); s.state.species_ref = hase_core::state::PkgRef { id: nid, version: nver, digest: ndig }; self.studio.msg = format!("Gespeichert: {} – der aktuelle Charakter nutzt sie jetzt", dir.display()); self.studio.new_name.clear(); }
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
        self.workflow_bar(ui);
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
                ui.strong("Spezieskörper aus der Formung");
                if self.studio.var_min == 0.0 && self.studio.var_max == 0.0 { self.studio.var_max = 1.0; }
                let nsc = s.sculpt_count();
                ui.weak("Körperbasis → Speziesmodell: Mit dem Pinsel („Direkt formen“) gestaltete Geometrie gehört entweder fest zum Spezieskörper (einbacken) oder wird bewusst zu einem Regler (Variation). Nicht jede Formung muss ein Regler werden.");
                let sp_label = s.packages.species[sel].label.clone();
                ui.label(format!("Ziel-Spezies: {sp_label} · aktuelle Formung: {nsc} Vertices · eingebacken: {} · Variationen: {}", s.packages.species[sel].shape.baked.len(), s.packages.species[sel].shape.variations.len()));
                if !editable { ui.colored_label(theme::WARN, "Mitgelieferte Spezies sind schreibgeschützt – links duplizieren oder als neue Spezies speichern."); }
                else {
                    if ui.add_enabled(nsc > 0, egui::Button::new("In Spezieskörper einbacken (feste Grundform)")).on_hover_text("Die Formung wird Teil des Körpers dieser Spezies – ohne Regler. Beispiel: Schädelform eines Asura.").clicked() {
                        self.studio.msg = match s.bake_into_species(sel) { Ok(n) => { self.studio.dirty = false; format!("{n} Vertices eingebacken") } Err(e) => e };
                    }
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.studio.var_name).hint_text("Name der Variation, z. B. Ohrlänge").desired_width(190.0));
                        ui.add(egui::DragValue::new(&mut self.studio.var_min).speed(0.05).prefix("min "));
                        ui.add(egui::DragValue::new(&mut self.studio.var_max).speed(0.05).prefix("max "));
                    });
                    if ui.add_enabled(nsc > 0 && !self.studio.var_name.trim().is_empty(), egui::Button::new("Als Variation (Regler) definieren")).on_hover_text("Die Formung wird zu einem benannten Regler dieser Spezies (Wert 1 = jetzige Formung). Die Charakter-Phase zeigt nur diese Regler.").clicked() {
                        let (nm, mn, mx) = (self.studio.var_name.trim().to_string(), self.studio.var_min.min(self.studio.var_max), self.studio.var_max.max(self.studio.var_min));
                        self.studio.msg = match s.define_variation(sel, &nm, mn, mx) { Ok(n) => { self.studio.var_name.clear(); format!("Variation „{nm}“ gespeichert ({n} Vertices)") } Err(e) => e };
                    }
                }
                let vars: Vec<String> = s.packages.species[sel].shape.variations.iter().map(|v| format!("{}  [{:.1} … {:.1}]  · {} Vertices", v.label, v.min, v.max, v.delta.len())).collect();
                for v in vars { ui.label(format!("• {v}")); }
                if !self.studio.msg.is_empty() { ui.colored_label(theme::TEXT2, &self.studio.msg); }
                ui.separator();
                ui.strong("Körperbasen");
                let bodies = s.packages.bodies.clone();
                for b in &bodies {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}  v{}  · {}{}", b.label, b.version, if b.status.is_empty() { "fertig" } else { &b.status }, if b.user { " · eigene" } else { "" }));
                        if b.user && ui.small_button("Aufbereiten").clicked() { match start_prep(s, b) { Ok(p) => { self.studio.prep = Some(p); self.studio.tab = 2; } Err(e) => self.studio.msg = e } }
                        if b.user && b.status == "configured" && ui.small_button("Aktivieren").clicked() { match s.activate_body(b) { Ok(()) => self.studio.msg = format!("{} aktiv", b.label), Err(e) => self.studio.msg = e } }
                    });
                }
                if s.generic && ui.button("Zur Referenz-Körperbasis zurück").clicked() { let r = s.reference_body.clone(); match s.activate_body(&r) { Ok(()) => { self.studio.prep = None; } Err(e) => self.studio.msg = e } }
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
                    let is_model = self.studio.import_path.as_ref().and_then(|p| p.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase())).map(|e| e == "vrm" || e == "glb").unwrap_or(false);
                    if is_model && rep.get(3).map(|r| r.ok == Some(true)).unwrap_or(false) {
                        if ui.button("Als Modell öffnen (Ansehen, Posen, Ausdrücke)").on_hover_text("Öffnet das Modell mit Skelett, Texturen und Blendshapes im Viewport").clicked() {
                            if let Some(p) = self.studio.import_path.clone() { self.studio.msg = match s.open_vrm(&p) { Ok(i) => i, Err(e) => e }; }
                        }
                    }
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
                ui.heading("Aufbereitung");
                match self.studio.prep.as_mut() {
                    None => { ui.weak("Wähle unter „Körperbasis“ einen importierten Entwurf und starte „Aufbereiten“."); }
                    Some(prep) => prep_ui(ui, s, prep),
                }
            }
            3 => {
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
            4 => {
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
            5 => {
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

fn load_model_cfg(pkg: &hase_core::packages::BodyPackage) -> Result<(Model, ImportConfig), String> {
    let dir = pkg.dir.clone().ok_or("kein Paketverzeichnis")?;
    let m: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("hase-package.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let src = m["dev"]["source"].as_str().ok_or("Quelle fehlt im Manifest")?.to_string();
    let path = dir.join("data").join(&src);
    let model = Model::open(&path)?;
    let mut cfg: ImportConfig = pkg.config.clone().and_then(|c| serde_json::from_value(c).ok()).unwrap_or_default();
    if cfg.source.is_empty() { cfg.source = src; cfg.scale = 1.0; cfg.primitive = model.prims.iter().enumerate().max_by_key(|(_, p)| (p.morphs, p.verts)).map(|(i, _)| i).unwrap_or(0); }
    Ok((model, cfg))
}

fn reload(s: &mut Session, prep: &mut Prep) {
    let path = prep.model.path.clone();
    match s.preview_import(&path, &prep.cfg, prep.pkg.clone()) {
        Ok(()) => {
            if prep.cfg.landmarks.is_empty() { prep.cfg.landmarks = import::default_landmarks(&s.body.base_pos); }
            for (n, r) in &prep.cfg.regions { if let Some(&i) = s.params.index.get(&format!("morph:{n}")) { s.params.defs[i].group = r.clone(); } }
            refresh_markers(s, prep);
            prep.msg = format!("{} Vertices, {} Morph-Targets geladen", s.body.base_pos.len(), s.body.morphs.names.len());
        }
        Err(e) => prep.msg = e,
    }
}

fn start_prep(s: &mut Session, pkg: &hase_core::packages::BodyPackage) -> Result<Prep, String> {
    let (model, cfg) = load_model_cfg(pkg)?;
    let mut prep = Prep { pkg: pkg.clone(), model, cfg, pick: Pick::None, hit: None, bone_sel: None, show_bones: false, msg: String::new() };
    reload(s, &mut prep);
    Ok(prep)
}

fn refresh_markers(s: &mut Session, prep: &Prep) {
    for (i, (k, _)) in LANDMARKS.iter().enumerate() {
        match prep.cfg.landmarks.get(*k) { Some(p) => s.set_marker(&format!("lm:{k}"), *p, 0.012, [0.35, 0.65, 1.0]), None => s.remove_marker(&format!("lm:{k}")) }
        let _ = i;
    }
    if prep.show_bones { for (i, j) in prep.model.joints.iter().enumerate() { let sel = prep.bone_sel == Some(i); s.set_marker(&format!("bone:{i}"), j.pos, if sel { 0.02 } else { 0.01 }, if sel { [1.0, 0.8, 0.2] } else { [0.7, 0.7, 0.75] }); } }
    else { for i in 0..prep.model.joints.len() { s.remove_marker(&format!("bone:{i}")); } }
}

/// Klick im Viewport während eines Pick-Modus.
pub fn on_pick(s: &mut Session, st: &mut StudioState, o: [f32; 3], d: [f32; 3]) {
    let Some(prep) = st.prep.as_mut() else { return };
    let Some((pos, vertex)) = s.pick(o, d) else { prep.msg = "Kein Treffer am Körper".into(); return };
    match prep.pick {
        Pick::Landmark(i) => { prep.cfg.landmarks.insert(LANDMARKS[i].0.to_string(), pos); prep.pick = Pick::None; refresh_markers(s, prep); prep.msg = format!("Landmarke „{}“ gesetzt", LANDMARKS[i].1); }
        Pick::Morphs => { s.set_marker("pick", pos, 0.01, [1.0, 0.5, 0.2]); prep.hit = Some(Hit { pos, vertex, morphs: s.morphs_at_vertex(vertex, 12) }); }
        Pick::None => {}
    }
}

fn prep_ui(ui: &mut egui::Ui, s: &mut Session, prep: &mut Prep) {
    ui.colored_label(theme::TEXT2, format!("{}  ·  {}", prep.pkg.label, prep.model.path.file_name().and_then(|f| f.to_str()).unwrap_or("")));
    if !prep.msg.is_empty() { ui.colored_label(theme::TEXT2, &prep.msg); }
    if prep.pick != Pick::None { ui.colored_label(theme::WARN, "Pick-Modus aktiv: Klick auf den Körper im Viewport (Orbit: ziehen)."); }
    // 1 Quelle
    ui.add_space(6.0); ui.strong("1  Quelle");
    let cur = prep.cfg.primitive.min(prep.model.prims.len().saturating_sub(1));
    let mut sel = cur;
    egui::ComboBox::from_id_salt("prim").selected_text(prep.model.prims.get(cur).map(|p| format!("{} · {} V · {} Morphs", p.label, p.verts, p.morphs)).unwrap_or_default()).show_ui(ui, |ui| { for (i, p) in prep.model.prims.iter().enumerate() { ui.selectable_value(&mut sel, i, format!("{} · {} V · {} Morphs{}", p.label, p.verts, p.morphs, if p.skinned { " · Skin" } else { "" })); } });
    let mut changed = sel != cur;
    ui.horizontal(|ui| { ui.label("Maßstab"); if ui.add(egui::DragValue::new(&mut prep.cfg.scale).speed(0.001).range(0.0001..=100.0)).changed() { changed = true; } ui.weak("(1 = Meter, 0.01 = cm)"); });
    if changed { prep.cfg.primitive = sel; prep.cfg.regions.clear(); prep.cfg.landmarks.clear(); reload(s, prep); }
    // 2 Landmarken
    ui.add_space(6.0); ui.strong("2  Landmarken am Modell");
    ui.weak("Setze die Punkte per Klick auf den Körper; daraus leiten sich die Regionsgrenzen ab.");
    for (i, (k, l)) in LANDMARKS.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{l}: {}", if prep.cfg.landmarks.contains_key(*k) { "gesetzt" } else { "–" }));
            let on = prep.pick == Pick::Landmark(i);
            if ui.selectable_label(on, "am Modell setzen").clicked() { prep.pick = if on { Pick::None } else { Pick::Landmark(i) }; }
        });
    }
    if ui.button("Aus Proportionen vorschlagen").clicked() { prep.cfg.landmarks = import::default_landmarks(&s.body.base_pos); refresh_markers(s, prep); }
    // 3 Regionen
    ui.add_space(6.0); ui.strong("3  Anatomische Regionen der Morphs");
    let n = s.params.defs.len();
    let unassigned = s.params.defs.iter().filter(|d| d.group == "Unzugeordnet").count();
    ui.label(format!("{} Morphs, davon {} ohne Region", n, unassigned));
    ui.horizontal(|ui| {
        if ui.button("Automatisch zuordnen (aus Verschiebungen)").on_hover_text("Ordnet jeden Morph anhand des Schwerpunkts seiner Verformung der nächstgelegenen Region zu – Trefferquote an MakeHuman ca. 64 %, bitte am Modell prüfen").clicked() {
            let sug = import::suggest_regions(&s.body, &prep.cfg.landmarks);
            for (name, r) in &sug { if let Some(&i) = s.params.index.get(&format!("morph:{name}")) { if s.params.defs[i].group == "Unzugeordnet" { s.params.defs[i].group = r.clone(); } } }
            prep.msg = format!("{} Morphs automatisch zugeordnet", sug.len());
        }
        let on = prep.pick == Pick::Morphs;
        if ui.selectable_label(on, "Am Modell prüfen").on_hover_text("Klick auf eine Körperstelle zeigt, welche Morphs dort wirken").clicked() { prep.pick = if on { Pick::None } else { Pick::Morphs }; }
    });
    if let Some(h) = &prep.hit {
        ui.colored_label(theme::TEXT2, format!("Stelle: Vertex {} · Region (Heuristik): {}", h.vertex, import::region_for_point(h.pos, &prep.cfg.landmarks)));
        let hits = h.morphs.clone();
        let mut assign: Option<(usize, String)> = None;
        let mut set: Option<(usize, f32)> = None;
        for (t, mag) in hits {
            let g = s.params.defs[t].group.clone();
            ui.horizontal(|ui| {
                let mut v = s.state.values[t];
                if ui.add(egui::Slider::new(&mut v, 0.0..=1.0).text(format!("{} ({:.1} mm)", s.params.defs[t].label, mag * 1000.0))).changed() { set = Some((t, v)); }
                let mut cur = g.clone();
                egui::ComboBox::from_id_salt(("reg", t)).width(130.0).selected_text(&g).show_ui(ui, |ui| { ui.selectable_value(&mut cur, "Unzugeordnet".to_string(), "Unzugeordnet"); for r in REGION_NAMES { ui.selectable_value(&mut cur, r.to_string(), r); } });
                if cur != g { assign = Some((t, cur)); }
            });
        }
        if let Some((t, v)) = set { s.set_param(t, v); }
        if let Some((t, r)) = assign { s.params.defs[t].group = r; }
        ui.weak("Region je Morph per Auswahlfeld; Regler zeigen die Wirkung sofort im Viewport.");
    }
    // 4 Bones
    ui.add_space(6.0); ui.strong("4  Skelett & Bone-Mapping (VRM-Humanoid)");
    if prep.model.joints.is_empty() { ui.colored_label(theme::WARN, "Kein Skelett im Modell – Rig in Blender ergänzen (Skin, Joints) und neu importieren."); }
    else {
        if ui.button("Automatisch zuordnen").clicked() { prep.cfg.bones = import::auto_bones(&prep.model.joints); }
        let before = prep.show_bones;
        ui.checkbox(&mut prep.show_bones, "Skelett im Viewport zeigen (Körper ausgeblendet)");
        if before != prep.show_bones { s.hide_body = prep.show_bones; refresh_markers(s, prep); }
        let names: Vec<String> = prep.model.joints.iter().map(|j| j.name.clone()).collect();
        let mut sel_bone = None;
        for (slot, _) in VRM_SLOTS.iter() {
            let mut cur = prep.cfg.bones.get(*slot).cloned().unwrap_or_default();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(*slot).color(if cur.is_empty() { theme::ERR } else { theme::TEXT }));
                let old = cur.clone();
                egui::ComboBox::from_id_salt(("bone", *slot)).width(170.0).selected_text(if cur.is_empty() { "– nicht zugeordnet –" } else { cur.as_str() }).show_ui(ui, |ui| { for n in &names { ui.selectable_value(&mut cur, n.clone(), n); } });
                if cur != old { prep.cfg.bones.insert(slot.to_string(), cur.clone()); sel_bone = names.iter().position(|n| *n == cur); }
            });
        }
        if sel_bone.is_some() { prep.bone_sel = sel_bone; refresh_markers(s, prep); }
        ui.weak(format!("{}/15 Pflicht-Bones zugeordnet · {} Joints", prep.cfg.bones.len(), names.len()));
    }
    // 5 Symmetrie
    ui.add_space(6.0); ui.strong("5  Symmetrie (links/rechts)");
    let pairs = import::detect_mirror(&s.body.morphs.names);
    ui.label(format!("{} Links/Rechts-Paare aus Namen erkannt ({} aktiv)", pairs.len(), s.params.defs.iter().filter(|d| d.mirror.is_some()).count() / 2));
    if ui.button("Erkannte Paare übernehmen").clicked() { prep.cfg.mirror = pairs.iter().map(|&(a, b)| [s.body.morphs.names[a].clone(), s.body.morphs.names[b].clone()]).collect(); for &(a, b) in &pairs { s.params.defs[a].mirror = Some(b); s.params.defs[b].mirror = Some(a); } }
    // 6 Prüfung & Speichern
    ui.add_space(6.0); ui.separator(); ui.strong("6  Prüfung");
    let unassigned = s.params.defs.iter().filter(|d| d.group == "Unzugeordnet").count();
    let lm_ok = LANDMARKS.iter().all(|(k, _)| prep.cfg.landmarks.contains_key(*k));
    let bones_ok = prep.cfg.bones.len() == 15;
    let checks = [
        Stage { name: "Geometrie & Morphs geladen", ok: Some(!s.body.morphs.names.is_empty()), detail: format!("{} Morph-Targets", s.body.morphs.names.len()) },
        Stage { name: "Landmarken", ok: Some(lm_ok), detail: "5 Landmarken am Modell gesetzt".into() },
        Stage { name: "Regionen", ok: Some(unassigned == 0), detail: format!("{unassigned} Morphs ohne Region") },
        Stage { name: "Rig / Bone-Mapping", ok: Some(bones_ok), detail: format!("{}/15 VRM-Pflicht-Bones", prep.cfg.bones.len()) },
        Stage { name: "Symmetrie", ok: Some(!prep.cfg.mirror.is_empty()), detail: format!("{} Paare übernommen", prep.cfg.mirror.len()) },
    ];
    for c in &checks { stage_row(ui, c); }
    let complete = checks.iter().all(|c| c.ok == Some(true));
    if ui.button(if complete { "Als Körperbasis speichern (vollständig konfiguriert)" } else { "Zwischenstand speichern (Entwurf)" }).clicked() {
        for d in &s.params.defs { if d.group != "Unzugeordnet" { prep.cfg.regions.insert(d.id.trim_start_matches("morph:").to_string(), d.group.clone()); } }
        let caps = if bones_ok { vec!["humanoid:vrm1-complete".to_string(), "morph:generic".to_string()] } else { vec!["morph:generic".to_string()] };
        let v = serde_json::to_value(&prep.cfg).unwrap();
        prep.msg = match prep.pkg.dir.as_ref().map(|d| hase_core::packages::save_body_config(d, &v, if complete { "configured" } else { "draft" }, caps)) {
            Some(Ok(())) => format!("Gespeichert ({})", if complete { "vollständig konfiguriert – als Körperbasis verwendbar" } else { "Entwurf" }),
            Some(Err(e)) => e, None => "kein Paketverzeichnis".into(),
        };
        if let Ok(pk) = hase_core::packages::Packages::load(&s.repo) { s.packages.bodies = pk.bodies; }
    }
}

impl App {
    /// Zielworkflow Import → Körperbasis → Spezies → Charakter mit Status je Schritt.
    fn workflow_bar(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_ref() else { return };
        let imported = self.studio.import.is_some() || s.packages.bodies.iter().any(|b| b.user);
        let prepping = self.studio.prep.is_some();
        let body_ready = s.packages.bodies.iter().any(|b| b.user && b.status == "configured");
        let species = s.packages.species.iter().any(|x| x.user);
        let steps = [("Import", if imported { "erledigt" } else { "offen" }, imported, 1usize), ("Körperbasis", if body_ready { "konfiguriert" } else if prepping { "in Arbeit" } else { "offen" }, body_ready, 2), ("Spezies", if species { "vorhanden" } else { "offen" }, species, 0), ("Charakter", "gestalten", false, 99)];
        ui.horizontal(|ui| {
            for (i, (n, st, done, tab)) in steps.iter().enumerate() {
                let col = if *done { egui::Color32::from_rgb(0x6C, 0xC4, 0x7F) } else { theme::TEXT2 };
                if ui.add(egui::Button::new(egui::RichText::new(format!("{n}\n{st}")).color(col).small()).min_size(egui::vec2(78.0, 34.0))).clicked() { if *tab == 99 { self.ws = crate::app::Workspace::Charakter; self.main = Main::Charakter; } else { self.studio.tab = *tab; } }
                if i < 3 { ui.weak(">"); }
            }
        });
        ui.separator();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_prepare_activate_chain() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let glb = repo.join("content/testcandidate/body_mpfb2_testcandidate.glb");
        if !glb.exists() { eprintln!("GLB fehlt – übersprungen"); return; }
        let mut s = Session::load(&repo).unwrap();
        let ref_id = s.packages.body.id.clone();
        let ref_params = s.params.defs.len();
        let dir = hase_core::packages::write_body_draft(&repo, "Kettentest", &glb, &[("Importiert".into(), Some(true), "x".into())]).unwrap();
        let pk = hase_core::packages::Packages::load(&repo).unwrap();
        let pkg = pk.bodies.iter().find(|b| b.id == "user.body.kettentest").cloned().expect("Entwurf gelistet");
        assert_eq!(pkg.status, "draft");
        let fp1 = pkg.digest.clone();

        let mut prep = start_prep(&mut s, &pkg).unwrap();
        assert!(s.generic && s.params.defs.len() == 548, "{}", s.params.defs.len());
        assert_eq!(prep.cfg.landmarks.len(), 5);
        // Strahl von vorn auf die Körpermitte trifft den Körper
        s.refresh();
        let (o, d) = ([0.0, s.body_height() * 0.6, 5.0], [0.0, 0.0, -1.0]);
        let (hit, v) = s.pick(o, d).expect("Treffer");
        assert!(hit[2] > 0.0, "Vorderseite getroffen, z={}", hit[2]);
        assert!(!s.morphs_at_vertex(v, 5).is_empty());
        // Auto-Zuordnung + Speichern
        let sug = import::suggest_regions(&s.body, &prep.cfg.landmarks);
        for (n, r) in &sug { if let Some(&i) = s.params.index.get(&format!("morph:{n}")) { s.params.defs[i].group = r.clone(); } }
        for d in &s.params.defs { if d.group != "Unzugeordnet" { prep.cfg.regions.insert(d.id.trim_start_matches("morph:").to_string(), d.group.clone()); } }
        let v = serde_json::to_value(&prep.cfg).unwrap();
        hase_core::packages::save_body_config(&dir, &v, "configured", vec!["morph:generic".into()]).unwrap();
        // Neu laden, aktivieren, Morph wirkt
        let pk = hase_core::packages::Packages::load(&repo).unwrap();
        let pkg2 = pk.bodies.iter().find(|b| b.id == "user.body.kettentest").cloned().unwrap();
        assert_eq!(pkg2.status, "configured");
        assert_ne!(pkg2.digest, fp1, "Konfigurationsänderung ändert den Fingerprint");
        s.activate_body(&pkg2).unwrap();
        assert!(s.params.defs.iter().any(|d| d.group == "Augen"));
        let i = s.params.defs.iter().position(|d| d.group == "Nase").unwrap();
        s.refresh();
        s.set_param(i, 1.0);
        let up = s.refresh();
        assert!(up.iter().any(|u| matches!(u, crate::render::MeshUpdate::Positions { id, .. } if id == "body")));
        // Fingerprint reagiert auf Austausch der Geometrie bei gleichem Manifest
        let before = hase_core::packages::data_fingerprint(&dir.join("data"), &repo.join(".hase-cache/fp-test.json"));
        std::fs::write(dir.join("data/extra.bin"), b"x").unwrap();
        let after = hase_core::packages::data_fingerprint(&dir.join("data"), &repo.join(".hase-cache/fp-test.json"));
        assert_ne!(before, after);
        // zurück zur Referenz
        let r = s.reference_body.clone();
        s.activate_body(&r).unwrap();
        assert!(!s.generic && s.packages.body.id == ref_id && s.params.defs.len() == ref_params);
        std::fs::remove_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(repo.join(".hase-cache/fp-test.json"));
    }

    /// Speziesunabhängigkeit: zweite, topologisch andere Körperbasis (Quaternius, 15 synthetische Morphs, Skelett) durch die ganze Kette.
    #[test]
    fn second_body_full_chain_import_species_character_save_load() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let glb = repo.join("content/testcandidate/body_b_quaternius_synthetic.glb");
        if !glb.exists() { eprintln!("body_b GLB fehlt – übersprungen"); return; }
        let mut s = Session::load(&repo).unwrap();
        let ref_body = s.packages.body.clone();
        let mh_verts = s.body.topo.vmap.len();
        let ref_params = s.params.defs.len();
        let dir = hase_core::packages::write_body_draft(&repo, "ZweiteBasisTest", &glb, &[]).unwrap();
        let cleanup = |d: &std::path::Path| { let _ = std::fs::remove_dir_all(d); };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let pk = hase_core::packages::Packages::load(&repo).unwrap();
            let pkg = pk.bodies.iter().find(|b| b.id == "user.body.zweitebasistest").cloned().unwrap();
            let mut prep = start_prep(&mut s, &pkg).unwrap();
            // andere Topologie, 15 Morphs
            assert!(s.generic);
            assert_ne!(s.body.topo.vmap.len(), mh_verts);
            assert_eq!(s.params.defs.len(), 15, "{:?}", s.params.defs.iter().map(|d| &d.id).collect::<Vec<_>>());
            // Aufbereitung: Regionen, Bones, Symmetrie
            let sug = import::suggest_regions(&s.body, &prep.cfg.landmarks);
            for (n, r) in &sug { let i = s.params.index[&format!("morph:{n}")]; s.params.defs[i].group = r.clone(); }
            for d in &s.params.defs { prep.cfg.regions.insert(d.id.trim_start_matches("morph:").to_string(), d.group.clone()); }
            prep.cfg.bones = import::auto_bones(&prep.model.joints);
            assert_eq!(prep.cfg.bones.len(), 15, "Bone-Mapping {:?}", prep.cfg.bones.keys().collect::<Vec<_>>());
            let mir = import::detect_mirror(&s.body.morphs.names);
            assert_eq!(mir.len(), 3, "{mir:?}");
            prep.cfg.mirror = mir.iter().map(|&(a, b)| [s.body.morphs.names[a].clone(), s.body.morphs.names[b].clone()]).collect();
            let v = serde_json::to_value(&prep.cfg).unwrap();
            hase_core::packages::save_body_config(&dir, &v, "configured", vec!["humanoid:vrm1-complete".into(), "morph:generic".into()]).unwrap();
            let pk = hase_core::packages::Packages::load(&repo).unwrap();
            let pkg_b = pk.bodies.iter().find(|b| b.id == pkg.id).cloned().unwrap();
            assert_eq!(pkg_b.status, "configured");
            s.packages.bodies = pk.bodies.clone();
            // aktivieren; Morph wirkt (größer)
            s.activate_body(&pkg_b).unwrap();
            s.refresh();
            let h0 = s.body_height();
            let i = s.params.index["morph:height_taller"];
            s.set_param(i, 1.0); s.refresh();
            assert!(s.body_height() > h0 * 1.08, "{} -> {}", h0, s.body_height());
            // Symmetrie: Schreiben auf links setzt rechts mit
            let l = s.params.index["morph:arm_longer_l"]; let r = s.params.index["morph:arm_longer_r"];
            s.symmetric = true; s.set_param(l, 0.5);
            assert_eq!(s.state.values[r], 0.5);
            // keine MakeHuman-Spezies sichtbar; Spezies auf Basis B erstellen
            assert!(s.species_indices().is_empty());
            let sp = Species { shape: Default::default(), id: "user.species.testwesen-b".into(), label: "Testwesen B".into(), version: "1.0.0".into(), defaults: [("morph:head_larger".to_string(), 0.4f32)].into_iter().collect(), limits: [("morph:head_larger".to_string(), [0.0f32, 0.8f32])].into_iter().collect(), traits: vec![], presets: vec![], components: vec![], stub: false, user: true, description: "Test".into(), body_id: s.packages.body.id.clone(), dir: None, digest: String::new() };
            let sdir = sp.write(&repo, &s.packages.body).unwrap();
            let mut sp = sp; sp.digest = hase_core::packages::file_digest(&sdir.join("hase-package.json")); sp.dir = Some(sdir.clone());
            s.packages.species.push(sp);
            let idx = s.packages.species.len() - 1;
            assert_eq!(s.species_indices(), vec![idx]);
            s.apply_species(idx);
            let hl = s.params.index["morph:head_larger"];
            assert!((s.state.values[hl] - 0.4).abs() < 1e-5);
            s.set_param(hl, 5.0);
            assert!((s.state.values[hl] - 0.8).abs() < 1e-5, "Grenze wirksam");
            // Charakter gestalten, speichern
            s.set_param(i, 0.7);
            let saved = s.state.to_json(&s.params, &s.packages.body.id);
            let values = s.state.values.clone();
            // zurück zur Referenz; Laden verlangt Körperbasis-Wechsel
            s.activate_body(&ref_body).unwrap();
            assert!(!s.generic && s.params.defs.len() == ref_params);
            match s.plan_load(&saved) { crate::session::LoadPlan::NeedsBody(b) => assert_eq!(b.id, pkg_b.id), _ => panic!("Wechsel erwartet") }
            s.activate_body(&pkg_b).unwrap();
            match s.plan_load(&saved) {
                crate::session::LoadPlan::Ready(st, issues) => { assert!(issues.is_empty(), "{issues:?}"); s.apply_state(st); }
                _ => panic!("bereit erwartet"),
            }
            assert_eq!(s.state.values, values);
            assert_eq!(s.state.species, "user.species.testwesen-b");
            // MakeHuman-Charakter bei aktiver Basis B: Wechsel nötig
            let mh_json = serde_json::json!({"formatVersion": "0.1-editor", "body": {"id": ref_body.id}, "species": {"id": "org.hase.species.human"}, "parameters": {}, "assets": []});
            assert!(matches!(s.plan_load(&mh_json), crate::session::LoadPlan::NeedsBody(_)));
            // Inhaltsänderung der Basis wird erkannt (Bindung des gespeicherten Charakters ist alt)
            std::fs::write(dir.join("data/austausch.bin"), b"neu").unwrap();
            let pk = hase_core::packages::Packages::load(&repo).unwrap();
            s.packages.body = pk.bodies.iter().find(|b| b.id == pkg.id).cloned().unwrap();
            match s.plan_load(&saved) { crate::session::LoadPlan::Ready(_, issues) => assert!(issues.iter().any(|i| i.contains("Inhalt der Körperbasis")), "{issues:?}"), _ => panic!() }
            cleanup(&sdir);
        }));
        cleanup(&dir);
        let _ = std::fs::remove_dir_all(repo.join("user-content/packages/user.species.testwesen-b"));
        if let Err(e) = result { std::panic::resume_unwind(e); }
    }
}
