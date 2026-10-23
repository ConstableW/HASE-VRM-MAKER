"""Test-Avatar B: MakeHuman-/MPFB2-CC0-Basis + CC0-Proxys (Augen, Haar, Shirt, Hose) -> VRM 1.0.

Pipeline (alles in MakeHuman-Einheiten dm, Umrechnung nach Metern am Ende):
  1. Basismesh (hm08) laden, Macro-/Detail-Targets additiv anwenden
  2. Skelett (MPFB2 game_engine-Rig) aus den Gelenk-Würfeln des *gemorphten* Körpers ableiten
  3. Skin-Gewichte aus weights.game_engine.json
  4. .mhclo-Proxys per Barycentric-Bindung an den gemorphten Körper anpassen, Gewichte vom Körper übernehmen
  5. VRM-1.0-Humanoid-Mapping (UE-Mannequin-Namen) + Export

Entwicklungswerkzeug (bpy + bpy-vrm-format lokal). Nur Daten aus MPFB2/MakeHuman (CC0) werden verwendet, kein MPFB2-Code.
Aufruf: python build_makehuman_vrm.py <repo-root>
"""
import gzip, json, sys
from pathlib import Path
import numpy as np
import bpy
import io_scene_vrm

sys.path.insert(0, str(Path(__file__).parent))
from ue_vrm_map import apply_mapping

root = Path(sys.argv[1])
cand = root / "content/testcandidate"
mh = cand / "mpfb2-cc0"
dl = cand / "downloads"
out = cand / "avatar_b_makehuman.vrm"

MACRO = [("macrodetails/caucasian-female-young", 1.0)]
DETAIL = []  # (gruppe/name, gewicht) – hier später Gesichts-/Körperanpassungen
PROXIES = [  # (Ordner, mhclo-Name, Slot)
    ("eyes/high-poly", "high-poly", "Eyes"),
    ("eyebrows/mindfront_eyebrows_01", "mindfront_eyebrows_01", "Eyebrows"),
    ("eyelashes/mindfront_eyelashes_01", "mindfront_eyelashes_01", "Eyelashes"),
    ("hair/bob01", "bob01", "Hair"),
    ("clothes/toigo_basic_tucked_t-shirt", "toigo_basic_tucked_t-shirt", "Shirt"),
    ("clothes/toigo_wool_pants", "toigo_wool_pants", "Pants"),
]
SKIN = dl / "skins/young_caucasian_female/young_lightskinned_female_diffuse.png"
EYE_TEX = dl / "eyes/materials/brown_eye.png"


# ---------- OBJ / mhclo / mhmat ----------
def parse_obj(path):
    v, vt, faces, fgroup, cur = [], [], [], [], "default"
    with open(path, encoding="utf8", errors="ignore") as f:
        for line in f:
            if line.startswith("v "):
                v.append([float(x) for x in line.split()[1:4]])
            elif line.startswith("vt "):
                vt.append([float(x) for x in line.split()[1:3]])
            elif line.startswith(("g ", "o ")):
                cur = line[2:].strip()
            elif line.startswith("f "):
                faces.append([tuple(int(i) - 1 if i else -1 for i in (t.split("/") + ["", ""])[:2]) for t in line.split()[1:]])
                fgroup.append(cur)
    return np.array(v), np.array(vt), faces, fgroup


def parse_mhclo(path):
    d = {"scale": {}, "verts": []}
    mode = None
    for line in open(path, encoding="utf8", errors="ignore"):
        t = line.split()
        if not t or t[0].startswith("#"):
            continue
        if t[0] == "verts":
            mode = "verts"
        elif t[0] in ("delete_verts", "weights", "subdivide", "tag", "z_depth", "max_pole", "uuid", "basemesh", "name"):
            if t[0] == "delete_verts":
                mode = None
        elif t[0] in ("x_scale", "y_scale", "z_scale"):
            d["scale"][t[0][0]] = (int(t[1]), int(t[2]), float(t[3]))
            mode = None
        elif t[0] in ("obj_file", "material"):
            d[t[0]] = " ".join(t[1:])
        elif mode == "verts" and len(t) == 9:
            d["verts"].append([float(x) if i >= 3 else int(x) for i, x in enumerate(t)])
    return d


