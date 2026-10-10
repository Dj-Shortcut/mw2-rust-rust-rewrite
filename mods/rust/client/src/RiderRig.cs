// The rider's pose. The local player's model is a separate root object
// (assets/prefabs/player/player_model.prefab) with a humanoid Animator, so bones are looked up
// through Unity's humanoid mapping instead of by name. The shared rider module turns the ride's
// state into joint positions; here they become bone rotations, late in the frame after the game's
// own animation: every bone is turned so that it points at its child's target. Run R (10 October
// 2026, issue #337) showed that a pose written at that point is what gets drawn.
// The module's ankles are the skeleton's ankle joints: it is given the plane those joints rest on
// when the soles are on the deck.
using System;
using Shortcut.RustMod;
using UnityEngine;

public static class RiderRig
{
    public const float AnkleHeight = 0.095f, ToeHeight = 0.025f, DeckTop = 0.085f;
    public static bool Bound, Enabled = true;
    public static Animator Anim;
    public static Transform Root, Pelvis, Neck, Head, LHip, LKnee, LFoot, LToe, RHip, RKnee, RFoot, RToe, LUpper, LFore, LHand, RUpper, RFore, RHand;
    public static SkateRiderRig Rig;
    public static float Thigh, Shin, Foot, UpperArm, Forearm, Crouch = 0.18f;
    // Checks for the test sessions: how far the joints ended up from where the module wanted them,
    // and how far they had moved again by the start of the next frame.
    public static float Miss, Drift;
    public static string MissAt = "", DriftAt = "", Error = "";
    public static int Applied, Refusals;
    private static Transform[] spine = new Transform[0];
    private static Vector3 lSole, rSole;
    private static readonly Transform[] marks = new Transform[9];
    private static readonly Vector3[] markAt = new Vector3[9];
    private static readonly string[] markName = { "left ankle", "right ankle", "left knee", "right knee", "neck", "head", "left hand", "right hand", "pelvis" };
    private static bool failed, marked;

    private static void Say(string m) { Out.Say("RIDER " + m); }
    private static float Dist(Transform a, Transform b) { return Vector3.Distance(a.position, b.position); }
    private static Vector3 V(SkateVector v) { return new Vector3((float)v.X, (float)v.Y, (float)v.Z); }

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
        Pelvis = a.GetBoneTransform(HumanBodyBones.Hips); Neck = a.GetBoneTransform(HumanBodyBones.Neck); Head = a.GetBoneTransform(HumanBodyBones.Head);
        LHip = a.GetBoneTransform(HumanBodyBones.LeftUpperLeg); LKnee = a.GetBoneTransform(HumanBodyBones.LeftLowerLeg); LFoot = a.GetBoneTransform(HumanBodyBones.LeftFoot); LToe = a.GetBoneTransform(HumanBodyBones.LeftToes);
        RHip = a.GetBoneTransform(HumanBodyBones.RightUpperLeg); RKnee = a.GetBoneTransform(HumanBodyBones.RightLowerLeg); RFoot = a.GetBoneTransform(HumanBodyBones.RightFoot); RToe = a.GetBoneTransform(HumanBodyBones.RightToes);
        LUpper = a.GetBoneTransform(HumanBodyBones.LeftUpperArm); LFore = a.GetBoneTransform(HumanBodyBones.LeftLowerArm); LHand = a.GetBoneTransform(HumanBodyBones.LeftHand);
        RUpper = a.GetBoneTransform(HumanBodyBones.RightUpperArm); RFore = a.GetBoneTransform(HumanBodyBones.RightLowerArm); RHand = a.GetBoneTransform(HumanBodyBones.RightHand);
        if (Pelvis == null || Neck == null || Head == null || LHip == null || LKnee == null || LFoot == null || LToe == null || RHip == null || RKnee == null || RFoot == null || RToe == null
            || LUpper == null || LFore == null || LHand == null || RUpper == null || RFore == null || RHand == null)
        { Say("humanoid mapping incomplete"); return false; }

        // The spine is every bone between the pelvis and the neck, whatever the avatar calls them.
        var chain = new System.Collections.Generic.List<Transform>();
        var t = Neck.parent;
        while (t != null && t != Pelvis && chain.Count < 8) { chain.Insert(0, t); t = t.parent; }
        if (t != Pelvis || chain.Count == 0) { Say("no spine between the pelvis and the neck"); return false; }
        spine = chain.ToArray();
        var back = Dist(Pelvis, spine[0]);
        for (var i = 0; i < spine.Length; i++) back += Dist(spine[i], i + 1 < spine.Length ? spine[i + 1] : Neck);

