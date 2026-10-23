VRMA-Konvertierung des Pakets "VRM-Assets-Pack-For-Silly-Tavern"
=================================================================

Quelle:
https://github.com/test157t/VRM-Assets-Pack-For-Silly-Tavern

Inhalt:
- 116 Animationen aus dem Ordner "animation"
- Konvertiert von BVH nach VRM Animation (VRMA)
- VRMC_vrm_animation Spezifikation 1.0
- BVH-Maßstab: 0,1 (ergibt beim Referenzskelett ca. 1,6-1,8 m)

Zielordner in SillyTavern:
/data/<user-handle>/assets/vrm/animation

Die Dateien wurden strukturell geprüft und zusätzlich mit dem offiziellen
@pixiv/three-vrm-animation Loader geladen.

Konverter-Basis:
https://github.com/vrm-c/bvh2vrma (MIT)
