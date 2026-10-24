//! Hautdetails: Masken aus Morph-Fußabdrücken (Lippen, Lider, Wangen, Nase) in UV-Raum und CPU-Komposition von Make-up, Sommersprossen
//! und lokalen Details (Muttermal, Tattoo-Motive, Narben) auf die Hauttextur. Ergebnis wird als Textur an den Renderer übergeben.
use crate::render::ImageRgba;
use hase_core::body::Body;
use hase_core::state::{Decal, SkinStyle};
use std::sync::Arc;

pub const N: usize = 1024;

pub struct Masks { pub lips: Vec<u8>, pub eyes: Vec<u8>, pub cheeks: Vec<u8>, pub nose: Vec<u8> }
impl Masks { pub fn empty() -> Masks { Masks { lips: vec![0; N * N], eyes: vec![0; N * N], cheeks: vec![0; N * N], nose: vec![0; N * N] } } }

/// Vertexgewichte (je OBJ-Vertex) aus der Verformungsstärke ausgewählter Targets, auf 0…1 normiert.
fn weights(body: &Body, lo: f32, hi: f32, pick: impl Fn(&str) -> bool) -> Vec<f32> {
    let mut w = vec![0f32; body.base_pos.len()];
    for (t, name) in body.morphs.names.iter().enumerate() {
        if !pick(name) { continue; }
        let (a, b) = (body.morphs.t_off[t] as usize, body.morphs.t_off[t + 1] as usize);
        for k in a..b { let d = body.morphs.t_delta[k]; let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(); let v = body.morphs.t_idx[k] as usize; w[v] = w[v].max(m); }
    }
    let mx = w.iter().cloned().fold(0f32, f32::max).max(1e-6);
    // glatter Einsatz zwischen lo und hi (Anteil der Maximalverformung)
    w.iter().map(|x| { let t = ((x / mx - lo) / (hi - lo)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }).collect()
}

fn raster(body: &Body, w: &[f32], lo: f32, hi: f32) -> Vec<u8> {
    let mut out = vec![0u8; N * N];
    let t = &body.topo;
    for tri in t.indices.chunks_exact(3) {
        let uv = [t.uv[tri[0] as usize], t.uv[tri[1] as usize], t.uv[tri[2] as usize]];
        let ww = [w[t.vmap[tri[0] as usize] as usize], w[t.vmap[tri[1] as usize] as usize], w[t.vmap[tri[2] as usize] as usize]];
        if ww.iter().all(|&x| x <= 0.0) { continue; }
        let p: Vec<[f32; 2]> = uv.iter().map(|u| [u[0] * N as f32, u[1] * N as f32]).collect();
        let (minx, maxx) = (p.iter().map(|q| q[0]).fold(f32::MAX, f32::min).floor().max(0.0) as usize, (p.iter().map(|q| q[0]).fold(f32::MIN, f32::max).ceil() as usize).min(N - 1));
        let (miny, maxy) = (p.iter().map(|q| q[1]).fold(f32::MAX, f32::min).floor().max(0.0) as usize, (p.iter().map(|q| q[1]).fold(f32::MIN, f32::max).ceil() as usize).min(N - 1));
        let den = (p[1][1] - p[2][1]) * (p[0][0] - p[2][0]) + (p[2][0] - p[1][0]) * (p[0][1] - p[2][1]);
        if den.abs() < 1e-9 { continue; }
        for y in miny..=maxy { for x in minx..=maxx {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let l0 = ((p[1][1] - p[2][1]) * (px - p[2][0]) + (p[2][0] - p[1][0]) * (py - p[2][1])) / den;
            let l1 = ((p[2][1] - p[0][1]) * (px - p[2][0]) + (p[0][0] - p[2][0]) * (py - p[2][1])) / den;
            let l2 = 1.0 - l0 - l1;
            if l0 < -0.02 || l1 < -0.02 || l2 < -0.02 { continue; }
            let v = (l0 * ww[0] + l1 * ww[1] + l2 * ww[2]).clamp(0.0, 1.0);
            // Kurve nach der Interpolation: dreiecksförmiges Ausbluten in große Dreiecke abschneiden
            let v = { let t = ((v - lo) / (hi - lo)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
            let o = &mut out[y * N + x]; *o = (*o).max((v * 255.0) as u8);
        } }
    }
    out
}

pub fn build_masks(body: &Body) -> Masks {
    let lips = weights(body, 0.15, 0.50, |n| n.starts_with("mouth/") && n.contains("lip-volume"));
    let eyes = weights(body, 0.10, 0.50, |n| n.starts_with("eyes/") && n.contains("eyefold") && !n.contains("bag"));
    let cheeks = weights(body, 0.10, 0.50, |n| n.starts_with("cheek/") && (n.contains("volume") || n.contains("bones")));
    let nose = weights(body, 0.30, 0.70, |n| n.starts_with("nose/") && (n.contains("scale") || n.contains("width1") || n.contains("hump")));
    Masks { lips: raster(body, &lips, 0.45, 0.80), eyes: raster(body, &eyes, 0.30, 0.70), cheeks: raster(body, &cheeks, 0.0, 1.0), nose: raster(body, &nose, 0.0, 1.0) }
}

fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393).wrapping_add((y as u32).wrapping_mul(668265263)).wrapping_add(seed.wrapping_mul(2246822519));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177); h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}

