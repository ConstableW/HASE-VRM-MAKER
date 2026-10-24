//! 3D-Renderer (wgpu): Meshes mit dynamischen Positionen/Normalen, Alpha-Masken, Bodenraster. Nutzbar im egui-Callback und offscreen.
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub struct ImageRgba { pub w: u32, pub h: u32, pub levels: Vec<Vec<u8>> }

pub struct MeshDesc {
    pub id: String,
    pub indices: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
    pub data: Vec<f32>, // pos(3)+nrm(3) je Vertex
    pub texture: Option<Arc<ImageRgba>>,
    pub tint: [f32; 3],
    pub mask: bool,
    pub cull_back: bool,
}

pub enum MeshUpdate { Create(MeshDesc), Positions { id: String, data: Vec<f32> }, Tint { id: String, tint: [f32; 3] }, Remove(String) }

#[repr(C)] #[derive(Clone, Copy, Pod, Zeroable)]
pub struct Globals { pub vp: [[f32; 4]; 4], pub eye: [f32; 4], pub key_dir: [f32; 4], pub fill_dir: [f32; 4], pub sky: [f32; 4], pub ground: [f32; 4], pub flags: [f32; 4] }
#[repr(C)] #[derive(Clone, Copy, Pod, Zeroable)]
struct MatU { tint: [f32; 4], flags: [f32; 4] }

struct GpuMesh { vb: wgpu::Buffer, uvb: wgpu::Buffer, ib: wgpu::Buffer, n_idx: u32, n_vert: usize, mat_buf: wgpu::Buffer, mat_bg: wgpu::BindGroup, mask: bool, cull_back: bool, has_tex: bool, _tex: Option<wgpu::Texture> }

pub struct SceneRenderer {
    pipe_opaque: wgpu::RenderPipeline, pipe_mask: wgpu::RenderPipeline, pipe_ground: wgpu::RenderPipeline,
    global_buf: wgpu::Buffer, global_bg: wgpu::BindGroup, mat_bgl: wgpu::BindGroupLayout, sampler: wgpu::Sampler,
    white: wgpu::TextureView, ground_bg: wgpu::BindGroup, ground_vb: wgpu::Buffer, ground_uv: wgpu::Buffer, ground_ib: wgpu::Buffer,
    meshes: HashMap<String, GpuMesh>, order: Vec<String>, gamma_flag: f32,
    pub stats: RenderStats,
}
#[derive(Default, Clone, Copy)]
pub struct RenderStats { pub meshes: usize, pub vertices: usize, pub triangles: usize, pub gpu_mb_est: f32 }

