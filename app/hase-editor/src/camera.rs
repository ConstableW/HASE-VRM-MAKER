//! Orbit-Kamera (Y-up, Blick auf +Z-Front des Charakters).
use crate::render::Globals;

#[derive(Clone, Copy)]
pub struct Camera { pub yaw: f32, pub pitch: f32, pub dist: f32, pub target: [f32; 3], pub fov: f32 }

/// Beleuchtungs-/Hintergrund-Preset des Viewports.
#[derive(Clone, Copy)]
pub struct Lighting { pub name: &'static str, pub key_dir: [f32; 4], pub key_col: [f32; 4], pub fill_dir: [f32; 4], pub sky: [f32; 4], pub ground: [f32; 4], pub bg_top: [f32; 4], pub bg_bot: [f32; 4] }

pub const LIGHTS: [Lighting; 3] = [
    Lighting { name: "Studio", key_dir: [0.45, 0.75, 0.6, 0.0], key_col: [1.0, 0.97, 0.92, 1.0], fill_dir: [-0.7, 0.2, 0.4, 0.0], sky: [0.62, 0.68, 0.78, 1.0], ground: [0.22, 0.2, 0.2, 1.0], bg_top: [0.10, 0.13, 0.19, 1.0], bg_bot: [0.03, 0.05, 0.08, 1.0] },
    Lighting { name: "Outdoor", key_dir: [-0.5, 0.8, 0.45, 0.0], key_col: [1.0, 0.92, 0.78, 1.0], fill_dir: [0.6, 0.15, 0.3, 0.0], sky: [0.55, 0.72, 0.95, 1.0], ground: [0.35, 0.32, 0.26, 1.0], bg_top: [0.42, 0.62, 0.88, 1.0], bg_bot: [0.78, 0.84, 0.90, 1.0] },
    Lighting { name: "Nacht", key_dir: [0.3, 0.5, 0.8, 0.0], key_col: [0.55, 0.65, 1.0, 1.0], fill_dir: [-0.6, 0.1, 0.3, 0.0], sky: [0.22, 0.26, 0.45, 1.0], ground: [0.08, 0.08, 0.14, 1.0], bg_top: [0.02, 0.03, 0.08, 1.0], bg_bot: [0.06, 0.05, 0.12, 1.0] },
];

pub const PRESETS: [(&str, Camera); 5] = [
    ("Vorne", Camera { yaw: 0.0, pitch: 0.04, dist: 3.0, target: [0.0, 0.9, 0.0], fov: 0.62 }),
    ("Seite", Camera { yaw: 1.5708, pitch: 0.04, dist: 3.4, target: [0.0, 0.88, 0.0], fov: 0.62 }),
    ("Hinten", Camera { yaw: 3.1416, pitch: 0.04, dist: 3.4, target: [0.0, 0.88, 0.0], fov: 0.62 }),
    ("Kopf", Camera { yaw: 0.0, pitch: 0.02, dist: 0.95, target: [0.0, 1.55, 0.0], fov: 0.62 }),
    ("Oberkörper", Camera { yaw: 0.35, pitch: 0.05, dist: 1.8, target: [0.0, 1.25, 0.0], fov: 0.62 }),
];

impl Default for Camera { fn default() -> Self { PRESETS[0].1 } }

type M4 = [[f32; 4]; 4];
fn mul(a: M4, b: M4) -> M4 { let mut r = [[0f32; 4]; 4]; for c in 0..4 { for rw in 0..4 { for k in 0..4 { r[c][rw] += a[k][rw] * b[c][k]; } } } r }
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn norm(v: [f32; 3]) -> [f32; 3] { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9); [v[0] / l, v[1] / l, v[2] / l] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }

impl Camera {
    pub fn eye(&self) -> [f32; 3] {
        let (sy, cy, sp, cp) = (self.yaw.sin(), self.yaw.cos(), self.pitch.sin(), self.pitch.cos());
        [self.target[0] + self.dist * sy * cp, self.target[1] + self.dist * sp, self.target[2] + self.dist * cy * cp]
    }
    pub fn orbit(&mut self, dx: f32, dy: f32) { self.yaw -= dx * 0.008; self.pitch = (self.pitch + dy * 0.008).clamp(-1.45, 1.45); }
    pub fn zoom(&mut self, scroll: f32) { self.dist = (self.dist * (1.0 - scroll * 0.0015)).clamp(0.25, 12.0); }
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let eye = self.eye(); let f = norm(sub(self.target, eye)); let r = norm(cross(f, [0.0, 1.0, 0.0])); let u = cross(r, f);
        let k = self.dist * 0.0012;
        for i in 0..3 { self.target[i] += (-r[i] * dx + u[i] * dy) * k; }
        self.target[1] = self.target[1].clamp(-0.5, 2.5);
    }
    /// Strahl durch einen Bildschirmpunkt (ndc x,y in -1…1, y nach oben).
    pub fn ray(&self, nx: f32, ny: f32, aspect: f32) -> ([f32; 3], [f32; 3]) {
        let eye = self.eye();
        let f = norm(sub(self.target, eye)); let s = norm(cross(f, [0.0, 1.0, 0.0])); let u = cross(s, f);
        let t = (self.fov / 2.0).tan();
        let d = norm([f[0] + s[0] * nx * t * aspect + u[0] * ny * t, f[1] + s[1] * nx * t * aspect + u[1] * ny * t, f[2] + s[2] * nx * t * aspect + u[2] * ny * t]);
        (eye, d)
    }
    pub fn forward(&self) -> [f32; 3] { norm(sub(self.target, self.eye())) }
    pub fn right(&self) -> [f32; 3] { norm(cross(self.forward(), [0.0, 1.0, 0.0])) }
    /// Weltpunkt → Normalisierte Gerätekoordinaten (x,y in -1…1, y nach oben).
    pub fn project(&self, p: [f32; 3], aspect: f32) -> Option<[f32; 2]> {
        let vp = self.globals(aspect).vp;
        let v = [p[0], p[1], p[2], 1.0];
        let mut c = [0f32; 4];
        for r in 0..4 { for k in 0..4 { c[r] += vp[k][r] * v[k]; } }
        if c[3] <= 1e-5 { return None; }
        Some([c[0] / c[3], c[1] / c[3]])
    }
    pub fn globals(&self, aspect: f32) -> Globals { self.globals_lit(aspect, &LIGHTS[0], true) }
    pub fn globals_lit(&self, aspect: f32, l: &Lighting, grid: bool) -> Globals {
        let eye = self.eye();
        let f = norm(sub(self.target, eye)); let s = norm(cross(f, [0.0, 1.0, 0.0])); let u = cross(s, f);
        let view: M4 = [[s[0], u[0], -f[0], 0.0], [s[1], u[1], -f[1], 0.0], [s[2], u[2], -f[2], 0.0], [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0]];
        let (n, fa) = (0.05f32, 60.0f32);
        let t = 1.0 / (self.fov / 2.0).tan();
        let proj: M4 = [[t / aspect, 0.0, 0.0, 0.0], [0.0, t, 0.0, 0.0], [0.0, 0.0, fa / (n - fa), -1.0], [0.0, 0.0, n * fa / (n - fa), 0.0]];
        Globals { vp: mul(proj, view), eye: [eye[0], eye[1], eye[2], 1.0], key_dir: l.key_dir, fill_dir: l.fill_dir, sky: l.sky, ground: l.ground, flags: [0.0, 0.5, if grid { 1.0 } else { 0.0 }, 0.0], key_col: l.key_col, bg_top: l.bg_top, bg_bot: l.bg_bot }
    }
}
