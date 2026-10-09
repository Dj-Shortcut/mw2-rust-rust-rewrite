// Loader probe plugin for issue #289: proves plugin load, an interop call into the game,
// and (optionally) class injection with a per-frame callback. Logs only; changes nothing.
using System;
using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using UnityEngine;

[BepInPlugin("claude.loaderprobe", "Loader Probe", "0.1.0")]
public class ProbePlugin : BasePlugin
{
    internal static ManualLogSource L;

    public override void Load()
    {
        L = Log;
        L.LogMessage("PROBE 1 load entered");
        try
        {
            L.LogMessage("PROBE 2 unityVersion=" + Application.unityVersion + " product=" + Application.productName + " version=" + Application.version);
        }
        catch (Exception e) { L.LogError("PROBE 2 failed: " + e); }

        if (System.IO.File.Exists(System.IO.Path.Combine(Paths.PluginPath, "inject.on")))
        {
            try
            {
                L.LogMessage("PROBE 3 registering behaviour");
                var c = AddComponent<ProbeBehaviour>();
                L.LogMessage("PROBE 4 behaviour added=" + (c != null));
            }
            catch (Exception e) { L.LogError("PROBE 3 failed: " + e); }
        }
        L.LogMessage("PROBE 9 load finished");
    }
}

public class ProbeBehaviour : MonoBehaviour
{
    public ProbeBehaviour(IntPtr p) : base(p) { }

    private int n;

    private void Update()
    {
        n++;
        if (n == 1 || n == 60 || n == 600 || n % 3000 == 0)
            ProbePlugin.L.LogMessage("PROBE 5 update frame " + n + " t=" + Time.realtimeSinceStartup);
    }
}
