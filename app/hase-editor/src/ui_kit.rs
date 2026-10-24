//! Wiederverwendbare Oberflächenbausteine im Stil des verbindlichen Mockups (Karten, Tabs, Navigationszeilen, Akkordeon, Parameterzeilen).
use crate::icons::{self, Icon};
use crate::theme;
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui};

pub const RADIUS: u8 = 8;

/// Karte (Panel-Fläche mit feinem Rahmen). `title` zeigt Icon + Überschrift im Kopf.
pub fn card<R>(ui: &mut Ui, title: Option<(Icon, &str)>, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new().fill(theme::PANEL).stroke(Stroke::new(1.0, theme::LINE)).corner_radius(CornerRadius::same(RADIUS)).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if let Some((ic, t)) = title {
            ui.horizontal(|ui| { icons::show(ui, ic, 18.0, theme::TEXT); ui.label(egui::RichText::new(t).strong().size(14.0)); });
            ui.add_space(4.0);
        }
        add(ui)
    }).inner
}

/// Zeile der linken Navigation: Icon + Beschriftung, ausgewählt = gefüllt mit Akzentrahmen.
pub fn nav_row(ui: &mut Ui, icon: Icon, label: &str, selected: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), Sense::click());
    let p = ui.painter();
    if selected { p.rect_filled(rect, CornerRadius::same(6), theme::ACTIVE); p.rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.0, theme::ACCENT), StrokeKind::Inside); }
    else if resp.hovered() { p.rect_filled(rect, CornerRadius::same(6), theme::HOVER); }
    icons::paint(p, egui::Rect::from_center_size(egui::pos2(rect.left() + 16.0, rect.center().y), egui::vec2(18.0, 18.0)), icon, if selected { Color32::WHITE } else { theme::TEXT2 });
    p.text(egui::pos2(rect.left() + 34.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(14.0), if selected { Color32::WHITE } else { theme::TEXT });
    resp
}

/// Kopf eines Akkordeon-Abschnitts (Icon, Titel, Pfeil).
pub fn acc_header(ui: &mut Ui, icon: Icon, label: &str, open: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(6), if open { theme::PANEL2 } else if resp.hovered() { theme::HOVER } else { theme::PANEL });
    if open { p.line_segment([rect.left_bottom() + egui::vec2(0.0, 0.0), rect.right_bottom()], Stroke::new(1.0, theme::LINE)); }
    icons::paint(p, egui::Rect::from_center_size(egui::pos2(rect.left() + 18.0, rect.center().y), egui::vec2(18.0, 18.0)), icon, theme::TEXT);
    p.text(egui::pos2(rect.left() + 38.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(14.5), theme::TEXT);
    icons::paint(p, egui::Rect::from_center_size(egui::pos2(rect.right() - 16.0, rect.center().y), egui::vec2(14.0, 14.0)), if open { Icon::ChevUp } else { Icon::ChevDown }, theme::TEXT2);
    resp
}

/// Segment-Tabs gleicher Breite. Liefert `true`, wenn die Auswahl geändert wurde.
pub fn tabs(ui: &mut Ui, items: &[&str], current: &mut usize) -> bool {
    let n = items.len().max(1) as f32;
    let gap = 4.0;
    let w = (ui.available_width() - gap * (n - 1.0)) / n;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (i, t) in items.iter().enumerate() {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 28.0), Sense::click());
            let sel = *current == i;
            let p = ui.painter();
            p.rect_filled(rect, CornerRadius::same(5), if sel { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::PANEL2 });
            p.rect_stroke(rect, CornerRadius::same(5), Stroke::new(1.0, if sel { theme::ACCENT } else { theme::LINE }), StrokeKind::Inside);
            p.text(rect.center(), Align2::CENTER_CENTER, *t, FontId::proportional(13.5), if sel { Color32::WHITE } else { theme::TEXT });
            if resp.clicked() && !sel { *current = i; changed = true; }
        }
    });
    changed
}

