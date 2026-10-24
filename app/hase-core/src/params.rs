//! Parameter-Katalog: aus der Target-Struktur (target.json) + Makro-Parameter. Parameter sind namespaced (`mh:`).
use crate::body::Morphs;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Signed, Unit, Macro }

#[derive(Clone, Debug)]
pub struct ParamDef {
    pub id: String,
    pub label: String,
    pub group: String,
    pub kind: Kind,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub neg: Option<usize>,
    pub pos: Option<usize>,
    pub mirror: Option<usize>, // Index des gespiegelten Parameters (links/rechts)
}

pub struct Catalog {
    pub defs: Vec<ParamDef>,
    pub index: HashMap<String, usize>,
    pub macro_targets: Vec<(usize, [usize; 3])>, // (Target-Index, [race, gender, age]-Kombination) – intern
    macro_idx: Option<[usize; 5]>,
}

const RACES: [&str; 3] = ["african", "asian", "caucasian"];
const GENDERS: [&str; 2] = ["female", "male"];
const AGES: [&str; 4] = ["baby", "child", "young", "old"];

pub fn macro_age_weights(a: f32) -> [f32; 4] {
    let a = a.clamp(0.0, 1.0);
    if a < 0.1875 { let t = a / 0.1875; [1.0 - t, t, 0.0, 0.0] }
    else if a < 0.5 { let t = (a - 0.1875) / 0.3125; [0.0, 1.0 - t, t, 0.0] }
    else { let t = (a - 0.5) / 0.5; [0.0, 0.0, 1.0 - t, t] }
}

fn humanize(name: &str) -> String { name.replace("-decr-incr", "").replace('-', " ") }

impl Catalog {
    pub fn build(morphs: &Morphs, target_json: &Path) -> Catalog {
        let mut defs: Vec<ParamDef> = Vec::new();
        let mk = |id: &str, label: &str, d: f32| ParamDef { id: id.into(), label: label.into(), group: "Makro".into(), kind: Kind::Macro, min: 0.0, max: 1.0, default: d, neg: None, pos: None, mirror: None };
        defs.push(mk("mh:macro-gender", "Geschlecht (0 weiblich … 1 männlich)", 0.5));
        defs.push(mk("mh:macro-age", "Alter", 0.5));
        defs.push(mk("mh:macro-race-african", "Anteil Typ afrikanisch", 1.0 / 3.0));
        defs.push(mk("mh:macro-race-asian", "Anteil Typ asiatisch", 1.0 / 3.0));
        defs.push(mk("mh:macro-race-caucasian", "Anteil Typ kaukasisch", 1.0 / 3.0));
        let macro_idx = [0, 1, 2, 3, 4];
        let mut macro_targets = Vec::new();
        for (ri, r) in RACES.iter().enumerate() { for (gi, g) in GENDERS.iter().enumerate() { for (ai, a) in AGES.iter().enumerate() {
            if let Some(t) = morphs.index_of(&format!("macrodetails/{r}-{g}-{a}")) { macro_targets.push((t, [ri, gi, ai])); }
        } } }
        if let Ok(f) = std::fs::File::open(target_json) {
            if let Ok(v) = serde_json::from_reader::<_, serde_json::Value>(f) {
                if let Some(obj) = v.as_object() {
                    let mut groups: Vec<&String> = obj.keys().collect();
                    groups.sort();
                    for g in groups {
                        if g == "genitals" || g == "macrodetails" { continue; }
                        let cats = match obj[g].get("categories").and_then(|c| c.as_array()) { Some(c) => c, None => continue };
                        for c in cats {
                            let name = c["name"].as_str().unwrap_or("");
                            let opp = &c["opposites"];
                            let s = |k: &str| opp[k].as_str().unwrap_or("");
                            let find = |t: &str| if t.is_empty() { None } else { morphs.index_of(&format!("{g}/{t}")) };
                            let sided = c["has_left_and_right"].as_bool().unwrap_or(false);
                            let variants: Vec<(String, &str, &str, &str)> = if sided {
                                vec![(format!("mh:l-{name}"), "l", s("negative-left"), s("positive-left")), (format!("mh:r-{name}"), "r", s("negative-right"), s("positive-right"))]
                            } else { vec![(format!("mh:{name}"), "", s("negative-unsided"), s("positive-unsided"))] };
                            let first = defs.len();
                            for (id, side, n, p) in variants {
                                let (neg, pos) = (find(n), find(p));
                                if neg.is_none() && pos.is_none() { continue; }
                                let kind = if neg.is_some() && pos.is_some() { Kind::Signed } else { Kind::Unit };
                                let side_label = match side { "l" => " (links)", "r" => " (rechts)", _ => "" };
                                // Nur negatives Target → Wert 0..1 auf negativem Target (Vorzeichen intern umgedreht)
                                let (neg2, pos2) = if kind == Kind::Unit && pos.is_none() { (None, neg) } else { (neg, pos) };
                                defs.push(ParamDef { id, label: format!("{}{}", humanize(name), side_label), group: g.clone(), kind, min: if kind == Kind::Signed { -1.0 } else { 0.0 }, max: 1.0, default: 0.0, neg: neg2, pos: pos2, mirror: None });
                            }
                            if sided && defs.len() == first + 2 { let (a, b) = (first, first + 1); defs[a].mirror = Some(b); defs[b].mirror = Some(a); }
                        }
                    }
                }
            }
        }
        // HASE-Anatomiemodell (Brust): signierte Parameter je Seite, links/rechts gekoppelt
        for (grp, p) in crate::anatomy::BREAST_PARAMS.iter().map(|p| ("breast", p)).chain(crate::anatomy::GLUTE_PARAMS.iter().map(|p| ("buttocks", p))) {
            let first = defs.len();
            for (side, label) in [("l", "links"), ("r", "rechts")] {
                let (pos, neg) = (morphs.index_of(&format!("hase/{}-{side}-pos", p.stem)), morphs.index_of(&format!("hase/{}-{side}-neg", p.stem)));
                if pos.is_none() { continue; }
                defs.push(ParamDef { id: format!("hase:{}-{side}", p.stem), label: format!("{} ({label})", p.label), group: grp.into(), kind: Kind::Signed, min: -1.0, max: 1.0, default: 0.0, neg, pos, mirror: None });
            }
            if defs.len() == first + 2 { defs[first].mirror = Some(first + 1); defs[first + 1].mirror = Some(first); }
        }
        let index = defs.iter().enumerate().map(|(i, d)| (d.id.clone(), i)).collect();
        Catalog { defs, index, macro_targets, macro_idx: Some(macro_idx) }
    }