def parse_mhmat(path):
    d = {}
    for line in open(path, encoding="utf8", errors="ignore"):
        t = line.split()
        if len(t) >= 2 and not t[0].startswith(("#", "//")):
            d[t[0]] = " ".join(t[1:])
    return d


def make_object(name, verts, faces, uvs=None):
    me = bpy.data.meshes.new(name)
    polys = [[v for v, _ in f] for f in faces]
    me.from_pydata(verts.tolist(), [], polys)
    if uvs is not None and len(uvs):
        layer = me.uv_layers.new(name="UVMap")
        i = 0
        for f in faces:
            for _, t in f:
                layer.data[i].uv = uvs[t] if t >= 0 else (0, 0)
                i += 1
    me.update()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def material(name, tex=None, alpha=False, color=(0.8, 0.8, 0.8, 1.0)):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Roughness"].default_value = 0.8
    if tex and Path(tex).exists():
        img = bpy.data.images.load(str(tex))
        tn = m.node_tree.nodes.new("ShaderNodeTexImage")
        tn.image = img
        m.node_tree.links.new(tn.outputs["Color"], bsdf.inputs["Base Color"])
        if alpha:
            m.node_tree.links.new(tn.outputs["Alpha"], bsdf.inputs["Alpha"])
    if alpha:
        m.blend_method = "HASHED" if hasattr(m, "blend_method") else None
    return m


# ---------- 1. Basis + Targets ----------
base_v, base_vt, base_f, base_g = parse_obj(mh / "base.obj")
nv = len(base_v)
for key, w in MACRO + DETAIL:
    with gzip.open(mh / "targets" / f"{key}.target.gz", "rt") as f:
        for line in f:
            if line.startswith("#") or not line.strip():
                continue
            i, x, y, z = line.split()[:4]
            if int(i) < nv:
                base_v[int(i)] += w * np.array([float(x), float(y), float(z)])

groups = {}
for f, g in zip(base_f, base_g):
    groups.setdefault(g, set()).update(v for v, _ in f)
body_set = groups["body"]
print("Basis:", nv, "Vertices, Körper:", len(body_set))

# ---------- 2. Skelett ----------
rig = json.load(open(mh / "rigs/rig.game_engine.json"))


def joint_pos(spec):
    if spec["strategy"] == "CUBE":
        return base_v[sorted(groups[spec["cube_name"]])].mean(axis=0)
    if spec["strategy"] == "MEAN":
        return base_v[spec["vertex_indices"]].mean(axis=0)
    raise ValueError(spec)


def to_blender(p):  # MakeHuman (x, y-up, z-forward) -> Blender (x, -z, y), dm -> m
    p = np.atleast_2d(p)
    return np.stack([p[:, 0], -p[:, 2], p[:, 1]], axis=1) * 0.1


bpy.ops.wm.read_factory_settings(use_empty=True)
io_scene_vrm.register()
if "io_scene_vrm" not in bpy.context.preferences.addons:
    bpy.context.preferences.addons.new().module = "io_scene_vrm"

arm_data = bpy.data.armatures.new("Armature")
arm = bpy.data.objects.new("Armature", arm_data)
bpy.context.scene.collection.objects.link(arm)
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="EDIT")
eb = {}
for name, b in rig.items():
    e = arm_data.edit_bones.new(name)
    e.head = to_blender(joint_pos(b["head"]))[0]
    e.tail = to_blender(joint_pos(b["tail"]))[0]
    if (e.tail - e.head).length < 1e-4:
        e.tail = e.head + type(e.head)((0, 0, 0.01))
    e.roll = b.get("roll", 0.0)
    eb[name] = e
for name, b in rig.items():
    if b.get("parent"):
        eb[name].parent = eb[b["parent"]]
bpy.ops.object.mode_set(mode="OBJECT")

# ---------- 3. Körper + Gewichte ----------
weights = json.load(open(mh / "rigs/weights.game_engine.json"))["weights"]
vert_w = [dict() for _ in range(nv)]
for bone, lst in weights.items():
    for vi, w in lst:
        vert_w[vi][bone] = w

body_ob = make_object("Body", to_blender(base_v), base_f, base_vt)
body_ob.data.materials.append(material("Skin", SKIN))


