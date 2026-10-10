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

[BepInPlugin("claude.loaderprobe", "Loader Probe", "0.3.0")]
public class ProbePlugin : BasePlugin
{
    internal static PLog L = new PLog();
    private static long lo, hi;

    public override void Load()
    {
        L.Inner = Log;
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
            case "world": WorldProbe.Push = false; return "added=" + (AddComponent<WorldProbe>() != null);
            case "worldpush": WorldProbe.Push = true; return "added=" + (AddComponent<WorldProbe>() != null);
            case "mute": WorldProbe.Mute = true; AudioListener.volume = 0f; return "volume=" + AudioListener.volume;
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

// Writes every line straight to plugins\\probe.log so a native crash cannot lose it.
public class PLog
{
    public ManualLogSource Inner;
    private readonly string file = System.IO.Path.Combine(Paths.PluginPath, "probe.log");
    public void LogMessage(string m) { System.IO.File.AppendAllText(file, m + "\r\n"); if (Inner != null) Inner.LogMessage(m); }
    public void LogError(string m) { System.IO.File.AppendAllText(file, "ERROR " + m + "\r\n"); if (Inner != null) Inner.LogError(m); }
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

// In-world probe. The game's own members are obfuscated in the client, so this uses only Unity
// engine API and game type names: it finds the local player object, shows a placeholder board
// and, when Push is set, drives the player's rigidbody for a few seconds at sprint-like speed
// to see whether the server accepts movement that does not come from the game's walk code.
public class WorldProbe : MonoBehaviour
{
    public WorldProbe(IntPtr p) : base(p) { }

    public static bool Push, Mute;
    private float nextLog, boardAt = -1f, pushStart = -1f, pushEnd = -1f, verdictAt = -1f;
    private int pushes;
    private BasePlayer local;
    private Rigidbody body;
    private PlayerWalkMovement walk;
    private GameObject board, marker;
    private Vector3 pushDir, before, after;
    private bool done;

    private static void Say(string m) { ProbePlugin.L.LogMessage("WORLD " + m); }
    private static string V(Vector3 v) { return v.x.ToString("F1") + "," + v.y.ToString("F1") + "," + v.z.ToString("F1"); }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        if (now < nextLog) return;
        nextLog = now + 2f;
        try { Tick(now); }
        catch (Exception e) { Say("tick threw: " + e.GetType().Name + ": " + e.Message); nextLog = now + 6f; }
    }

    private void Tick(float now)
    {
        if (Mute) AudioListener.volume = 0f;
        var cam = Camera.main;
        if (cam == null) { if (local != null) { Say("camera gone; local player cleared"); local = null; } return; }
        var camPos = cam.transform.position;
        if (local == null)
        {
            var players = UnityEngine.Object.FindObjectsOfType<BasePlayer>();
            var n = players == null ? 0 : players.Length;
            BasePlayer best = null; var bestD = 999f;
            for (var i = 0; i < n; i++)
            {
                var d = Vector3.Distance(players[i].transform.position, camPos);
                if (d < bestD) { bestD = d; best = players[i]; }
            }
            Say("cam=" + V(camPos) + " players=" + n + (best != null ? " nearest=" + bestD.ToString("F1") + "m name=" + best.gameObject.name : ""));
            if (best == null || bestD > 4f) return;
            local = best;
            body = local.GetComponent<Rigidbody>();
            walk = local.GetComponent<PlayerWalkMovement>();
            Say("LOCAL found pos=" + V(local.transform.position) + " rigidbody=" + (body != null) + (body != null ? " kinematic=" + body.isKinematic : "") + " walk=" + (walk != null) + (walk != null ? " walkEnabled=" + walk.enabled : ""));
            MakeBoard(cam);
            boardAt = now;
            return;
        }
        var pos = local.transform.position;
        Say("pos=" + V(pos) + (body != null ? " vel=" + V(body.linearVelocity) + " kinematic=" + body.isKinematic : "") + (walk != null ? " walkEnabled=" + walk.enabled : "") + " board=" + (board != null));
        if (!Push || done || body == null) return;
        if (pushStart < 0f && verdictAt < 0f && now - boardAt > 60f + pushes * 40f)
        {
            var f = cam.transform.forward; f.y = 0f;
            pushDir = f.sqrMagnitude > 0.01f ? f.normalized : Vector3.forward;
            before = pos; pushStart = now; pushEnd = now + 3f;
            Say("PUSH " + (pushes + 1) + " start from " + V(before) + " dir=" + V(pushDir) + " kinematic=" + body.isKinematic);
        }
        if (verdictAt > 0f && now >= verdictAt)
        {
            var moved = Vector3.Distance(before, after); var kept = Vector3.Distance(before, pos);
            Say("PUSH " + (pushes + 1) + " verdict: moved=" + moved.ToString("F1") + "m, 6s later still " + kept.ToString("F1") + "m from start => " + (moved < 1f ? "no movement" : kept > moved * 0.7f ? "KEPT (no snap-back seen)" : "SNAPPED BACK"));
            verdictAt = -1f; pushes++;
            if (pushes >= 2) { done = true; System.IO.File.WriteAllText(System.IO.Path.Combine(Paths.PluginPath, "probe.done"), "done"); Say("DONE"); }
        }
    }

    private void FixedUpdate()
    {
        if (pushStart < 0f || body == null) return;
        try
        {
            if (Time.realtimeSinceStartup < pushEnd)
            {
                if (walk != null) walk.enabled = false;
                var v = pushDir * 5f; v.y = body.linearVelocity.y;
                body.linearVelocity = v;
            }
            else
            {
                if (walk != null) walk.enabled = true;
                after = local.transform.position;
                Say("PUSH " + (pushes + 1) + " end at " + V(after));
                pushStart = -1f; verdictAt = Time.realtimeSinceStartup + 6f;
            }
        }
        catch (Exception e) { Say("push threw: " + e.GetType().Name + ": " + e.Message); pushStart = -1f; if (walk != null) walk.enabled = true; }
    }

    private void MakeBoard(Camera cam)
    {
        try
        {
            // Board-sized box on the ground ahead of the player, and a small marker at eye height.
            var fwd = cam.transform.forward; fwd.y = 0f; fwd = fwd.sqrMagnitude > 0.01f ? fwd.normalized : Vector3.forward;
            board = Box("skate_probe_board", new Vector3(0.22f, 0.04f, 0.8f), new Color(1f, 0.1f, 0.6f));
            board.transform.position = local.transform.position + fwd * 1.6f + Vector3.up * 0.12f;
            board.transform.rotation = Quaternion.LookRotation(fwd, Vector3.up);
            marker = Box("skate_probe_marker", new Vector3(0.25f, 0.25f, 0.25f), new Color(0.1f, 1f, 0.2f));
            marker.transform.position = cam.transform.position + cam.transform.forward * 2.2f;
            Say("BOARD created at " + V(board.transform.position) + " marker at " + V(marker.transform.position));
        }
        catch (Exception e) { Say("board threw: " + e.GetType().Name + ": " + e.Message); }
    }

    private static GameObject Box(string name, Vector3 size, Color color)
    {
        GameObject go;
        try
        {
            go = GameObject.CreatePrimitive(PrimitiveType.Cube);
            var col = go.GetComponent<Collider>();
            if (col != null) UnityEngine.Object.Destroy(col);
        }
        catch (Exception e)
        {
            Say("CreatePrimitive unavailable (" + e.GetType().Name + "); building a mesh");
            go = new GameObject();
            var mesh = new Mesh();
            mesh.vertices = new[] {
                new Vector3(-.5f,-.5f,-.5f), new Vector3(.5f,-.5f,-.5f), new Vector3(.5f,.5f,-.5f), new Vector3(-.5f,.5f,-.5f),
                new Vector3(-.5f,-.5f,.5f), new Vector3(.5f,-.5f,.5f), new Vector3(.5f,.5f,.5f), new Vector3(-.5f,.5f,.5f) };
            mesh.triangles = new[] { 0,2,1, 0,3,2, 4,5,6, 4,6,7, 0,1,5, 0,5,4, 2,3,7, 2,7,6, 1,2,6, 1,6,5, 0,4,7, 0,7,3 };
            mesh.RecalculateNormals(); mesh.RecalculateBounds();
            go.AddComponent<MeshFilter>().sharedMesh = mesh;
            go.AddComponent<MeshRenderer>();
        }
        go.name = name;
        go.transform.localScale = size;
        var r = go.GetComponent<Renderer>();
        Shader sh = null;
        foreach (var n in new[] { "Hidden/Internal-Colored", "Sprites/Default", "Unlit/Color", "Standard" }) { sh = Shader.Find(n); if (sh != null) { Say("shader " + n); break; } }
        if (sh != null) { var m = new Material(sh); m.color = color; r.material = m; }
        else Say("no known shader found; keeping the default material");
        UnityEngine.Object.DontDestroyOnLoad(go);
        return go;
    }
}
