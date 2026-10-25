//! Screenshot-Modus (ohne Fenster): rendert mit demselben SceneRenderer in eine Textur und speichert PNG. Dient Tests/CI.
use crate::camera::{Camera, PRESETS};
use crate::render::SceneRenderer;
use crate::session::Session;
use std::path::Path;

fn arg<'a>(args: &'a [String], k: &str) -> Option<&'a str> { args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).map(|s| s.as_str()) }

pub fn run(repo: &Path, out: &str, args: &[String]) -> i32 {
    let (w, h) = arg(args, "--size").and_then(|s| s.split_once('x')).and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))).unwrap_or((900, 1000));
    let mut s = match Session::load(repo) { Ok(s) => s, Err(e) => { eprintln!("{e}"); return 1; } };
    if let Some(p) = arg(args, "--vrm") { match s.open_vrm(std::path::Path::new(p)) { Ok(i) => eprintln!("{i}"), Err(e) => { eprintln!("--vrm: {e}"); return 1; } } if let Some(av) = arg(args, "--av") { for kv in av.split(',') { if let Some((k, v)) = kv.split_once('=') { if let (Some(i), Ok(f)) = (hase_core::avatar::index_of(k), v.parse::<f32>()) { s.avatar_set(i, f); } } } if let Some(v) = s.vrm.as_mut() { if args.iter().any(|a| a == "--av-unlinked") { v.avatar.eye_linked = false; } } } if let Some(ft) = arg(args, "--feat") { for kv in ft.split(',') { if let Some((k, pi)) = kv.split_once(':') { if let Some(f) = s.feature_mut(k) { f.enabled = true; if let Ok(pi) = pi.parse::<usize>() { if let Some(pr) = hase_core::features::presets(k).get(pi) { f.vals = pr.1.to_vec(); } } } } } } if let Some(ex) = arg(args, "--expr") { for kv in ex.split(',') { if let Some((k, v)) = kv.split_once('=') { s.set_expression(k, v.parse().unwrap_or(0.0)); } } } }
    if arg(args, "--vrm").is_none() { if let Some(ft) = arg(args, "--feat") { for kv in ft.split(',') { if let Some((k, pi)) = kv.split_once(':') { if let Some(f) = s.feature_mut(k) { f.enabled = true; if let Ok(pi) = pi.parse::<usize>() { if let Some(pr) = hase_core::features::presets(k).get(pi) { f.vals = pr.1.to_vec(); } } } } } } }
    if let Some(i) = arg(args, "--species").and_then(|x| x.parse::<usize>().ok()) { s.apply_species(i); }
    if let Some(sk) = arg(args, "--skin") { s.set_skin(sk); }
    let assets = arg(args, "--assets").unwrap_or("eyebrows/mindfront_eyebrows_01,eyelashes/mindfront_eyelashes_01,hair/short02");
    let list: Vec<&str> = assets.split(',').filter(|x| !x.is_empty()).collect();
    s.default_assets_blocking(&list);
    if let Some(sets) = arg(args, "--set") {
        for kv in sets.split(',') { if let Some((k, v)) = kv.split_once('=') { match (s.params.canonical(k), v.parse::<f32>()) { (Some(i), Ok(f)) => s.set_param(i, f), _ => eprintln!("Parameter unbekannt/ungültig: {kv}") } } }
    }
    if let Some(js) = arg(args, "--style") { match serde_json::from_str::<serde_json::Value>(js) { Ok(v) => { let st = hase_core::state::Style::from_json(&v); s.edit_style(0, |x| *x = st); } Err(e) => eprintln!("--style: {e}") } }
    if let Some(p) = arg(args, "--pose") { match s.load_clip(std::path::Path::new(p)) { Ok(()) => { s.play.playing = false; s.play.t = arg(args, "--pose-t").and_then(|x| x.parse().ok()).unwrap_or(0.5); } Err(e) => eprintln!("--pose: {e}") } }
    let mut cam = Camera::default();
    if let Some(c) = arg(args, "--cam") { let v: Vec<f32> = c.split(',').filter_map(|x| x.parse().ok()).collect(); if v.len() == 4 { cam = Camera { yaw: v[0], pitch: 0.0, dist: v[1], target: [0.0, v[2] * s.body_height() / 1.75, 0.0], fov: 0.62 }; let _ = v[3]; } }
    if let Some(v) = arg(args, "--view") { if let Some((_, c)) = PRESETS.iter().find(|(n, _)| n.to_lowercase().starts_with(&v.to_lowercase())) { cam = *c; } }
    if let Some(y) = arg(args, "--yaw").and_then(|x| x.parse::<f32>().ok()) { cam.yaw = y; }

    let mut off = match Offscreen::new() { Ok(o) => o, Err(e) => { eprintln!("{e}"); return 1; } };
    let t0 = std::time::Instant::now();
    let upd = s.refresh();
    let rgba = match off.render(upd, cam.globals(w as f32 / h as f32), w, h) { Ok(d) => d, Err(e) => { eprintln!("{e}"); return 1; } };
    image::RgbaImage::from_raw(w, h, rgba).expect("bild").save(out).expect("png");
    let (v, t) = s.counts();
    eprintln!("Screenshot {out} ({w}x{h}) – Morph {:.3} ms, Fit {:.3} ms, Vertices {v}, Dreiecke {t}, Szene+Render {:.0} ms, Parameter {}", s.perf.morph_ms, s.perf.fit_ms, t0.elapsed().as_secs_f64() * 1e3, s.params.defs.len());
    0
}

