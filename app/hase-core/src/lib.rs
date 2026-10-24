//! hase-core: UI- und GPU-freier Kern von HASE VRM MAKER (Daten, Parameter, Morph-Evaluator, Fitting, Packages, Projektzustand).
//! Architekturregeln: docs/02-architecture/performance-architecture.md (Snapshot-Zustand, inkrementelle Neuberechnung, Sparse-Morphs, Binär-Cache).
pub mod assets;
pub mod body;
pub mod eval;
pub mod fit;
pub mod obj;
pub mod packages;
pub mod params;
pub mod state;

use std::path::{Path, PathBuf};

/// Sucht die Repository-Wurzel (Ordner mit `content/testcandidate`) ab `start` aufwärts.
pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut p = start.to_path_buf();
    loop {
        if p.join("content/testcandidate/mpfb2-cc0/base.obj").exists() { return Some(p); }
        if !p.pop() { return None; }
    }
}
