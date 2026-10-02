"""SPDX-License-Identifier: Apache-2.0

Generate this project's own models using Blender, without external files.
Run: blender -b --python scripts/generate_authored_content.py -- --output assets/authored
"""

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector


def material(name, color, metallic=0.0, roughness=0.6):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*color, 1)
    mat.use_nodes = True
    node = mat.node_tree.nodes.get("Principled BSDF")
    node.inputs["Base Color"].default_value = (*color, 1)
    node.inputs["Metallic"].default_value = metallic
    node.inputs["Roughness"].default_value = roughness
    return mat


def finish(obj, name, mat, bone=None):
    obj.name = name
    obj.data.materials.append(mat)
    if bone:
        obj.vertex_groups.new(name=bone).add(list(range(len(obj.data.vertices))), 1, "REPLACE")
    return obj


def box(name, center, size, mat, bone=None, rotation=(0, 0, 0), bevel=0.015):
    bpy.ops.mesh.primitive_cube_add(size=1, location=center, rotation=rotation)
    obj = bpy.context.object
    obj.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel:
        mod = obj.modifiers.new("Rounded edges", "BEVEL")
        mod.width = min(bevel, min(size) / 4)
        mod.segments = 2
        bpy.ops.object.modifier_apply(modifier=mod.name)
    return finish(obj, name, mat, bone)


def ellipsoid(name, center, size, mat, bone=None):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=8, radius=1, location=center)
    obj = bpy.context.object
    obj.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    for polygon in obj.data.polygons:
        polygon.use_smooth = True
    return finish(obj, name, mat, bone)


def tube(name, start, end, radius, mat, bone=None, vertices=12):
    start, end = Vector(start), Vector(end)
    direction = end - start
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius,
        depth=direction.length, location=(start + end) / 2)
    obj = bpy.context.object
    obj.rotation_mode = "QUATERNION"
    obj.rotation_quaternion = direction.to_track_quat("Z", "Y")
    return finish(obj, name, mat, bone)


def clear():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for action in list(bpy.data.actions):
        bpy.data.actions.remove(action)
    for datablocks in (bpy.data.meshes, bpy.data.materials, bpy.data.armatures):
        for item in list(datablocks):
            if item.users == 0:
                datablocks.remove(item)


