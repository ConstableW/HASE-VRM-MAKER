//! egui-CPU-Kosten (Layout + Tessellation) für Parameter-Panels und Bibliotheken. Misst NICHT die GPU-Zeit des UI-Renderings.
use crate::stats::*;
use serde_json::{json, Value};
use std::time::Instant;

fn run_frames<F: FnMut(&mut egui::Ui)>(frames: usize, mut ui: F) -> (Stats, Stats, usize) {
    let ctx = egui::Context::default();
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let mut shapes_n = 0;
    for f in 0..frames + 30 {
        let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0))), time: Some(f as f64 / 60.0), ..Default::default() };
        let t0 = Instant::now();
        let out = ctx.run_ui(raw, |root| ui(root));
        let t1 = Instant::now();
        let prims = ctx.tessellate(out.shapes, out.pixels_per_point);
        let t2 = Instant::now();
        if f >= 30 { a.push((t1 - t0).as_secs_f64() * 1e3); b.push((t2 - t1).as_secs_f64() * 1e3); shapes_n = prims.len(); }
    }
    (stats(a), stats(b), shapes_n)
}

pub fn run(names: &[String]) -> Value {
    let n = names.len();
    let mut vals = vec![0f32; n];
    let mut out = serde_json::Map::new();
    let frames = 600;

    let (l, t, p) = run_frames(frames, |ctx| {
        egui::Panel::left("params").default_size(420.0).show(ctx, |ui| {
            ui.heading("Parameter");
            egui::ScrollArea::vertical().show_rows(ui, 24.0, n, |ui, range| {
                for i in range { ui.add(egui::Slider::new(&mut vals[i], -1.0..=1.0).text(&names[i])); }
            });
        });
    });
    out.insert("parameterpanel_virtualisiert_610".into(), json!({"layout_ms": l, "tessellation_ms": t, "clipped_primitives": p}));

    let (l, t, p) = run_frames(frames, |ctx| {
        egui::Panel::left("params").default_size(420.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| { for i in 0..n { ui.add(egui::Slider::new(&mut vals[i], -1.0..=1.0).text(&names[i])); } });
        });
    });
    out.insert("parameterpanel_nicht_virtualisiert_610_worstcase".into(), json!({"layout_ms": l, "tessellation_ms": t, "clipped_primitives": p}));

    let lib: Vec<String> = (0..10_000).map(|i| format!("asset_{i:05}")).collect();
    let mut sel = 0usize;
    let (l, t, p) = run_frames(frames, |ctx| {
        egui::Panel::right("lib").default_size(300.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show_rows(ui, 22.0, lib.len(), |ui, range| { for i in range { if ui.selectable_label(sel == i, &lib[i]).clicked() { sel = i; } } });
        });
    });
    out.insert("bibliothek_virtualisiert_10000".into(), json!({"layout_ms": l, "tessellation_ms": t, "clipped_primitives": p}));

    let (l, t, p) = run_frames(frames, |ctx| {
        egui::Panel::left("params").default_size(420.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show_rows(ui, 24.0, n, |ui, range| { for i in range { vals[i] = ((i as f32) * 0.01 + ui.input(|s| s.time as f32)).sin(); ui.add(egui::Slider::new(&mut vals[i], -1.0..=1.0).text(&names[i])); } });
        });
        ctx.ctx().request_repaint();
    });
    out.insert("parameterpanel_virtualisiert_mit_wertaenderung_je_frame".into(), json!({"layout_ms": l, "tessellation_ms": t, "clipped_primitives": p}));

    // Hintergrundlast (Export-/Fitting-Simulation): 2 Worker-Threads mit Volllast, UI-Thread getrennt
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let workers: Vec<_> = (0..2).map(|i| { let st = stop.clone(); std::thread::spawn(move || { let mut x = 1.0f64 + i as f64; while !st.load(std::sync::atomic::Ordering::Relaxed) { for _ in 0..100_000 { x = (x * 1.000001).sin().abs() + 1.0; } std::hint::black_box(x); } }) }).collect();
    let (l, t, p) = run_frames(frames, |ctx| {
        egui::Panel::left("params").default_size(420.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show_rows(ui, 24.0, n, |ui, range| { for i in range { ui.add(egui::Slider::new(&mut vals[i], -1.0..=1.0).text(&names[i])); } });
        });
    });
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    for w in workers { let _ = w.join(); }
    out.insert("parameterpanel_virtualisiert_waehrend_2_worker_threads_volllast".into(), json!({"layout_ms": l, "tessellation_ms": t, "clipped_primitives": p}));
    Value::Object(out)
}
