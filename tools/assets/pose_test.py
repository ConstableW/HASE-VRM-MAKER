"""Wendet eine .vrma auf eine .vrm an und rendert ein Frame (Skinning-Test). Aufruf: python pose_test.py <avatar.vrm> <anim.vrma> <out.png> <frame>"""
import sys, math, bpy, io_scene_vrm
bpy.ops.wm.read_factory_settings(use_empty=True)
io_scene_vrm.register()
if "io_scene_vrm" not in bpy.context.preferences.addons: bpy.context.preferences.addons.new().module="io_scene_vrm"
bpy.ops.import_scene.vrm(filepath=sys.argv[1])
arm=[o for o in bpy.data.objects if o.type=='ARMATURE'][0]
bpy.context.view_layer.objects.active=arm; arm.select_set(True)
print(bpy.ops.import_scene.vrma(filepath=sys.argv[2], armature_object_name=arm.name))
act=arm.animation_data.action; print('action',act.name,act.frame_range)
sc=bpy.context.scene; sc.frame_set(int(sys.argv[4]))
sc.render.engine="CYCLES"; sc.cycles.samples=24; sc.cycles.device="CPU"; sc.render.resolution_x,sc.render.resolution_y=700,900
sc.world=bpy.data.worlds.new("w"); sc.world.color=(0.55,0.6,0.65)
sun=bpy.data.objects.new("sun",bpy.data.lights.new("sun","SUN")); sun.data.energy=4; sun.rotation_euler=(math.radians(50),0,math.radians(-30)); sc.collection.objects.link(sun)
cam=bpy.data.objects.new("cam",bpy.data.cameras.new("cam")); sc.collection.objects.link(cam); sc.camera=cam; cam.data.lens=80
cam.location=(0,-5.0,0.9); cam.rotation_euler=(math.radians(90),0,0)
sc.render.filepath=sys.argv[3]; bpy.ops.render.render(write_still=True); print('ok')
