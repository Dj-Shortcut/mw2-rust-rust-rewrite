using System;
using Shortcut.RustMod;
using UnityEngine;
using UnityEngine.InputSystem;

public static class Scenarios
{
    public static string Name = "", Phase = "";
    public static bool Active { get { return Name != ""; } }
    // A check that plays a controller rides through the plugin's own input, getting on and off included.
    public static bool Plays { get { return Name == "pad"; } }
    private static float startAt = -1f, startGame, yaw0;
    private static int phase, next, lands;
    private static bool done;

    private static void Say(string m) { Out.Say("SCENARIO " + m); }

    public static void Abort() { Restore(); SkatePad.FeedEnd(); Name = ""; }

    private static void Restore()
    {
        SkateKeys.Scripted = false; SkateKeys.Reset();
        SkateCamera.Fixed = false; SkateCamera.Chase = true; SkateRide.Frozen = false; SkateRide.Manual = false; SkateBoard.ShowLift = 0f;
    }

    public static void Update(float now)
    {
        if (done) return;
        if (startAt < 0f)
        {
            if (now - SkateRig.AwakeAt < 4f) return;
            startAt = now; startGame = Time.time; yaw0 = SkateCamera.LookYaw;
            SkateKeys.Scripted = true; SkateKeys.Reset(); SkateKeys.LookYaw = yaw0;
            Say(Name + " begins, view yaw=" + yaw0.ToString("F0") + " at " + Out.V1(SkateRig.LocalT.position));
        }
        // The checks that ride are timed in the game's own seconds: a client that draws few frames
        // runs slower than the clock, and presses timed by the clock would come before their turn.
        var s = now - startAt; var game = Time.time - startGame;
        SkateSfx.Listen();
        if (SkateRide.Lands != lands) { lands = SkateRide.Lands; Say("landed: " + SkateRide.LastAir + " | mode=" + SkateRide.Mode + " switch=" + SkateRide.Trick.Switch + " pos=" + Out.V1(SkateRide.Position)); }
        if (Name == "ride") Ride(game);
        else if (Name == "pose") Pose(s, now);
        else if (Name == "pad") Pad(game, now);
        else Trick(game);
    }

    private static bool Enter(int to, string label)
    {
        if (to == phase) return false;
        if (phase > 0 && Name == "pose")
            Say("  rider: applied=" + RiderRig.Applied + " refused=" + RiderRig.Refusals + (RiderRig.Error != "" ? " (" + RiderRig.Error + ")" : "") + " miss=" + RiderRig.Miss.ToString("F3") + " at " + RiderRig.MissAt
                + " | look=" + RiderRig.LookState);
        phase = to; Phase = phase + " " + label;
        Say("phase " + Phase + " | " + SkateRide.Status());
        return true;
    }