impl SceneRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, color_format: wgpu::TextureFormat, depth_format: wgpu::TextureFormat, samples: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("scene"), source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()) });
        let g_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }] });
        let mat_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None }] });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&g_bgl), Some(&mat_bgl)], immediate_size: 0 });
        let vbuf = [
            wgpu::VertexBufferLayout { array_stride: 24, step_mode: wgpu::VertexStepMode::Vertex, attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3] },
            wgpu::VertexBufferLayout { array_stride: 8, step_mode: wgpu::VertexStepMode::Vertex, attributes: &wgpu::vertex_attr_array![2 => Float32x2] },
        ];
        let mk = |fs: &str, cull: Option<wgpu::Face>| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { label: Some(fs), layout: Some(&layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[Some(vbuf[0].clone()), Some(vbuf[1].clone())] },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some(fs), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: color_format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: cull, front_face: wgpu::FrontFace::Ccw, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: depth_format, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::LessEqual), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState { count: samples.max(1), mask: !0, alpha_to_coverage_enabled: false }, multiview_mask: None, cache: None });
        let pipe_opaque = mk("fs_main", Some(wgpu::Face::Back));
        let pipe_mask = mk("fs_main", None);
        let pipe_ground = mk("fs_ground", None);
        let global_buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("globals"), size: std::mem::size_of::<Globals>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let global_bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &g_bgl, entries: &[wgpu::BindGroupEntry { binding: 0, resource: global_buf.as_entire_binding() }] });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { address_mode_u: wgpu::AddressMode::Repeat, address_mode_v: wgpu::AddressMode::Repeat, mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, mipmap_filter: wgpu::MipmapFilterMode::Linear, anisotropy_clamp: 8, ..Default::default() });
        let white_tex = device.create_texture_with_data(queue, &wgpu::TextureDescriptor { label: Some("white"), size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[] }, wgpu::util::TextureDataOrder::LayerMajor, &[255, 255, 255, 255]);
        let white = white_tex.create_view(&Default::default());
        let gmat = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::bytes_of(&MatU { tint: [1.0; 4], flags: [0.0; 4] }), usage: wgpu::BufferUsages::UNIFORM });
        let ground_bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &mat_bgl, entries: &[wgpu::BindGroupEntry { binding: 0, resource: gmat.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&white) }, wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) }] });
        let gv: Vec<f32> = vec![-10.0, 0.0, -10.0, 0.0, 1.0, 0.0, 10.0, 0.0, -10.0, 0.0, 1.0, 0.0, 10.0, 0.0, 10.0, 0.0, 1.0, 0.0, -10.0, 0.0, 10.0, 0.0, 1.0, 0.0];
        let ground_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&gv), usage: wgpu::BufferUsages::VERTEX });
        let ground_uv = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&[0f32; 8]), usage: wgpu::BufferUsages::VERTEX });
        let ground_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&[0u32, 2, 1, 0, 3, 2]), usage: wgpu::BufferUsages::INDEX });
        Self { pipe_opaque, pipe_mask, pipe_ground, global_buf, global_bg, mat_bgl, sampler, white, ground_bg, ground_vb, ground_uv, ground_ib, meshes: HashMap::new(), order: vec![], gamma_flag: if color_format.is_srgb() { 0.0 } else { 1.0 }, stats: Default::default() }
    }

    pub fn apply_updates(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, updates: Vec<MeshUpdate>) {
        for u in updates {
            match u {
                MeshUpdate::Create(d) => {
                    let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&d.id), contents: bytemuck::cast_slice(&d.data), usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST });
                    let uvb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&d.uv), usage: wgpu::BufferUsages::VERTEX });
                    let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&d.indices), usage: wgpu::BufferUsages::INDEX });
                    let (tex, view, has_tex) = match &d.texture {
                        Some(img) => {
                            let t = device.create_texture_with_data(queue, &wgpu::TextureDescriptor { label: Some(&d.id), size: wgpu::Extent3d { width: img.w, height: img.h, depth_or_array_layers: 1 }, mip_level_count: img.levels.len() as u32, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8UnormSrgb, usage: wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[] },
                                wgpu::util::TextureDataOrder::LayerMajor, &img.levels.concat());
                            let v = t.create_view(&Default::default()); (Some(t), v, true)
                        }
                        None => (None, self.white.clone(), false),
                    };
                    let mu = MatU { tint: [d.tint[0], d.tint[1], d.tint[2], 1.0], flags: [d.mask as u32 as f32, has_tex as u32 as f32, 0.0, 0.0] };
                    let mat_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::bytes_of(&mu), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST });
                    let mat_bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &self.mat_bgl, entries: &[wgpu::BindGroupEntry { binding: 0, resource: mat_buf.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) }, wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) }] });
                    if !self.order.contains(&d.id) { self.order.push(d.id.clone()); }
                    self.meshes.insert(d.id.clone(), GpuMesh { vb, uvb, ib, n_idx: d.indices.len() as u32, n_vert: d.uv.len(), mat_buf, mat_bg, mask: d.mask, cull_back: d.cull_back, has_tex, _tex: tex });
                }
                MeshUpdate::Positions { id, data } => { if let Some(m) = self.meshes.get(&id) { queue.write_buffer(&m.vb, 0, bytemuck::cast_slice(&data)); } }
                MeshUpdate::Tint { id, tint } => { if let Some(m) = self.meshes.get(&id) { let mu = MatU { tint: [tint[0], tint[1], tint[2], 1.0], flags: [m.mask as u32 as f32, m.has_tex as u32 as f32, 0.0, 0.0] }; queue.write_buffer(&m.mat_buf, 0, bytemuck::bytes_of(&mu)); } }
                MeshUpdate::Remove(id) => { self.meshes.remove(&id); self.order.retain(|x| *x != id); }
            }
        }
        self.stats = RenderStats { meshes: self.meshes.len(), vertices: self.meshes.values().map(|m| m.n_vert).sum(), triangles: self.meshes.values().map(|m| m.n_idx as usize / 3).sum(), gpu_mb_est: self.meshes.values().map(|m| (m.n_vert * 32 + m.n_idx as usize * 4) as f32).sum::<f32>() / 1048576.0 };
    }

    pub fn write_globals(&self, queue: &wgpu::Queue, mut g: Globals) { g.flags[0] = self.gamma_flag; queue.write_buffer(&self.global_buf, 0, bytemuck::bytes_of(&g)); }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_bind_group(0, &self.global_bg, &[]);
        // Boden
        pass.set_pipeline(&self.pipe_ground);
        pass.set_bind_group(1, &self.ground_bg, &[]);
        pass.set_vertex_buffer(0, self.ground_vb.slice(..)); pass.set_vertex_buffer(1, self.ground_uv.slice(..)); pass.set_index_buffer(self.ground_ib.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..6, 0, 0..1);
        for want_mask in [false, true] {
            for id in &self.order {
                let Some(m) = self.meshes.get(id) else { continue };
                if m.mask != want_mask { continue; }
                pass.set_pipeline(if m.mask || !m.cull_back { &self.pipe_mask } else { &self.pipe_opaque });
                pass.set_bind_group(1, &m.mat_bg, &[]);
                pass.set_vertex_buffer(0, m.vb.slice(..)); pass.set_vertex_buffer(1, m.uvb.slice(..)); pass.set_index_buffer(m.ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..m.n_idx, 0, 0..1);
            }
        }
    }
}
