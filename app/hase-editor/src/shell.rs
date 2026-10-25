//! Hauptlayout nach dem verbindlichen Mockup: Kopfzeile, linke Navigation, Viewport mit Werkzeugen, rechter Editor, untere Presetleiste.
use crate::app::{App, Tool, Workspace};
use crate::camera::{LIGHTS, PRESETS};
use crate::icons::{self, Icon};
use crate::regions::{self, Main, CLOTHES_KINDS, REGIONS};
use crate::theme;
use crate::ui_kit::{self, acc_header, card, icon_tile, nav_row, tabs};
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Margin, Stroke, StrokeKind};

fn bg_frame(l: i8, r: i8) -> egui::Frame { egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: l, right: r, top: 8, bottom: 8 }) }

impl App {
    pub(crate) fn layout(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.header(ui, ctx);
        egui::Panel::bottom("status").frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin::symmetric(14, 3))).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status).size(12.0).color(theme::TEXT2));
                if let Some(e) = &self.error { ui.colored_label(theme::ERR, e); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(egui::RichText::new("HASE VRM MAKER – Entwicklungsstand").size(11.0).color(theme::TEXT2)); });
            });
        });
        self.load_modal(ctx);
        self.base_picker_window(ctx);
        self.lib_pending();
        self.lib_window(ctx);
        let vrm = self.session.as_ref().map(|s| s.vrm.is_some()).unwrap_or(false);
        if !self.panels_hidden {
            egui::Panel::left("left").frame(bg_frame(8, 4)).default_size(262.0).min_size(220.0).max_size(360.0).show(ui, |ui| self.left_column(ui, vrm));
            egui::Panel::right("right").frame(bg_frame(4, 8)).default_size(420.0).min_size(320.0).max_size(620.0).show(ui, |ui| self.right_column(ui, ctx, vrm));
            if self.ws == Workspace::Charakter && !self.expert && !vrm && self.rtab != 3 {
                egui::Panel::bottom("presetbar").frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: 4, right: 4, top: 0, bottom: 8 })).min_size(150.0).default_size(176.0).show(ui, |ui| self.preset_bar(ui));
            }
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: 4, right: 4, top: 8, bottom: 8 })).show(ui, |ui| self.viewport(ui));
    }

    // ---------------------------------------------------------------------------------------------------------- Kopfzeile
    fn header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::Panel::top("nav").frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin { left: 14, right: 14, top: 8, bottom: 4 })).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                // Logo
                let (lr, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                logo(ui.painter(), lr);
                ui.vertical(|ui| { ui.add_space(2.0); ui.label(egui::RichText::new("HASE VRM MAKER").strong().size(18.0)); ui.label(egui::RichText::new("Character Studio").size(13.0).color(theme::TEXT2)); });
                ui.add_space(26.0);
                for (i, (t, sub, ws)) in [("Charakter", "Anatomie & Merkmale", Workspace::Charakter), ("Outfit", "Kleidung & Accessoires", Workspace::Outfit), ("Finalisieren", "Presets, Export & DB", Workspace::Finalisieren)].into_iter().enumerate() {
                    if self.ws_tab(ui, i + 1, t, sub, self.ws == ws) { self.switch_ws(ws); }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let sz = egui::vec2(68.0, 54.0);
                    let r = icon_tile(ui, Icon::Settings, "Einstellungen", sz, false, "Ansicht, Leistung, Hilfe");
                    egui::Popup::menu(&r).show(|ui| {
                        ui.set_min_width(220.0);
                        ui.checkbox(&mut self.show_stats, "Leistungsanzeige (F3)");
                        ui.checkbox(&mut self.grid, "Bodenraster");
                        if let Some(s) = self.session.as_mut() { ui.checkbox(&mut s.symmetric, "Symmetrie (links = rechts)"); }
                        ui.separator();
                        ui.label("Maus: links drehen · rechts/Mitte verschieben · Rad zoomen");
                        ui.label("Doppelklick: Kamera zurücksetzen · 1–5: Ansichten");
                        ui.label("Regler: Doppelklick = Standardwert · Strg+Z/Y = Undo/Redo");
                    });
                    if icon_tile(ui, Icon::Species, "Werkzeuge", sz, self.ws == Workspace::Studio, "Körperbasen & Spezies erstellen, importieren, prüfen").clicked() { self.switch_ws(Workspace::Studio); }
                    if icon_tile(ui, Icon::Export, "Export", sz, false, "VRM 1.0 exportieren (Finalisieren › Export)").clicked() { self.goto_export(); }
                    if icon_tile(ui, Icon::Save, "Speichern", sz, false, "In der Bibliothek speichern (Strg+S)").clicked() { self.quick_save(); }
                    if icon_tile(ui, Icon::Library, "Bibliothek", sz, self.lib.open, "Charaktere, Vorlagen, Varianten, Snapshots").clicked() { self.lib.open = !self.lib.open; }
                    let r = icon_tile(ui, Icon::Project, "Projekt", sz, false, "Datei: Laden, Speichern unter, Modell öffnen");
                    egui::Popup::menu(&r).show(|ui| {
                        ui.set_min_width(240.0);
                        if ui.button("Neuer Charakter (realistisch, MakeHuman-Basis)").clicked() { if let Some(s) = self.session.as_mut() { if s.vrm.is_some() { s.close_vrm(); } s.new_character(); } self.lib.current = None; ui.close(); }
                        if ui.button("Neuer Anime-/VTuber-Charakter …").on_hover_text("Basis wählen (z. B. ein VRoid-Export) und von Grund auf gestalten").clicked() { if let Some(s) = self.session.as_ref() { self.bases = crate::anime_bases::list(&s.repo); } self.base_picker = true; ui.close(); }
                        if ui.button("Aus Datei laden …").clicked() { self.load_dialog(); ui.close(); }
                        if ui.button("Als Datei speichern …").clicked() { self.save_dialog(); ui.close(); }
                        ui.separator();
                        if ui.button("VRM/GLB als Basis öffnen …").on_hover_text("Datei ansehen und als Ausgangsform für neue Charaktere verwenden").clicked() { self.open_vrm_dialog(); ui.close(); }
                        ui.separator();
                        if ui.button("Beenden").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                    });
                });
            });
        });
    }

    fn ws_tab(&mut self, ui: &mut egui::Ui, n: usize, title: &str, sub: &str, active: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(214.0, 48.0), egui::Sense::click());
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::same(8), if active { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::PANEL });
        p.rect_stroke(rect, CornerRadius::same(8), Stroke::new(if active { 1.5 } else { 1.0 }, if active { theme::ACCENT } else { theme::LINE }), StrokeKind::Inside);
        let nb = egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 8.0), egui::vec2(32.0, 32.0));
        p.rect_filled(nb, CornerRadius::same(6), if active { Color32::from_rgb(0x0F, 0x2F, 0x57) } else { theme::PANEL2 });
        p.rect_stroke(nb, CornerRadius::same(6), Stroke::new(1.0, if active { theme::ACCENT } else { theme::LINE }), StrokeKind::Inside);
        p.text(nb.center(), Align2::CENTER_CENTER, n.to_string(), FontId::proportional(18.0), Color32::WHITE);
        p.text(egui::pos2(rect.left() + 50.0, rect.top() + 17.0), Align2::LEFT_CENTER, title, FontId::proportional(16.5), Color32::WHITE);
        p.text(egui::pos2(rect.left() + 50.0, rect.top() + 34.0), Align2::LEFT_CENTER, sub, FontId::proportional(11.5), if active { Color32::from_rgb(0xC6, 0xDA, 0xF2) } else { theme::TEXT2 });
        resp.clicked()
    }

    pub(crate) fn switch_ws(&mut self, ws: Workspace) {
        self.ws = ws;
        self.main = match ws { Workspace::Charakter => Main::Charakter, Workspace::Outfit => Main::Kleidung, Workspace::Finalisieren => Main::Animation, Workspace::Studio => Main::Charakter };
        self.region = usize::MAX; self.sub = 0; self.asset_filter.clear();
        if self.saved_cam.is_some() { self.saved_cam = None; }
        self.camera = crate::camera::Camera::default();
    }

    // ---------------------------------------------------------------------------------------------------------- Linke Spalte
    fn left_column(&mut self, ui: &mut egui::Ui, vrm: bool) {
        match self.ws {
            Workspace::Studio => { card(ui, Some((Icon::Species, "Körperbasen & Spezies")), |ui| self.studio_library(ui)); return; }
            _ if vrm && self.ws != Workspace::Outfit && self.ws != Workspace::Finalisieren => {
                let h = ui.available_height();
                card(ui, Some((Icon::Areas, "Avatar erstellen")), |ui| {
                    let list_h = (ui.available_height() - 84.0).max(100.0);
                    egui::ScrollArea::vertical().id_salt("navlist_vrm").max_height(list_h).auto_shrink([false, false]).show(ui, |ui| { ui.spacing_mut().item_spacing.y = 2.0; self.avatar_area_list(ui); });
                    ui_kit::hr(ui); ui.add_space(4.0);
                    ui.spacing_mut().item_spacing.y = 4.0;
                    if ui.add_sized([ui.available_width(), 28.0], egui::Button::new("Ansicht zurücksetzen")).clicked() { self.camera = crate::camera::Camera::default(); self.saved_cam = None; }
                    if ui.add_sized([ui.available_width(), 28.0], egui::Button::new("Alle Regler zurücksetzen")).clicked() { if let Some(s) = self.session.as_mut() { let all: Vec<usize> = (0..hase_core::avatar::DEFS.len()).collect(); s.avatar_reset(&all); } }
                });
                let _ = h;
                return;
            }
            _ => {}
        }
        if self.ws == Workspace::Charakter { self.species_card(ui); ui.add_space(8.0); }
        let h = ui.available_height();
        card(ui, Some((Icon::Areas, match self.ws { Workspace::Charakter => "Bereiche", Workspace::Outfit => "Outfit", _ => "Finalisieren" })), |ui| {
            let list_h = (ui.available_height() - 84.0).max(100.0);
            let _ = h;
            egui::ScrollArea::vertical().id_salt("navlist").max_height(list_h).auto_shrink([false, false]).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                match self.ws {
                    Workspace::Charakter => self.area_list(ui),
                    Workspace::Outfit => { for (i, l) in CLOTHES_KINDS.iter().enumerate() { if nav_row(ui, [Icon::Outfit, Icon::Torso, Icon::Leg, Icon::Foot, Icon::Star][i], l, self.sub == i).clicked() { self.sub = i; } } }
                    _ => {
                        for (i, (l, ic)) in [("Posen & Animation", Icon::Pose), ("Export (VRM 1.0)", Icon::Export)].iter().enumerate() { if nav_row(ui, *ic, l, self.export.sub == i).clicked() { self.export.sub = i; } }
                        if nav_row(ui, Icon::Library, "Vorlagen & Varianten", self.lib.open).clicked() { self.lib.open = true; }
                    }
                }
            });
            ui_kit::hr(ui);
            ui.add_space(4.0);
            ui.spacing_mut().item_spacing.y = 4.0;
            if ui.add_sized([ui.available_width(), 28.0], egui::Button::new("Ansicht zurücksetzen")).clicked() { self.camera = crate::camera::Camera::default(); self.saved_cam = None; }
            if let Some(s) = self.session.as_mut() {
                let on = s.symmetric;
                let (r, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
                ui.painter().rect_filled(r, CornerRadius::same(6), if on { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::PANEL2 });
                ui.painter().rect_stroke(r, CornerRadius::same(6), Stroke::new(1.0, if on { theme::ACCENT } else { theme::LINE }), StrokeKind::Inside);
                icons::paint(ui.painter(), egui::Rect::from_center_size(egui::pos2(r.left() + 18.0, r.center().y), egui::vec2(16.0, 16.0)), Icon::Symmetry, if on { Color32::WHITE } else { theme::TEXT2 });
                ui.painter().text(egui::pos2(r.left() + 34.0, r.center().y), Align2::LEFT_CENTER, if on { "Symmetrie: an" } else { "Symmetrie: aus" }, FontId::proportional(13.5), if on { Color32::WHITE } else { theme::TEXT2 });
                if resp.on_hover_text("Links und rechts gemeinsam bearbeiten").clicked() { s.symmetric = !on; }
            }
        });
    }

    fn species_card(&mut self, ui: &mut egui::Ui) {
        if self.session.as_ref().map(|s| s.vrm.is_some()).unwrap_or(true) { return; }
        card(ui, Some((Icon::Species, "Spezies")), |ui| {
            {
                let s = self.session.as_mut().unwrap();
                let compat = s.species_indices();
                let cur = s.species_opt().unwrap_or(usize::MAX);
                let mut sel = cur;
                let pos = compat.iter().position(|&i| i == cur);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let w = ui.available_width() - 2.0 * 30.0 - 8.0;
                    if ui.add_sized([30.0, 28.0], egui::Button::new("‹")).clicked() { if let Some(p) = pos { sel = compat[(p + compat.len() - 1) % compat.len()]; } }
                    egui::ComboBox::from_id_salt("species").width(w - 14.0).selected_text(s.species_opt().map(|i| s.packages.species[i].label.clone()).unwrap_or_else(|| "(keine Spezies)".into())).show_ui(ui, |ui| {
                        for &i in &compat { let sp = &s.packages.species[i]; ui.selectable_value(&mut sel, i, format!("{}{}", sp.label, if sp.stub { "  (Stub)" } else { "" })); }
                    });
                    if ui.add_sized([30.0, 28.0], egui::Button::new("›")).clicked() { if let Some(p) = pos { sel = compat[(p + 1) % compat.len()]; } }
                });
                if sel != cur && sel != usize::MAX { s.apply_species(sel); self.rows_key.1 = usize::MAX; }
            }
            // Preset-Vorschau (erste fünf)
            let presets: Vec<(String, std::collections::BTreeMap<String, f32>)> = { let s = self.session.as_ref().unwrap(); s.species_opt().map(|i| s.packages.species[i].presets.iter().take(5).map(|p| (p.label.clone(), p.params.clone())).collect()).unwrap_or_default() };
            if !presets.is_empty() {
                ui.add_space(6.0);
                let tw = ((ui.available_width() - 4.0 * 4.0) / 5.0).floor();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (label, map) in presets {
                        let st = self.session.as_ref().unwrap().state_with(&map);
                        if self.tile_card(ui, tw, tw * 1.2, st, "head", "", false) { if let Some(s) = self.session.as_mut() { s.apply_map(&map); } }
                        let _ = label;
                    }
                });
            }
        });
    }

    /// Bereichsliste der Charakter-Phase: Gesamtansicht, Gesicht, Körper, Haare & Stil.
    fn area_list(&mut self, ui: &mut egui::Ui) {
        let mut go: Option<(Main, usize, usize)> = None;
        if nav_row(ui, Icon::Overview, "Gesamtansicht", self.main == Main::Charakter && self.sub == 0).clicked() { go = Some((Main::Charakter, usize::MAX, 0)); }
        for (m, t) in [(Main::Gesicht, "Gesicht"), (Main::Koerper, "Körper")] {
            ui_kit::section(ui, t);
            for (i, r) in regions::regions_of(m) { if nav_row(ui, icons::for_region(r.label), r.label, self.main == m && self.region == i).clicked() { go = Some((m, i, 0)); } }
        }
        ui_kit::section(ui, "Haare & Stil");
        if nav_row(ui, Icon::Hair, "Frisur", self.main == Main::Haare).clicked() { go = Some((Main::Haare, usize::MAX, 0)); }
        for (i, (l, ic)) in [("Haut", Icon::Skin), ("Make-up", Icon::Makeup), ("Details", Icon::Details), ("Augenfarbe (siehe Augen)", Icon::Eye), ("Brauen & Wimpern", Icon::Lashes)].iter().enumerate() {
            if nav_row(ui, *ic, l, self.main == Main::Stil && self.sub == i).clicked() { if i == 3 { go = Some((Main::Gesicht, REGIONS.iter().position(|r| r.label == "Augen").unwrap(), 0)); } else { go = Some((Main::Stil, usize::MAX, i)); } }
        }
        if let Some((m, r, sub)) = go {
            self.main = m; self.region = r; self.sub = sub;
            if self.rtab != 0 && self.rtab != 3 || m == Main::Haare || m == Main::Stil { if self.rtab != 3 { self.rtab = 1; } }
            if r != usize::MAX { self.open_regions.clear(); self.open_regions.insert(r); self.scroll_to = Some(r); self.focus(REGIONS[r].cam); }
            else if m == Main::Haare || (m == Main::Stil && sub == 1) { self.focus(regions::CAM_HEAD); } else { self.focus(regions::CAM_BODY); }
        }
    }

    // ---------------------------------------------------------------------------------------------------------- Rechte Spalte
    fn right_column(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, vrm: bool) {
        if self.session.is_none() { card(ui, None, |ui| { ui.label("Lade …"); }); return; }
        let ws = self.ws;
        card(ui, None, |ui| {
            ui.set_min_height(ui.available_height() - 22.0);
            if self.lib.open { egui::ScrollArea::vertical().id_salt("libscroll").auto_shrink([false, false]).show(ui, |ui| { ui.set_width(ui.available_width()); self.lib_panel(ui, ctx); }); return; }
            if ws == Workspace::Charakter && !vrm {
                let mut t = self.rtab;
                if tabs(ui, &["Presets", "Parameter", "Fein", "Erweitert"], &mut t) { self.rtab = t; self.expert = t == 3; }
                ui.add_space(6.0);
            }
            egui::ScrollArea::vertical().id_salt("rightscroll").auto_shrink([false, false]).show(ui, |ui| { ui.set_width(ui.available_width()); self.detail_panel(ui, ctx); });
        });
    }

    /// Akkordeon der Regionen einer Hauptgruppe; `fine` = nur Feinparameter (Position/Winkel, Asymmetrie).
    pub(crate) fn region_accordion(&mut self, ui: &mut egui::Ui, main: Main, fine: bool) {
        let vrm = self.session.as_ref().map(|s| s.vrm.is_some()).unwrap_or(false);
        let regs: Vec<usize> = regions::regions_of(main).map(|(i, _)| i).filter(|&i| !vrm || hase_core::avatar::DEFS.iter().any(|d| d.region == REGIONS[i].label)).collect();
        if vrm && fine { ui.colored_label(theme::TEXT2, "Für Avatare gibt es keine getrennte Feinabstimmung – alle Regler stehen unter „Parameter“."); return; }
        for i in regs {
            let r = &REGIONS[i];
            let open = self.open_regions.contains(&i);
            let resp = acc_header(ui, icons::for_region(r.label), r.label, open);
            if self.scroll_to == Some(i) { resp.scroll_to_me(Some(egui::Align::TOP)); self.scroll_to = None; }
            if resp.clicked() {
                if open { self.open_regions.remove(&i); } else { self.open_regions.insert(i); self.region = i; self.focus(r.cam); }
            }
            if open {
                ui.add_space(6.0);
                if vrm { self.avatar_rows(ui, r.label); } else if r.label == "Augen" && !fine { self.eye_studio(ui); } else { self.region_sliders(ui, Some(i), false, fine); }
                ui.add_space(8.0);
            }
        }
    }

    // ---------------------------------------------------------------------------------------------------------- Viewport-Overlays
    pub(crate) fn viewport_overlays(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let p = ui.painter().clone();
        // abgerundete Ecken + Rahmen
        round_mask(&p, rect, 10.0, theme::BG);
        p.rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        // Werkzeuge links
        let mut y = rect.top() + 12.0;
        for (t, ic, n, tip) in [(Tool::Pan, Icon::Move, "Bewegen", "Ziehen verschiebt die Ansicht"), (Tool::Orbit, Icon::Rotate, "Rotieren", "Ziehen dreht die Kamera"), (Tool::Zoom, Icon::Zoom, "Zoomen", "Ziehen zoomt (auch Mausrad)"), (Tool::Focus, Icon::Focus, "Fokus", "Klick auf den Körper zentriert die Kamera dort"), (Tool::Region, Icon::Region, "Bereich", "Klick auf eine Körperstelle öffnet den passenden Bearbeitungsbereich")] {
            let r = egui::Rect::from_min_size(egui::pos2(rect.left() + 12.0, y), egui::vec2(58.0, 58.0));
            if ui.put(r, |ui: &mut egui::Ui| icon_tile(ui, ic, n, egui::vec2(58.0, 58.0), self.tool == t, tip)).clicked() { self.tool = t; }
            y += 64.0;
        }
        // Licht/Hintergrund oben rechts
        let w = 3.0 * 74.0;
        let top = rect.top() + 12.0;
        let seg = egui::Rect::from_min_size(egui::pos2(rect.right() - w - 14.0, top), egui::vec2(w, 28.0));
        p.rect_filled(seg, CornerRadius::same(6), Color32::from_rgba_unmultiplied(16, 24, 36, 220));
        for (i, l) in LIGHTS.iter().enumerate() {
            let r = egui::Rect::from_min_size(seg.min + egui::vec2(i as f32 * 74.0, 0.0), egui::vec2(74.0, 28.0));
            let resp = ui.interact(r, egui::Id::new(("light", i)), egui::Sense::click());
            let sel = self.light == i;
            if sel { p.rect_filled(r.shrink(2.0), CornerRadius::same(5), theme::ACTIVE); p.rect_stroke(r.shrink(2.0), CornerRadius::same(5), Stroke::new(1.0, theme::ACCENT), StrokeKind::Inside); }
            else if resp.hovered() { p.rect_filled(r.shrink(2.0), CornerRadius::same(5), theme::HOVER); }
            p.text(r.center(), Align2::CENTER_CENTER, l.name, FontId::proportional(13.0), Color32::WHITE);
            if resp.clicked() { self.light = i; }
        }
        p.rect_stroke(seg, CornerRadius::same(6), Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        // Hintergrund-/Licht-Vorschaufelder
        for (i, l) in LIGHTS.iter().enumerate() {
            let r = egui::Rect::from_min_size(egui::pos2(seg.left() + i as f32 * 74.0 + 17.0, seg.bottom() + 8.0), egui::vec2(40.0, 40.0));
            let resp = ui.interact(r, egui::Id::new(("lightsw", i)), egui::Sense::click());
            let c = |a: [f32; 4]| Color32::from_rgb((a[0].powf(1.0 / 2.2) * 255.0) as u8, (a[1].powf(1.0 / 2.2) * 255.0) as u8, (a[2].powf(1.0 / 2.2) * 255.0) as u8);
            let mut mesh = egui::Mesh::default();
            mesh.colored_vertex(r.left_top(), c(l.bg_top)); mesh.colored_vertex(r.right_top(), c(l.bg_top)); mesh.colored_vertex(r.right_bottom(), c(l.bg_bot)); mesh.colored_vertex(r.left_bottom(), c(l.bg_bot));
            mesh.add_triangle(0, 1, 2); mesh.add_triangle(0, 2, 3);
            p.add(egui::Shape::mesh(mesh));
            p.rect_stroke(r, CornerRadius::same(5), Stroke::new(if self.light == i { 2.0 } else { 1.0 }, if self.light == i { theme::ACCENT } else if resp.hovered() { theme::TEXT2 } else { theme::LINE }), StrokeKind::Inside);
            if resp.on_hover_text(l.name).clicked() { self.light = i; }
        }
        // Kameraansichten rechts
        let mut y = rect.top() + 132.0;
        let cam_w = 118.0;
        let views = [PRESETS.len()];
        let _ = views;
        let panel = egui::Rect::from_min_size(egui::pos2(rect.right() - cam_w - 14.0, y - 6.0), egui::vec2(cam_w, PRESETS.len() as f32 * 40.0 + 6.0));
        p.rect_filled(panel, CornerRadius::same(8), Color32::from_rgba_unmultiplied(16, 24, 36, 205));
        p.rect_stroke(panel, CornerRadius::same(8), Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        for (i, (n, c)) in PRESETS.iter().enumerate() {
            let r = egui::Rect::from_min_size(egui::pos2(panel.left() + 4.0, y), egui::vec2(cam_w - 8.0, 36.0));
            let resp = ui.interact(r, egui::Id::new(("camv", i)), egui::Sense::click());
            let active = (self.camera.yaw - c.yaw).abs() < 0.02 && (self.camera.dist - c.dist).abs() < 0.02 && (self.camera.target[1] - c.target[1]).abs() < 0.02;
            if active { p.rect_filled(r, CornerRadius::same(5), theme::ACTIVE); p.rect_stroke(r, CornerRadius::same(5), Stroke::new(1.0, theme::ACCENT), StrokeKind::Inside); } else if resp.hovered() { p.rect_filled(r, CornerRadius::same(5), theme::HOVER); }
            icons::paint(&p, egui::Rect::from_center_size(egui::pos2(r.left() + 18.0, r.center().y), egui::vec2(22.0, 22.0)), [Icon::Overview, Icon::Overview, Icon::Back, Icon::Head, Icon::Torso][i.min(4)], if active { Color32::WHITE } else { theme::TEXT2 });
            p.text(egui::pos2(r.left() + 36.0, r.center().y), Align2::LEFT_CENTER, *n, FontId::proportional(13.0), Color32::WHITE);
            if resp.clicked() { self.camera = *c; self.saved_cam = None; }
            y += 40.0;
        }
        // untere Leiste (Undo/Redo/Zurücksetzen) + Vollbild
        let (can_u, can_r) = self.session.as_ref().map(|s| (s.history.can_undo(), s.history.can_redo())).unwrap_or((false, false));
        let bw = 4.0 * 44.0;
        let bar = egui::Rect::from_min_size(egui::pos2(rect.center().x - bw / 2.0, rect.bottom() - 52.0), egui::vec2(bw, 40.0));
        p.rect_filled(bar, CornerRadius::same(8), Color32::from_rgba_unmultiplied(16, 24, 36, 215));
        p.rect_stroke(bar, CornerRadius::same(8), Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        let mut act = 0;
        for (i, (ic, tip, en)) in [(Icon::Undo, "Rückgängig (Strg+Z)", can_u), (Icon::Redo, "Wiederholen (Strg+Y)", can_r), (Icon::Reset, "Ansicht zurücksetzen", true), (Icon::Image, "Bodenraster ein/aus", true)].iter().enumerate() {
            let r = egui::Rect::from_min_size(bar.min + egui::vec2(i as f32 * 44.0, 0.0), egui::vec2(44.0, 40.0));
            let resp = ui.interact(r, egui::Id::new(("bar", i)), egui::Sense::click());
            if resp.hovered() && *en { p.rect_filled(r.shrink(3.0), CornerRadius::same(6), theme::HOVER); }
            icons::paint(&p, egui::Rect::from_center_size(r.center(), egui::vec2(22.0, 22.0)), *ic, if !*en { theme::LINE } else if i == 3 && self.grid { theme::ACCENT } else { theme::TEXT });
            if resp.on_hover_text(*tip).clicked() && *en { act = i + 1; }
        }
        match act { 1 => { if let Some(s) = self.session.as_mut() { s.undo(); } } 2 => { if let Some(s) = self.session.as_mut() { s.redo(); } } 3 => { self.camera = crate::camera::Camera::default(); self.saved_cam = None; } 4 => self.grid = !self.grid, _ => {} }
        let fs = egui::Rect::from_min_size(egui::pos2(rect.right() - 54.0, rect.bottom() - 52.0), egui::vec2(40.0, 40.0));
        p.rect_filled(fs, CornerRadius::same(8), Color32::from_rgba_unmultiplied(16, 24, 36, 215));
        p.rect_stroke(fs, CornerRadius::same(8), Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        let resp = ui.interact(fs, egui::Id::new("fullscreen"), egui::Sense::click());
        icons::paint(&p, egui::Rect::from_center_size(fs.center(), egui::vec2(22.0, 22.0)), Icon::Fullscreen, if resp.hovered() || self.panels_hidden { theme::ACCENT } else { theme::TEXT });
        if resp.on_hover_text("Seitenleisten aus-/einblenden (Viewport maximieren)").clicked() { self.panels_hidden = !self.panels_hidden; }
        // Leistungsanzeige unten links
        let stats = format!("{:.0} FPS\n{:.1} ms{}", 1000.0 / self.frame_avg.max(0.1), self.frame_avg, self.session.as_ref().map(|s| { let (v, t) = s.counts(); format!("\nDreiecke {:.0}k\nVertices {:.0}k", t as f32 / 1000.0, v as f32 / 1000.0) }).unwrap_or_default());
        let sr = egui::Rect::from_min_size(egui::pos2(rect.left() + 12.0, rect.bottom() - 96.0), egui::vec2(108.0, 84.0));
        p.rect_filled(sr, CornerRadius::same(8), Color32::from_rgba_unmultiplied(16, 24, 36, 200));
        p.rect_stroke(sr, CornerRadius::same(8), Stroke::new(1.0, theme::LINE), StrokeKind::Inside);
        p.text(sr.left_top() + egui::vec2(10.0, 8.0), Align2::LEFT_TOP, stats, FontId::monospace(12.0), Color32::from_rgb(210, 222, 240));
        if self.session.is_none() { p.text(rect.center(), Align2::CENTER_CENTER, if self.error.is_some() { "Fehler beim Laden" } else { "Lade Charakterdaten …" }, FontId::proportional(18.0), Color32::WHITE); }
    }
}

/// Logo-Marke (stilisierte Schwingen in Akzentblau).
fn logo(p: &egui::Painter, r: egui::Rect) {
    let c = r.center(); let s = r.width() / 40.0;
    let pt = |x: f32, y: f32| egui::pos2(c.x + x * s, c.y + y * s);
    let blue = theme::ACCENT; let deep = Color32::from_rgb(0x2A, 0x6F, 0xC4);
    p.add(egui::Shape::convex_polygon(vec![pt(-17.0, -16.0), pt(-4.0, -4.0), pt(-6.0, 17.0), pt(-18.0, 4.0)], deep, Stroke::NONE));
    p.add(egui::Shape::convex_polygon(vec![pt(17.0, -16.0), pt(4.0, -4.0), pt(6.0, 17.0), pt(18.0, 4.0)], blue, Stroke::NONE));
    p.add(egui::Shape::convex_polygon(vec![pt(-3.0, -18.0), pt(0.0, -6.0), pt(3.0, -18.0), pt(0.0, -14.0)], Color32::from_rgb(0xB6, 0xD7, 0xFF), Stroke::NONE));
}

/// Deckt die vier Ecken des Rechtecks so ab, dass eine abgerundete Form entsteht (3D-Viewport kann nicht selbst abgerundet werden).
fn round_mask(p: &egui::Painter, rect: egui::Rect, r: f32, color: Color32) {
    let mut mesh = egui::Mesh::default();
    for (corner, sx, sy) in [(rect.left_top(), 1.0f32, 1.0f32), (rect.right_top(), -1.0, 1.0), (rect.right_bottom(), -1.0, -1.0), (rect.left_bottom(), 1.0, -1.0)] {
        let center = egui::pos2(corner.x + sx * r, corner.y + sy * r);
        let n = 8;
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(corner, color);
        for i in 0..=n {
            let a = (i as f32 / n as f32) * std::f32::consts::FRAC_PI_2;
            // Bogen von (corner + (0, r·sy)) bis (corner + (r·sx, 0))
            let (dx, dy) = (-sx * a.cos(), -sy * a.sin());
            let (dx, dy) = (dx * 1.0, dy * 1.0);
            let _ = (dx, dy);
            let ang = a;
            let q = egui::pos2(center.x - sx * r * ang.sin(), center.y - sy * r * ang.cos());
            mesh.colored_vertex(q, color);
        }
        for i in 0..n { mesh.add_triangle(base, base + 1 + i, base + 2 + i); }
    }
    p.add(egui::Shape::mesh(mesh));
}

impl App {
    /// Navigation für Avatar-Modus: Gesicht/Körper-Bereiche mit Reglern, dann Farben und Modell/Ausdrücke.
    fn avatar_area_list(&mut self, ui: &mut egui::Ui) {
        let mut go: Option<(Main, usize, usize)> = None;
        if nav_row(ui, Icon::Overview, "Gesamtansicht", self.main == Main::Charakter).clicked() { go = Some((Main::Charakter, usize::MAX, 0)); }
        for (m, t) in [(Main::Gesicht, "Gesicht"), (Main::Koerper, "Körper")] {
            ui_kit::section(ui, t);
            for (i, r) in regions::regions_of(m) { if !hase_core::avatar::DEFS.iter().any(|d| d.region == r.label) { continue; } if nav_row(ui, icons::for_region(r.label), r.label, self.main == m && self.region == i).clicked() { go = Some((m, i, 0)); } }
        }
        ui_kit::section(ui, "Aussehen");
        for (i, (l, ic)) in [("Farben", Icon::Skin), ("Modell & Ausdrücke", Icon::Overview)].iter().enumerate() { if nav_row(ui, *ic, l, self.main == Main::Stil && self.sub == i).clicked() { go = Some((Main::Stil, usize::MAX, i)); } }
        if let Some((m, r, sub)) = go {
            self.main = m; self.region = r; self.sub = sub;
            if r != usize::MAX { self.open_regions.clear(); self.open_regions.insert(r); self.scroll_to = Some(r); self.focus(REGIONS[r].cam); } else { self.focus(regions::CAM_BODY); }
        }
    }

    /// Regler einer Region im Avatar-Modus.
    pub(crate) fn avatar_rows(&mut self, ui: &mut egui::Ui, region: &str) {
        let idx: Vec<usize> = hase_core::avatar::DEFS.iter().enumerate().filter(|(_, d)| d.region == region).map(|(i, _)| i).collect();
        let Some(s) = self.session.as_mut() else { return };
        let Some(vr) = s.vrm.as_ref() else { return };
        let (vals, mut linked) = (vr.avatar.vals.clone(), vr.avatar.eye_linked);
        let has_sides = idx.iter().any(|&i| hase_core::avatar::mirror(i).is_some());
        if has_sides {
            if ui.checkbox(&mut linked, "Links und rechts gemeinsam bearbeiten").changed() { if let Some(v) = s.vrm.as_mut() { v.avatar.eye_linked = linked; } }
            ui.add_space(2.0);
        }
        let mut set: Option<(usize, f32)> = None;
        for &i in &idx {
            let d = &hase_core::avatar::DEFS[i];
            if linked && d.id.ends_with("-r") { continue; }
            let label = if linked { d.label.replace(" (links)", "") } else { d.label.to_string() };
            if let Some(v) = ui_kit::param_row(ui, &label, vals[i], d.min..=d.max, d.def, "") { set = Some((i, v)); }
        }
        if let Some((i, v)) = set { s.avatar_set(i, v); }
        ui.add_space(4.0);
        if ui.add(egui::Button::new(egui::RichText::new("Bereich zurücksetzen").size(12.5))).clicked() { s.avatar_reset(&idx); }
    }

    /// Farben je Teilegruppe (Haut, Haare, Iris …).
    pub(crate) fn avatar_colors(&mut self, ui: &mut egui::Ui) {
        self.panel_title(ui, Icon::Skin, "Farben");
        let Some(s) = self.session.as_mut() else { return };
        let Some(v) = s.vrm.as_ref() else { return };
        let mats: Vec<String> = v.model.parts.iter().map(|p| p.mat_name.to_lowercase()).collect();
        let groups: Vec<(&str, Vec<usize>)> = [("Haut", vec!["skin"]), ("Haare", vec!["hair"]), ("Augen (Iris)", vec!["eyeiris"]), ("Augenweiß", vec!["eyewhite"]), ("Brauen", vec!["facebrow"]), ("Lippen / Mund", vec!["facemouth"]), ("Wimpern / Lidstrich", vec!["eyeline"])]
            .into_iter().map(|(n, keys)| (n, mats.iter().enumerate().filter(|(_, m)| keys.iter().any(|k| m.contains(k))).map(|(i, _)| i).collect::<Vec<usize>>())).filter(|(_, l)| !l.is_empty()).collect();
        if groups.is_empty() { ui.colored_label(theme::TEXT2, "Dieses Modell hat keine erkennbaren Materialgruppen (VRoid-Namen erwartet)."); }
        let cur: Vec<[f32; 3]> = groups.iter().map(|(_, l)| v.tints.get(&l[0]).copied().unwrap_or([1.0; 3])).collect();
        let mut changes: Vec<(usize, [f32; 3])> = vec![];
        for (gi, (n, _)) in groups.iter().enumerate() { ui.horizontal(|ui| { let mut c = cur[gi]; if ui.color_edit_button_rgb(&mut c).changed() { changes.push((gi, c)); } ui.label(*n); if ui.small_button("Original").clicked() { changes.push((gi, [1.0; 3])); } }); }
        ui.add_space(4.0);
        ui.colored_label(theme::TEXT2, "Haare und Iris werden über die Helligkeitszeichnung der Textur umgefärbt, die übrigen Gruppen multipliziert.");
        for (gi, c) in changes { for &pi in &groups[gi].1 { s.set_part_tint(pi, c); } }
    }
}

impl App {
    /// Auswahl der Ausgangsbasis für neue Anime-/VTuber-Charaktere.
    pub(crate) fn base_picker_window(&mut self, ctx: &egui::Context) {
        if !self.base_picker { return; }
        let mut close = false;
        let mut start: Option<crate::anime_bases::AnimeBase> = None;
        let mut add = false;
        egui::Window::new("Neuer Charakter – Basis wählen").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).default_width(520.0).show(ctx, |ui| {
            ui.colored_label(theme::TEXT2, "Eine Basis liefert Körpermesh, Skelett und Gesichtsaufbau (z. B. ein nackter VRoid-Export). Aus ihr gestaltest du einen neuen Charakter: Gesicht, Augen, Körper und Farben per Regler. Basen bleiben lokal auf diesem Rechner.");
            ui.add_space(8.0);
            if self.bases.is_empty() { ui.label("Noch keine Basis registriert."); }
            for b in self.bases.clone() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&b.name).strong());
                    ui.colored_label(theme::TEXT2, format!("{:.1} MB", b.size as f64 / 1048576.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui_kit::primary_button(ui, "Charakter erstellen").clicked() { start = Some(b.clone()); } });
                });
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Neue Basis aus VRM-Datei …").clicked() { add = true; }
                if ui.button("Abbrechen").clicked() { close = true; }
            });
        });
        if add {
            if let Some(p) = rfd::FileDialog::new().add_filter("VRM / GLB", &["vrm", "glb"]).pick_file() {
                let repo = self.session.as_ref().map(|s| s.repo.clone());
                if let Some(repo) = repo { match crate::anime_bases::register(&repo, &p, None) { Ok(b) => { self.bases = crate::anime_bases::list(&repo); start = Some(b); } Err(e) => self.status = e } }
            }
        }
        if let Some(b) = start { self.start_avatar(&b); close = true; }
        if close { self.base_picker = false; }
    }
}