fn lin(c: [f32; 3]) -> [f32; 3] { c } // Texturraum ist sRGB; Farben aus dem UI sind ebenfalls sRGB-nah

/// Alpha einer Dekor-Form an Pixelposition (px,py) relativ zum Zentrum (in Pixeln), Größe `r` in Pixeln, Rotation `rot`.
fn shape_alpha(shape: u8, dx: f32, dy: f32, r: f32, rot: f32) -> f32 {
    let (c, s) = (rot.cos(), rot.sin());
    let (x, y) = ((dx * c + dy * s) / r, (-dx * s + dy * c) / r);
    let aa = |d: f32| (0.5 - d * r).clamp(0.0, 1.0); // d in Einheiten → Kante ~1 px weich
    match shape {
        0 => { let d = (x * x + y * y).sqrt(); (1.0 - ((d - 0.85) / 0.25).clamp(0.0, 1.0)).clamp(0.0, 1.0) }
        1 => { // Stern (5 Zacken) über Polarradius
            let a = y.atan2(x); let k = 5.0; let rr = 0.45 + 0.55 * (0.5 + 0.5 * (k * a).cos()).powf(2.2);
            let d = (x * x + y * y).sqrt() - rr; aa(d)
        }
        2 => { // Herz
            let (hx, hy) = (x * 1.1, -y * 1.1 + 0.15); let v = (hx * hx + hy * hy - 0.6).powi(3) - hx * hx * hy * hy * hy;
            (1.0 - (v / 0.04).clamp(0.0, 1.0)).max(0.0) * (1.0 - (v.max(0.0) / 0.04).clamp(0.0, 1.0))
        }
        3 => { let d = ((x * x + y * y).sqrt() - 0.75).abs(); aa(d - 0.12) }
        4 => { let d = (x.abs().min(y.abs())).max(0.0); let inside = x.abs().max(y.abs()) < 1.0; if inside { aa(d - 0.14) } else { 0.0 } }
        _ => { // Narbe: länglicher Strich mit leichter Wulstform
            let w = 0.14 * (1.0 - (x / 3.0).abs().min(1.0).powi(2)); let inside = x.abs() < 3.0;
            if inside { aa(y.abs() - w) * (0.75 + 0.25 * (x * 9.0).sin().abs()) } else { 0.0 }
        }
    }
}