def operator():
    cloth = material("Uniform olive", (0.20, 0.24, 0.17))
    tan = material("Equipment canvas", (0.34, 0.29, 0.19))
    black = material("Mask and rubber", (0.022, 0.025, 0.024))
    white = material("Mask painted motif", (0.70, 0.69, 0.59))
    lens = material("Protective lenses", (0.09, 0.12, 0.10), 0.55, 0.18)
    skin = material("Hands", (0.45, 0.29, 0.20))
    metal = material("Equipment metal", (0.13, 0.15, 0.14), 0.65, 0.4)
    bones = {
        "root": ((0, 0, 0), (0, 0, 0.12), None),
        "pelvis": ((0, 0, 0.86), (0, 0, 1.03), "root"),
        "spine": ((0, 0, 1.03), (0, 0, 1.23), "pelvis"),
        "chest": ((0, 0, 1.23), (0, 0, 1.46), "spine"),
        "head": ((0, 0, 1.46), (0, 0, 1.78), "chest"),
    }
    box("Pelvis", (0, 0, 0.96), (0.32, 0.23, 0.23), cloth, "pelvis")
    box("Uniform torso", (0, 0, 1.21), (0.41, 0.24, 0.42), cloth, "chest", bevel=0.045)
    box("Plate carrier", (0, -0.13, 1.23), (0.34, 0.07, 0.34), tan, "chest")
    box("Backpack", (0, 0.16, 1.24), (0.29, 0.12, 0.33), tan, "chest")
    for x in (-0.11, 0, 0.11):
        box("Magazine pouch", (x, -0.19, 1.13), (0.085, 0.06, 0.12), tan, "chest")
    for z in (1.19, 1.23, 1.27, 1.31):
        box("Carrier webbing", (0, -0.174, z), (0.31, 0.009, 0.014), black, "chest", bevel=0.002)
    tube("Neck", (0, 0, 1.43), (0, 0, 1.53), 0.065, black, "head")
    ellipsoid("Balaclava", (0, -0.01, 1.635), (0.105, 0.10, 0.145), black, "head")
    ellipsoid("Helmet", (0, 0, 1.735), (0.12, 0.115, 0.064), tan, "head")
    box("Helmet mounting plate", (0, -0.11, 1.745), (0.042, 0.015, 0.034), metal, "head")
    for x in (-0.108, 0.108):
        ellipsoid("Headset cup", (x, 0, 1.64), (0.025, 0.044, 0.056), black, "head")
    tube("Microphone", (0.12, -0.025, 1.60), (0.052, -0.112, 1.555), 0.006, black, "head")
    for x in (-0.047, 0.047):
        box("Goggles", (x, -0.105, 1.663), (0.086, 0.023, 0.035), lens, "head", bevel=0.01)
        box("Mask cheek marking", (x, -0.103, 1.604), (0.034, 0.01, 0.035), white, "head", bevel=0.008)
    for x in (-0.035, -0.0175, 0, 0.0175, 0.035):
        box("Mask tooth marking", (x, -0.107, 1.56), (0.012, 0.009, 0.025), white, "head", bevel=0.002)
    for side, sign in (("L", 1), ("R", -1)):
        shoulder, elbow, wrist = (sign * 0.22, 0, 1.39), (sign * 0.34, 0, 1.14), (sign * 0.40, -0.035, 0.94)
        hip, knee, ankle = (sign * 0.10, 0, 0.91), (sign * 0.115, 0, 0.54), (sign * 0.115, 0, 0.13)
        bones.update({
            "upper_arm." + side: (shoulder, elbow, "chest"),
            "forearm." + side: (elbow, wrist, "upper_arm." + side),
            "hand." + side: (wrist, (sign * 0.41, -0.04, 0.85), "forearm." + side),
            "thigh." + side: (hip, knee, "pelvis"),
            "calf." + side: (knee, ankle, "thigh." + side),
            "foot." + side: (ankle, (sign * 0.115, -0.17, 0.075), "calf." + side),
        })
        tube("Upper sleeve", shoulder, elbow, 0.075, cloth, "upper_arm." + side)
        tube("Lower sleeve", elbow, wrist, 0.060, cloth, "forearm." + side)
        ellipsoid("Elbow pad", elbow, (0.07, 0.077, 0.068), black, "forearm." + side)
        ellipsoid("Glove", (sign * 0.405, -0.037, 0.89), (0.043, 0.045, 0.072), black, "hand." + side)
        tube("Trouser thigh", hip, knee, 0.10, cloth, "thigh." + side)
        tube("Trouser calf", knee, ankle, 0.077, cloth, "calf." + side)
        box("Knee protector", (sign * 0.115, -0.075, 0.55), (0.12, 0.045, 0.13), black, "calf." + side)
        box("Boot", (sign * 0.115, -0.055, 0.085), (0.135, 0.26, 0.15), black, "foot." + side)
        box("Boot sole", (sign * 0.115, -0.055, 0.025), (0.145, 0.27, 0.035), black, "foot." + side, bevel=0.008)
    parts = list(bpy.context.scene.objects)
    bpy.ops.object.select_all(action="DESELECT")
    for part in parts:
        part.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.join()
    body = bpy.context.object
    body.name = "Operator mesh"
    bpy.ops.object.armature_add(enter_editmode=True)
    rig = bpy.context.object
    rig.name = "Operator rig"
    for bone in list(rig.data.edit_bones):
        rig.data.edit_bones.remove(bone)
    for name, (head, tail, parent) in bones.items():
        bone = rig.data.edit_bones.new(name)
        bone.head, bone.tail = head, tail
        if parent:
            bone.parent = rig.data.edit_bones[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    body.modifiers.new("Skeleton deformation", "ARMATURE").object = rig
    body.parent = rig
    rig.animation_data_create()
    for action_name in ("Idle", "Walk", "Aim"):
        rig.animation_data.action = bpy.data.actions.new(action_name)
        for frame in (1, 7, 13, 19, 25):
            phase = (frame - 1) / 24 * math.tau
            for pose in rig.pose.bones:
                pose.rotation_mode = "XYZ"
                pose.rotation_euler = (0, 0, 0)
                if action_name == "Walk":
                    sign = 1 if pose.name.endswith(".L") else -1
                    if pose.name.startswith("thigh"):
                        pose.rotation_euler.x = sign * math.sin(phase) * 0.35
                    elif pose.name.startswith("calf"):
                        pose.rotation_euler.x = max(0, -sign * math.sin(phase)) * 0.40
                    elif pose.name.startswith("upper_arm"):
                        pose.rotation_euler.x = -sign * math.sin(phase) * 0.23
                elif action_name == "Idle" and pose.name == "chest":
                    pose.rotation_euler.x = math.sin(phase) * 0.012
                elif action_name == "Aim" and pose.name.startswith("upper_arm"):
                    pose.rotation_euler.x = -0.85
                pose.keyframe_insert(data_path="rotation_euler", frame=frame)
        rig.animation_data.action.use_fake_user = True
    rig.animation_data.action = bpy.data.actions["Idle"]
    bpy.context.scene.frame_set(1)


def carbine():
    steel = material("Weapon steel", (0.075, 0.085, 0.085), 0.8, 0.35)
    polymer = material("Weapon polymer", (0.075, 0.080, 0.060), 0.05, 0.65)
    tan = material("Weapon furniture", (0.33, 0.28, 0.18), 0.0, 0.65)
    box("Receiver", (0, 0.025, 0), (0.058, 0.22, 0.075), steel, bevel=0.009)
    box("Handguard", (0, -0.19, 0.006), (0.060, 0.20, 0.065), tan, bevel=0.009)
    tube("Barrel", (0, -0.30, 0.008), (0, -0.50, 0.008), 0.011, steel)
    tube("Muzzle device", (0, -0.49, 0.008), (0, -0.54, 0.008), 0.016, steel)
    tube("Buffer tube", (0, 0.13, 0.009), (0, 0.31, 0.009), 0.017, steel)
    box("Stock", (0, 0.28, -0.005), (0.055, 0.17, 0.065), tan)
    box("Buttpad", (0, 0.375, -0.015), (0.062, 0.025, 0.11), polymer, bevel=0.004)
    box("Grip", (0, 0.09, -0.09), (0.036, 0.047, 0.105), polymer, rotation=(0.22, 0, 0))
    box("Magazine", (0, -0.055, -0.108), (0.024, 0.073, 0.15), steel, rotation=(-0.10, 0, 0))
    for side in (-1, 1):
        for z in (-0.08, -0.11, -0.14, -0.17):
            box("Magazine pressed rib", (side * 0.013, -0.05, z), (0.004, 0.055, 0.004), polymer, bevel=0.001)
        tube("Trigger guard", (side * 0.016, 0.012, -0.046), (side * 0.016, 0.08, -0.054), 0.004, steel)
    box("Trigger", (0, 0.047, -0.056), (0.009, 0.007, 0.025), steel, bevel=0.002)
    for y in [0.11 - i * 0.017 for i in range(23)]:
        box("Top rail tooth", (0, y, 0.049), (0.055, 0.008, 0.012), steel, bevel=0.002)
    for side in (-1, 1):
        for y in (-0.24, -0.20, -0.16, -0.12):
            box("Handguard vent", (side * 0.031, y, 0.008), (0.004, 0.025, 0.015), polymer, bevel=0.002)
    tube("Sight housing", (0, -0.04, 0.084), (0, -0.11, 0.084), 0.021, steel)
    box("Sight mount", (0, -0.06, 0.061), (0.03, 0.055, 0.015), steel)


def skateboard():
    grip = material("Deck grip", (0.025, 0.027, 0.027), 0, 0.95)
    wood = material("Deck laminated edge", (0.44, 0.26, 0.10))
    paint = material("Deck underside", (0.30, 0.055, 0.035))
    silver = material("Truck alloy", (0.33, 0.36, 0.38), 0.8, 0.4)
    urethane = material("Wheel urethane", (0.78, 0.73, 0.54), 0, 0.8)
    vertices, faces = [], []
    rows = 17
    for layer in (0, 1):
        for i in range(rows):
            y = (i / (rows - 1) - 0.5) * 0.80
            width = 0.10 * math.sqrt(max(0.08, 1 - (abs(y) / 0.435) ** 4))
            z = max(0, abs(y) - 0.27) ** 2 * 3.5 + layer * 0.012
            vertices.extend([(-width, y, z), (width, y, z)])
    for layer in (0, 1):
        offset = layer * rows * 2
        for i in range(rows - 1):
            a = offset + i * 2
            face = (a, a + 1, a + 3, a + 2)
            faces.append(face if layer else tuple(reversed(face)))
    for i in range(rows - 1):
        a, b = i * 2, (i + 1) * 2
        faces.extend([(a, b, b + rows * 2, a + rows * 2),
            (a + 1, a + 1 + rows * 2, b + 1 + rows * 2, b + 1)])
    faces.extend([(0, rows * 2, rows * 2 + 1, 1),
        (rows * 2 - 2, rows * 2 - 1, rows * 4 - 1, rows * 4 - 2)])
    mesh = bpy.data.meshes.new("Concave deck")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new("Skateboard deck", mesh)
    bpy.context.collection.objects.link(obj)
    for mat in (paint, grip, wood):
        mesh.materials.append(mat)
    for polygon in mesh.polygons:
        polygon.material_index = 0 if polygon.index < rows - 1 else 1 if polygon.index < 2 * (rows - 1) else 2
    for y in (-0.245, 0.245):
        box("Truck base", (0, y, -0.010), (0.065, 0.065, 0.016), silver, bevel=0.003)
        tube("Truck hanger", (-0.09, y, -0.045), (0.09, y, -0.045), 0.012, silver)
        tube("Truck kingpin", (0, y - 0.014, -0.012), (0, y, -0.047), 0.008, silver)
        for x in (-0.10, 0.10):
            tube("Wheel", (x - 0.018, y, -0.045), (x + 0.018, y, -0.045), 0.028, urethane, vertices=16)
            tube("Wheel bearing", (x - 0.019, y, -0.045), (x + 0.019, y, -0.045), 0.011, silver)


def construction():
    wood = material("Construction timber", (0.32, 0.20, 0.10))
    metal = material("Construction fasteners", (0.15, 0.17, 0.17), 0.65)
    cell = 120 * 0.0254
    box("Foundation", (cell / 2, cell / 2, -0.15), (cell, cell, 0.30), wood)
    for x in [0.09 + i * 0.19 for i in range(16)]:
        box("Floorboard", (x, cell / 2, 0.02), (0.175, cell - 0.06, 0.04), wood, bevel=0.004)
    for x in (0.06, cell - 0.06):
        box("Wall upright", (x, 0, cell / 2), (0.12, 0.16, cell), wood)
    for z in [0.10 + i * 0.19 for i in range(16)]:
        box("Wallboard", (cell / 2, 0, z), (cell, 0.11, 0.17), wood, bevel=0.004)
    for z in (0.18, cell - 0.18):
        for x in (0.06, cell - 0.06):
            tube("Fastener", (x, -0.064, z), (x, -0.055, z), 0.008, metal)


def preview(path):
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    points = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
    lo = Vector([min(p[k] for p in points) for k in range(3)])
    hi = Vector([max(p[k] for p in points) for k in range(3)])
    center, radius = (lo + hi) / 2, max((hi - lo).length / 2, 0.3)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 24
    scene.cycles.use_denoising = False
    scene.world.use_nodes = True
    scene.world.node_tree.nodes["Background"].inputs[0].default_value = (0.035, 0.045, 0.065, 1)
    scene.world.node_tree.nodes["Background"].inputs[1].default_value = 0.6
    for offset, power, size in [((-3, -4, 5), 550, 3), ((3, -2, 3), 350, 2), ((1, 3, 4), 700, 2)]:
        bpy.ops.object.light_add(type="AREA", location=center + Vector(offset) * radius)
        light = bpy.context.object
        light.data.energy = power * radius * radius
        light.data.shape, light.data.size = "DISK", size * radius
        light.rotation_euler = (center - light.location).to_track_quat("-Z", "Y").to_euler()
    bpy.ops.object.camera_add(location=center + Vector((1.8, -3.0, 1.05)) * radius)
    camera = bpy.context.object
    camera.rotation_euler = (center - camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.type, camera.data.ortho_scale = "ORTHO", radius * 2.7
    scene.camera = camera
    scene.render.resolution_x, scene.render.resolution_y = 800, 800
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.filepath = str(path)
    bpy.ops.render.render(write_still=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--preview-dir", type=Path)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
    bpy.context.preferences.filepaths.save_version = 0
    args.output.mkdir(parents=True, exist_ok=True)
    if args.preview_dir:
        args.preview_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"schema": 1, "license": "CC0-1.0", "units": "meters", "source": "scripts/generate_authored_content.py",
        "generator_blender": bpy.app.version_string, "assets": []}
    for name, generate in [("masked_operator", operator), ("carbine", carbine),
            ("skateboard", skateboard), ("timber_construction", construction)]:
        clear()
        generate()
        bpy.context.view_layer.update()
        meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
        points = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
        entry = {"name": name, "file": name + ".glb", "vertices": sum(len(obj.data.vertices) for obj in meshes),
            "triangles": sum(len(poly.vertices) - 2 for obj in meshes for poly in obj.data.polygons),
            "bounds": [[min(p[k] for p in points) for k in range(3)], [max(p[k] for p in points) for k in range(3)]],
            "animations": [action.name for action in bpy.data.actions if action.users]}
        bpy.ops.object.select_all(action="SELECT")
        destination = args.output / entry["file"]
        bpy.ops.export_scene.gltf(filepath=str(destination.resolve()), export_format="GLB",
            use_selection=True, export_animations=True, export_skins=True,
            export_cameras=False, export_lights=False, export_animation_mode="ACTIONS")
        entry["sha256"] = hashlib.sha256(destination.read_bytes()).hexdigest()
        entry["bytes"] = destination.stat().st_size
        manifest["assets"].append(entry)
        bpy.ops.wm.save_as_mainfile(filepath=str((args.output / (name + ".blend")).resolve()))
        if args.preview_dir:
            preview((args.preview_dir / (name + ".png")).resolve())
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print("Generated four independently authored assets:", args.output.resolve())


if __name__ == "__main__":
    main()