/// Wiederverwendbarer Offscreen-Renderer (eigenes wgpu-Gerät): Screenshots, Thumbnails. Hält die Szene zwischen Aufrufen.
pub struct Offscreen { device: wgpu::Device, queue: wgpu::Queue, r: SceneRenderer, pub adapter: String }

impl Offscreen {
    pub fn new() -> Result<Offscreen, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })).map_err(|e| format!("Kein GPU-Adapter: {e}"))?;
        let name = adapter.get_info().name;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).map_err(|e| e.to_string())?;
        let r = SceneRenderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Depth32Float, 4);
        Ok(Offscreen { device, queue, r, adapter: name })
    }

    /// Wendet Mesh-Updates an und rendert ein RGBA-Bild (w×h, 4×MSAA).
    pub fn render(&mut self, updates: Vec<crate::render::MeshUpdate>, globals: crate::render::Globals, w: u32, h: u32) -> Result<Vec<u8>, String> {
        let (device, queue) = (&self.device, &self.queue);
        let fmt = wgpu::TextureFormat::Rgba8Unorm;
        self.r.apply_updates(device, queue, updates);
        self.r.write_globals(queue, globals);
        let mk = |fmt, samples, usage| device.create_texture(&wgpu::TextureDescriptor { label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: samples, dimension: wgpu::TextureDimension::D2, format: fmt, usage, view_formats: &[] });
        let msaa = mk(fmt, 4, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let resolve = mk(fmt, 1, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
        let depth = mk(wgpu::TextureFormat::Depth32Float, 4, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let (vm, vr, vd) = (msaa.create_view(&Default::default()), resolve.create_view(&Default::default()), depth.create_view(&Default::default()));
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor { label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &vm, depth_slice: None, resolve_target: Some(&vr), ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.12, b: 0.16, a: 1.0 }), store: wgpu::StoreOp::Discard } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &vd, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }), stencil_ops: None }),
                timestamp_writes: None, occlusion_query_set: None, multiview_mask: None });
            self.r.draw(&mut pass);
        }
        let bpr = (w * 4 + 255) / 256 * 256;
        let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (bpr * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        enc.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo { texture: &resolve, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(h) } }, wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buf.slice(..).map_async(wgpu::MapMode::Read, move |res| { let _ = tx.send(res); });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        if rx.recv().map(|r| r.is_err()).unwrap_or(true) { return Err("Readback fehlgeschlagen".into()); }
        let data = buf.slice(..).get_mapped_range().map_err(|e| e.to_string())?;
        let mut out = vec![255u8; (w * h * 4) as usize];
        for y in 0..h { let src = (y * bpr) as usize; let dst = (y * w * 4) as usize; out[dst..dst + (w * 4) as usize].copy_from_slice(&data[src..src + (w * 4) as usize]); }
        for px in out.chunks_exact_mut(4) { px[3] = 255; }
        Ok(out)
    }
}

