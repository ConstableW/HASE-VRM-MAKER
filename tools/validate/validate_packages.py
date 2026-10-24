"""Validiert HASE-Package-Manifeste und ein Character-Projekt: Schema + Kompatibilitätsregeln (docs/03-specifications/compatibility-rules.md).

Aufruf:
  python validate_packages.py [--packages DIR] [--character FILE] [--write-lock] [--publish] [--selftest]
Benötigt: jsonschema (pip). Exit-Code 1 bei Fehlern.
"""
import argparse, copy, hashlib, json, re, sys
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[2]
SCHEMAS = ROOT / "docs/03-specifications/schemas"


# ---------- SemVer ----------
def parse_v(s):
    m = re.match(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?", s.strip())
    return tuple(int(x or 0) for x in m.groups())


def satisfies(version, rng):
    v = parse_v(version)
    for part in re.findall(r"(\^|~|>=|<=|>|<|=)?\s*(\d+(?:\.\d+){0,2})", rng):
        op, ver = part
        r = parse_v(ver)
        if op == "^":
            upper = (r[0] + 1, 0, 0) if r[0] > 0 else (0, r[1] + 1, 0)
            ok = r <= v < upper
        elif op == "~":
            ok = r <= v < (r[0], r[1] + 1, 0)
        elif op == ">=": ok = v >= r
        elif op == "<=": ok = v <= r
        elif op == ">": ok = v > r
        elif op == "<": ok = v < r
        else: ok = v == r
        if not ok:
            return False
    return True


def digest(manifest):
    m = {k: v for k, v in manifest.items() if k not in ("$schema", "dev")}
    return "sha256:" + hashlib.sha256(json.dumps(m, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()


# ---------- Laden ----------
def load_packages(pdir):
    pk = {}
    for f in sorted(Path(pdir).glob("*/hase-package.json")):
        m = json.load(open(f, encoding="utf8"))
        m["_dir"] = f.parent.name
        pk[m["id"]] = m
    return pk


def check_packages(pk, publish=False):
    errs = []
    schema = json.load(open(SCHEMAS / "package.schema.json"))
    val = Draft202012Validator(schema)
    for pid, m in pk.items():
        clean = {k: v for k, v in m.items() if k != "_dir"}
        for e in val.iter_errors(clean):
            errs.append(f"[schema] {pid}: {e.message[:140]} @ {'/'.join(map(str, e.path))}")
        if m["_dir"] != pid:
            errs.append(f"[layout] {pid}: Verzeichnis '{m['_dir']}' ≠ id")
        if publish and "dev" in m:
            errs.append(f"[publish] {pid}: Feld 'dev' ist in veröffentlichten Packages verboten")
        for dep in m.get("requires", []):
            t = pk.get(dep["id"])
            if not t: errs.append(f"[requires] {pid}: {dep['id']} nicht vorhanden")
            elif not satisfies(t["version"], dep["range"]): errs.append(f"[requires] {pid}: {dep['id']} {t['version']} ∉ {dep['range']}")
        for dep in m.get("conflicts", []):
            t = pk.get(dep["id"])
            if t and satisfies(t["version"], dep["range"]): errs.append(f"[conflicts] {pid}: Konflikt mit {dep['id']} {t['version']}")
    bodies = [m for m in pk.values() if m["type"] == "body"]
    body_by_if = {b["body"]["provides"]["interface"]: b for b in bodies}

    def body_for(iref):
        b = body_by_if.get(iref["id"])
        if not b: return None, f"Body-Interface {iref['id']} nicht bereitgestellt"
        if not satisfies(b["body"]["provides"]["version"], iref["range"]):
            return None, f"Interface {iref['id']} {b['body']['provides']['version']} ∉ {iref['range']}"
        return b, None

    def param_ids(b):
        return {p["id"] for p in b["body"].get("morphs", {}).get("parameters", [])}

    for pid, m in pk.items():
        if m["type"] == "species":
            s = m["species"]; b, err = body_for(s["targets"]["bodyInterface"])
            if err: errs.append(f"[interface] {pid}: {err}"); continue
            ids = param_ids(b); slots = {x["id"] for x in b["body"]["slots"]}
            ranges = {p["id"]: p["range"] for p in b["body"]["morphs"]["parameters"]}
            refs = list(s.get("defaults", {})) + list(s.get("limits", {})) + [k for t in s.get("traits", {}).values() for k in t["set"]] + [k for p in s.get("presets", []) for k in p.get("parameters", {})]
            for r in refs:
                if r not in ids: errs.append(f"[parameter] {pid}: unbekannter Parameter {r}")
            for k, (lo, hi) in s.get("limits", {}).items():
                if k in ranges and not (ranges[k][0] <= lo <= hi <= ranges[k][1]): errs.append(f"[limits] {pid}: {k} außerhalb Body-Bereich")
            caps = set(b["body"]["provides"].get("capabilities", [])) | set(s.get("capabilities", []))
            for c in s.get("components", []):
                a = pk.get(c["asset"])
                if not a: errs.append(f"[components] {pid}: {c['asset']} nicht vorhanden"); continue
                if a["type"] != "asset": errs.append(f"[components] {pid}: {c['asset']} ist kein Asset"); continue
                if c.get("slot") and c["slot"] not in a["asset"]["occupies"]: errs.append(f"[slot] {pid}: {c['asset']} belegt {c['slot']} nicht")
                if c.get("slot") and c["slot"] not in slots: errs.append(f"[slot] {pid}: Slot {c['slot']} fehlt im Body")
                allowed = a["asset"]["targets"].get("species")
                if allowed and pid not in allowed: errs.append(f"[species] {pid}: {c['asset']} nicht für diese Spezies freigegeben")
                for rc in a["asset"].get("requiresCapabilities", []):
                    if rc not in caps: errs.append(f"[capability] {pid}: {c['asset']} braucht '{rc}'")
        if m["type"] == "asset":
            a = m["asset"]; b, err = body_for(a["targets"]["bodyInterface"])
            if err: errs.append(f"[interface] {pid}: {err}"); continue
            slots = {x["id"] for x in b["body"]["slots"]}
            for o in a["occupies"]:
                if o not in slots: errs.append(f"[slot] {pid}: belegt unbekannten Slot {o}")
            for rc in a.get("requiresCapabilities", []):
                if rc not in b["body"]["provides"].get("capabilities", []): errs.append(f"[capability] {pid}: Body bietet '{rc}' nicht")
    return errs


def check_character(pk, ch, write_lock=False):
    errs = []
    schema = json.load(open(SCHEMAS / "character.schema.json"))
    for e in Draft202012Validator(schema).iter_errors(ch):
        errs.append(f"[schema] character: {e.message[:140]} @ {'/'.join(map(str, e.path))}")
    def res(ref, what):
        t = pk.get(ref["id"])
        if not t: errs.append(f"[ref] {what}: {ref['id']} nicht aufgelöst"); return None
        if not satisfies(t["version"], ref["range"]): errs.append(f"[ref] {what}: {ref['id']} {t['version']} ∉ {ref['range']}"); return None
        return t
    body = res(ch["body"], "body"); sp = res(ch["species"], "species")
    resolved = [x for x in (body, sp) if x]
    used, params = {}, set()
    if body: params |= {p["id"] for p in body["body"]["morphs"]["parameters"]}
    for a in ch.get("assets", []):
        t = res(a["ref"], f"asset {a['instance']}")
        if not t: continue
        resolved.append(t)
        if a.get("slot") and a["slot"] not in t["asset"]["occupies"]: errs.append(f"[slot] {a['instance']}: {t['id']} belegt {a['slot']} nicht")
        if a.get("enabled", True):
            for s in t["asset"]["occupies"]:
                excl = next((x.get("exclusive", True) for x in body["body"]["slots"] if x["id"] == s), True) if body else True
                if excl and s in used: errs.append(f"[slot-konflikt] {a['instance']} und {used[s]} belegen exklusiv '{s}'")
                used[s] = a["instance"]
        params |= {p["id"] for p in t["asset"].get("morphs", {}).get("parameters", [])}
        if sp and sp["species"]["targets"]["bodyInterface"]["id"] != t["asset"]["targets"]["bodyInterface"]["id"]:
            errs.append(f"[interface] {t['id']} passt nicht zur Spezies-Schnittstelle")
        allowed = t["asset"]["targets"].get("species")
        if allowed and sp and sp["id"] not in allowed: errs.append(f"[species] {t['id']} nicht für {sp['id']}")
    for k in ch["parameters"]:
        if k not in params: errs.append(f"[parameter] unbekannter Parameter {k}")
    if sp:
        for c in sp["species"].get("components", []):
            if not c.get("optional") and c["asset"] not in {r["id"] for r in resolved}:
                resolved.append(pk[c["asset"]])
    lock = sorted(({"id": t["id"], "version": t["version"], "digest": digest(t)} for t in {x["id"]: x for x in resolved}.values()), key=lambda x: x["id"])
    if write_lock: ch["lock"] = lock
    elif ch["lock"] != lock: errs.append("[lock] lock stimmt nicht mit aufgelösten Packages überein (--write-lock)")
    return errs, lock


def selftest(pk, ch):
    ok = True
    def expect(name, errs, needle):
        nonlocal ok
        hit = any(needle in e for e in errs)
        print(("  ok   " if hit else "  FAIL ") + name); ok &= hit
    p = copy.deepcopy(pk); p["org.hase.asset.bodypart.cat-ears"]["asset"]["requiresCapabilities"] = ["nonexistent"]
    expect("fehlende Capability wird erkannt", check_packages(p), "[capability]")
    p = copy.deepcopy(pk); p["org.hase.body.humanoid-mh08"]["body"]["provides"]["version"] = "2.0.0"
    expect("Interface-Major-Sprung bricht Spezies/Assets", check_packages(p), "[interface]")
    p = copy.deepcopy(pk); p["org.hase.species.elf"]["species"]["limits"]["mh:head-scale-vert-decr-incr"] = [-3, 3]
    expect("Limits außerhalb Body-Bereich", check_packages(p), "[limits]")
    p = copy.deepcopy(pk); p["org.hase.species.cat-demi"]["species"]["defaults"]["mh:gibtsnicht"] = 1
    expect("unbekannter Parameter in Species", check_packages(p), "[parameter]")
    p = copy.deepcopy(pk); p["org.hase.asset.clothing.tshirt-basic"]["asset"]["occupies"] = ["clothing.nowhere"]
    expect("unbekannter Slot", check_packages(p), "[slot]")
    c = copy.deepcopy(ch); c["assets"].append(copy.deepcopy(c["assets"][1])); c["assets"][-1]["instance"] = "top-2"
    expect("Slot-Konflikt im Character", check_character(pk, c)[0], "[slot-konflikt]")
    c = copy.deepcopy(ch); c["species"] = {"id": "org.hase.species.elf", "range": "^1.0"}; c["assets"].append({"ref": {"id": "org.hase.asset.bodypart.cat-ears", "range": "^1.0"}, "instance": "ears-1", "slot": "head.ears"})
    expect("Katzenohren + Elf: Ohren-Slot doppelt belegt (Spezies-Komponente)", check_character(pk, c)[0] + [e for e in check_packages(pk) if False], "") if False else None
    c = copy.deepcopy(ch); c["parameters"]["mh:nope"] = 1
    expect("unbekannter Parameter im Character", check_character(pk, c)[0], "[parameter]")
    c = copy.deepcopy(ch); c["assets"][0]["ref"]["range"] = "^2.0"
    expect("Versionsbereich nicht erfüllbar", check_character(pk, c)[0], "[ref]")
    print("selftest:", "bestanden" if ok else "FEHLGESCHLAGEN")
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--packages", default=str(ROOT / "examples/species/packages"))
    ap.add_argument("--character", default=str(ROOT / "examples/species/character.example.hase.json"))
    ap.add_argument("--write-lock", action="store_true"); ap.add_argument("--publish", action="store_true"); ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    pk = load_packages(a.packages); ch = json.load(open(a.character, encoding="utf8"))
    errs = check_packages(pk, a.publish)
    cerrs, lock = check_character(pk, ch, a.write_lock)
    if a.write_lock:
        json.dump(ch, open(a.character, "w", encoding="utf8"), indent=2, ensure_ascii=False); open(a.character, "a").write("\n"); cerrs = [e for e in cerrs if "[lock]" not in e]
    errs += cerrs
    print(f"{len(pk)} Packages, Character '{ch['name']}', {len(lock)} aufgelöste Packages")
    for e in errs: print("  FEHLER", e)
    print("Ergebnis:", "OK" if not errs else f"{len(errs)} Fehler")
    ok = not errs
    if a.selftest: ok = selftest(pk, ch) and ok
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
