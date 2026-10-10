// The rider's pose. The local player's model is a separate root object
// (assets/prefabs/player/player_model.prefab) with a humanoid Animator, so bones are looked up
// through Unity's humanoid mapping instead of by name (probe run M, 10 October 2026: pelvis,
// spine1-3, neck, head, l_hip/l_knee/l_foot/l_toe, clavicle/upperarm/forearm/hand).
// Each frame, after the game's own animation, joint targets are turned into bone rotations:
// every bone is turned so that it points at its child's target. Limb lengths are measured on the
// real skeleton once. The stance generator here is a placeholder until the shared SkateRider
// module (issue #337) supplies joint positions.
using System;
using UnityEngine;

public static class RiderRig
{
    public static bool Bound, Enabled = true;
    public static Animator Anim;
    public static Transform Root, Pelvis, Spine1, Spine2, Spine3, Neck, Head, LHip, LKnee, LFoot, LToe, RHip, RKnee, RFoot, RToe, LUpper, LFore, LHand, RUpper, RFore, RHand;
    public static float Thigh, Shin, Foot, UpperArm, Forearm, HipHalf, HipDrop, Spine, NeckLen, Ankle;
    private static bool failed;

    private static void Say(string m) { Out.Say("RIDER " + m); }

    // Finds the player model nearest to the player and measures it. Returns false until there is one.
    public static bool Bind(Transform playerT)
    {
        Bound = false;
        var models = UnityEngine.Object.FindObjectsOfType<PlayerModel>();
        PlayerModel best = null; var bd = 2f;
        for (var i = 0; i < models.Length; i++)
        {
            var d = Vector3.Distance(models[i].transform.position, playerT.position);
            if (d < bd) { bd = d; best = models[i]; }
        }
        if (best == null) return false;
        var a = best.GetComponentInChildren<Animator>(true);
        if (a == null || !a.isHuman) { Say("model without a humanoid animator"); return false; }
        Anim = a; Root = best.transform;
        Pelvis = a.GetBoneTransform(HumanBodyBones.Hips); Spine1 = a.GetBoneTransform(HumanBodyBones.Spine); Spine2 = a.GetBoneTransform(HumanBodyBones.Chest); Spine3 = a.GetBoneTransform(HumanBodyBones.UpperChest);
        Neck = a.GetBoneTransform(HumanBodyBones.Neck); Head = a.GetBoneTransform(HumanBodyBones.Head);
        LHip = a.GetBoneTransform(HumanBodyBones.LeftUpperLeg); LKnee = a.GetBoneTransform(HumanBodyBones.LeftLowerLeg); LFoot = a.GetBoneTransform(HumanBodyBones.LeftFoot); LToe = a.GetBoneTransform(HumanBodyBones.LeftToes);
        RHip = a.GetBoneTransform(HumanBodyBones.RightUpperLeg); RKnee = a.GetBoneTransform(HumanBodyBones.RightLowerLeg); RFoot = a.GetBoneTransform(HumanBodyBones.RightFoot); RToe = a.GetBoneTransform(HumanBodyBones.RightToes);
        LUpper = a.GetBoneTransform(HumanBodyBones.LeftUpperArm); LFore = a.GetBoneTransform(HumanBodyBones.LeftLowerArm); LHand = a.GetBoneTransform(HumanBodyBones.LeftHand);
        RUpper = a.GetBoneTransform(HumanBodyBones.RightUpperArm); RFore = a.GetBoneTransform(HumanBodyBones.RightLowerArm); RHand = a.GetBoneTransform(HumanBodyBones.RightHand);
        if (Pelvis == null || Spine1 == null || Neck == null || Head == null || LHip == null || LKnee == null || LFoot == null || RHip == null || RKnee == null || RFoot == null
            || LUpper == null || LFore == null || LHand == null || RUpper == null || RFore == null || RHand == null)
        { Say("humanoid mapping incomplete"); return false; }
        Thigh = (Dist(LHip, LKnee) + Dist(RHip, RKnee)) * 0.5f; Shin = (Dist(LKnee, LFoot) + Dist(RKnee, RFoot)) * 0.5f;
        Foot = LToe != null && RToe != null ? (Dist(LFoot, LToe) + Dist(RFoot, RToe)) * 0.5f : 0.16f;
        UpperArm = (Dist(LUpper, LFore) + Dist(RUpper, RFore)) * 0.5f; Forearm = (Dist(LFore, LHand) + Dist(RFore, RHand)) * 0.5f;
        HipHalf = Dist(LHip, RHip) * 0.5f;
        HipDrop = Vector3.Distance(Pelvis.position, (LHip.position + RHip.position) * 0.5f);
        Spine = Dist(Pelvis, Neck); NeckLen = Dist(Neck, Head);
        Ankle = 0.095f;
        failed = false; Bound = true;
        Say("bound to " + Root.gameObject.name + " thigh=" + Thigh.ToString("F3") + " shin=" + Shin.ToString("F3") + " foot=" + Foot.ToString("F3") + " upperArm=" + UpperArm.ToString("F3") + " forearm=" + Forearm.ToString("F3")
            + " hipHalf=" + HipHalf.ToString("F3") + " hipDrop=" + HipDrop.ToString("F3") + " spine=" + Spine.ToString("F3") + " neck=" + NeckLen.ToString("F3"));
        return true;
    }

    private static float Dist(Transform a, Transform b) { return Vector3.Distance(a.position, b.position); }