/// `--export datei.vrm [--outfit] [--assets …] [--set …] [--style …]`: Export ohne Fenster (Tests/CI).
pub fn run_export(repo: &Path, out: &str, args: &[String]) -> i32 {
    let mut s = match Session::load(repo) { Ok(s) => s, Err(e) => { eprintln!("{e}"); return 1; } };
    let assets = arg(args, "--assets").unwrap_or("eyebrows/mindfront_eyebrows_01,eyelashes/mindfront_eyelashes_01,hair/short02");
    let list: Vec<&str> = assets.split(',').filter(|x| !x.is_empty()).collect();
    s.default_assets_blocking(&list);
    if let Some(sets) = arg(args, "--set") { for kv in sets.split(',') { if let Some((k, v)) = kv.split_once('=') { if let (Some(i), Ok(f)) = (s.params.canonical(k), v.parse::<f32>()) { s.set_param(i, f); } } } }
    if let Some(js) = arg(args, "--style") { if let Ok(v) = serde_json::from_str::<serde_json::Value>(js) { let st = hase_core::state::Style::from_json(&v); s.edit_style(0, |x| *x = st); } }
    let _ = s.refresh();
    if let Some(p) = arg(args, "--vrm") { if let Err(e) = s.open_vrm(std::path::Path::new(p)) { eprintln!("--vrm: {e}"); return 1; } if let Some(av) = arg(args, "--av") { for kv in av.split(',') { if let Some((k, v)) = kv.split_once('=') { if let (Some(i), Ok(f)) = (hase_core::avatar::index_of(k), v.parse::<f32>()) { s.avatar_set(i, f); } } } } if let Some(ft) = arg(args, "--feat") { for kv in ft.split(',') { if let Some((k, pi)) = kv.split_once(':') { if let Some(f) = s.feature_mut(k) { f.enabled = true; if let Ok(pi) = pi.parse::<usize>() { if let Some(pr) = hase_core::features::presets(k).get(pi) { f.vals = pr.1.to_vec(); } } } } } } let _ = s.refresh(); }
    let o = crate::session::ExportOptions { with_outfit: args.iter().any(|a| a == "--outfit"), expressions: true, name: "HASE Export-Test".into(), author: "HASE VRM MAKER".into(), license_url: "https://vrm.dev/licenses/1.0/".into(), commercial: "personalNonProfit".into(), redistribution: true };
    let t0 = std::time::Instant::now();
    let (doc, report) = match s.build_export(&o) { Ok(r) => r, Err(e) => { eprintln!("Export: {e}"); return 1; } };
    let t1 = t0.elapsed();
    let glb = match hase_core::export::write_glb(&doc) { Ok(g) => g, Err(e) => { eprintln!("Export: {e}"); return 1; } };
    if let Err(e) = std::fs::write(out, &glb) { eprintln!("{e}"); return 1; }
    for r in &report { eprintln!("{r}"); }
    eprintln!("VRM {out}: {:.1} MB, Zusammenstellung {:.0} ms, Schreiben {:.0} ms", glb.len() as f64 / 1048576.0, t1.as_secs_f64() * 1e3, (t0.elapsed() - t1).as_secs_f64() * 1e3);
    0
}
