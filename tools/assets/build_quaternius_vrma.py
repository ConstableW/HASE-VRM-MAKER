"""Quaternius Universal Animation Library (CC0, UE-Mannequin-Skeleton) -> VRMA (VRM Animation 1.0).

Entwicklungswerkzeug (bpy + bpy-vrm-format, nur lokal). Aufruf: python build_quaternius_vrma.py <repo-root>
Ausgabe: content/testcandidate/animations/<Bibliothek>/<Aktion>.vrma
"""
import sys
from pathlib import Path
import bpy
import io_scene_vrm

sys.path.insert(0, str(Path(__file__).parent))
from ue_vrm_map import apply_mapping

root = Path(sys.argv[1])
dl = root / "content/testcandidate/downloads"
LIBS = {
    "UAL1": dl / "Universal Animation Library[Standard]/Unreal-Godot/UAL1_Standard.glb",
    "UAL2": dl / "Universal Animation Library 2[Standard]/Unreal-Godot/UAL2_Standard.glb",
}
bpy.ops.wm.read_factory_settings(use_empty=True)
io_scene_vrm.register()
if "io_scene_vrm" not in bpy.context.preferences.addons:
    bpy.context.preferences.addons.new().module = "io_scene_vrm"

for lib, path in LIBS.items():
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    for a in list(bpy.data.actions):
        bpy.data.actions.remove(a)
    bpy.ops.import_scene.gltf(filepath=str(path))
    arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
    print(lib, "fehlende Bones:", apply_mapping(arm))
    bpy.context.view_layer.objects.active = arm
    outdir = root / "content/testcandidate/animations" / lib
    outdir.mkdir(parents=True, exist_ok=True)
    ok, fail = 0, []
    for act in sorted(bpy.data.actions, key=lambda a: a.name):
        if act.name == "A_TPose":
            continue
        arm.animation_data_create().action = act
        bpy.context.scene.frame_start, bpy.context.scene.frame_end = int(act.frame_range[0]), int(act.frame_range[1])
        bpy.ops.object.select_all(action="DESELECT")
        arm.select_set(True)
        try:
            bpy.ops.export_scene.vrma(filepath=str(outdir / f"{act.name}.vrma"), armature_object_name=arm.name)
            ok += 1
        except Exception as e:  # noqa: BLE001
            fail.append((act.name, str(e).splitlines()[-1][:100]))
    print(lib, "exportiert:", ok, "fehlgeschlagen:", fail)
