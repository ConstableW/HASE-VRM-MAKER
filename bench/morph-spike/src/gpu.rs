//! wgpu-Spike: GPU-Morphing (Compute), Skinning (Vertex Pulling), Schattenpass, MSAA-Hauptpass; Offscreen, ohne Fenster.
use crate::data::*;
use crate::stats::*;
use bytemuck::{Pod, Zeroable};
use serde_json::{json, Value};
use std::time::Instant;
use wgpu::util::DeviceExt;

#[repr(C)] #[derive(Clone, Copy, Pod, Zeroable)]
struct MorphParams { nv: u32, copies: u32, nt: u32, pad: u32 }
#[repr(C)] #[derive(Clone, Copy, Pod, Zeroable)]
struct Scene { view_proj: [[f32; 4]; 4], light_vp: [[f32; 4]; 4], light_dir: [f32; 4], n: [u32; 4], grid: [f32; 4] }

fn mat_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut r = [[0f32; 4]; 4];
    for c in 0..4 { for rw in 0..4 { for k in 0..4 { r[c][rw] += a[k][rw] * b[c][k]; } } }
    r
}
fn perspective(fovy: f32, aspect: f32, n: f32, f: f32) -> [[f32; 4]; 4] {
    let t = 1.0 / (fovy / 2.0).tan();
    [[t / aspect, 0.0, 0.0, 0.0], [0.0, t, 0.0, 0.0], [0.0, 0.0, f / (n - f), -1.0], [0.0, 0.0, n * f / (n - f), 0.0]]
}
fn ortho(h: f32, n: f32, f: f32) -> [[f32; 4]; 4] {
    [[1.0 / h, 0.0, 0.0, 0.0], [0.0, 1.0 / h, 0.0, 0.0], [0.0, 0.0, 1.0 / (n - f), 0.0], [0.0, 0.0, n / (n - f), 1.0]]
}
fn look_at(eye: [f32; 3], at: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let norm = |v: [f32; 3]| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(); [v[0] / l, v[1] / l, v[2] / l] };
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let f = norm(sub(at, eye)); let s = norm(cross(f, up)); let u = cross(s, f);
    [[s[0], u[0], -f[0], 0.0], [s[1], u[1], -f[1], 0.0], [s[2], u[2], -f[2], 0.0], [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0]]
}

struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn f(&mut self) -> f32 { (self.next() >> 40) as f32 / 16777216.0 } }

pub struct Setup { pub copies: u32, pub frames: usize, pub w: u32, pub h: u32 }

