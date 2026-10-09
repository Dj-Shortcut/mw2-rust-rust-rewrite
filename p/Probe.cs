// Loader probe plugin for issue #289. Runs the steps listed in plugins\steps.txt (comma separated),
// logging before and after each one so a native crash identifies the failing step. Logs only.
using System;
using System.Diagnostics;
using System.Reflection;
using System.Runtime.InteropServices;
using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using Il2CppInterop.Runtime;
using UnityEngine;

[BepInPlugin("claude.loaderprobe", "Loader Probe", "0.2.0")]
public class ProbePlugin : BasePlugin
{
    internal static ManualLogSource L;
    private static long lo, hi;

    public override void Load()
    {
        L = Log;
        var path = System.IO.Path.Combine(Paths.PluginPath, "steps.txt");
        var steps = System.IO.File.Exists(path) ? System.IO.File.ReadAllText(path).Trim() : "";
        L.LogMessage("PROBE load entered steps=" + steps);
        foreach (ProcessModule m in Process.GetCurrentProcess().Modules)
            if (m.ModuleName.Equals("GameAssembly.dll", StringComparison.OrdinalIgnoreCase))
            { lo = (long)m.BaseAddress; hi = lo + m.ModuleMemorySize; }
        L.LogMessage("PROBE GameAssembly range=" + lo.ToString("X") + "-" + hi.ToString("X"));
        foreach (var s in steps.Split(new[] { ',' }, StringSplitOptions.RemoveEmptyEntries))
        {
            L.LogMessage("STEP " + s + " begin");
            try { L.LogMessage("STEP " + s + " ok: " + Run(s.Trim())); }
            catch (Exception e) { L.LogError("STEP " + s + " threw: " + e); }
        }
        L.LogMessage("PROBE load finished");
    }

    private string Run(string s)
    {
        switch (s)
        {
            case "api": return Api();
            case "ptr": return Ptrs(typeof(Application), "get_productName") + " || " + Ptrs(typeof(Application), "get_unityVersion") + " || " + Ptrs(typeof(Il2CppSystem.Math), "Max_Public_Static_Int32_Int32_Int32");
            case "max": return Il2CppSystem.Math.Max(3, 5).ToString();
            case "uver": return Application.unityVersion;
            case "prod": return Application.productName;
            case "frame": return Time.frameCount.ToString();
            case "plat": return Application.platform.ToString();
            case "log": UnityEngine.Debug.Log("probe debug log"); return "logged";
            case "inject": return "added=" + (AddComponent<ProbeBehaviour>() != null);
            default: return "unknown step";
        }
    }

    private static string Where(IntPtr p)
    {
        var v = (long)p;
        return v.ToString("X") + (v >= lo && v < hi ? "(GA+" + (v - lo).ToString("X") + ")" : "(out)");
    }

    private static string Describe(IntPtr mi)
    {
        if (mi == IntPtr.Zero) return "null";
        var name = Marshal.PtrToStringAnsi(IL2CPP.il2cpp_method_get_name(mi));
        var cls = Marshal.PtrToStringAnsi(IL2CPP.il2cpp_class_get_name(IL2CPP.il2cpp_method_get_class(mi)));
        var raw = "";
        for (var i = 0; i < 6; i++) raw += " q" + i + "=" + Where(Marshal.ReadIntPtr(mi, i * 8));
        return cls + "::" + name + " params=" + IL2CPP.il2cpp_method_get_param_count(mi) + " token=" + IL2CPP.il2cpp_method_get_token(mi).ToString("X") + raw;
    }

    private static string Ptrs(Type t, string part)
    {
        var r = "";
        foreach (var f in t.GetFields(BindingFlags.NonPublic | BindingFlags.Public | BindingFlags.Static))
            if (f.Name.StartsWith("NativeMethodInfoPtr_") && f.Name.Contains(part))
                r += "[" + f.Name + " -> " + Describe((IntPtr)f.GetValue(null)) + "]";
        return r == "" ? "no field for " + part : r;
    }

    private static string Api()
    {
        var klass = Il2CppClassPointerStore<Application>.NativeClassPtr;
        var r = "class=" + Where(klass) + " name=" + Marshal.PtrToStringAnsi(IL2CPP.il2cpp_class_get_name(klass));
        var iter = IntPtr.Zero; IntPtr m; var n = 0;
        while ((m = IL2CPP.il2cpp_class_get_methods(klass, ref iter)) != IntPtr.Zero)
        {
            n++;
            var name = Marshal.PtrToStringAnsi(IL2CPP.il2cpp_method_get_name(m));
            if (name == "get_productName" || name == "get_unityVersion") r += " {" + Describe(m) + "}";
        }
        return r + " methods=" + n;
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
            ProbePlugin.L.LogMessage("PROBE update frame " + n + " t=" + Time.realtimeSinceStartup);
    }
}