    private static void Finish()
    {
        done = true; Phase = "done";
        SkateRide.Dismount("the scenario ended"); Restore();
        System.IO.File.WriteAllText(System.IO.Path.Combine(BepInEx.Paths.PluginPath, "skate.done"), "done");
        var real = Time.realtimeSinceStartup - startAt;
        Say("DONE pace=" + (real > 0.5f ? (Time.time - startGame) / real : 1f).ToString("F2") + " top speed=" + SkateRide.Top.ToString("F1") + " pull-backs=" + SkateRide.Resets + " late calls=" + LateDriver.Calls + " pops=" + SkateRide.Pops + " lands=" + SkateRide.Lands + " bails=" + SkateRide.Bails
            + " refused presses=" + SkateRide.Refused + " score=" + SkateRide.Trick.TotalPoints + " grind scans=" + SkateGrind.Scans + " flicks=" + SkatePad.Flicks + " audio=" + SkateSfx.State + " | loudest output: " + SkateSfx.Heard);
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

    private struct View
    {
        public readonly string Label, State; public readonly float Bearing, Value;
        public View(string label, float bearing, string state, float value) { Label = label; Bearing = bearing; State = state; Value = value; }
    }

    // Value is the point in the push for a push view, the board's flip in degrees for a flip view
    // and the part of the bail that has passed for a bail view.
    private static readonly View[] views = {
        new View("stance, seen from the rider's front", 90f, "stance", 0f), new View("stance, seen along the board from behind", 180f, "stance", 0f),
        new View("stance, seen from the rider's back", 270f, "stance", 0f), new View("stance, seen along the board from ahead", 0f, "stance", 0f),
        new View("stance, seen from above", 90f, "above", 0f),
        new View("push, foot coming down", 90f, "push", 0.2f), new View("push, foot on the ground", 90f, "push", 0.45f),
        new View("push, foot on the ground, seen from behind", 180f, "push", 0.45f), new View("push, foot leaving the ground", 90f, "push", 0.7f),
        new View("air, half a kickflip", 120f, "flip", 180f), new View("air, quarter flip", 90f, "flip", 90f),
        new View("air with a grab", 90f, "grab", 0f), new View("air with a grab, seen from behind", 180f, "grab", 0f),
        new View("bail, stumbling", 90f, "bail", 0.2f), new View("bail, going down", 90f, "bail", 0.45f),
        new View("bail, down", 90f, "bail", 0.62f), new View("bail, getting up", 90f, "bail", 0.85f),
        new View("switch stance", 90f, "switch", 0f), new View("grind", 90f, "grind", 0f),
        new View("manual", 90f, "manual", 0f), new View("manual, seen from behind", 180f, "manual", 0f),
        new View("first person", 90f, "first", 0f),
        new View("moving, pushing off", 115f, "move", 0f), new View("moving, turning round", 115f, "turn", 0f),
        new View("finish", 90f, "end", 0f) };
    private static float stepAt;
    private static bool keyWas;

    private static void Pose(float s, float now)
    {
        var advance = phase == 0;
        try { var kb = Keyboard.current; var k = kb != null && SkateKeys.Down(kb.kKey); if (k && !keyWas) advance = true; keyWas = k; }
        catch (Exception) { }
        var state = phase == 0 ? "" : views[phase - 1].State;
        var moving = state == "move" || state == "turn";
        if (now - stepAt > (moving ? 7f : 45f)) advance = true;
        // The moving views hold walking pace, which the server accepts without its skate plugin.
        if (moving) SkateKeys.Push = SkateRide.Speed < 2.4f;
        if (state == "bail") SkateRide.ModeAt = now - views[phase - 1].Value * SkateRide.BailSeconds;
        if (!advance) return;
        stepAt = now;
        var v = views[phase];
        var still = v.State != "move" && v.State != "turn" && v.State != "end";
        Enter(phase + 1, v.Label + (still ? "   (K: next)" : ""));
        if (phase == 1) { SkateRig.MountNow(yaw0); SkateRide.Frozen = true; }
        if (v.State == "end") { Finish(); return; }
        SkateCamera.Fixed = v.State != "first"; SkateCamera.Chase = v.State != "first";
        SkateCamera.Bearing = v.Bearing;
        SkateCamera.FixedHeight = v.State == "above" ? 2.9f : 1.15f; SkateCamera.FixedDistance = v.State == "above" ? 1.6f : 2.7f;
        var air = v.State == "flip" || v.State == "grab";
        SkateRide.Mode = air ? RideMode.Air : v.State == "bail" ? RideMode.Bail : v.State == "grind" ? RideMode.Grind : RideMode.Ground;
        SkateRide.PushPhase = v.State == "push" ? v.Value : 0f;
        SkateRide.FlipDeg = v.State == "flip" ? (int)v.Value : 0;
        SkateRide.Grab = v.State == "grab";
        SkateRide.Manual = v.State == "manual";
        SkateBoard.ShowLift = air ? 0.7f : 0f;
        SkateRide.Trick = new SkateTrickState(v.State == "switch");
        if (v.State == "bail") SkateRide.ModeAt = now - v.Value * SkateRide.BailSeconds;
        if (v.State == "move") { SkateRide.Frozen = false; SkateRide.Mode = RideMode.Ground; }
        if (v.State == "turn") SkateKeys.LookYaw = yaw0 + 180f;
    }

    // The ride with a controller, without one: its states are played into the plugin as the reader
    // outside the game would share them, and the keyboard is left to itself. The pace is walking
    // pace, which the server accepts without its skate plugin.
    private const string OlliePath = "0:0,-100 250:0,-20 262:0,60 274:0,100 330:0,0", KickflipPath = "0:0,-100 250:-20,-30 262:-55,45 274:-70,70 330:0,0",
        HeelflipPath = "0:0,-100 250:20,-30 262:55,45 274:70,70 330:0,0";
    private static readonly float[] padAt = { 1f, 2f, 4.5f, 5.5f, 6.5f, 8f, 10.5f, 13f, 15.5f, 16.9f, 17.6f, 17.75f, 18.4f, 20f, 22f, 23f };
    private static readonly string[] padDo = { "y", "push", "right", "left", "straight", "ollie", "kickflip", "heelflip", "manual", "release", "ollie", "grab", "release", "brake", "y", "finish" };
    private static long padUs = 1000000;
    private static float padLast, padX, padRightY, padTrigger, padTapUntil;
    private static bool padPush, padBrake;

    private static void Pad(float s, float now)
    {
        if (phase == 0)
        {
            Enter(1, "controller");
            SkateKeys.Scripted = false; padLast = now;
            SkateCamera.Fixed = false; SkateCamera.Chase = true;
        }
        padUs += (long)((now - padLast) * 1000000f); padLast = now;
        while (next < padAt.Length && s >= padAt[next])
        {
            var what = padDo[next++];
            Phase = (next + 1) + " " + what;
            Say("controller " + what + " | " + SkateRide.Status() + " rides=" + (SkateKeys.Pad ? "controller" : "keyboard") + " flicks=" + SkatePad.Flicks);
            if (what == "y") padTapUntil = now + 0.12f;
            if (what == "push") padPush = true;
            if (what == "right") padX = 0.7f;
            if (what == "left") padX = -0.7f;
            if (what == "straight") padX = 0f;
            if (what == "ollie") Flick(OlliePath, now);
            if (what == "kickflip") Flick(KickflipPath, now);
            if (what == "heelflip") Flick(HeelflipPath, now);
            if (what == "manual") padRightY = -0.45f;
            if (what == "grab") padTrigger = 1f;
            if (what == "release") { padRightY = 0f; padTrigger = 0f; }
            if (what == "brake") { padBrake = true; padPush = false; }
            if (what == "finish") { SkatePad.FeedEnd(); Finish(); return; }
        }
        var buttons = (padPush && SkateRide.On && SkateRide.Mode == RideMode.Ground && SkateRide.Speed < 2.2f ? SkatePad.A : 0) | (padBrake ? SkatePad.B : 0) | (now < padTapUntil ? SkatePad.Y : 0);
        SkatePad.Feed(padUs, buttons, 0f, padTrigger, padX, 0f, 0f, padRightY, now);
    }

    // The right stick along a path, milliseconds:x,y in percent, played between two frames.
    private static void Flick(string path, float now)
    {
        var start = padUs;
        foreach (var part in path.Split(' '))
        {
            var a = part.Split(':'); var xy = a[1].Split(',');
            padUs = start + long.Parse(a[0]) * 1000;
            SkatePad.Feed(padUs, 0, 0f, padTrigger, padX, 0f, int.Parse(xy[0]) / 100f, int.Parse(xy[1]) / 100f, now);
        }
    }

    private static readonly float[] trickAt = { 3f, 6f, 8.9f, 9f, 9.4f, 11f, 14f, 16.9f, 17f, 17.5f, 20f, 20.12f, 23f, 23.45f, 24.6f, 26.2f, 28f, 30.5f };
    private static readonly string[] trickDo = { "ollie", "ollie+kick", "left", "ollie", "release", "turn", "ollie+heel", "right+grab", "ollie", "release", "ollie+kick", "kick", "ollie", "kick", "manual", "release", "brake", "finish" };

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
            if (what == "manual") SkateKeys.Manual = true;
            if (what == "release") SkateKeys.Left = SkateKeys.Right = SkateKeys.Crouch = SkateKeys.Manual = false;
            if (what == "turn") SkateKeys.LookYaw = yaw0 + 180f;
            if (what == "brake") SkateKeys.Brake = true;
            if (what == "finish") Finish();
        }
    }
}
