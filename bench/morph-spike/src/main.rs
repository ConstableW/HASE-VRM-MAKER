mod cpu;
mod data;
mod gpu;
mod stats;
mod ui;
use serde_json::json;
use std::path::PathBuf;
use std::time::Instant;

fn repo_root() -> PathBuf {
    let mut p = std::env::current_dir().unwrap();
    for _ in 0..5 { if p.join("content/testcandidate").exists() { return p; } if !p.pop() { break; } }
    panic!("Repo-Wurzel (content/testcandidate) nicht gefunden – im Repo ausführen")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("all");
    let root = repo_root();
    let mh = root.join("content/testcandidate/mpfb2-cc0");
    let rss0 = stats::rss_mb();
    let t0 = Instant::now();
    let mesh = data::load_mesh(&mh.join("base.obj"));
    let t_mesh = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = Instant::now();
    let morphs = data::load_morphs(&mh.join("targets"), &mesh);
    let t_morph = t1.elapsed().as_secs_f64() * 1e3;
    let t2 = Instant::now();
    let skin = data::load_skin(&mh.join("rigs/weights.game_engine.json"), &mesh);
    let t_skin = t2.elapsed().as_secs_f64() * 1e3;
    let rss1 = stats::rss_mb();
    let mut result = json!({
        "umgebung": { "os": std::env::consts::OS, "arch": std::env::consts::ARCH, "cpus": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0) },
        "laden": { "mesh_ms": t_mesh, "morphs_ms": t_morph, "skin_ms": t_skin, "rss_vorher_mb": rss0, "rss_nachher_mb": rss1, "rss_delta_mb": rss1 - rss0 }
    });
    if mode == "cpu" || mode == "all" { result["cpu"] = cpu::run(&mesh, &morphs, &skin); }
    if mode == "ui" || mode == "all" { result["ui_egui"] = ui::run(&morphs.names); }
    if mode == "gpu" || mode == "all" {
        let arg = |k: &str, d: u32| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(d);
        for copies in [1u32, arg("--copies", 6)] {
            result[format!("gpu_kopien_{copies}")] = gpu::run(&mesh, &morphs, &skin, gpu::Setup { copies, frames: arg("--frames", 600) as usize, w: 1920, h: 1080 });
        }
    }
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
