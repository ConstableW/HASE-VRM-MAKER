"""Humanoid-Mapping des UE-Mannequin-Skeletons (Quaternius) auf VRM 1.0."""
import re

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



def apply_mapping(arm):
    """Setzt das VRM-1.0-Humanoid-Mapping am Armature; gibt fehlende Bones zurück."""
    ext = arm.data.vrm_addon_extension
    ext.spec_version = "1.0"
    hb = ext.vrm1.humanoid.human_bones
    missing = []
    for vrm_name, bone in MAP.items():
        if bone not in arm.data.bones:
            missing.append(bone)
            continue
        getattr(hb, re.sub(r"(?<!^)(?=[A-Z])", "_", vrm_name).lower()).node.bone_name = bone
    return missing