/// Kleine Umschalter-Gruppe (z. B. Links/Rechts) mit fester Breite je Segment.
pub fn segmented(ui: &mut Ui, items: &[&str], current: &mut usize, seg_w: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for (i, t) in items.iter().enumerate() {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(seg_w, 24.0), Sense::click());
            let sel = *current == i; let p = ui.painter();
            p.rect_filled(rect, CornerRadius::same(5), if sel { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { theme::PANEL2 });
            p.rect_stroke(rect, CornerRadius::same(5), Stroke::new(1.0, if sel { theme::ACCENT } else { theme::LINE }), StrokeKind::Inside);
            p.text(rect.center(), Align2::CENTER_CENTER, *t, FontId::proportional(12.5), if sel { Color32::WHITE } else { theme::TEXT });
            if resp.clicked() && !sel { *current = i; changed = true; }
        }
    });
    changed
}

/// Quadratischer Icon-Knopf mit Beschriftung darunter (Viewport-Werkzeugleiste, Kopfzeile).
pub fn icon_tile(ui: &mut Ui, icon: Icon, label: &str, size: egui::Vec2, selected: bool, tip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let p = ui.painter();
    let fill = if selected { theme::ACTIVE } else if resp.hovered() { theme::HOVER } else { Color32::from_rgba_unmultiplied(16, 24, 36, 215) };
    p.rect_filled(rect, CornerRadius::same(6), fill);
    p.rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.0, if selected { theme::ACCENT } else { theme::LINE }), StrokeKind::Inside);
    let has_label = !label.is_empty();
    let ic_rect = egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.center().y - if has_label { 6.0 } else { 0.0 }), egui::vec2(22.0, 22.0));
    icons::paint(p, ic_rect, icon, if selected { Color32::WHITE } else { theme::TEXT });
    if has_label { p.text(egui::pos2(rect.center().x, rect.bottom() - 9.0), Align2::CENTER_CENTER, label, FontId::proportional(10.0), if selected { Color32::WHITE } else { theme::TEXT2 }); }
    if tip.is_empty() { resp } else { resp.on_hover_text(tip) }
}

/// Parameterzeile: Beschriftung links, Regler, Zahlenfeld rechts. Rückgabe: `Some(neuer Wert)` bei Änderung; Doppelklick auf den Regler = Standardwert.
pub fn param_row(ui: &mut Ui, label: &str, value: f32, range: std::ops::RangeInclusive<f32>, default: f32, tip: &str) -> Option<f32> {
    let mut v = value;
    let mut out = None;
    let total = ui.available_width();
    let (lw, vw) = ((total * 0.42).clamp(110.0, 210.0), 52.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (lr, lresp) = ui.allocate_exact_size(egui::vec2(lw, 22.0), Sense::hover());
        let mut job = egui::text::LayoutJob::simple_singleline(label.to_string(), FontId::proportional(13.5), theme::TEXT);
        job.wrap = egui::text::TextWrapping { max_width: lw, max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
        let galley = ui.painter().layout_job(job);
        ui.painter().galley(egui::pos2(lr.left(), lr.center().y - galley.size().y / 2.0), galley, theme::TEXT);
        lresp.on_hover_text(if tip.is_empty() { label } else { tip });
        let sw = (total - lw - vw - 16.0).max(60.0);
        ui.spacing_mut().slider_width = sw;
        let r = ui.add(egui::Slider::new(&mut v, range.clone()).show_value(false));
        if r.changed() { out = Some(v); }
        if r.double_clicked() { out = Some(default); }
        let (lo, hi) = (*range.start(), *range.end());
        let mut dv = out.unwrap_or(v);
        let d = ui.add_sized([vw, 22.0], egui::DragValue::new(&mut dv).range(lo..=hi.max(lo + 1e-4)).speed(0.005).fixed_decimals(2));
        if d.changed() { out = Some(dv); }
    });
    out
}

/// Dünne Trennlinie in Panelbreite.
pub fn hr(ui: &mut Ui) { let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover()); ui.painter().rect_filled(r, 0.0, theme::LINE); }

/// Kleiner Überschriftentext in Großbuchstaben (Abschnittsüberschrift).
pub fn section(ui: &mut Ui, t: &str) { ui.add_space(6.0); ui.label(egui::RichText::new(t.to_uppercase()).size(11.0).color(theme::TEXT2).strong()); ui.add_space(2.0); }

/// Gefüllter Hauptknopf (Akzent).
pub fn primary_button(ui: &mut Ui, label: &str) -> Response {
    ui.add(egui::Button::new(egui::RichText::new(label).color(Color32::WHITE).strong()).fill(theme::ACTIVE).stroke(Stroke::new(1.0, theme::ACCENT)).min_size(egui::vec2(0.0, 30.0)))
}