        Thigh = (Dist(LHip, LKnee) + Dist(RHip, RKnee)) * 0.5f; Shin = (Dist(LKnee, LFoot) + Dist(RKnee, RFoot)) * 0.5f;
        Foot = (Dist(LFoot, LToe) + Dist(RFoot, RToe)) * 0.5f;
        UpperArm = (Dist(LUpper, LFore) + Dist(RUpper, RFore)) * 0.5f; Forearm = (Dist(LFore, LHand) + Dist(RFore, RHand)) * 0.5f;
        var hipWidth = Dist(LHip, RHip);
        var hipDrop = Vector3.Distance(Pelvis.position, (LHip.position + RHip.position) * 0.5f);
        var drop = AnkleHeight - ToeHeight;
        var footFlat = (float)Math.Sqrt(Math.Max(0.0001f, Foot * Foot - drop * drop));
        // The module puts the neck 0.12 of the spine above the chest, and the chest a spine above the hips.
        Rig = new SkateRiderRig((Thigh + Shin) * 0.945f, hipWidth, Thigh, Shin, footFlat, (back + hipDrop) / 1.12f, Dist(LUpper, RUpper), UpperArm, Forearm, Dist(Neck, Head));
        // Which way the soles face, seen from each foot bone, while the player stands.
        var up = Root.rotation * Vector3.up;
        lSole = Quaternion.Inverse(LFoot.rotation) * up; rSole = Quaternion.Inverse(RFoot.rotation) * up;
        marks[0] = LFoot; marks[1] = RFoot; marks[2] = LKnee; marks[3] = RKnee; marks[4] = Neck; marks[5] = Head; marks[6] = LHand; marks[7] = RHand; marks[8] = Pelvis;
        failed = false; marked = false; Bound = true; Error = "";
        Say("bound to " + Root.gameObject.name + " spine bones=" + spine.Length + " thigh=" + Thigh.ToString("F3") + " shin=" + Shin.ToString("F3") + " foot=" + Foot.ToString("F3") + " upperArm=" + UpperArm.ToString("F3")
            + " forearm=" + Forearm.ToString("F3") + " hips=" + hipWidth.ToString("F3") + " hipDrop=" + hipDrop.ToString("F3") + " back=" + back.ToString("F3") + " shoulders=" + Rig.ShoulderWidth.ToString("F3")
            + " neck=" + Rig.NeckToHeadLength.ToString("F3") + " headScale=" + Out.V(Head.localScale));
        return true;
    }

    // Turn a bone so that the line to its child points along the wanted direction.
    private static void Aim(Transform bone, Transform child, Vector3 want)
    {
        var cur = child.position - bone.position;
        if (cur.sqrMagnitude < 0.000001f || want.sqrMagnitude < 0.000001f) return;
        bone.rotation = Quaternion.FromToRotation(cur, want) * bone.rotation;
    }

    // Roll a bone about its own axis until the given bone-local direction points as near to `want` as that roll allows.
    private static void Roll(Transform bone, Vector3 axis, Vector3 local, Vector3 want)
    {
        var cur = bone.rotation * local;
        cur = cur - axis * Vector3.Dot(cur, axis); want = want - axis * Vector3.Dot(want, axis);
        if (cur.sqrMagnitude < 0.0001f || want.sqrMagnitude < 0.0001f) return;
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

    // Called late in the frame, after the game has animated the model. `feet` is where the board
    // touches the ground under the rider, `up` the board's unflipped up direction.
    public static void Frame(Vector3 feet, Vector3 up, bool legsOnly, float frameSeconds)
    {
        if (!Bound || !Enabled || failed) return;
        try
        {
            if (Pelvis == null) { Bound = false; return; }
            double ux = up.x, uy = up.y, uz = up.z;
            var ul = Math.Sqrt(ux * ux + uy * uy + uz * uz);
            if (ul < 0.5 || uy / ul < 0.2) { ux = 0; uy = 1; uz = 0; ul = 1; }
            ux /= ul; uy /= ul; uz /= ul;
            var rad = SkateRide.NoseYaw * Math.PI / 180.0;
            double fx = Math.Sin(rad), fy = 0, fz = Math.Cos(rad);
            var d = fx * ux + fy * uy + fz * uz;
            fx -= ux * d; fy -= uy * d; fz -= uz * d;
            var fl = Math.Sqrt(fx * fx + fy * fy + fz * fz);
            if (fl < 0.2) return;
            fx /= fl; fy /= fl; fz /= fl;
            var upV = new Vector3((float)ux, (float)uy, (float)uz);
            var plane = feet + upV * (DeckTop + AnkleHeight - (float)SkatePose.FootLift);

            var now = Time.realtimeSinceStartup;
            var mode = SkateRide.Mode;
            // Tucked in the air, folded right down to reach the board for a grab.
            var want = mode == RideMode.Air ? (SkateRide.Grab ? 0.95f : 0.5f) : mode == RideMode.Grind ? 0.38f : SkateRide.Braking ? 0.4f : 0.18f;
            var since = now - SkateRide.LandAt;
            if (mode == RideMode.Ground && since < 0.35f) want += (1f - since / 0.35f) * SkateRide.Clamp(SkateRide.LandImpact / 8f, 0.2f, 1f) * 0.45f;
            Crouch += (SkateRide.Clamp(want, 0f, 1f) - Crouch) * SkateRide.Clamp(frameSeconds * 14f, 0f, 1f);
            // The module leans toward the board's own right; the ride leans toward the right of travel.
            var lean = SkateRide.Clamp(SkateRide.Trick.Switch ? -SkateRide.Lean : SkateRide.Lean, -(float)SkatePose.MaximumLean, (float)SkatePose.MaximumLean);
            var speed = Math.Min((float)SkateMotion.MaximumGroundSpeed, Math.Abs(SkateRide.Speed));
            var input = new SkateRiderInput(new SkateVector(plane.x, plane.y, plane.z), new SkateVector(fx, fy, fz), new SkateVector(ux, uy, uz), SkateStance.Regular, SkateRide.TailFirst,
                speed, lean, Crouch, mode == RideMode.Air, mode == RideMode.Ground && SkateRide.PushPhase > 0f, SkateRide.Clamp(SkateRide.PushPhase, 0f, 1f),
                SkateRide.FlipDeg, SkateRide.Grab, mode == RideMode.Bail, SkateKeys.LookYaw % 360f);
            SkateRiderPose pose; string error;
            if (!SkateRider.TryCreate(Rig, input, out pose, out error))
            {
                if (Refusals++ == 0 || error != Error) Say("pose refused: " + error);
                Error = error; return;
            }
            // The module keeps the pushing foot at deck height; the ground is a deck lower. How far
            // down the foot is follows the module's stroke: down by a quarter, up again at the end.
            var phase = input.PushPhase; var down = 0f;
            if (input.Pushing) down = phase < 0.25 ? Smooth((float)phase / 0.25f) : phase < 0.85 ? 1f : 1f - Smooth(((float)phase - 0.85f) / 0.15f);
            Apply(pose, upV, legsOnly, down * DeckTop, SkateRide.TailFirst);
            Applied++;
        }
        catch (Exception e) { failed = true; Say("pose threw " + e.GetType().Name + ": " + e.Message); }
    }

    private static float Smooth(float t) { t = t < 0f ? 0f : t > 1f ? 1f : t; return t * t * (3f - 2f * t); }

    private static void Apply(SkateRiderPose p, Vector3 up, bool legsOnly, float pushDrop, bool pushLeft)
    {
        // Pelvis: turn it so the hips line up with the pose and the spine starts upward, then move
        // it so the middle of the hip joints is where the pose has it.
        var hipsNow = RHip.position - LHip.position; var upNow = spine[0].position - Pelvis.position;
        if (hipsNow.sqrMagnitude > 0.000001f && upNow.sqrMagnitude > 0.000001f)
        {
            var from = Quaternion.LookRotation(Vector3.Cross(hipsNow.normalized, upNow.normalized), upNow.normalized);
            var to = Quaternion.LookRotation(V(p.PelvisForward), V(p.PelvisUp));
            Pelvis.rotation = to * Quaternion.Inverse(from) * Pelvis.rotation;
        }
        Pelvis.position = Pelvis.position + ((V(p.LeftHip) + V(p.RightHip)) * 0.5f - (LHip.position + RHip.position) * 0.5f);

        Miss = 0f; MissAt = "";
        if (!legsOnly)
        {
            // Spine: straight along the chest's up, then swung as one piece so the neck is on its target.
            var chestUp = V(p.ChestUp);
            for (var i = 0; i < spine.Length; i++) Aim(spine[i], i + 1 < spine.Length ? spine[i + 1] : Neck, chestUp);
            Aim(spine[0], Neck, V(p.Neck) - spine[0].position);
            Aim(Neck, Head, V(p.Head) - V(p.Neck));
            // The head turns toward where the rider looks, as far as a neck goes.
            var chestForward = V(p.ChestForward); var headForward = V(p.HeadForward);
            var cf = chestForward - chestUp * Vector3.Dot(chestForward, chestUp); var hf = headForward - chestUp * Vector3.Dot(headForward, chestUp);
            if (cf.sqrMagnitude > 0.0001f && hf.sqrMagnitude > 0.0001f)
            {
                cf = cf.normalized; hf = hf.normalized;
                var turn = (float)(Math.Atan2(Vector3.Dot(Vector3.Cross(cf, hf), chestUp), Vector3.Dot(cf, hf)) * 180.0 / Math.PI);
                turn = SkateRide.Clamp(turn, -75f, 75f);
                Neck.rotation = SkateRide.Turn(turn * 0.4f, chestUp) * Neck.rotation;
                Head.rotation = SkateRide.Turn(turn * 0.6f, chestUp) * Head.rotation;
            }
            Note(Neck, V(p.Neck), "neck");
            Limb(LUpper, LFore, LHand, V(p.LeftHand), V(p.LeftElbow), UpperArm, Forearm, "left hand");
            Limb(RUpper, RFore, RHand, V(p.RightHand), V(p.RightElbow), UpperArm, Forearm, "right hand");
        }

        // The rear foot pushes on the ground beside the board (the front one when rolling tail first).
        var lower = up * pushDrop;
        var la = V(p.LeftAnkle) - (pushLeft ? lower : Vector3.zero); var ra = V(p.RightAnkle) - (pushLeft ? Vector3.zero : lower);
        Limb(LHip, LKnee, LFoot, la, V(p.LeftKnee), Thigh, Shin, "left ankle");
        Limb(RHip, RKnee, RFoot, ra, V(p.RightKnee), Thigh, Shin, "right ankle");
        // Feet: toes toward the pose's toes, lowered from the ankle plane to the sole, soles flat.
        var drop = up * (AnkleHeight - ToeHeight);
        Aim(LFoot, LToe, V(p.LeftToe) - (pushLeft ? lower : Vector3.zero) - drop - LFoot.position);
        Aim(RFoot, RToe, V(p.RightToe) - (pushLeft ? Vector3.zero : lower) - drop - RFoot.position);
        Roll(LFoot, (LToe.position - LFoot.position).normalized, lSole, up);
        Roll(RFoot, (RToe.position - RFoot.position).normalized, rSole, up);

        for (var i = 0; i < marks.Length; i++) markAt[i] = marks[i].position - (i == 8 ? Root.position : Pelvis.position);
        marked = true;
    }

    // A two-bone limb onto its target, bending toward the pose's middle joint.
    private static void Limb(Transform a, Transform b, Transform c, Vector3 target, Vector3 hint, float upper, float lower, string name)
    {
        var mid = Bend(a.position, target, upper, lower, hint - a.position);
        Aim(a, b, mid - a.position);
        Aim(b, c, target - b.position);
        Note(c, target, name);
    }

    private static void Note(Transform bone, Vector3 target, string name)
    {
        var miss = Vector3.Distance(bone.position, target);
        if (miss > Miss) { Miss = miss; MissAt = name; }
    }

    // At the start of the next frame, before the game animates again: has anything moved the joints since the pose was written?
    public static void Check()
    {
        if (!marked || !Bound || failed) return;
        marked = false;
        try
        {
            if (Pelvis == null || Root == null) return;
            Drift = 0f; DriftAt = "";
            for (var i = 0; i < marks.Length; i++)
            {
                var d = Vector3.Distance(marks[i].position - (i == 8 ? Root.position : Pelvis.position), markAt[i]);
                if (d > Drift) { Drift = d; DriftAt = markName[i]; }
            }
        }
        catch (Exception e) { failed = true; Say("check threw " + e.GetType().Name + ": " + e.Message); }
    }
}
