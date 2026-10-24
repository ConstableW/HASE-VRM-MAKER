mod app;
mod camera;
mod export_ui;
mod icons;
mod shell;
mod ui_kit;
mod offscreen;
mod regions;
mod studios;
mod library;
mod library_ui;
mod render;
mod session;
mod skin;
mod studio;
mod theme;
mod thumbs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|x| x.to_path_buf()));
    let repo = exe_dir.as_deref().and_then(hase_core::find_repo_root)
        .or_else(|| std::env::current_dir().ok().and_then(|d| hase_core::find_repo_root(&d)));
    let Some(repo) = repo else {
        eprintln!("Repository-Wurzel nicht gefunden (Ordner content/testcandidate). Bitte die exe innerhalb des Repository-Klons starten.");
        std::process::exit(2);
    };
    if let Some(i) = args.iter().position(|a| a == "--export") {
        let out = args.get(i + 1).cloned().unwrap_or_else(|| "export.vrm".into());
        std::process::exit(offscreen::run_export(&repo, &out, &args));
    }
    if let Some(i) = args.iter().position(|a| a == "--screenshot") {
        let out = args.get(i + 1).cloned().unwrap_or_else(|| "screenshot.png".into());
        std::process::exit(offscreen::run(&repo, &out, &args));
    }
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1760.0, 990.0]).with_maximized(true).with_min_inner_size([900.0, 600.0]).with_title("HASE VRM MAKER"),
        multisampling: 4,
        depth_buffer: 32,
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    if let Err(e) = eframe::run_native("HASE VRM MAKER", native, Box::new(move |cc| { let mut a = app::App::new(cc, repo.clone()); a.open_on_start = args.iter().position(|x| x == "--open").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from); Ok(Box::new(a)) })) {
        eprintln!("Start fehlgeschlagen: {e}");
        std::process::exit(1);
    }
}