pub fn run(mesh: &Mesh, morphs: &Morphs, skin: &Skin, setup: Setup) -> Value {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })) {
        Ok(a) => a, Err(e) => return json!({"fehler": format!("kein Adapter: {e}")}),
    };
    let info = adapter.get_info();
    let has_ts = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
    let mut limits = wgpu::Limits::default();
    limits.max_storage_buffer_binding_size = adapter.limits().max_storage_buffer_binding_size.min(1 << 29);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: if has_ts { wgpu::Features::TIMESTAMP_QUERY } else { wgpu::Features::empty() },
        required_limits: limits, ..Default::default()
    })).expect("device");

    let nv = mesh.pos.len() as u32; let nt = morphs.names.len() as u32; let copies = setup.copies; let nb = skin.bones as u32;
    let mut gpu_bytes: u64 = 0;
    let mut mk = |label: &str, data: &[u8], usage: wgpu::BufferUsages| { gpu_bytes += data.len() as u64; device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(label), contents: data, usage }) };
    let st = wgpu::BufferUsages::STORAGE;
    let base: Vec<[f32; 4]> = mesh.pos.iter().map(|p| [p[0], p[1], p[2], 1.0]).collect();
    let vdelta: Vec<[f32; 4]> = morphs.v_delta.iter().map(|d| [d[0], d[1], d[2], 0.0]).collect();
    let b_base = mk("base", bytemuck::cast_slice(&base), st);
    let b_off = mk("v_off", bytemuck::cast_slice(&morphs.v_off), st);
    let b_tgt = mk("v_tgt", bytemuck::cast_slice(&morphs.v_tgt), st);
    let b_delta = mk("v_delta", bytemuck::cast_slice(&vdelta), st);
    let b_w = mk("weights", &vec![0u8; (nt * copies * 4) as usize], st | wgpu::BufferUsages::COPY_DST);
    let b_out = mk("morphed", &vec![0u8; (nv * copies * 16) as usize], st);
    let b_mp = mk("morph_params", bytemuck::bytes_of(&MorphParams { nv, copies, nt, pad: 0 }), wgpu::BufferUsages::UNIFORM);
    let b_idx = mk("indices", bytemuck::cast_slice(&mesh.tris), wgpu::BufferUsages::INDEX);
    let joints: Vec<[u32; 4]> = skin.joints.clone();
    let b_j = mk("joints", bytemuck::cast_slice(&joints), st);
    let b_sw = mk("skinw", bytemuck::cast_slice(&skin.weights), st);
    let b_bones = mk("bones", &vec![0u8; (copies * nb * 64) as usize], st | wgpu::BufferUsages::COPY_DST);
    let b_scene = mk("scene", &vec![0u8; std::mem::size_of::<Scene>()], wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()) });
    let (w, h) = (setup.w, setup.h);
    let mut tex = |label: &str, size: (u32, u32), fmt: wgpu::TextureFormat, usage: wgpu::TextureUsages, samples: u32, bpp: u64| {
        gpu_bytes += size.0 as u64 * size.1 as u64 * bpp * samples as u64;
        device.create_texture(&wgpu::TextureDescriptor { label: Some(label), size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: samples, dimension: wgpu::TextureDimension::D2, format: fmt, usage, view_formats: &[] })
    };
    let color_ms = tex("color_ms", (w, h), wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureUsages::RENDER_ATTACHMENT, 4, 4);
    let color_rs = tex("color_resolve", (w, h), wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, 1, 4);
    let depth_ms = tex("depth_ms", (w, h), wgpu::TextureFormat::Depth32Float, wgpu::TextureUsages::RENDER_ATTACHMENT, 4, 4);
    let shadow = tex("shadow", (2048, 2048), wgpu::TextureFormat::Depth32Float, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 1, 4);
    let (v_cms, v_crs, v_dms, v_sh) = (color_ms.create_view(&Default::default()), color_rs.create_view(&Default::default()), depth_ms.create_view(&Default::default()), shadow.create_view(&Default::default()));
    let samp = device.create_sampler(&wgpu::SamplerDescriptor { compare: Some(wgpu::CompareFunction::LessEqual), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });

    let bgl_entry = |b: u32, vis: wgpu::ShaderStages, ty: wgpu::BindingType| wgpu::BindGroupLayoutEntry { binding: b, visibility: vis, ty, count: None };
    let ro = |vis| wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None };
    let cs = wgpu::ShaderStages::COMPUTE;
    let vs = wgpu::ShaderStages::VERTEX; let fs = wgpu::ShaderStages::FRAGMENT; let vf = vs | fs;
    let uni = wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None };
    let bgl_c = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries: &[
        bgl_entry(0, cs, uni), bgl_entry(1, cs, ro(cs)), bgl_entry(2, cs, ro(cs)), bgl_entry(3, cs, ro(cs)), bgl_entry(4, cs, ro(cs)), bgl_entry(5, cs, ro(cs)),
        bgl_entry(6, cs, wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }) ] });
    let bgl_r = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries: &[
        bgl_entry(0, vf, uni), bgl_entry(1, vs, ro(vs)), bgl_entry(2, vs, ro(vs)), bgl_entry(3, vs, ro(vs)), bgl_entry(4, vs, ro(vs)),
        bgl_entry(5, fs, wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }),
        bgl_entry(6, fs, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison)) ] });
    fn be(b: u32, buf: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> { wgpu::BindGroupEntry { binding: b, resource: buf.as_entire_binding() } }
    let bg_c = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &bgl_c, entries: &[be(0, &b_mp), be(1, &b_base), be(2, &b_off), be(3, &b_tgt), be(4, &b_delta), be(5, &b_w), be(6, &b_out)] });
    let bg_r = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &bgl_r, entries: &[be(0, &b_scene), be(1, &b_out), be(2, &b_j), be(3, &b_sw), be(4, &b_bones),
        wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&v_sh) }, wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&samp) }] });

    let bgl_s = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries: &[
        bgl_entry(0, vf, uni), bgl_entry(1, vs, ro(vs)), bgl_entry(2, vs, ro(vs)), bgl_entry(3, vs, ro(vs)), bgl_entry(4, vs, ro(vs))] });
    let bg_s = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &bgl_s, entries: &[be(0, &b_scene), be(1, &b_out), be(2, &b_j), be(3, &b_sw), be(4, &b_bones)] });
    let pl_s = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&bgl_s)], immediate_size: 0 });
    let pl_c = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&bgl_c)], immediate_size: 0 });
    let pl_r = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&bgl_r)], immediate_size: 0 });
    let p_morph = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: Some(&pl_c), module: &shader, entry_point: Some("morph"), compilation_options: Default::default(), cache: None });
    let prim = wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() };
    let depth_state = |fmt| Some(wgpu::DepthStencilState { format: fmt, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Less), stencil: Default::default(), bias: Default::default() });
    let p_shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { label: None, layout: Some(&pl_s),
        vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_shadow"), compilation_options: Default::default(), buffers: &[] },
        fragment: None, primitive: prim, depth_stencil: depth_state(wgpu::TextureFormat::Depth32Float), multisample: Default::default(), multiview_mask: None, cache: None });
    let p_main = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { label: None, layout: Some(&pl_r),
        vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
        fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8Unorm, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
        primitive: prim, depth_stencil: depth_state(wgpu::TextureFormat::Depth32Float), multisample: wgpu::MultisampleState { count: 4, mask: !0, alpha_to_coverage_enabled: false }, multiview_mask: None, cache: None });

    // Szene
    let cols = (copies as f32).sqrt().ceil().max(1.0);
    let (spacing_x, spacing_y) = (0.9f32, 0.9f32);
    let rows = ((copies as f32) / cols).ceil();
    let center = [(cols - 1.0) * spacing_x / 2.0, (rows - 1.0) * spacing_y / 2.0, 0.8];
    let extent = (cols * spacing_x).max(rows * spacing_y).max(2.0);
    let view = look_at([center[0], center[1] - extent * 1.6 - 1.5, 1.0 + extent * 0.3], center, [0.0, 0.0, 1.0]);
    let proj = perspective(0.8, w as f32 / h as f32, 0.1, 100.0);
    let light_dir = [0.4f32, -0.6, 0.7];
    let lview = look_at([center[0] + light_dir[0] * 8.0, center[1] + light_dir[1] * 8.0, center[2] + light_dir[2] * 8.0], center, [0.0, 0.0, 1.0]);
    let scene = Scene { view_proj: mat_mul(proj, view), light_vp: mat_mul(ortho(extent + 1.5, 0.1, 30.0), lview), light_dir: [light_dir[0], light_dir[1], light_dir[2], 0.0], n: [nv, copies, nb, 0], grid: [spacing_x, spacing_y, cols, 0.0] };
    queue.write_buffer(&b_scene, 0, bytemuck::bytes_of(&scene));

    let q_set = has_ts.then(|| device.create_query_set(&wgpu::QuerySetDescriptor { label: None, ty: wgpu::QueryType::Timestamp, count: 6 }));
    let q_res = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 6 * 8, usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let q_read = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 6 * 8, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    let period = queue.get_timestamp_period() as f64;

    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    let n_idx = mesh.tris.len() as u32;
    let idle_bones: Vec<[[f32; 4]; 4]> = vec![[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]; (copies * nb) as usize];
    let mut results = serde_json::Map::new();

    // Szenarien: (Name, aktive Targets, Compute?, Render?)
    let scenarios: Vec<(&str, usize, bool, bool)> = vec![
        ("statisch_render", 0, false, true), ("morph_1_target_drag", 1, true, true), ("morph_64_targets_drag", 64, true, true),
        ("morph_alle_targets_drag", nt as usize, true, true), ("nur_compute_alle_targets", nt as usize, true, false),
    ];
    for (name, k, do_morph, do_render) in scenarios {
        let mut active: Vec<usize> = (0..nt as usize).collect();
        for i in 0..active.len() { let j = i + (rng.next() as usize) % (active.len() - i); active.swap(i, j); }
        active.truncate(k);
        let (mut wall, mut cpu_enc, mut g_total, mut g_comp, mut g_shadow, mut g_main) = (vec![], vec![], vec![], vec![], vec![], vec![]);
        let total_frames = setup.frames + 60;
        for f in 0..total_frames {
            let t0 = Instant::now();
            if do_morph {
                let mut wv = vec![0f32; (nt * copies) as usize];
                for c in 0..copies as usize { for &t in &active { wv[c * nt as usize + t] = ((f as f32 * 0.07 + t as f32).sin()) * 0.8; } }
                queue.write_buffer(&b_w, 0, bytemuck::cast_slice(&wv));
            }
            let mut bones = idle_bones.clone();
            for c in 0..copies as usize { for b in 0..nb as usize { let a = 0.15 * (f as f32 * 0.05 + b as f32).sin(); let (s, co) = a.sin_cos(); bones[c * nb as usize + b] = [[co, s, 0.0, 0.0], [-s, co, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]; } }
            queue.write_buffer(&b_bones, 0, bytemuck::cast_slice(&bones));
            let mut enc = device.create_command_encoder(&Default::default());
            if do_morph {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: q_set.as_ref().map(|q| wgpu::ComputePassTimestampWrites { query_set: q, beginning_of_pass_write_index: Some(0), end_of_pass_write_index: Some(1) }) });
                pass.set_pipeline(&p_morph); pass.set_bind_group(0, &bg_c, &[]);
                pass.dispatch_workgroups((nv * copies + 63) / 64, 1, 1);
            } else if let Some(q) = &q_set { let _ = q; }
            if do_render {
                {
                    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor { label: None, color_attachments: &[], depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &v_sh, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                        timestamp_writes: q_set.as_ref().map(|q| wgpu::RenderPassTimestampWrites { query_set: q, beginning_of_pass_write_index: Some(2), end_of_pass_write_index: Some(3) }), occlusion_query_set: None, multiview_mask: None });
                    pass.set_pipeline(&p_shadow); pass.set_bind_group(0, &bg_s, &[]); pass.set_index_buffer(b_idx.slice(..), wgpu::IndexFormat::Uint32);
                    for c in 0..copies { pass.draw_indexed(0..n_idx, 0, c..c + 1); }
                }
                {
                    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor { label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &v_cms, depth_slice: None, resolve_target: Some(&v_crs), ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.2, g: 0.22, b: 0.25, a: 1.0 }), store: wgpu::StoreOp::Discard } })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &v_dms, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }), stencil_ops: None }),
                        timestamp_writes: q_set.as_ref().map(|q| wgpu::RenderPassTimestampWrites { query_set: q, beginning_of_pass_write_index: Some(4), end_of_pass_write_index: Some(5) }), occlusion_query_set: None, multiview_mask: None });
                    pass.set_pipeline(&p_main); pass.set_bind_group(0, &bg_r, &[]); pass.set_index_buffer(b_idx.slice(..), wgpu::IndexFormat::Uint32);
                    for c in 0..copies { pass.draw_indexed(0..n_idx, 0, c..c + 1); }
                }
            }
            if let Some(q) = &q_set { enc.resolve_query_set(q, 0..6, &q_res, 0); enc.copy_buffer_to_buffer(&q_res, 0, &q_read, 0, 48); }
            queue.submit([enc.finish()]);
            let enc_done = t0.elapsed().as_secs_f64() * 1e3;
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            let wall_ms = t0.elapsed().as_secs_f64() * 1e3;
            if f >= 60 {
                wall.push(wall_ms); cpu_enc.push(enc_done);
                if q_set.is_some() {
                    let (tx, rx) = std::sync::mpsc::channel();
                    q_read.slice(..).map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
                    let _ = device.poll(wgpu::PollType::wait_indefinitely());
                    if rx.recv().map(|r| r.is_ok()).unwrap_or(false) {
                        let d = q_read.slice(..).get_mapped_range().expect("map");
                        let t: &[u64] = bytemuck::cast_slice(&d);
                        let ms = |a: usize, b: usize| (t[b].saturating_sub(t[a])) as f64 * period / 1e6;
                        if do_morph { g_comp.push(ms(0, 1)); }
                        if do_render { g_shadow.push(ms(2, 3)); g_main.push(ms(4, 5)); }
                        g_total.push(if do_morph { ms(0, 1) } else { 0.0 } + if do_render { ms(2, 3) + ms(4, 5) } else { 0.0 });
                        drop(d); q_read.unmap();
                    }
                }
            }
        }
        let mut s = serde_json::Map::new();
        s.insert("aktive_targets".into(), json!(k));
        s.insert("frame_wall_ms".into(), json!(stats(wall)));
        s.insert("cpu_encode_submit_ms".into(), json!(stats(cpu_enc)));
        if !g_total.is_empty() {
            s.insert("gpu_gesamt_ms".into(), json!(stats(g_total)));
            if !g_comp.is_empty() { s.insert("gpu_morph_compute_ms".into(), json!(stats(g_comp))); }
            if !g_shadow.is_empty() { s.insert("gpu_schatten_ms".into(), json!(stats(g_shadow))); s.insert("gpu_hauptpass_ms".into(), json!(stats(g_main))); }
        }
        results.insert(name.into(), serde_json::Value::Object(s));
    }
    json!({
        "adapter": { "name": info.name, "backend": format!("{:?}", info.backend), "typ": format!("{:?}", info.device_type), "treiber": info.driver, "treiber_info": info.driver_info, "timestamp_query": has_ts },
        "szene": { "kopien": copies, "aufloesung": [w, h], "msaa": 4, "schatten_aufloesung": 2048, "dreiecke_gesamt": (mesh.tris.len() / 3) as u32 * copies, "vertices_gesamt": nv * copies, "aktive_gpu_bytes_berechnet_mb": gpu_bytes as f64 / 1048576.0 },
        "szenarien": results
    })
}