    /// Katalog für importierte Körperbasen: ein Parameter (0…1) je Morph-Target, Gruppe = zugeordnete Region.
    pub fn from_morphs(names: &[String], region_of: &std::collections::BTreeMap<String, String>, mirror: &[(usize, usize)]) -> Catalog {
        let mut defs: Vec<ParamDef> = names.iter().enumerate().map(|(t, n)| ParamDef { id: format!("morph:{n}"), label: n.replace(['_', '.'], " "), group: region_of.get(n).cloned().unwrap_or_else(|| "Unzugeordnet".into()), kind: Kind::Unit, min: 0.0, max: 1.0, default: 0.0, neg: None, pos: Some(t), mirror: None }).collect();
        for &(a, b) in mirror { if a < defs.len() && b < defs.len() { defs[a].mirror = Some(b); defs[b].mirror = Some(a); } }
        let index = defs.iter().enumerate().map(|(i, d)| (d.id.clone(), i)).collect();
        Catalog { defs, index, macro_targets: vec![], macro_idx: None }
    }

    /// Löst eine Parameter-ID auf (auch ältere Kurzformen der Beispiel-Packages → kanonische Katalog-ID).
    pub fn canonical(&self, id: &str) -> Option<usize> {
        if let Some(&i) = self.index.get(id) { return Some(i); }
        const ALIAS: &[(&str, &str)] = &[("mh:jaw-drop", "mh:chin-jaw-drop-decr-incr")];
        if let Some((_, to)) = ALIAS.iter().find(|(k, _)| *k == id) { return self.index.get(*to).copied(); }
        for suf in ["-decr-incr", "-down-up", "-in-out", "-backward-forward", "-square-round"] { if let Some(&i) = self.index.get(&format!("{id}{suf}")) { return Some(i); } }
        None
    }

    pub fn defaults(&self) -> Vec<f32> { self.defs.iter().map(|d| d.default).collect() }

    /// Wertevektor (aligned zu `defs`) -> Zielgewichte je Target.
    pub fn target_weights(&self, values: &[f32], nt: usize, out: &mut Vec<f32>) {
        out.clear();
        out.resize(nt, 0.0);
        for (d, &v) in self.defs.iter().zip(values) {
            match d.kind {
                Kind::Signed => { if let Some(t) = d.neg { out[t] += (-v).max(0.0); } if let Some(t) = d.pos { out[t] += v.max(0.0); } }
                Kind::Unit => { if let Some(t) = d.pos { out[t] += v.max(0.0); } }
                Kind::Macro => {}
            }
        }
        let Some(mi) = self.macro_idx else { return };
        let [g, a] = [values[mi[0]], values[mi[1]]];
        let races = [values[mi[2]], values[mi[3]], values[mi[4]]];
        let rs: f32 = races.iter().sum::<f32>().max(1e-6);
        let gw = [1.0 - g.clamp(0.0, 1.0), g.clamp(0.0, 1.0)];
        let aw = macro_age_weights(a);
        for &(t, [ri, gi, ai]) in &self.macro_targets { out[t] += races[ri] / rs * gw[gi] * aw[ai]; }
    }
}
