//! Finalisieren: VRM-Export mit Profilen, Metadaten, Vorabinformationen, Bericht und Verlustliste. Schreiben läuft im Hintergrund-Thread.
use crate::app::App;
use crate::session::ExportOptions;
use crate::theme;
use eframe::egui;
use std::sync::mpsc::{channel, Receiver};

pub struct ExportState {
    pub with_outfit: bool,
    pub expressions: bool,
    pub format: usize,
    pub name: String,
    pub author: String,
    pub commercial: usize,
    pub redistribution: bool,
    pub report: Vec<String>,
    pub msg: String,
    pub rx: Option<Receiver<Result<String, String>>>,
    pub busy: bool,
    pub sub: usize,
    pub clips: Vec<(String, std::path::PathBuf)>,
    pub clips_loaded: bool,
    pub filter: String,
    pub clip_msg: String,
}

impl Default for ExportState {
    fn default() -> Self { Self { with_outfit: false, expressions: true, format: 0, name: "HASE Charakter".into(), author: String::new(), commercial: 0, redistribution: true, report: vec![], msg: String::new(), rx: None, busy: false, sub: 0, clips: vec![], clips_loaded: false, filter: String::new(), clip_msg: String::new() } }
}

const COMMERCIAL: [(&str, &str); 3] = [("personalNonProfit", "Privat, nicht kommerziell"), ("personalProfit", "Privat, auch kommerziell (z. B. Streaming)"), ("corporation", "Unternehmen")];

impl App {
    pub(crate) fn export_poll(&mut self) {
        if let Some(rx) = self.export.rx.as_ref() {
            if let Ok(r) = rx.try_recv() { self.export.busy = false; self.export.rx = None; self.export.msg = match r { Ok(m) => m, Err(e) => format!("Export fehlgeschlagen: {e}") }; }
        }
    }

