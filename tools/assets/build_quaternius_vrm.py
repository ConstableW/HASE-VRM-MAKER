"""Test-Avatar A: Quaternius Universal Base Character (CC0) + Haare -> VRM 1.0.

Entwicklungswerkzeug. Benötigt PyPI-Pakete `bpy` (GPL) und `bpy-vrm-format` (MIT OR GPL-3.0-or-later), nur lokal verwendet.
Aufruf: python build_quaternius_vrm.py <repo-root> [Female|Male] [Hair_Buns]
"""
import sys
from pathlib import Path
import bpy
import io_scene_vrm

root = Path(sys.argv[1])
sex = sys.argv[2] if len(sys.argv) > 2 else "Female"
hair = sys.argv[3] if len(sys.argv) > 3 else "Hair_Buns"
base = root / "content/testcandidate/downloads/Universal Base Characters[Standard]"
out = root / f"content/testcandidate/avatar_a_quaternius_{sex.lower()}.vrm"

bpy.ops.wm.read_factory_settings(use_empty=True)
io_scene_vrm.register()
if "io_scene_vrm" not in bpy.context.preferences.addons:
    bpy.context.preferences.addons.new().module = "io_scene_vrm"

bpy.ops.import_scene.gltf(filepath=str(base / f"Base Characters/Godot - UE/Superhero_{sex}_FullBody.gltf"))
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
before = set(bpy.data.objects)
bpy.ops.import_scene.gltf(filepath=str(base / f"Hairstyles/Rigged to Head Bone/glTF (Godot -Unreal)/{hair}.gltf"))
new = set(bpy.data.objects) - before
print("Haar-Objekte:", [(o.type, o.name) for o in new])
hair_arm = [o for o in new if o.type == "ARMATURE"]
for o in new:
    if o.type == "MESH":
        o.parent = arm
        for m in o.modifiers:
            if m.type == "ARMATURE":
                m.object = arm
for a in hair_arm:
    bpy.data.objects.remove(a)
for o in list(bpy.data.objects):
    if o.name.startswith("Icosphere"):
        bpy.data.objects.remove(o)

# --- Humanoid-Mapping UE-Mannequin-Skeleton -> VRM 1.0 ---
MAP = {
    "hips": "pelvis", "spine": "spine_01", "chest": "spine_02", "upperChest": "spine_03",
    "neck": "neck_01", "head": "Head",
    "leftShoulder": "clavicle_l", "leftUpperArm": "upperarm_l", "leftLowerArm": "lowerarm_l", "leftHand": "hand_l",
    "rightShoulder": "clavicle_r", "rightUpperArm": "upperarm_r", "rightLowerArm": "lowerarm_r", "rightHand": "hand_r",
    "leftUpperLeg": "thigh_l", "leftLowerLeg": "calf_l", "leftFoot": "foot_l", "leftToes": "ball_l",
    "rightUpperLeg": "thigh_r", "rightLowerLeg": "calf_r", "rightFoot": "foot_r", "rightToes": "ball_r",
}
for side, s in (("left", "l"), ("right", "r")):
    for f_vrm, f_ue in (("Thumb", "thumb"), ("Index", "index"), ("Middle", "middle"), ("Ring", "ring"), ("Little", "pinky")):
        names = ("Metacarpal", "Proximal", "Distal") if f_vrm == "Thumb" else ("Proximal", "Intermediate", "Distal")
        for i, n in enumerate(names):
            MAP[f"{side}{f_vrm}{n}"] = f"{f_ue}_0{i + 1}_{s}"

bpy.context.view_layer.objects.active = arm
ext = arm.data.vrm_addon_extension
ext.spec_version = "1.0"
hb = ext.vrm1.humanoid.human_bones
missing = []
for vrm_name, bone in MAP.items():
    if bone not in arm.data.bones:
        missing.append(bone)
        continue
    getattr(hb, _snake := __import__("re").sub(r"(?<!^)(?=[A-Z])", "_", vrm_name).lower()).node.bone_name = bone
print("fehlende Bones:", missing)

m = ext.vrm1.meta
m.vrm_name = "HASE Testavatar A (Quaternius)"
m.authors.add().value = "Quaternius (CC0); Zusammenstellung HASE VRM MAKER"
m.license_url = "https://vrm.dev/licenses/1.0/"
m.avatar_permission = "everyone"
m.commercial_usage = "personalProfit"
m.credit_notation = "unnecessary"
m.modification = "allowModificationRedistribution"
m.allow_redistribution = True
m.copyright_information = "Basismodell und Haare: Quaternius, CC0 1.0"

bpy.ops.object.select_all(action="DESELECT")
arm.select_set(True)
res = bpy.ops.export_scene.vrm(filepath=str(out), armature_object_name=arm.name)
print("export:", res, out, out.stat().st_size if out.exists() else "fehlt")