def add_weights(ob, per_vertex_weights):
    names = sorted({b for d in per_vertex_weights for b in d})
    vg = {n: ob.vertex_groups.new(name=n) for n in names}
    for i, d in enumerate(per_vertex_weights):
        tot = sum(d.values()) or 1.0
        for b, w in d.items():
            if w > 1e-4:
                vg[b].add([i], w / tot, "REPLACE")
    mod = ob.modifiers.new("Armature", "ARMATURE")
    mod.object = arm
    ob.parent = arm


add_weights(body_ob, vert_w)

# Hilfsgeometrie (Tights, Joint-Würfel, Wimpern-Helper) entfernen
import bmesh  # noqa: E402

bm = bmesh.new()
bm.from_mesh(body_ob.data)
bm.verts.ensure_lookup_table()
bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.index not in body_set], context="VERTS")
bm.to_mesh(body_ob.data)
bm.free()

# ---------- 4. Proxys ----------
body_idx = np.array(sorted(body_set))
for folder, stem, slot in PROXIES:
    d = parse_mhclo(dl / folder / f"{stem}.mhclo")
    pv, pvt, pf, _ = parse_obj(dl / folder / d["obj_file"])
    sc = {}
    for ax, i in (("x", 0), ("y", 1), ("z", 2)):
        v1, v2, den = d["scale"][ax]
        sc[i] = abs(base_v[v1][i] - base_v[v2][i]) / den
    fit = np.zeros((len(d["verts"]), 3))
    pw = []
    for k, row in enumerate(d["verts"]):
        a, b, c, w0, w1, w2, o0, o1, o2 = row
        fit[k] = w0 * base_v[a] + w1 * base_v[b] + w2 * base_v[c] + np.array([o0 * sc[0], o1 * sc[1], o2 * sc[2]])
        acc = {}
        for vi, w in ((a, w0), (b, w1), (c, w2)):
            for bone, bw in vert_w[vi].items():
                acc[bone] = acc.get(bone, 0.0) + w * bw
        if not acc:  # Bindung an Hilfs-Vertices ohne Gewichte -> nächster Körper-Vertex
            near = body_idx[np.argmin(((base_v[body_idx] - fit[k]) ** 2).sum(axis=1))]
            acc = dict(vert_w[near])
        pw.append(acc)
    assert len(fit) == len(pv), (slot, len(fit), len(pv))
    ob = make_object(slot, to_blender(fit), pf, pvt)
    mat_info = parse_mhmat(dl / folder / d["material"]) if d.get("material") else {}
    tex = None
    if slot == "Eyes":
        tex = EYE_TEX
    elif "diffuseTexture" in mat_info:
        tex = (dl / folder / d["material"]).parent / mat_info["diffuseTexture"]
    alpha = mat_info.get("transparent", "False").lower() == "true"
    ob.data.materials.append(material(slot, tex, alpha=alpha, color=(0.9, 0.9, 0.9, 1.0) if tex else (0.8, 0.8, 0.8, 1.0)))
    add_weights(ob, pw)
    print(slot, "Vertices", len(pv))

# ---------- 5. VRM ----------
bpy.context.view_layer.objects.active = arm
print("fehlende Bones:", apply_mapping(arm))
ext = arm.data.vrm_addon_extension
m = ext.vrm1.meta
m.vrm_name = "HASE Testavatar B (MakeHuman CC0)"
m.authors.add().value = "MakeHuman community assets (CC0); Zusammenstellung HASE VRM MAKER"
m.license_url = "https://vrm.dev/licenses/1.0/"
m.avatar_permission = "everyone"
m.commercial_usage = "personalProfit"
m.credit_notation = "unnecessary"
m.modification = "allowModificationRedistribution"
m.allow_redistribution = True
m.copyright_information = "Basismesh, Targets, Rig, Haare, Augen, Kleidung: MakeHuman/MPFB2-Community-Assets, CC0 1.0"

bpy.ops.object.select_all(action="DESELECT")
arm.select_set(True)
print("export:", bpy.ops.export_scene.vrm(filepath=str(out), armature_object_name=arm.name), out.stat().st_size if out.exists() else "fehlt")
