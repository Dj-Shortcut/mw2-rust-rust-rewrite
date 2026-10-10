// Scripted sessions for unattended checks. A scenario starts four seconds after the player stands
// up, drives the ride with synthetic input, names its phase on screen so screenshots explain
// themselves, and ends the session by writing plugins\probe.done.
//   ride   out along the reverse of the view direction and back: push, coast, brake, turn
//   pose   slow ride that shows the rider pose first person, from a side camera with the pose on
//          and off, and from the chase camera; then tries the console command route by changing
//          the time of day, and leaves half a minute to inspect the console by hand
//   dump   log what the rider model is made of
using System;
using UnityEngine;

public static class Scenarios
{
    public static string Name = "", Phase = "";
    public static bool Active { get { return Name != ""; } }
    private static float startAt = -1f, yaw0;
    private static int phase;
    private static bool done;

    private static void Say(string m) { Out.Say("SCENARIO " + m); }

    public static void Update(float now)
    {
        if (done) return;
        if (startAt < 0f)
        {
            if (now - SkateRig.AwakeAt < 4f) return;
            startAt = now; yaw0 = SkateCamera.LookYaw;
            Say(Name + " begins, view yaw=" + yaw0.ToString("F0") + " at " + Out.V1(SkateRig.LocalT.position));
        }
        var s = now - startAt;
        if (Name == "ride") Ride(s);
        else if (Name == "pose") Pose(s);
        else { Phase = "model dump"; ModelDump.Run(); Finish(); }
        SideCamera.Follow();
    }

    private static bool Enter(int next, string label)
    {
        if (next == phase) return false;
        phase = next; Phase = phase + " " + label;
        Say("phase " + Phase + " | " + SkateRide.Status());
        return true;
    }

    private static void Finish()
    {
        done = true; Phase = "done";
        SkateRide.Dismount(); SkateRide.Synth = false; SkateCamera.Chase = false; SideCamera.Set(false);
        System.IO.File.WriteAllText(System.IO.Path.Combine(BepInEx.Paths.PluginPath, "probe.done"), "done");
        Say("DONE top speed=" + SkateRide.Top.ToString("F1") + " pull-backs=" + SkateRide.Resets + " late calls=" + LateDriver.Calls);
    }

    private static void Ride(float s)
    {
        var next = s < 2f ? 1 : s < 4f ? 2 : s < 5.5f ? 3 : s < 7.5f ? 4 : s < 9.5f ? 5 : s < 11.5f ? 6 : s < 13f ? 7 : s < 15f ? 8 : 9;
        if (!Enter(next, next == 1 ? "push out" : next == 2 ? "coast" : next == 3 ? "brake" : next == 4 ? "turn round" : next == 5 ? "push back" : next == 6 ? "coast" : next == 7 ? "brake" : next == 8 ? "dismount" : "finish")) return;
        if (phase == 1) { SkateRide.Synth = true; SkateRide.SynthYaw = yaw0 + 180f; SkateRig.MountNow(yaw0 + 180f); }
        SkateRide.SynthPush = phase == 1 || phase == 5;
        SkateRide.SynthBrake = phase == 3 || phase == 7;
        SkateRide.SynthYaw = phase <= 3 ? yaw0 + 180f : yaw0;
        if (phase == 8) SkateRide.Dismount();
        if (phase == 9) Finish();
    }

    private static void Pose(float s)
    {
        var next = s < 5f ? 1 : s < 11f ? 2 : s < 16f ? 3 : s < 26f ? 4 : s < 30f ? 5 : s < 36f ? 6 : s < 42f ? 7 : s < 46f ? 8 : s < 76f ? 9 : 10;
        // Hold walking pace, which the server accepts without its skate plugin.
        if (phase >= 1 && phase <= 4) SkateRide.SynthPush = SkateRide.Speed < 2.4f;
        if (!Enter(next, next == 1 ? "first person, pose on" : next == 2 ? "side camera, pose on" : next == 3 ? "side camera, pose off" : next == 4 ? "chase camera, pose on, turning round"
            : next == 5 ? "chase camera, braking" : next == 6 ? "console: midnight through onSubmit" : next == 7 ? "console: noon through onEndEdit" : next == 8 ? "console: noon through onSubmit"
            : next == 9 ? "idle, inspect by hand" : "finish")) return;
        switch (phase)
        {
            case 1: SkateRide.Synth = true; SkateRide.SynthBrake = false; SkateRide.SynthYaw = yaw0; SkateRig.MountNow(yaw0); RiderRig.Enabled = true; break;
            case 2: SideCamera.Set(true); break;
            case 3: RiderRig.Enabled = false; break;
            case 4: RiderRig.Enabled = true; SideCamera.Set(false); SkateCamera.Chase = true; SkateRide.SynthYaw = yaw0 + 180f; break;
            case 5: SkateRide.SynthPush = false; SkateRide.SynthBrake = true; break;
            case 6: Say("console " + GameConsole.Send("env.time 0", false)); break;
            case 7: Say("console " + GameConsole.Send("env.time 12", true)); break;
            case 8: Say("console " + GameConsole.Send("env.time 12", false)); break;
            case 9: SkateRide.Dismount(); SkateRide.Synth = false; SkateCamera.Chase = false; break;
            default: Finish(); break;
        }
    }
}

// A second camera beside the rider, for looking at the pose from outside in test sessions.
// It renders the whole scene a second time, so play uses the chase camera instead.
public static class SideCamera
{
    public static bool On { get { return cam != null; } }
    private static Camera cam;

    public static void Set(bool on)
    {
        if (!on) { if (cam != null) { UnityEngine.Object.Destroy(cam.gameObject); cam = null; } return; }
        if (cam != null) return;
        var main = Camera.main;
        if (main == null) return;
        var go = new GameObject("skate_side_camera");
        var c = go.AddComponent<Camera>();
        c.CopyFrom(main);
        c.depth = main.depth + 5f;
        UnityEngine.Object.DontDestroyOnLoad(go);
        cam = c;
    }

    public static void Follow()
    {
        if (cam == null || SkateRig.LocalT == null) return;
        var p = SkateRig.LocalT.position;
        var f = SkateRide.Dir(SkateRide.Yaw);
        var side = SkateRide.Dir(SkateRide.Yaw + 90f);
        cam.transform.position = p + side * 2.6f + Vector3.up * 1.25f + f * 1.1f;
        cam.transform.LookAt(p + Vector3.up * 0.85f);
    }
}

// On-screen text (IMGUI, which probe run M showed to work in this client). English only.
public static class SkateHud
{
    private static GUIStyle style;

    public static void Draw()
    {
        if (!Scenarios.Active && !SkateRide.On) return;
        if (style == null)
        {
            style = new GUIStyle(GUI.skin.label);
            style.fontSize = 26; style.fontStyle = FontStyle.Bold;
            style.normal.textColor = Color.white;
        }
        if (Scenarios.Active) GUI.Label(new Rect(40f, 110f, 1500f, 44f), "SKATE TEST  " + Scenarios.Phase, style);
        else GUI.Label(new Rect(40f, 110f, 600f, 44f), "SKATE  " + (Math.Abs(SkateRide.Speed) * 3.6f).ToString("F0") + " km/h", style);
    }
}
