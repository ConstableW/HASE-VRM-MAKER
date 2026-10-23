"""Rendert eine Vorschau (Front + Seite) einer .vrm per Blender/Cycles-CPU. Aufruf: python render_preview.py <datei.vrm> <out-prefix>"""
import sys, math
import bpy, io_scene_vrm
from mathutils import Vector

bpy.ops.wm.read_factory_settings(use_empty=True)
io_scene_vrm.register()
if "io_scene_vrm" not in bpy.context.preferences.addons:
    bpy.context.preferences.addons.new().module = "io_scene_vrm"
bpy.ops.import_scene.vrm(filepath=sys.argv[1])
sc = bpy.context.scene
sc.render.engine = "CYCLES"; sc.cycles.samples = 24; sc.cycles.device = "CPU"
sc.render.resolution_x, sc.render.resolution_y = 700, 1000
sc.world = bpy.data.worlds.new("w"); sc.world.color = (0.55, 0.6, 0.65)
sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN")); sun.data.energy = 4; sun.rotation_euler = (math.radians(50), 0, math.radians(-30))
sc.collection.objects.link(sun)
cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam")); sc.collection.objects.link(cam); sc.camera = cam
cam.data.lens = 70
for name, loc, rot in (("front", (0, -6.5, 0.85), (90, 0, 0)), ("side", (6.5, 0, 0.85), (90, 0, 90))):
    cam.location = loc; cam.rotation_euler = [math.radians(a) for a in rot]
    sc.render.filepath = f"{sys.argv[2]}_{name}.png"
    bpy.ops.render.render(write_still=True)
print("ok")