    // Turn a bone so that the line to its child points along the wanted direction.
    private static void Aim(Transform bone, Transform child, Vector3 want)
    {
        var cur = child.position - bone.position;
        if (cur.sqrMagnitude < 0.000001f || want.sqrMagnitude < 0.000001f) return;
        bone.rotation = Quaternion.FromToRotation(cur, want) * bone.rotation;
    }

    // Middle joint of a two-bone limb: root, target, the two lengths and the side the joint bends toward.
    public static Vector3 Bend(Vector3 root, Vector3 target, float upper, float lower, Vector3 pole)
    {
        var to = target - root; var d = to.magnitude;
        var max = (upper + lower) * 0.999f; var min = Math.Abs(upper - lower) * 1.001f + 0.001f;
        if (d > max) d = max; if (d < min) d = min;
        var axis = to.sqrMagnitude > 0.000001f ? to.normalized : Vector3.down;
        var along = (upper * upper - lower * lower + d * d) / (2f * d);
        var h2 = upper * upper - along * along;
        var side = pole - axis * Vector3.Dot(pole, axis);
        side = side.sqrMagnitude > 0.000001f ? side.normalized : Vector3.Cross(axis, Vector3.right).normalized;
        return root + axis * along + side * (float)Math.Sqrt(h2 > 0f ? h2 : 0f);
    }

    // Called late in the frame, after the game has animated the model.
    public static void Apply(Vector3 boardPos, Vector3 f, Vector3 u, float lean, float crouch, bool legsOnly)
    {
        if (!Bound || !Enabled || failed) return;
        try
        {
            if (Pelvis == null) { Bound = false; return; }
            var r = Vector3.Cross(u, f);
            // Regular stance: left foot leads, chest toward the board's right, opened a little toward travel.
            var c = (r * 0.82f + f * 0.57f).normalized;
            var bodyRight = Vector3.Cross(u, c);
            var deck = boardPos + u * 0.10f;
            var leftAnkle = deck + f * 0.21f + u * Ankle;
            var rightAnkle = deck - f * 0.21f + u * Ankle;
            var reach = Thigh + Shin;
            var leanRad = lean * (float)Math.PI / 180f;
            var pelvis = deck + u * (Ankle + reach * (0.84f - 0.22f * crouch) + HipDrop) + c * (0.05f + 0.10f * crouch) + r * (float)Math.Sin(leanRad) * 0.25f;
            if (legsOnly) pelvis = pelvis - c * 0.05f;

            // Pelvis: place it, then turn it so the hips line up with the body's right and the spine starts upward.
            var hipsNow = RHip.position - LHip.position; var upNow = Spine1.position - Pelvis.position;
            if (hipsNow.sqrMagnitude > 0.000001f && upNow.sqrMagnitude > 0.000001f)
            {
                var fromFwd = Vector3.Cross(hipsNow.normalized, upNow.normalized);
                var toFwd = Vector3.Cross(bodyRight, u);
                var from = Quaternion.LookRotation(fromFwd, upNow.normalized);
                var to = Quaternion.LookRotation(toFwd, u);
                Pelvis.rotation = to * Quaternion.Inverse(from) * Pelvis.rotation;
            }
            Pelvis.position = pelvis;

            if (!legsOnly)
            {
                var bend = (12f + 22f * crouch) * (float)Math.PI / 180f;
                var spineDir = (u * (float)Math.Cos(bend) + c * (float)Math.Sin(bend) + r * (float)Math.Sin(leanRad) * 0.6f).normalized;
                Aim(Spine1, Spine2 != null ? Spine2 : Neck, spineDir);
                if (Spine2 != null) Aim(Spine2, Spine3 != null ? Spine3 : Neck, spineDir);
                if (Spine3 != null) Aim(Spine3, Neck, spineDir);
                Aim(Neck, Head, (u * 0.9f + c * 0.25f + f * 0.2f).normalized);
            }

            // Legs.
            var lk = Bend(LHip.position, leftAnkle, Thigh, Shin, (c + f * 0.35f).normalized);
            Aim(LHip, LKnee, lk - LHip.position); Aim(LKnee, LFoot, leftAnkle - LKnee.position);
            var rk = Bend(RHip.position, rightAnkle, Thigh, Shin, (c - f * 0.35f).normalized);
            Aim(RHip, RKnee, rk - RHip.position); Aim(RKnee, RFoot, rightAnkle - RKnee.position);
            if (LToe != null) Aim(LFoot, LToe, (c + f * 0.25f).normalized * Foot - u * (Ankle - 0.02f));
            if (RToe != null) Aim(RFoot, RToe, c * Foot - u * (Ankle - 0.02f));

            if (!legsOnly)
            {
                // Arms: leading hand ahead and out, trailing hand behind and out, elbows low.
                var arm = (UpperArm + Forearm) * 0.9f;
                var lh = LUpper.position + (f * 0.55f - u * 0.72f + c * 0.25f).normalized * arm;
                var le = Bend(LUpper.position, lh, UpperArm, Forearm, (-u * 0.7f - c * 0.4f).normalized);
                Aim(LUpper, LFore, le - LUpper.position); Aim(LFore, LHand, lh - LFore.position);
                var rh = RUpper.position + (-f * 0.55f - u * 0.72f + c * 0.25f).normalized * arm;
                var re = Bend(RUpper.position, rh, UpperArm, Forearm, (-u * 0.7f - c * 0.4f).normalized);
                Aim(RUpper, RFore, re - RUpper.position); Aim(RFore, RHand, rh - RFore.position);
            }
        }
        catch (Exception e) { failed = true; Say("pose threw " + e.GetType().Name + ": " + e.Message); }
    }
}
