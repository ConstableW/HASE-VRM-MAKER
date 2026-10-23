"""Rendert Gesichts-Close-ups einer .vrm mit aktivierten VRM-Expressions. Aufruf: python render_expressions.py <datei.vrm> <out-prefix> [expr ...]"""
import sys, math
import bpy, io_scene_vrm

bpy.ops.wm.read_factory_settings(use_empty=True)
io_scene_vrm.register()
if "io_scene_vrm" not in bpy.context.preferences.addons:
    bpy.context.preferences.addons.new().module = "io_scene_vrm"
bpy.ops.import_scene.vrm(filepath=sys.argv[1])
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
sc = bpy.context.scene
sc.render.engine = "CYCLES"; sc.cycles.samples = 24; sc.cycles.device = "CPU"
sc.render.resolution_x, sc.render.resolution_y = 600, 600
sc.world = bpy.data.worlds.new("w"); sc.world.color = (0.55, 0.6, 0.65)
sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN")); sun.data.energy = 4; sun.rotation_euler = (math.radians(55), 0, math.radians(-25)); sc.collection.objects.link(sun)
cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam")); sc.collection.objects.link(cam); sc.camera = cam; cam.data.lens = 100
head = arm.pose.bones["head"].head if "head" in arm.pose.bones else arm.pose.bones["Head"].head
z = (arm.matrix_world @ head).z + 0.05
cam.location = (0, -1.05, z); cam.rotation_euler = (math.radians(90), 0, 0)
preset = arm.data.vrm_addon_extension.vrm1.expressions.preset
for expr in (sys.argv[3:] or ["neutral", "blink", "aa", "happy"]):
    for n in ("blink", "aa", "happy", "sad", "oh", "ou", "ee", "ih", "surprised", "relaxed"):
        getattr(preset, n).preview = 0.0
    if expr != "neutral":
        getattr(preset, expr).preview = 1.0
    bpy.context.view_layer.update()
    sc.render.filepath = f"{sys.argv[2]}_{expr}.png"
    bpy.ops.render.render(write_still=True)
print("ok")
