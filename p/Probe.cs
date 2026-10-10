// Loader probe plugin for issue #289. Runs the steps listed in plugins\steps.txt (comma separated),
// logging before and after each one so a native crash identifies the failing step. Logs only.
using System;
using System.Diagnostics;
using System.Reflection;
using System.Runtime.InteropServices;
using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using HarmonyLib;
using Il2CppInterop.Runtime;
using Il2CppInterop.Runtime.Attributes;
using UnityEngine;

[BepInPlugin("claude.loaderprobe", "Loader Probe", "0.6.0")]
public class ProbePlugin : BasePlugin
{
    internal static PLog L = new PLog();
    internal static ProbePlugin Instance;
    private static long lo, hi;

    public override void Load()
    {
        L.Inner = Log; Instance = this;
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
            case "move": return "added=" + (AddComponent<MoveProbe>() != null);
            case "mute": MoveProbe.Mute = true; AudioListener.volume = 0f; return "volume=" + AudioListener.volume;
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

// In-world movement probe. The game's own members are obfuscated in the client, so this uses only
// Unity engine API, game type names and Unity message names. What earlier runs established
// (10 October 2026, build 25824447):
//   - The local player is moved by a separate root object, assets/prefabs/player/player_movement.prefab:
//     PlayerWalkMovement, a dynamic Rigidbody (mass 0.5, no engine gravity, rotation frozen) and a
//     CapsuleCollider (height 1.8, radius 0.5). The BasePlayer object only follows it.
//   - With the walk component enabled, a velocity written from an ordinary FixedUpdate is gone by
//     the next step: the game's own fixed step runs later and cancels it.
//   - With the walk component disabled the body keeps the velocity, but the player is pulled back
//     toward where the component was switched off, several times a second.
// So this run keeps the walk component on and writes the velocity after the game's fixed step, in
// two ways: from a component created after the walk component exists, and from a Harmony postfix
// on BaseMovement.FixedUpdate. Timeline, in seconds after the player stands up:
//   push 1 at 6 (late component), patch at 16, push 2 at 18 (postfix), then with whichever moved
//   the player further: 3 at 30 (walking pace), 4 at 42 (faster than sprinting), 5 at 54 (upward
//   kick), 6 at 66 (one shove, then left to the game). Done at 80.
public class MoveProbe : MonoBehaviour
{
    public MoveProbe(IntPtr p) : base(p) { }

    public static bool Mute;
    private static readonly string[] Modes = { "", "5 m/s from a late component", "5 m/s from a postfix on the walk step", "2.5 m/s, walking pace",
        "8 m/s, faster than sprinting", "5 m/s plus upward kick", "one 5 m/s shove, then left to the game" };
    private static readonly float[] Speeds = { 0f, 5f, 5f, 2.5f, 8f, 5f, 5f };
    private readonly float[] movedBy = new float[7];
    private float nextTick, nextTrace, lastTraceAt, wakeAt = -1f, pushEnd = -1f, verdictAt = -1f;
    private int ticks, standTicks, stage, lastMode;
    private BasePlayer local;
    private Transform localT, moveT;
    private PlayerWalkMovement walk;
    private Rigidbody body;
    private GameObject board;
    private bool trace, done, patched, lateAdded;
    private Vector3 camBefore, camAfter, lastTraceCam;

    private static void Say(string m) { ProbePlugin.L.LogMessage("MOVE " + m); }
    private static string V(Vector3 v) { return v.x.ToString("F2") + "," + v.y.ToString("F2") + "," + v.z.ToString("F2"); }
    private static float H(Vector3 a, Vector3 b) { a.y = 0f; b.y = 0f; return Vector3.Distance(a, b); }
    [HideFromIl2Cpp]
    private string Rel(float now) { return (wakeAt > 0f ? now - wakeAt : 0f).ToString("F1"); }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        try
        {
            if (pushEnd > 0f && now >= pushEnd) EndPush(now);
            if (trace && now >= nextTrace) { nextTrace = now + 0.25f; Trace(now); }
        }
        catch (Exception e) { Say("update threw: " + e.GetType().Name + ": " + e.Message); Driver.Active = false; pushEnd = -1f; }
        if (now < nextTick) return;
        nextTick = now + 1f; ticks++;
        try { Tick(now); }
        catch (Exception e) { Say("tick threw: " + e.GetType().Name + ": " + e.Message); nextTick = now + 5f; }
    }

    [HideFromIl2Cpp]
    private void Tick(float now)
    {
        if (Mute) AudioListener.volume = 0f;
        var cam = Camera.main;
        if (cam == null) { if (local != null && ticks % 10 == 0) Say("no main camera"); return; }
        var cp = cam.transform.position;
        if (local == null)
        {
            var players = UnityEngine.Object.FindObjectsOfType<BasePlayer>();
            var n = players == null ? 0 : players.Length;
            BasePlayer best = null; var bestD = 999f;
            for (var i = 0; i < n; i++)
            {
                var d = Vector3.Distance(players[i].transform.position, cp);
                if (d < bestD) { bestD = d; best = players[i]; }
            }
            if (ticks % 20 == 1) Say("cam=" + V(cp) + " players=" + n + (best != null ? " nearest=" + bestD.ToString("F1") + "m" : ""));
            if (best == null || bestD > 4f) return;
            var walks = UnityEngine.Object.FindObjectsOfType<PlayerWalkMovement>();
            PlayerWalkMovement w = null; var wd = 999f;
            for (var i = 0; i < walks.Length; i++)
            {
                var d = Vector3.Distance(walks[i].transform.position, cp);
                if (d < wd) { wd = d; w = walks[i]; }
            }
            if (w == null || wd > 4f) { if (ticks % 5 == 0) Say("player found but no walk component near the camera yet (count=" + walks.Length + ")"); return; }
            local = best; localT = best.transform; walk = w; moveT = w.transform; body = w.gameObject.GetComponent<Rigidbody>();
            Say("LOCAL found player=" + V(localT.position) + " movement object at " + V(moveT.position) + " body=" + (body != null) + " cam=" + V(cp));
            MakeBoard(cam);
            return;
        }
        var up = cp.y - localT.position.y;
        if (wakeAt < 0f)
        {
            standTicks = up > 1.0f ? standTicks + 1 : 0;
            if (ticks % 10 == 0) Say("waiting for the player to stand: camera height=" + up.ToString("F2"));
            if (standTicks >= 2) { wakeAt = now; trace = true; Say("AWAKE camera height=" + up.ToString("F2") + " player=" + V(localT.position)); }
            return;
        }
        if (body == null) { if (!done) { done = true; Say("no rigidbody on the movement object; nothing to push"); Finish(); } return; }
        var t = now - wakeAt;
        if (verdictAt > 0f && now >= verdictAt) Verdict(cam);
        if (!lateAdded && t >= 3f)
        {
            lateAdded = true;
            Say("LATE component: adding");
            try { Say("LATE component: added=" + (ProbePlugin.Instance.AddComponent<LateDriver>() != null)); }
            catch (Exception e) { Say("LATE component threw " + e.GetType().Name + ": " + e.Message); }
        }
        if (!patched && t >= 16f)
        {
            patched = true;
            Say("PATCH: applying a postfix to BaseMovement.FixedUpdate");
            Driver.Patch();
            Say("PATCH: " + Driver.Patched);
        }
        if (stage <= 5 && t >= 6f + stage * 12f) { stage++; StartPush(stage, cam, now); }
        else if (stage == 6 && t >= 80f && !done) { done = true; trace = false; Finish(); }
    }

    [HideFromIl2Cpp]
    private void Finish()
    {
        Driver.Active = false;
        System.IO.File.WriteAllText(System.IO.Path.Combine(Paths.PluginPath, "probe.done"), "done");
        Say("DONE late component calls=" + Driver.LateCalls + " postfix calls=" + Driver.PatchCalls);
    }

    [HideFromIl2Cpp]
    private void Trace(float now)
    {
        var cam = Camera.main; if (cam == null || local == null) return;
        var cp = cam.transform.position;
        if (Vector3.Distance(cp, lastTraceCam) < 0.05f && now - lastTraceAt < 4f) return;
        lastTraceCam = cp; lastTraceAt = now;
        Say("  t=" + Rel(now) + " cam=" + V(cp) + " player=" + V(localT.position) + " mover=" + V(moveT.position) + (body != null ? " vel=" + V(body.linearVelocity) : ""));
    }

    [HideFromIl2Cpp]
    private void StartPush(int m, Camera cam, float now)
    {
        var f = Quaternion.Euler(0f, cam.transform.eulerAngles.y, 0f) * Vector3.forward;
        var kind = m == 1 ? 1 : m == 2 ? 2 : movedBy[2] > movedBy[1] + 0.5f ? 2 : 1;
        camBefore = cam.transform.position; lastMode = m; pushEnd = now + 3f;
        Say("PUSH " + m + " (" + Modes[m] + ") start t=" + Rel(now) + " via " + (kind == 1 ? "late component" : "postfix") + " cam=" + V(camBefore) + " yaw=" + cam.transform.eulerAngles.y.ToString("F0") + " vel=" + V(body.linearVelocity));
        Driver.Begin(body, (m % 2 == 1 ? f : -f) * Speeds[m], kind, m == 5 ? 6f : 0f, m == 6);
    }

    [HideFromIl2Cpp]
    private void EndPush(float now)
    {
        Driver.Active = false;
        var cam = Camera.main; camAfter = cam != null ? cam.transform.position : camBefore;
        movedBy[lastMode] = H(camBefore, camAfter);
        Say("PUSH " + lastMode + " end: camera moved " + movedBy[lastMode].ToString("F2") + " m, height change " + (camAfter.y - camBefore.y).ToString("F2") + " m, velocity writes=" + Driver.Steps + " backward jumps=" + Driver.Resets + " vel=" + V(body.linearVelocity));
        pushEnd = -1f; verdictAt = now + 6f;
    }

    [HideFromIl2Cpp]
    private void Verdict(Camera cam)
    {
        var moved = H(camBefore, camAfter); var kept = H(camBefore, cam.transform.position);
        Say("PUSH " + lastMode + " verdict: moved=" + moved.ToString("F1") + " m, 6 s later " + kept.ToString("F1") + " m from start => " + (moved < 1f ? "NO MOVEMENT" : kept > moved * 0.7f ? "KEPT" : "SNAPPED BACK"));
        verdictAt = -1f;
    }

    private void LateUpdate()
    {
        try
        {
            if (board != null && wakeAt > 0f && localT != null)
            {
                // The board rides at the player's feet and points where the camera looks.
                var cam = Camera.main;
                if (cam != null) board.transform.rotation = Quaternion.Euler(0f, cam.transform.eulerAngles.y, 0f);
                board.transform.position = localT.position + Vector3.up * 0.06f;
            }
        }
        catch (Exception e) { Say("late update threw: " + e.GetType().Name + ": " + e.Message); board = null; }
    }

    [HideFromIl2Cpp]
    private void MakeBoard(Camera cam)
    {
        try
        {
            // Board-sized box on the ground ahead of the player until the player stands up.
            var fwd = Quaternion.Euler(0f, cam.transform.eulerAngles.y, 0f) * Vector3.forward;
            board = Box("skate_probe_board", new Vector3(0.22f, 0.04f, 0.8f), new Color(1f, 0.1f, 0.6f));
            board.transform.position = localT.position + fwd * 1.6f + Vector3.up * 0.12f;
            board.transform.rotation = Quaternion.LookRotation(fwd, Vector3.up);
            Say("BOARD created at " + V(board.transform.position));
        }
        catch (Exception e) { Say("board threw: " + e.GetType().Name + ": " + e.Message); }
    }

    private static GameObject Box(string name, Vector3 size, Color color)
    {
        var go = GameObject.CreatePrimitive(PrimitiveType.Cube);
        var col = go.GetComponent<Collider>();
        if (col != null) UnityEngine.Object.Destroy(col);
        go.name = name;
        go.transform.localScale = size;
        var sh = Shader.Find("Hidden/Internal-Colored");
        if (sh != null) { var m = new Material(sh); m.color = color; go.GetComponent<Renderer>().material = m; }
        else Say("shader Hidden/Internal-Colored not found; keeping the default material");
        UnityEngine.Object.DontDestroyOnLoad(go);
        return go;
    }
}

// Writes the wanted velocity into the movement body. Called either from LateDriver.FixedUpdate
// (Kind 1) or from the Harmony postfix on BaseMovement.FixedUpdate (Kind 2).
public static class Driver
{
    public static Rigidbody Body;
    public static Vector3 Velocity, LastPos;
    public static bool Active, Once;
    public static int Kind, Steps, Resets, LateCalls, PatchCalls;
    public static float Kick;
    public static string Patched = "not attempted";

