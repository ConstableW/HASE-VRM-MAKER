"""Zweite Test-Körperbasis (andere Topologie als MakeHuman): Quaternius Universal Base Character (CC0) + **synthetische** Shape Keys.

Testfixture für die Speziesunabhängigkeit des Importwegs (nicht als Produktinhalt gedacht): Die Shape Keys sind einfache, regionsbasierte
Verformungen (Kopf, Schultern, Torso, Hüfte, Arme, Beine, Gesamtgröße) auf Basis der Mesh-Proportionen; links/rechts getrennt benannt.
Entwicklungswerkzeug, benötigt PyPI `bpy`. Aufruf: python build_test_body_b.py <repo-root>
Ergebnis: content/testcandidate/body_b_quaternius_synthetic.glb (nicht versioniert, *.glb ist ignoriert).
"""
import sys
from pathlib import Path
import bpy

root = Path(sys.argv[1])
src = root / "content/testcandidate/downloads/Universal Base Characters[Standard]/Base Characters/Godot - UE/Superhero_Female_FullBody.gltf"
out = root / "content/testcandidate/body_b_quaternius_synthetic.glb"
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(src))
meshes = [o for o in bpy.data.objects if o.type == "MESH"]
print("Meshes:", [(o.name, len(o.data.vertices)) for o in meshes])
body = max(meshes, key=lambda o: len(o.data.vertices))
for o in meshes:
    if o is not body:
        bpy.data.objects.remove(o)
me = body.data
vs = [v.co.copy() for v in me.vertices]
zs = [v.z for v in vs]; xs = [v.x for v in vs]
zmin, zmax = min(zs), max(zs); H = zmax - zmin
cx = (min(xs) + max(xs)) / 2
print("Höhe", H, "Vertices", len(vs))

def sstep(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)

def frac(v): return (v.z - zmin) / H
def side(v): return v.x - cx  # Blender: +x = Charakter links (glTF -x/+x je nach Import) – nur für Benennung

if me.shape_keys is None:
    body.shape_key_add(name="Basis", from_mix=False)

def key(name, fn):
    k = body.shape_key_add(name=name, from_mix=False)
    for i, v in enumerate(vs):
        k.data[i].co = fn(i, v)

def vec(x, y, z):
    from mathutils import Vector
    return Vector((x, y, z))

head_c = vec(cx, 0, zmin + H * 0.93)
key("height_taller", lambda i, v: vec(v.x, v.y, zmin + (v.z - zmin) * 1.12))
key("head_larger", lambda i, v: head_c + (v - head_c) * (1 + 0.25 * sstep(0.82, 0.86, frac(v))) if frac(v) > 0.8 else v)
key("head_wider", lambda i, v: vec(cx + (v.x - cx) * (1 + 0.25 * sstep(0.8, 0.88, frac(v))), v.y, v.z))
key("neck_longer", lambda i, v: vec(v.x, v.y, v.z + 0.06 * H * sstep(0.78, 0.84, frac(v))))
key("shoulders_wider", lambda i, v: vec(cx + (v.x - cx) * (1 + 0.3 * sstep(0.66, 0.74, frac(v)) * (1 - sstep(0.80, 0.86, frac(v)))), v.y, v.z))
key("chest_deeper", lambda i, v: vec(v.x, v.y - 0.05 * H * sstep(0.62, 0.7, frac(v)) * (1 - sstep(0.74, 0.8, frac(v))) * (1 if v.y < 0 else 0.3), v.z))
key("belly_fuller", lambda i, v: vec(v.x, v.y - 0.06 * H * sstep(0.5, 0.56, frac(v)) * (1 - sstep(0.6, 0.64, frac(v))) * (1 if v.y < 0 else 0), v.z))
key("hips_wider", lambda i, v: vec(cx + (v.x - cx) * (1 + 0.25 * sstep(0.42, 0.48, frac(v)) * (1 - sstep(0.52, 0.58, frac(v)))), v.y, v.z))
key("legs_longer", lambda i, v: vec(v.x, v.y, zmin + (v.z - zmin) * 1.15 if frac(v) < 0.47 else v.z + (0.47 * H * 0.15)))
for nm, sgn in (("l", 1), ("r", -1)):
    def arm_len(i, v, sgn=sgn):
        a = abs(v.x - cx) / H
        on = (v.x - cx) * sgn > 0 and a > 0.17 and 0.55 < frac(v) < 0.8
        return vec(v.x + sgn * 0.12 * H * sstep(0.17, 0.4, a), v.y, v.z) if on else v
    key(f"arm_longer_{nm}", arm_len)
    def arm_thick(i, v, sgn=sgn):
        a = abs(v.x - cx) / H
        on = (v.x - cx) * sgn > 0 and a > 0.15 and 0.55 < frac(v) < 0.8
        if not on: return v
        c = vec(v.x, 0, zmin + H * 0.72)
        d = vec(0, v.y, v.z - c.z)
        return vec(v.x, v.y * 1.3, c.z + d.z * 1.3)
    key(f"arm_thicker_{nm}", arm_thick)
    def thigh(i, v, sgn=sgn):
        on = (v.x - cx) * sgn > 0 and 0.28 < frac(v) < 0.48
        if not on: return v
        c = cx + sgn * 0.08 * H
        return vec(c + (v.x - c) * 1.3, v.y * 1.3, v.z)
    key(f"thigh_thicker_{nm}", thigh)
print("Shape Keys:", [k.name for k in me.shape_keys.key_blocks])
for o in bpy.data.objects: o.select_set(True)
bpy.ops.export_scene.gltf(filepath=str(out), export_format="GLB", export_morph=True, export_skins=True, export_apply=False)
print("OK", out)