pub fn compose(base: &ImageRgba, masks: &Masks, st: &SkinStyle) -> Arc<ImageRgba> {
    // Basis auf N×N bringen (Mip-Ebene mit passender Größe wählen)
    let mut lvl = 0usize; let (mut w, mut h) = (base.w as usize, base.h as usize);
    while w > N && lvl + 1 < base.levels.len() { lvl += 1; w /= 2; h = (h / 2).max(1); }
    let src = &base.levels[lvl];
    let mut out = vec![255u8; N * N * 4];
    for y in 0..N { for x in 0..N { let (sx, sy) = (x * w / N, y * h / N); let i = (sy * w + sx) * 4; let o = (y * N + x) * 4; out[o..o + 4].copy_from_slice(&src[i..i + 4]); } }
    let blend = |px: &mut [u8], c: [f32; 3], a: f32| { if a <= 0.0 { return; } let a = a.clamp(0.0, 1.0); for k in 0..3 { px[k] = (px[k] as f32 * (1.0 - a) + c[k] * 255.0 * a) as u8; } };
    let (lc, sc, bc, fc) = (lin(st.lips.color), lin(st.shadow.color), lin(st.blush.color), lin(st.freckles.color));
    let fsize = (6.0 / st.freckle_size.max(0.3)).max(2.0);
    for y in 0..N { for x in 0..N {
        let i = y * N + x; let px = &mut out[i * 4..i * 4 + 4];
        if st.blush.amount > 0.0 { let m = masks.cheeks[i] as f32 / 255.0; blend(px, bc, m * st.blush.amount * 0.55); }
        if st.shadow.amount > 0.0 { let m = masks.eyes[i] as f32 / 255.0; blend(px, sc, m * st.shadow.amount * 0.7); }
        if st.lips.amount > 0.0 { let m = masks.lips[i] as f32 / 255.0; blend(px, lc, m * st.lips.amount * 0.85); }
        if st.freckles.amount > 0.0 {
            let m = (masks.cheeks[i] as f32 / 255.0).max(masks.nose[i] as f32 / 255.0);
            if m > 0.12 {
                for (layer, cs) in [(0u32, fsize), (1, fsize * 1.41)] {
                    let (cx, cy) = ((x as f32 / cs).floor() as i32, (y as f32 / cs).floor() as i32);
                    let seed = st.freckle_seed.wrapping_add(layer * 7919);
                    let (jx, jy) = (hash2(cx, cy, seed), hash2(cx + 91, cy + 17, seed));
                    let (fx, fy) = ((cx as f32 + 0.15 + 0.7 * jx) * cs, (cy as f32 + 0.15 + 0.7 * jy) * cs);
                    let d = ((x as f32 - fx).powi(2) + (y as f32 - fy).powi(2)).sqrt();
                    let r = cs * (0.10 + 0.22 * hash2(cx + 5, cy + 3, seed).powi(2));
                    let cluster = 0.5 + 0.5 * (hash2(cx / 3, cy / 3, seed ^ 0x55) * 2.0 - 0.3).clamp(0.0, 1.0); // Häufungen statt Gleichverteilung
                    let present = hash2(cx + 33, cy + 71, seed) < st.freckles.amount * m.powf(1.4) * 0.55 * cluster * 1.6;
                    if present { blend(px, fc, (1.0 - (d / r)).clamp(0.0, 1.0) * 0.75); }
                }
            }
        }
    } }
    // lokale Details
    for d in &st.decals { draw_decal(&mut out, d, &blend); }
    // Mip-Kette
    let mut levels = vec![out]; let (mut lw, mut lh) = (N, N);
    while lw > 1 && lh > 1 {
        let prev = levels.last().unwrap(); let (nw, nh) = (lw / 2, lh / 2); let mut next = vec![255u8; nw * nh * 4];
        for y in 0..nh { for x in 0..nw { for c in 0..4 { let g = |xx: usize, yy: usize| prev[((yy * lw) + xx) * 4 + c] as u32; next[(y * nw + x) * 4 + c] = ((g(2 * x, 2 * y) + g(2 * x + 1, 2 * y) + g(2 * x, 2 * y + 1) + g(2 * x + 1, 2 * y + 1) + 2) / 4) as u8; } } }
        levels.push(next); lw = nw; lh = nh;
    }
    Arc::new(ImageRgba { w: N as u32, h: N as u32, levels })
}

fn draw_decal(out: &mut [u8], d: &Decal, blend: &dyn Fn(&mut [u8], [f32; 3], f32)) {
    let r = (d.size * N as f32).max(1.5);
    let (cx, cy) = (d.u * N as f32, d.v * N as f32);
    let ext = (r * if d.shape == 5 { 3.2 } else { 1.3 }).ceil() as i32;
    for dy in -ext..=ext { for dx in -ext..=ext {
        let (x, y) = (cx as i32 + dx, cy as i32 + dy);
        if x < 0 || y < 0 || x >= N as i32 || y >= N as i32 { continue; }
        let a = shape_alpha(d.shape, x as f32 + 0.5 - cx, y as f32 + 0.5 - cy, r, d.rot);
        if a > 0.0 { let o = (y as usize * N + x as usize) * 4; blend(&mut out[o..o + 4], d.color, a * d.opacity); }
    } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dump_masks() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let body = Body::load(&repo.join("content/testcandidate/mpfb2-cc0"), &repo.join(".hase-cache")).unwrap();
        let m = build_masks(&body);
        for (n, d) in [("lips", &m.lips), ("eyes", &m.eyes), ("cheeks", &m.cheeks), ("nose", &m.nose)] {
            let img = image::GrayImage::from_raw(N as u32, N as u32, d.clone()).unwrap();
            let _ = img.save(std::env::temp_dir().join(format!("mask_{n}.png")));
            assert!(d.iter().any(|&x| x > 128), "{n} leer");
        }
    }
}

