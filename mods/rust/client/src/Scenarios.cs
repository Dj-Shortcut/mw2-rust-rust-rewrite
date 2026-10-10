using System;
using Shortcut.RustMod;
using UnityEngine;
using UnityEngine.InputSystem;

public static class Scenarios
{
    public static string Name = "", Phase = "";
    public static bool Active { get { return Name != ""; } }
    private static float startAt = -1f, yaw0;
    private static int phase, next, lands;
    private static bool done;

    private static void Say(string m) { Out.Say("SCENARIO " + m); }

    public static void Abort() { Restore(); Name = ""; }

    private static void Restore()
    {
        SkateKeys.Scripted = false; SkateKeys.Reset();
        SkateCamera.Fixed = false; SkateCamera.Chase = true; SkateRide.Frozen = false; SkateBoard.ShowLift = 0f; RiderRig.Enabled = true;
    }

    public static void Update(float now)
    {
        if (done) return;
        if (startAt < 0f)
        {
            if (now - SkateRig.AwakeAt < 4f) return;
            startAt = now; yaw0 = SkateCamera.LookYaw;
            SkateKeys.Scripted = true; SkateKeys.Reset(); SkateKeys.LookYaw = yaw0;
            Say(Name + " begins, view yaw=" + yaw0.ToString("F0") + " at " + Out.V1(SkateRig.LocalT.position));
        }
        var s = now - startAt;
        if (SkateRide.Lands != lands) { lands = SkateRide.Lands; Say("landed: " + SkateRide.LastAir + " | mode=" + SkateRide.Mode + " switch=" + SkateRide.Trick.Switch + " pos=" + Out.V1(SkateRide.Position)); }
        if (Name == "ride") Ride(s);
        else if (Name == "pose") Pose(s, now);
        else if (Name == "trick") Trick(s);
        else { Phase = "model dump"; ModelDump.Run(); Finish(); }
    }

    private static bool Enter(int to, string label)
    {
        if (to == phase) return false;
        if (phase > 0 && Name == "pose")
            Say("  rider: applied=" + RiderRig.Applied + " refused=" + RiderRig.Refusals + (RiderRig.Error != "" ? " (" + RiderRig.Error + ")" : "") + " miss=" + RiderRig.Miss.ToString("F3") + " at " + RiderRig.MissAt
                + " moved-after-write=" + RiderRig.Drift.ToString("F3") + " at " + RiderRig.DriftAt + " | look=" + RiderRig.LookState + (RiderRig.DriftNote != "" ? " | " + RiderRig.DriftNote : ""));
        phase = to; Phase = phase + " " + label;
        Say("phase " + Phase + " | " + SkateRide.Status());
        return true;
    }

    private static void Finish()
    {
        done = true; Phase = "done";
        SkateRide.Dismount("the scenario ended"); Restore();
        System.IO.File.WriteAllText(System.IO.Path.Combine(BepInEx.Paths.PluginPath, "probe.done"), "done");
        Say("DONE top speed=" + SkateRide.Top.ToString("F1") + " pull-backs=" + SkateRide.Resets + " late calls=" + LateDriver.Calls + " pops=" + SkateRide.Pops + " lands=" + SkateRide.Lands + " bails=" + SkateRide.Bails
            + " refused presses=" + SkateRide.Refused + " score=" + SkateRide.Trick.TotalPoints + " grind scans=" + SkateGrind.Scans + " audio=" + SkateSfx.State);
    }

    private static void Ride(float s)
    {
        var to = s < 2f ? 1 : s < 4f ? 2 : s < 5.5f ? 3 : s < 7.5f ? 4 : s < 9.5f ? 5 : s < 11.5f ? 6 : s < 13f ? 7 : s < 15f ? 8 : 9;
        if (!Enter(to, to == 1 ? "push out" : to == 2 ? "coast" : to == 3 ? "brake" : to == 4 ? "turn round" : to == 5 ? "push back" : to == 6 ? "coast" : to == 7 ? "brake" : to == 8 ? "dismount" : "finish")) return;
        if (phase == 1) { SkateKeys.LookYaw = yaw0 + 180f; SkateRig.MountNow(yaw0 + 180f); }
        SkateKeys.Push = phase == 1 || phase == 5;
        SkateKeys.Brake = phase == 3 || phase == 7;
        SkateKeys.LookYaw = phase <= 3 ? yaw0 + 180f : yaw0;
        if (phase == 8) SkateRide.Dismount("the scenario stepped off");
        if (phase == 9) Finish();
    }