    /// Animationsclips (VRMA) unter content/testcandidate/animations/*/ einmalig einlesen.
    fn scan_clips(&mut self) {
        let Some(s) = self.session.as_ref() else { return };
        let root = s.repo.join("content/testcandidate/animations");
        let mut v = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&root) { for pack in rd.flatten() { if let Ok(rd2) = std::fs::read_dir(pack.path()) { for f in rd2.flatten() { let p = f.path(); if p.extension().and_then(|x| x.to_str()) == Some("vrma") { v.push((format!("{} · {}", pack.file_name().to_string_lossy(), p.file_stem().unwrap().to_string_lossy().replace('_', " ")), p)); } } } } }
        v.sort(); self.export.clips = v; self.export.clips_loaded = true;
    }

    /// Panel für ein geöffnetes VRM-Modell: Informationen, Ausdrücke, einzelne Blendshapes, schließen.
    pub(crate) fn vrm_panel(&mut self, ui: &mut egui::Ui) {
        let s = self.session.as_mut().unwrap();
        let Some(v) = s.vrm.as_ref() else { return };
        let (name, parts, bones, hum, spring, path) = (v.model.name.clone(), v.model.parts.iter().map(|p| (p.name.clone(), p.pos.len(), p.targets.len())).collect::<Vec<_>>(), v.model.bones.len(), v.model.humanoid.len(), v.model.spring_bones, v.path.clone());
        let exprs: Vec<String> = v.model.expressions.iter().map(|e| e.0.clone()).collect();
        let cur: std::collections::BTreeMap<String, f32> = v.expr.clone();
        ui.heading(&name); ui.separator();
        ui.colored_label(theme::TEXT2, path.display().to_string());
        ui.label(format!("{} Teile · {} Bones · {} humanoide Zuordnungen{}", parts.len(), bones, hum, if spring { " · Spring Bones vorhanden" } else { "" }));
        for (n, vcount, tcount) in &parts { ui.colored_label(theme::TEXT2, format!("• {n}: {vcount} Vertices, {tcount} Blendshapes")); }
        ui.add_space(4.0);
        ui.colored_label(theme::WARN, "Spring Bones werden in der Vorschau nicht simuliert. Kleidung anprobieren: 2 Outfit · Posen: 3 Finalisieren.");
        {
            let v = s.vrm.as_ref().unwrap();
            let rows: Vec<(usize, String, bool, bool)> = v.model.parts.iter().enumerate().map(|(i, p)| (i, format!("{} ({})", p.name, p.pos.len()), !v.hidden.contains(&i), v.skin_parts.contains(&i))).collect();
            let info = v.dress_info.clone();
            ui.separator(); ui.strong("Teile");
            ui.colored_label(theme::TEXT2, "„Haut“ = Teile, auf die die Kleidung angepasst wird (Standard: der Körper).");
            let mut act: Option<(usize, bool, bool)> = None;
            for (i, n, vis, skin) in rows { ui.horizontal(|ui| { let (mut a, mut b) = (vis, skin); let c1 = ui.checkbox(&mut a, "").on_hover_text("sichtbar").changed(); let c2 = ui.checkbox(&mut b, "Haut").changed(); ui.label(n); if c1 { act = Some((i, true, a)); } if c2 { act = Some((i, false, b)); } }); }
            if let Some((i, is_vis, val)) = act { if is_vis { s.set_part_hidden(i, !val); } else { s.set_skin_part(i, val); } }
            if !info.is_empty() { ui.colored_label(theme::TEXT2, info); }
        }
        ui.separator(); ui.strong("Ausdrücke");
        let mut set: Option<(String, f32)> = None;
        for n in &exprs { let mut val = cur.get(n).copied().unwrap_or(0.0); if crate::ui_kit::slider(ui, n.as_str(), &mut val, 0.0..=1.0) { set = Some((n.clone(), val)); } }
        if let Some((n, val)) = set { s.set_expression(&n, val); }
        ui.horizontal(|ui| {
            if ui.button("Ausdrücke zurücksetzen").clicked() { for n in &exprs { s.set_expression(n, 0.0); } let keys: Vec<(usize, usize)> = s.vrm.as_ref().map(|v| v.raw.keys().copied().collect()).unwrap_or_default(); for (p, t) in keys { s.set_raw_target(p, t, 0.0); } }
            if ui.button("Modell schließen").on_hover_text("Zurück zum MakeHuman-Charakter").clicked() { s.close_vrm(); }
        });
        ui.separator();
        egui::CollapsingHeader::new("Einzelne Blendshapes").default_open(false).show(ui, |ui| {
            let list: Vec<(usize, usize, String)> = { let v = s.vrm.as_ref().unwrap(); v.model.parts.iter().enumerate().flat_map(|(pi, p)| p.targets.iter().enumerate().map(move |(ti, t)| (pi, ti, t.0.clone()))).collect() };
            let raw: std::collections::BTreeMap<(usize, usize), f32> = s.vrm.as_ref().unwrap().raw.clone();
            let mut chg: Option<(usize, usize, f32)> = None;
            egui::ScrollArea::vertical().max_height(320.0).id_salt("rawbs").show_rows(ui, 24.0, list.len(), |ui, range| { for r in range { let (pi, ti, n) = &list[r]; let mut val = raw.get(&(*pi, *ti)).copied().unwrap_or(0.0); if crate::ui_kit::slider(ui, n.as_str(), &mut val, 0.0..=1.0) { chg = Some((*pi, *ti, val)); } } });
            if let Some((p, t, val)) = chg { s.set_raw_target(p, t, val); }
        });
    }

    pub(crate) fn anim_panel(&mut self, ui: &mut egui::Ui) {
        use crate::icons::Icon;
        if !self.export.clips_loaded { self.scan_clips(); }
        self.panel_title(ui, Icon::Pose, "Posen & Animation");
        {
            let s = self.session.as_mut().unwrap();
            if s.generic && s.vrm.is_none() { ui.colored_label(theme::ERR, "Für importierte Körperbasen noch nicht unterstützt (Rig und Gewichte fehlen)."); return; }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if crate::ui_kit::icon_tile(ui, Icon::Reset, "Ruhepose", egui::vec2(72.0, 48.0), s.play.clip.is_none(), "Zurück zur Ruhepose (A-Pose)").clicked() { s.stop_pose(); }
                if s.play.clip.is_some() {
                    let playing = s.play.playing;
                    if crate::ui_kit::icon_tile(ui, if playing { Icon::Pause } else { Icon::Play }, if playing { "Pause" } else { "Abspielen" }, egui::vec2(72.0, 48.0), playing, "").clicked() { s.play.playing = !playing; }
                    let lp = s.play.looping;
                    if crate::ui_kit::icon_tile(ui, Icon::Rotate, "Schleife", egui::vec2(72.0, 48.0), lp, "Animation wiederholen").clicked() { s.play.looping = !lp; }
                }
            });
            if let Some(c) = s.play.clip.as_ref() {
                let dur = c.duration;
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("{} · {:.1} s", c.name.replace('_', " "), dur)).color(theme::TEXT2));
                let mut t = s.play.t;
                if crate::ui_kit::slider(ui, "Zeit (s)", &mut t, 0.0..=dur) { s.play.t = t; s.play.playing = false; }
                crate::ui_kit::slider(ui, "Tempo", &mut s.play.speed, 0.1..=2.0);
            }
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| { crate::icons::show(ui, Icon::Search, 16.0, theme::TEXT2); ui.add(egui::TextEdit::singleline(&mut self.export.filter).hint_text("Suche Posen und Animationen …").desired_width(f32::INFINITY)); });
        let f = self.export.filter.to_lowercase();
        let list: Vec<usize> = self.export.clips.iter().enumerate().filter(|(_, (n, _))| f.is_empty() || n.to_lowercase().contains(&f)).map(|(i, _)| i).collect();
        ui.colored_label(theme::TEXT2, format!("{} Clips (CC0, Quaternius Universal Animation Library). Kleidung und Haare folgen über die Körpergewichte.", list.len()));
        if !self.export.clip_msg.is_empty() { ui.colored_label(theme::WARN, &self.export.clip_msg); }
        ui.add_space(4.0);
        let cur = self.session.as_ref().and_then(|s| s.play.clip.as_ref().map(|c| c.name.clone())).unwrap_or_default();
        let mut pick: Option<usize> = None;
        ui.spacing_mut().item_spacing.y = 2.0;
        for &i in &list {
            let n = self.export.clips[i].0.clone();
            let label = n.split_once(" · ").map(|x| x.1).unwrap_or(&n).to_string();
            let active = !cur.is_empty() && cur.replace('_', " ") == label.replace('_', " ");
            if crate::ui_kit::nav_row(ui, Icon::Pose, &label, active).clicked() { pick = Some(i); }
        }
        if let Some(i) = pick { let p = self.export.clips[i].1.clone(); let s = self.session.as_mut().unwrap(); self.export.clip_msg = match s.load_clip(&p) { Ok(()) => String::new(), Err(e) => e }; }
    }

    pub(crate) fn export_panel(&mut self, ui: &mut egui::Ui) {
        use crate::icons::Icon;
        if self.session.is_none() { return; }
        self.panel_title(ui, Icon::Export, "Exportieren (VRM 1.0)");
        let s = self.session.as_ref().unwrap();
        if s.vrm.is_some() { ui.colored_label(theme::TEXT2, "Geöffnetes Modell: Es wird mit eigenem Skelett, Körper, Haaren und Ausdrücken exportiert; anprobierte Kleidung kommt dazu (Profil „mit Outfit“)."); ui.add_space(4.0); }
        let ex = &mut self.export;
        crate::ui_kit::section(ui, "Profil");
        let mut prof = ex.with_outfit as usize;
        if crate::ui_kit::tabs(ui, &["Basischarakter", "Mit Outfit"], &mut prof) { ex.with_outfit = prof == 1; }
        ui.colored_label(theme::TEXT2, if ex.with_outfit { "Charakter mit getragener Kleidung (verdeckte Körperflächen entfallen)." } else { "Nackter Basischarakter ohne Kleidung." });
        ui.add_space(4.0);
        crate::ui_kit::section(ui, "Format");
        crate::ui_kit::tabs(ui, &["VRM 1.0", "glTF (GLB)", "OBJ"], &mut ex.format);
        if ex.format == 1 { ui.colored_label(theme::TEXT2, "Schlichtes glTF 2.0: Skelett, Skinning, Morph-Targets, Texturen. Ohne VRM-Metadaten, Expressions-Zuordnung und Spring Bones."); }
        if ex.format == 2 { ui.colored_label(theme::TEXT2, "Statisches Mesh in Ruhepose (OBJ + MTL + PNG), ohne Skelett und Morphs."); }
        ui.checkbox(&mut ex.expressions, "Expressions (Blinzeln, Vokale, Gefühle)");
        crate::ui_kit::section(ui, "Metadaten");
        egui::Grid::new("meta").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label("Name"); ui.add(egui::TextEdit::singleline(&mut ex.name).desired_width(240.0)); ui.end_row();
            ui.label("Autor"); ui.add(egui::TextEdit::singleline(&mut ex.author).desired_width(240.0)); ui.end_row();
            ui.label("Nutzung"); egui::ComboBox::from_id_salt("commercial").width(240.0).selected_text(COMMERCIAL[ex.commercial].1).show_ui(ui, |ui| { for (i, c) in COMMERCIAL.iter().enumerate() { ui.selectable_value(&mut ex.commercial, i, c.1); } }); ui.end_row();
        });
        ui.checkbox(&mut ex.redistribution, "Weitergabe erlaubt");
        crate::ui_kit::section(ui, "Vorabinformationen");
        let (v, t) = s.counts();
        crate::ui_kit::card(ui, None, |ui| {
            ui.label(format!("Zielprofil: VRM 1.0 · {} Bones", if s.vrm.is_some() { s.vrm.as_ref().map(|x| x.model.bones.len()).unwrap_or(0) } else { 53 }));
            ui.label(format!("Sichtbar im Editor: {v} Vertices, {t} Dreiecke ({} Assets)", s.active_asset_count()));
            if t > 70_000 { ui.colored_label(theme::WARN, "Über 70.000 Dreiecke – für manche Avatar-Plattformen zu hoch."); }
            if s.generic { ui.colored_label(theme::ERR, "Export für importierte Körperbasen ist noch nicht unterstützt (Rig- und Gewichtsübernahme aus dem Import fehlt)."); }
            ui.colored_label(theme::TEXT2, "Wird nicht exportiert: Haar-Farbverlauf (nur Mittelfarbe), Augen-Reflexion, einzelne Hautdetail-Ebenen (eingerechnet), Pupillenform; Haarphysik/Spring Bones sind noch nicht gesetzt.");
        });
        ui.add_space(8.0);
        let enabled = !ex.busy && !s.generic;
        let busy = ex.busy;
        let mut go = false;
        ui.add_enabled_ui(enabled, |ui| { if crate::ui_kit::primary_button(ui, if busy { "Schreibe …" } else { "Exportieren …" }).clicked() { go = true; } });
        if go { self.export_start(); }
        if !self.export.msg.is_empty() { ui.add_space(4.0); ui.colored_label(theme::TEXT2, &self.export.msg); }
        for r in &self.export.report { ui.colored_label(if r.starts_with("Verlust") { theme::WARN } else { theme::TEXT2 }, r); }
    }

    fn export_start(&mut self) {
        let ex = &self.export;
        let fmt = ex.format;
        let (ext, fname) = [("vrm", "VRM 1.0"), ("glb", "glTF (GLB)"), ("obj", "Wavefront OBJ")][fmt.min(2)];
        let default = format!("{}.{ext}", hase_core::packages::slug(&ex.name));
        let Some(path) = rfd::FileDialog::new().add_filter(fname, &[ext]).set_file_name(default).save_file() else { return };
        let o = ExportOptions { with_outfit: ex.with_outfit, expressions: ex.expressions, name: ex.name.clone(), author: if ex.author.trim().is_empty() { "HASE VRM MAKER".into() } else { ex.author.clone() }, license_url: "https://vrm.dev/licenses/1.0/".into(), commercial: COMMERCIAL[ex.commercial].0.into(), redistribution: ex.redistribution };
        let Some(s) = self.session.as_mut() else { return };
        let t0 = std::time::Instant::now();
        match s.build_export(&o) {
            Err(e) => { self.export.msg = format!("Export nicht möglich: {e}"); }
            Ok((doc, report)) => {
                let build_ms = t0.elapsed().as_secs_f64() * 1e3;
                self.export.report = report; self.export.busy = true; self.export.msg = "Schreibe Datei …".into();
                let (tx, rx) = channel(); self.export.rx = Some(rx);
                let ctx_repaint = self.ctx.clone();
                std::thread::spawn(move || {
                    let t1 = std::time::Instant::now();
                    let r = if fmt == 2 { hase_core::export::write_obj(&doc, &path).map(|(t, f)| format!("Gespeichert: {} ({t} Dreiecke; weitere Dateien: {})", path.display(), f.join(", "))) } else { hase_core::export::write_glb_opt(&doc, fmt == 0).and_then(|glb| {
                        std::fs::write(&path, &glb).map_err(|e| e.to_string())?;
                        // Rücklesen-Prüfung: die Datei muss als glTF parsebar sein
                        let (m, mat, im, mt) = hase_core::export::verify_glb(&glb)?;
                        Ok(format!("Gespeichert: {} ({:.1} MB, {m} Meshes, {mat} Materialien, {im} Bilder, {mt} Morph-Targets; Zusammenstellung {build_ms:.0} ms, Schreiben {:.0} ms im Hintergrund; Rücklesen OK)", path.display(), glb.len() as f64 / 1048576.0, t1.elapsed().as_secs_f64() * 1e3))
                    }) };
                    let _ = tx.send(r);
                    ctx_repaint.request_repaint();
                });
            }
        }
    }
}