    private static void Say(string m) { ProbePlugin.L.LogMessage("MOVE " + m); }
    private static string V(Vector3 v) { return v.x.ToString("F2") + "," + v.y.ToString("F2") + "," + v.z.ToString("F2"); }

    public static void Begin(Rigidbody body, Vector3 velocity, int kind, float kick, bool once)
    {
        Body = body; Velocity = velocity; Kind = kind; Kick = kick; Once = once; Steps = 0; Resets = 0; Active = true;
    }

    public static void Patch()
    {
        try
        {
            var target = AccessTools.Method(typeof(BaseMovement), "FixedUpdate");
            if (target == null) { Patched = "BaseMovement.FixedUpdate not found"; return; }
            var post = typeof(Driver).GetMethod("AfterWalk", BindingFlags.Public | BindingFlags.Static);
            new Harmony("claude.loaderprobe").Patch(target, null, new HarmonyMethod(post));
            Patched = "ok";
        }
        catch (Exception e) { Patched = "threw " + e.GetType().Name + ": " + e.Message; }
    }

    public static void AfterWalk()
    {
        PatchCalls++;
        if (Kind == 2) Apply();
    }

    public static void Apply()
    {
        if (!Active || Body == null) return;
        try
        {
            var cur = Body.linearVelocity; var pos = Body.position;
            Steps++;
            if (Steps > 1 && Vector3.Dot(pos - LastPos, Velocity.normalized) < -0.05f)
            {
                Resets++;
                if (Resets <= 5) Say("  backward jump " + Resets + " at write " + Steps + ": " + V(LastPos) + " -> " + V(pos));
            }
            if (Steps <= 4 || Steps == 20 || Steps == 60) Say("  write#" + Steps + " found vel=" + V(cur) + " pos=" + V(pos));
            LastPos = pos;
            if (Once && Steps > 1) return;
            var v = Velocity; v.y = cur.y;
            if (Kick != 0f) { v.y = Kick; Kick = 0f; }
            Body.linearVelocity = v;
        }
        catch (Exception e) { Active = false; Say("driver threw " + e.GetType().Name + ": " + e.Message); }
    }
}

// Created only after the game's walk component exists, so that its FixedUpdate is queued behind
// the game's when both have the default script order.
public class LateDriver : MonoBehaviour
{
    public LateDriver(IntPtr p) : base(p) { }

    private void FixedUpdate()
    {
        Driver.LateCalls++;
        if (Driver.Kind == 1) Driver.Apply();
    }
}