#[cfg(test)]
mod perf_tests {
    use super::*;
    #[test]
    fn compose_timing() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let body = Body::load(&repo.join("content/testcandidate/mpfb2-cc0"), &repo.join(".hase-cache")).unwrap();
        let t0 = std::time::Instant::now(); let m = build_masks(&body); let t_masks = t0.elapsed();
        let base = crate::session::load_image(&repo.join("content/testcandidate/downloads/skins/young_caucasian_female/young_lightskinned_female_diffuse.png"), 2048);
        let Ok(base) = base else { eprintln!("Skin nicht gefunden – übersprungen"); return };
        let mut st = SkinStyle::default(); st.lips.amount = 0.8; st.blush.amount = 0.6; st.freckles.amount = 0.8; st.shadow.amount = 0.5;
        let t1 = std::time::Instant::now(); let _ = compose(&base, &m, &st); let t_comp = t1.elapsed();
        eprintln!("MASKEN {:?} KOMPOSITION {:?}", t_masks, t_comp);
    }
}

#[cfg(test)]
mod lip_debug {
    use super::*;
    #[test]
    fn lip_mask_variants_overlay() {
        let repo = hase_core::find_repo_root(&std::env::current_dir().unwrap()).unwrap();
        let body = Body::load(&repo.join("content/testcandidate/mpfb2-cc0"), &repo.join(".hase-cache")).unwrap();
        let Ok(base) = crate::session::load_image(&repo.join("content/testcandidate/downloads/skins/young_caucasian_female/young_lightskinned_female_diffuse.png"), 1024) else { return };
        let variants: Vec<(&str, Vec<u8>)> = vec![
            ("A_aktuell", raster(&body, &weights(&body, 0.06, 0.40, |n| n.starts_with("mouth/") && (n.contains("lip-volume") || n.contains("lip-middle") || n.contains("cupidsbow") || n.contains("philtrum"))), 0.38, 0.78)),
            ("B_volume", raster(&body, &weights(&body, 0.15, 0.50, |n| n.starts_with("mouth/") && n.contains("lip-volume")), 0.45, 0.80)),
            ("C_volume_eng", raster(&body, &weights(&body, 0.30, 0.65, |n| n.starts_with("mouth/") && n.contains("lip-volume")), 0.50, 0.85)),
            ("D_volume+width", raster(&body, &weights(&body, 0.25, 0.60, |n| n.starts_with("mouth/") && (n.contains("lip-volume") || n.contains("lip-width"))), 0.50, 0.85)),
        ];
        let (x0, y0, x1, y1) = (800usize, 470usize, 980usize, 640usize);
        let (cw, ch) = (x1 - x0, y1 - y0);
        let mut sheet = image::RgbaImage::new((cw * 4) as u32, ch as u32);
        for (k, (_, m)) in variants.iter().enumerate() {
            for y in 0..ch { for x in 0..cw {
                let (sx, sy) = (x0 + x, y0 + y);
                let i = (sy * 1024 + sx) * 4; let mut px = [base.levels[0][i], base.levels[0][i + 1], base.levels[0][i + 2], 255];
                let a = m[sy * N + sx] as f32 / 255.0;
                px[0] = (px[0] as f32 * (1.0 - a * 0.6) + 255.0 * a * 0.6) as u8; px[1] = (px[1] as f32 * (1.0 - a * 0.6)) as u8; px[2] = (px[2] as f32 * (1.0 - a * 0.6)) as u8;
                sheet.put_pixel((k * cw + x) as u32, y as u32, image::Rgba(px));
            } }
        }
        let big = image::imageops::resize(&sheet, sheet.width() * 2, sheet.height() * 2, image::imageops::FilterType::Nearest);
        let _ = big.save(std::env::temp_dir().join("lip_variants.png"));
        let _ = image::imageops::resize(&image::RgbaImage::from_fn(cw as u32, ch as u32, |x, y| { let i = ((y0 + y as usize) * 1024 + x0 + x as usize) * 4; image::Rgba([base.levels[0][i], base.levels[0][i + 1], base.levels[0][i + 2], 255]) }), cw as u32 * 2, ch as u32 * 2, image::imageops::FilterType::Nearest).save(std::env::temp_dir().join("lip_base.png"));
    }
}
