"""Baut den Testcandidate-Body aus den MPFB2-CC0-Daten (content/testcandidate/mpfb2-cc0).

Entwicklungswerkzeug, kein Produktbestandteil. Benötigt das PyPI-Paket `bpy` (GPL, nur lokal verwendet).
Aufruf:  python build_testcandidate_body.py <repo-root>
Ausgabe: content/testcandidate/body_mpfb2_testcandidate.glb (Mesh + Shape Keys)
"""
import gzip, json, sys
from pathlib import Path
import bpy, bmesh

root = Path(sys.argv[1])
src = root / "content/testcandidate/mpfb2-cc0"
out = root / "content/testcandidate/body_mpfb2_testcandidate.glb"
GROUPS = ["macrodetails", "head", "forehead", "eyebrows", "eyes", "nose", "mouth", "chin", "cheek", "ears",
          "neck", "torso", "arms", "hands", "hip", "legs", "feet", "stomach", "pelvis", "breast", "buttocks"]

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.wm.obj_import(filepath=str(src / "base.obj"), use_split_groups=False, use_split_objects=False)
obj = bpy.context.selected_objects[0]
obj.name = "HASE_TestBody"
me = obj.data
nv = len(me.vertices)
print("vertices", nv)

# Vertexgruppen aus OBJ-Gruppen sind nicht garantiert; Hilfsgeometrie wird über base.obj-Gruppen bestimmt.
helper_vertices, current, in_body = set(), None, None
with open(src / "base.obj") as f:
    for line in f:
        if line.startswith("g "):
            current = line[2:].strip()
        elif line.startswith("f ") and current != "body":
            for tok in line.split()[1:]:
                helper_vertices.add(int(tok.split("/")[0]) - 1)
body_vertices = set()
current = None
with open(src / "base.obj") as f:
    for line in f:
        if line.startswith("g "):
            current = line[2:].strip()
        elif line.startswith("f ") and current == "body":
            for tok in line.split()[1:]:
                body_vertices.add(int(tok.split("/")[0]) - 1)
helper_only = helper_vertices - body_vertices
print("helper-only vertices", len(helper_only))

obj.shape_key_add(name="Basis", from_mix=False)
count = 0
for g in GROUPS:
    for p in sorted((src / "targets" / g).glob("*.target.gz")):
        key = obj.shape_key_add(name=f"{g}__{p.name[:-len('.target.gz')]}", from_mix=False)
        key.value = 0.0
        with gzip.open(p, "rt") as f:
            for line in f:
                if line.startswith("#") or not line.strip():
                    continue
                i, x, y, z = line.split()[:4]
                i = int(i)
                if i < nv:
                    d = key.data[i].co
                    d.x += float(x); d.y += float(y); d.z += float(z)
        count += 1
print("shape keys", count)

# Hilfsgeometrie entfernen (Shape Keys folgen automatisch)
bpy.context.view_layer.objects.active = obj
bpy.ops.object.mode_set(mode="EDIT")
bm = bmesh.from_edit_mesh(me)
bm.verts.ensure_lookup_table()
for v in bm.verts:
    v.select = v.index in helper_only
bmesh.update_edit_mesh(me)
bpy.ops.mesh.delete(type="VERT")
bpy.ops.object.mode_set(mode="OBJECT")

# MakeHuman-Einheiten (dm) -> Meter, Y-up/Z-up wie Blender-OBJ-Import belassen
obj.scale = (0.1, 0.1, 0.1)
bpy.ops.object.transform_apply(scale=True)
print("final vertices", len(obj.data.vertices), "polys", len(obj.data.polygons), "height m", obj.dimensions.z)

bpy.ops.export_scene.gltf(filepath=str(out), export_format="GLB", export_morph=True, export_morph_normal=False, export_morph_tangent=False, export_try_sparse_sk=True, export_apply=False)
print("written", out, out.stat().st_size)
