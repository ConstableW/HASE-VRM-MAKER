//! Finalisieren: VRM-Export mit Profilen, Metadaten, Vorabinformationen, Bericht und Verlustliste. Schreiben läuft im Hintergrund-Thread.
use crate::app::App;
use crate::session::ExportOptions;
use crate::theme;
use eframe::egui;
use std::sync::mpsc::{channel, Receiver};

pub struct ExportState {
    pub with_outfit: bool,
    pub expressions: bool,
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
    fn default() -> Self { Self { with_outfit: false, expressions: true, name: "HASE Charakter".into(), author: String::new(), commercial: 0, redistribution: true, report: vec![], msg: String::new(), rx: None, busy: false, sub: 0, clips: vec![], clips_loaded: false, filter: String::new(), clip_msg: String::new() } }
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

    pub(crate) fn anim_panel(&mut self, ui: &mut egui::Ui) {
        if !self.export.clips_loaded { self.scan_clips(); }
        ui.heading("Posen & Animation"); ui.separator();
        {
            let s = self.session.as_mut().unwrap();
            if s.generic { ui.colored_label(theme::ERR, "Für importierte Körperbasen noch nicht unterstützt (Rig und Gewichte fehlen)."); return; }
            ui.horizontal(|ui| {
                if ui.button("Ruhepose (A-Pose)").clicked() { s.stop_pose(); }
                if s.play.clip.is_some() {
                    if ui.button(if s.play.playing { "Pause" } else { "Abspielen" }).clicked() { s.play.playing = !s.play.playing; }
                    ui.checkbox(&mut s.play.looping, "Schleife");
                }
            });
            if let Some(c) = s.play.clip.as_ref() {
                let dur = c.duration;
                ui.label(format!("{} · {:.1} s", c.name.replace('_', " "), dur));
                let mut t = s.play.t;
                if ui.add(egui::Slider::new(&mut t, 0.0..=dur).text("Zeit (s)")).changed() { s.play.t = t; s.play.playing = false; }
                ui.add(egui::Slider::new(&mut s.play.speed, 0.1..=2.0).text("Tempo"));
            }
        }
        ui.horizontal(|ui| { ui.label("Suche:"); ui.text_edit_singleline(&mut self.export.filter); });
        let f = self.export.filter.to_lowercase();
        let list: Vec<usize> = self.export.clips.iter().enumerate().filter(|(_, (n, _))| f.is_empty() || n.to_lowercase().contains(&f)).map(|(i, _)| i).collect();
        ui.weak(format!("{} Clips (CC0, Quaternius Universal Animation Library als VRMA). Die Pose nutzt das aktuelle Rig des Charakters; Kleidung/Haare folgen über die Körpergewichte.", list.len()));
        if !self.export.clip_msg.is_empty() { ui.colored_label(theme::WARN, &self.export.clip_msg); }
        let mut pick: Option<usize> = None;
        egui::ScrollArea::vertical().id_salt("clips").auto_shrink([false; 2]).show_rows(ui, 22.0, list.len(), |ui, range| { for r in range { let i = list[r]; if ui.selectable_label(false, &self.export.clips[i].0).clicked() { pick = Some(i); } } });
        if let Some(i) = pick { let p = self.export.clips[i].1.clone(); let s = self.session.as_mut().unwrap(); self.export.clip_msg = match s.load_clip(&p) { Ok(()) => String::new(), Err(e) => e }; }
    }

    pub(crate) fn export_panel(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.session.as_ref() else { return };
        ui.heading("Exportieren (VRM 1.0)"); ui.separator();
        let ex = &mut self.export;
        ui.strong("Profil");
        ui.radio_value(&mut ex.with_outfit, false, "Basischarakter ohne Outfit");
        ui.radio_value(&mut ex.with_outfit, true, "Charakter mit Outfit (verdeckte Körperflächen entfallen)");
        ui.checkbox(&mut ex.expressions, "Expressions (Blinzeln, Vokale, Gefühle) aus MakeHuman-Targets");
        ui.add_space(6.0);
        ui.strong("Metadaten");
        egui::Grid::new("meta").num_columns(2).show(ui, |ui| {
            ui.label("Name"); ui.text_edit_singleline(&mut ex.name); ui.end_row();
            ui.label("Autor"); ui.text_edit_singleline(&mut ex.author); ui.end_row();
            ui.label("Nutzung"); egui::ComboBox::from_id_salt("commercial").selected_text(COMMERCIAL[ex.commercial].1).show_ui(ui, |ui| { for (i, c) in COMMERCIAL.iter().enumerate() { ui.selectable_value(&mut ex.commercial, i, c.1); } }); ui.end_row();
        });
        ui.checkbox(&mut ex.redistribution, "Weitergabe erlaubt");
        ui.add_space(6.0);
        ui.strong("Vorabinformationen");
        let (v, t) = s.counts();
        ui.label(format!("Zielprofil: VRM 1.0 (generisch) · Skelett 53 Bones, 15 Pflicht-Bones im Humanoid-Mapping"));
        ui.label(format!("Sichtbar im Editor: {v} Vertices, {t} Dreiecke ({} Assets)", s.active_asset_count()));
        if t > 70_000 { ui.colored_label(theme::WARN, "Hinweis: über 70.000 Dreiecke – für manche VTuber-/Avatar-Plattformen zu hoch."); }
        if s.generic { ui.colored_label(theme::ERR, "Export für importierte Körperbasen ist noch nicht unterstützt (Rig- und Gewichtsübernahme aus dem Import fehlt)."); }
        ui.colored_label(theme::TEXT2, "Wird nicht exportiert: Haar-Farbverlauf/Highlights (nur Mittelfarbe), Augen-Reflexion, einzelne Hautdetail-Ebenen (eingerechnet), Pupillenform; Haarphysik/Spring Bones sind noch nicht gesetzt.");
        ui.add_space(8.0);
        let enabled = !ex.busy && !s.generic;
        if ui.add_enabled(enabled, egui::Button::new(if ex.busy { "Schreibe …" } else { "Exportieren …" })).clicked() { self.export_start(); }
        if !self.export.msg.is_empty() { ui.colored_label(theme::TEXT2, &self.export.msg); }
        for r in &self.export.report { ui.colored_label(if r.starts_with("Verlust") { theme::WARN } else { theme::TEXT2 }, r); }
    }

    fn export_start(&mut self) {
        let ex = &self.export;
        let default = format!("{}.vrm", hase_core::packages::slug(&ex.name));
        let Some(path) = rfd::FileDialog::new().add_filter("VRM 1.0", &["vrm"]).set_file_name(default).save_file() else { return };
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
                    let r = hase_core::export::write_glb(&doc).and_then(|glb| {
                        std::fs::write(&path, &glb).map_err(|e| e.to_string())?;
                        // Rücklesen-Prüfung: die Datei muss als glTF parsebar sein
                        let (m, mat, im, mt) = hase_core::export::verify_glb(&glb)?;
                        Ok(format!("Gespeichert: {} ({:.1} MB, {m} Meshes, {mat} Materialien, {im} Bilder, {mt} Morph-Targets; Zusammenstellung {build_ms:.0} ms, Schreiben {:.0} ms im Hintergrund; Rücklesen OK)", path.display(), glb.len() as f64 / 1048576.0, t1.elapsed().as_secs_f64() * 1e3))
                    });
                    let _ = tx.send(r);
                    ctx_repaint.request_repaint();
                });
            }
        }
    }
}