    private static readonly string[] poseLabels = {
        "stance, seen from the rider's front", "stance, seen along the board from behind", "stance, seen from the rider's back", "stance, seen along the board from ahead", "stance, seen from above",
        "the game's own pose (ours off), from the same side", "the game's own pose (ours off), from behind",
        "push, foot on the ground", "push, seen from behind", "air, half a kickflip", "air, quarter flip with a grab", "air with a grab, seen from behind", "bail", "switch stance", "grind",
        "first person", "moving, pushing off", "moving, turning round", "finish" };
    private static readonly float[] poseBearing = { 90f, 180f, 270f, 0f, 90f, 90f, 180f, 90f, 180f, 120f, 90f, 180f, 90f, 90f, 90f, 90f, 115f, 115f, 90f };
    private const int PoseOurs = 6, PosePush = 8, PoseFlip = 10, PoseGrab = 11, PoseBail = 13, PoseSwitch = 14, PoseGrind = 15, PoseFirst = 16, PoseMove = 17, PoseTurn = 18, PoseEnd = 19;
    private static float stepAt;
    private static bool keyWas;

    private static void Pose(float s, float now)
    {
        var advance = phase == 0;
        try { var kb = Keyboard.current; var k = kb != null && kb.kKey.isPressed; if (k && !keyWas) advance = true; keyWas = k; }
        catch (Exception) { }
        if (now - stepAt > (phase >= PoseMove ? 7f : 45f)) advance = true;
        // The moving phases hold walking pace, which the server accepts without its skate plugin.
        if (phase >= PoseMove) SkateKeys.Push = SkateRide.Speed < 2.4f;
        if (phase == PoseBail) SkateRide.ModeAt = now - 0.3f;
        if (!advance) return;
        stepAt = now;
        Enter(phase + 1, poseLabels[phase] + (phase + 1 < PoseMove ? "   (K: next)" : ""));
        if (phase == 1)
        {
            ModelDump.Renderers();
            SkateRig.MountNow(yaw0);
            SkateRide.Frozen = true;
        }
        if (phase == PoseEnd) { Finish(); return; }
        SkateCamera.Fixed = phase != PoseFirst; SkateCamera.Chase = phase != PoseFirst;
        SkateCamera.Bearing = poseBearing[phase - 1];
        SkateCamera.FixedHeight = phase == 5 ? 2.9f : 1.15f; SkateCamera.FixedDistance = phase == 5 ? 1.6f : 2.7f;
        RiderRig.Enabled = phase != PoseOurs && phase != PoseOurs + 1;
        var air = phase == PoseFlip || phase == PoseGrab || phase == PoseGrab + 1;
        SkateRide.Mode = air ? RideMode.Air : phase == PoseBail ? RideMode.Bail : phase == PoseGrind ? RideMode.Grind : RideMode.Ground;
        SkateRide.PushPhase = phase == PosePush || phase == PosePush + 1 ? 0.45f : 0f;
        SkateRide.FlipDeg = phase == PoseFlip ? 180 : air ? 90 : 0;
        SkateRide.Grab = phase == PoseGrab || phase == PoseGrab + 1;
        SkateBoard.ShowLift = air ? 0.7f : 0f;
        SkateRide.Trick = new SkateTrickState(phase == PoseSwitch);
        if (phase == PoseMove) { SkateRide.Frozen = false; SkateRide.Mode = RideMode.Ground; }
        if (phase == PoseTurn) SkateKeys.LookYaw = yaw0 + 180f;
    }

    private static readonly float[] trickAt = { 3f, 6f, 8.9f, 9f, 9.4f, 11f, 14f, 16.9f, 17f, 17.5f, 20f, 20.12f, 23f, 23.45f, 26f, 28.5f };
    private static readonly string[] trickDo = { "ollie", "ollie+kick", "left", "ollie", "release", "turn", "ollie+heel", "right+grab", "ollie", "release", "ollie+kick", "kick", "ollie", "kick", "brake", "finish" };

    private static void Trick(float s)
    {
        if (phase == 0)
        {
            Enter(1, "riding");
            SkateRig.MountNow(yaw0);
            SkateCamera.Fixed = true; SkateCamera.Bearing = 100f; SkateCamera.FixedDistance = 4.4f; SkateCamera.FixedHeight = 1.4f;
        }
        SkateKeys.Push = !SkateKeys.Brake && SkateRide.Mode == RideMode.Ground && SkateRide.Speed < 2.4f;
        while (next < trickAt.Length && s >= trickAt[next])
        {
            var what = trickDo[next++];
            Phase = (next + 1) + " " + what;
            Say("press " + what + " | " + SkateRide.Status());
            if (what.Contains("ollie")) SkateKeys.JumpPressed = true;
            if (what.Contains("kick")) SkateKeys.FlipUp = true;
            if (what.Contains("heel")) SkateKeys.FlipDown = true;
            if (what.Contains("left")) { SkateKeys.Left = true; SkateKeys.LeftPressed = true; }
            if (what.Contains("right")) { SkateKeys.Right = true; SkateKeys.RightPressed = true; }
            if (what.Contains("grab")) SkateKeys.Crouch = true;
            if (what == "release") SkateKeys.Left = SkateKeys.Right = SkateKeys.Crouch = false;
            if (what == "turn") SkateKeys.LookYaw = yaw0 + 180f;
            if (what == "brake") SkateKeys.Brake = true;
            if (what == "finish") Finish();
        }
    }
}
