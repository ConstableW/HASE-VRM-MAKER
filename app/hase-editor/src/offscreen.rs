//! Screenshot-Modus (ohne Fenster): rendert mit demselben SceneRenderer in eine Textur und speichert PNG. Dient Tests/CI.
use crate::camera::{Camera, PRESETS};
use crate::render::SceneRenderer;
use crate::session::Session;
use std::path::Path;

fn arg<'a>(args: &'a [String], k: &str) -> Option<&'a str> { args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).map(|s| s.as_str()) }

pub fn run(repo: &Path, out: &str, args: &[String]) -> i32 {
    let (w, h) = arg(args, "--size").and_then(|s| s.split_once('x')).and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))).unwrap_or((900, 1000));
    let mut s = match Session::load(repo) { Ok(s) => s, Err(e) => { eprintln!("{e}"); return 1; } };
    if let Some(i) = arg(args, "--species").and_then(|x| x.parse::<usize>().ok()) { s.apply_species(i); }
    if let Some(sk) = arg(args, "--skin") { s.set_skin(sk); }
    let assets = arg(args, "--assets").unwrap_or("eyebrows/mindfront_eyebrows_01,eyelashes/mindfront_eyelashes_01,hair/short02,clothes/toigo_basic_tucked_t-shirt,clothes/toigo_wool_pants");
    let list: Vec<&str> = assets.split(',').filter(|x| !x.is_empty()).collect();
    s.default_assets_blocking(&list);
    if let Some(sets) = arg(args, "--set") {
        for kv in sets.split(',') { if let Some((k, v)) = kv.split_once('=') { match (s.params.index.get(k).copied(), v.parse::<f32>()) { (Some(i), Ok(f)) => s.set_param(i, f), _ => eprintln!("Parameter unbekannt/ungültig: {kv}") } } }
    }
    let mut cam = Camera::default();
    if let Some(v) = arg(args, "--view") { if let Some((_, c)) = PRESETS.iter().find(|(n, _)| n.to_lowercase().starts_with(&v.to_lowercase())) { cam = *c; } }
    if let Some(y) = arg(args, "--yaw").and_then(|x| x.parse::<f32>().ok()) { cam.yaw = y; }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })) { Ok(a) => a, Err(e) => { eprintln!("Kein GPU-Adapter: {e}"); return 1; } };
    eprintln!("Adapter: {:?}", adapter.get_info().name);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("device");
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let mut r = SceneRenderer::new(&device, &queue, fmt, wgpu::TextureFormat::Depth32Float, 4);
    let t0 = std::time::Instant::now();
    let upd = s.refresh();
    r.apply_updates(&device, &queue, upd);
    r.write_globals(&queue, cam.globals(w as f32 / h as f32));
    let mk = |fmt, samples, usage| device.create_texture(&wgpu::TextureDescriptor { label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: samples, dimension: wgpu::TextureDimension::D2, format: fmt, usage, view_formats: &[] });
    let msaa = mk(fmt, 4, wgpu::TextureUsages::RENDER_ATTACHMENT);
    let resolve = mk(fmt, 1, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
    let depth = mk(wgpu::TextureFormat::Depth32Float, 4, wgpu::TextureUsages::RENDER_ATTACHMENT);
    let (vm, vr, vd) = (msaa.create_view(&Default::default()), resolve.create_view(&Default::default()), depth.create_view(&Default::default()));
    let mut enc = device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor { label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &vm, depth_slice: None, resolve_target: Some(&vr), ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.2, g: 0.22, b: 0.26, a: 1.0 }), store: wgpu::StoreOp::Discard } })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &vd, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }), stencil_ops: None }),
            timestamp_writes: None, occlusion_query_set: None, multiview_mask: None });
        r.draw(&mut pass);
    }
    let bpr = (w * 4 + 255) / 256 * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (bpr * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    enc.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo { texture: &resolve, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(h) } }, wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
    queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    buf.slice(..).map_async(wgpu::MapMode::Read, move |res| { let _ = tx.send(res); });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    if rx.recv().map(|r| r.is_err()).unwrap_or(true) { eprintln!("Readback fehlgeschlagen"); return 1; }
    let data = buf.slice(..).get_mapped_range().expect("map");
    let mut img = image::RgbaImage::new(w, h);
    for y in 0..h { for x in 0..w { let o = (y * bpr + x * 4) as usize; img.put_pixel(x, y, image::Rgba([data[o], data[o + 1], data[o + 2], 255])); } }
    img.save(out).expect("png");
    let (v, t) = s.counts();
    eprintln!("Screenshot {out} ({w}x{h}) – Morph {:.3} ms, Fit {:.3} ms, Vertices {v}, Dreiecke {t}, Szene+Render {:.0} ms, Parameter {}", s.perf.morph_ms, s.perf.fit_ms, t0.elapsed().as_secs_f64() * 1e3, s.params.defs.len());
    0
}
