// Entry point of the Shortcut skate client, a BepInEx IL2CPP plugin for the Rust PC client.
// The launch script writes plugins\steps.txt (comma separated) to choose what runs:
//   skate        the playable mod
//   skatetest    a scripted ride, out and back, that ends the session by itself
//   posetest     the rider and board in each state, standing still and moving, from fixed cameras
//   tricktest    a scripted ride with ollies, flips, spins and a grab
//   modeldump    log what the rider model is made of
//   mute         silence the game
//   nogrind      leave edge grinding off
//   api, ptr, max, uver, prod, frame, plat, log, inject   loader diagnostics from issue #289
// Every line also goes straight to plugins\probe.log, so a native crash cannot lose it.
using System;
using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using UnityEngine;

[BepInPlugin("shortcut.skate.client", "Shortcut Skate Client", SkatePlugin.Version)]
public class SkatePlugin : BasePlugin
{
    public const string Version = "0.11.0";
    internal static SkatePlugin Instance;

    public override void Load()
    {
        Instance = this;
        Out.Inner = Log;
        var path = System.IO.Path.Combine(Paths.PluginPath, "steps.txt");
        var steps = System.IO.File.Exists(path) ? System.IO.File.ReadAllText(path).Trim() : "skate";
        Out.Say("PROBE load entered steps=" + steps);
        foreach (var s in steps.Split(new[] { ',' }, StringSplitOptions.RemoveEmptyEntries))
        {
            Out.Say("STEP " + s + " begin");
            try { Out.Say("STEP " + s + " ok: " + Run(s.Trim())); }
            catch (Exception e) { Out.Err("STEP " + s + " threw: " + e); }
        }
        Out.Say("PROBE load finished");
    }

    private string Run(string s)
    {
        switch (s)
        {
            case "skate": return "added=" + (AddComponent<SkateRig>() != null);
            case "skatetest": Scenarios.Name = "ride"; return "added=" + (AddComponent<SkateRig>() != null);
            case "posetest": Scenarios.Name = "pose"; return "added=" + (AddComponent<SkateRig>() != null);
            case "tricktest": Scenarios.Name = "trick"; return "added=" + (AddComponent<SkateRig>() != null);
            case "nogrind": SkateGrind.Enabled = false; return "grinding off";
            case "modeldump": Scenarios.Name = "dump"; return "added=" + (AddComponent<SkateRig>() != null);
            case "mute": SkateRig.Mute = true; AudioListener.volume = 0f; return "volume=" + AudioListener.volume;
            default: return LoaderSteps.Run(this, s);
        }
    }
}

public static class Out
{
    public static ManualLogSource Inner;
    private static readonly string file = System.IO.Path.Combine(Paths.PluginPath, "probe.log");

    public static void Say(string m) { System.IO.File.AppendAllText(file, m + "\r\n"); if (Inner != null) Inner.LogMessage(m); }
    public static void Err(string m) { System.IO.File.AppendAllText(file, "ERROR " + m + "\r\n"); if (Inner != null) Inner.LogError(m); }
    public static string V(Vector3 v) { return v.x.ToString("F2") + "," + v.y.ToString("F2") + "," + v.z.ToString("F2"); }
    public static string V1(Vector3 v) { return v.x.ToString("F1") + "," + v.y.ToString("F1") + "," + v.z.ToString("F1"); }
    public static string V3(Vector3 v) { return v.x.ToString("F3") + "," + v.y.ToString("F3") + "," + v.z.ToString("F3"); }

    // One engine call that may be missing from the client build: report it instead of losing the rest.
    public static string Try(Func<string> f)
    {
        try { return f(); }
        catch (Exception e) { return "unavailable(" + (e.Message != null && e.Message.Contains("unstripping") ? "stripped" : e.GetType().Name + ": " + e.Message) + ")"; }
    }

    public static string Chain(Transform t)
    {
        var s = ""; var p = t.parent; var n = 0;
        while (p != null && n++ < 8) { s = p.gameObject.name + "/" + s; p = p.parent; }
        return s == "" ? "(root)" : s;
    }
}
