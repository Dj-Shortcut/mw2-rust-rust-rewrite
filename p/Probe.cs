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
using Il2CppInterop.Runtime.Attributes;
using Il2CppInterop.Runtime.InteropTypes;
using UnityEngine;

[BepInPlugin("claude.loaderprobe", "Loader Probe", "0.4.0")]
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
// Unity engine API and game type names. It finds the local player object, dumps what the player is
// made of (components, nearby rigidbodies and colliders) asleep and again awake, shows a placeholder
// board at the player's feet, and then tries six ways of moving the player for three seconds each,
// to learn which one the game and the server accept. Timeline, in seconds after waking up:
//   3 scan, 3-16 free window for walking with W (reference), 16/28/40/52/64/76 pushes 1-6, 88 done.
public class MoveProbe : MonoBehaviour
{
    public MoveProbe(IntPtr p) : base(p) { }

    public static bool Mute;
    private static readonly string[] Modes = { "", "velocity in FixedUpdate", "velocity in FixedUpdate with walk components disabled",
        "Rigidbody.MovePosition in FixedUpdate", "velocity in Update, FixedUpdate and LateUpdate",
        "translate the player transform in LateUpdate", "translate the body transform in LateUpdate" };
    private float nextTick, nextTrace, lastTraceAt, wakeAt = -1f, pushEnd = -1f, verdictAt = -1f;
    private int ticks, standTicks, stage, mode, lastMode, fixedSteps, budget;
    private BasePlayer local;
    private Transform localT;
    private GameObject board;
    private readonly System.Collections.Generic.List<Rigidbody> bodies = new System.Collections.Generic.List<Rigidbody>();
    private readonly System.Collections.Generic.List<Behaviour> walks = new System.Collections.Generic.List<Behaviour>();
    private bool inputOk = true, wDown, trace, done;
    private Vector3 dir, camBefore, camAfter, lastTraceCam;

    private static void Say(string m) { ProbePlugin.L.LogMessage("MOVE " + m); }
    private static string V(Vector3 v) { return v.x.ToString("F2") + "," + v.y.ToString("F2") + "," + v.z.ToString("F2"); }
    private static float H(Vector3 a, Vector3 b) { a.y = 0f; b.y = 0f; return Vector3.Distance(a, b); }
    [HideFromIl2Cpp]
    private string Rel(float now) { return (wakeAt > 0f ? now - wakeAt : 0f).ToString("F1"); }

    private static string Cls(Il2CppObjectBase o)
    {
        try { return Marshal.PtrToStringAnsi(IL2CPP.il2cpp_class_get_name(IL2CPP.il2cpp_object_get_class(o.Pointer))); }
        catch (Exception e) { return "?" + e.GetType().Name; }
    }

    private static string Comps(GameObject go)
    {
        var s = "";
        try
        {
            var cs = go.GetComponents<Component>();
            for (var i = 0; i < cs.Length; i++)
            {
                var c = cs[i];
                if (c == null) { s += " <missing>"; continue; }
                s += " " + Cls(c);
                var b = c.TryCast<Behaviour>();
                if (b != null && !b.enabled) s += "(off)";
            }
        }
        catch (Exception e) { s += " comps threw " + e.GetType().Name + ": " + e.Message; }
        return s;
    }

    private static string Chain(Transform t)
    {
        var s = ""; var p = t.parent; var n = 0;
        while (p != null && n++ < 8) { s += "/" + p.gameObject.name; p = p.parent; }
        return s == "" ? "(root)" : s;
    }

    [HideFromIl2Cpp]
    private void Tree(Transform t, int depth, int maxDepth)
    {
        if (budget-- <= 0) return;
        var pad = new string(' ', depth * 2);
        var go = t.gameObject;
        Say(pad + "- " + go.name + (go.activeSelf ? "" : " (inactive)") + " L" + go.layer + " :" + Comps(go));
        var n = t.childCount;
        if (depth >= maxDepth) { if (n > 0) Say(pad + "  .. " + n + " children"); return; }
        for (var i = 0; i < n; i++) Tree(t.GetChild(i), depth + 1, maxDepth);
    }

    [HideFromIl2Cpp]
    private void Scan(string tag, Camera cam)
    {
        var cp = cam.transform.position;
        Say("SCAN " + tag + " cam=" + V(cp) + " camGo=" + cam.gameObject.name + " camParent=" + Chain(cam.transform) + " camComps:" + Comps(cam.gameObject));
        try { Say("  physics simulationMode=" + Physics.simulationMode + " fixedDt=" + Time.fixedDeltaTime.ToString("F4") + " timeScale=" + Time.timeScale + " gravity=" + V(Physics.gravity)); }
        catch (Exception e) { Say("  physics info threw " + e.GetType().Name + ": " + e.Message); }
        Say("  player go=" + local.gameObject.name + " pos=" + V(localT.position) + " parent=" + Chain(localT) + " children=" + localT.childCount);
        budget = 40; Tree(localT, 0, 2);
        bodies.Clear(); walks.Clear();
        var seen = new System.Collections.Generic.HashSet<int>();
        try
        {
            var w = UnityEngine.Object.FindObjectsOfType<PlayerWalkMovement>();
            Say("  PlayerWalkMovement active instances=" + w.Length);
            for (var i = 0; i < w.Length && i < 6; i++)
            {
                var go = w[i].gameObject; var d = Vector3.Distance(go.transform.position, cp);
                Say("   walk[" + i + "] go=" + go.name + " d=" + d.ToString("F1") + " enabled=" + w[i].enabled + " onPlayer=" + (go.GetInstanceID() == local.gameObject.GetInstanceID()) + " parent=" + Chain(go.transform) + " comps:" + Comps(go));
                if (d < 4f && seen.Add(w[i].GetInstanceID())) walks.Add(w[i]);
            }
        }
        catch (Exception e) { Say("  walk scan threw " + e.GetType().Name + ": " + e.Message); }
        try
        {
            var m = UnityEngine.Object.FindObjectsOfType<BaseMovement>();
            Say("  BaseMovement active instances=" + m.Length);
            for (var i = 0; i < m.Length && i < 6; i++)
            {
                var go = m[i].gameObject; var d = Vector3.Distance(go.transform.position, cp);
                Say("   move[" + i + "] class=" + Cls(m[i]) + " go=" + go.name + " d=" + d.ToString("F1") + " enabled=" + m[i].enabled + " parent=" + Chain(go.transform));
                if (d < 4f && seen.Add(m[i].GetInstanceID())) walks.Add(m[i]);
            }
        }
        catch (Exception e) { Say("  movement scan threw " + e.GetType().Name + ": " + e.Message); }
        try
        {
            var all = Resources.FindObjectsOfTypeAll(Il2CppType.Of<BaseMovement>());
            Say("  BaseMovement including inactive objects and assets=" + all.Length);
            for (var i = 0; i < all.Length && i < 6; i++)
            {
                var c = all[i].TryCast<Component>();
                if (c != null) Say("   all[" + i + "] class=" + Cls(c) + " go=" + c.gameObject.name + " activeInHierarchy=" + c.gameObject.activeInHierarchy + " scene=" + c.gameObject.scene.name + " parent=" + Chain(c.transform));
            }
        }
        catch (Exception e) { Say("  all-movement scan threw " + e.GetType().Name + ": " + e.Message); }
        try
        {
            var rbs = UnityEngine.Object.FindObjectsOfType<Rigidbody>();
            var near = 0;
            for (var i = 0; i < rbs.Length; i++)
            {
                var rb = rbs[i]; var d = Vector3.Distance(rb.position, cp);
                if (d > 3f) continue;
                near++;
                if (near > 10) continue;
                Say("   rb go=" + rb.gameObject.name + " d=" + d.ToString("F1") + " kin=" + rb.isKinematic + " grav=" + rb.useGravity + " mass=" + rb.mass.ToString("F1") + " vel=" + V(rb.linearVelocity) + " constraints=" + rb.constraints + " interp=" + rb.interpolation + " parent=" + Chain(rb.transform) + " comps:" + Comps(rb.gameObject));
                if (!rb.isKinematic) bodies.Add(rb);
            }
            Say("  rigidbodies total=" + rbs.Length + " within 3 m=" + near + " not kinematic=" + bodies.Count);
        }
        catch (Exception e) { Say("  rigidbody scan threw " + e.GetType().Name + ": " + e.Message); }
        if (bodies.Count == 0)
        {
            var own = local.GetComponent<Rigidbody>();
            if (own != null) { bodies.Add(own); Say("  no moving body near the camera; using the player's own kinematic rigidbody for the push tests"); }
        }
        try
        {
            var cols = Physics.OverlapSphere(cp, 2.2f, -1, QueryTriggerInteraction.Collide);
            Say("  colliders within 2.2 m=" + cols.Length);
            for (var i = 0; i < cols.Length && i < 18; i++)
            {
                var c = cols[i]; var ar = c.attachedRigidbody;
                Say("   col " + Cls(c) + " go=" + c.gameObject.name + " L" + c.gameObject.layer + (c.isTrigger ? " trigger" : "") + (c.enabled ? "" : " (off)") + " rb=" + (ar != null ? ar.gameObject.name + (ar.isKinematic ? "(kin)" : "(dynamic)") : "none") + " parent=" + Chain(c.transform));
            }
        }
        catch (Exception e) { Say("  collider scan threw " + e.GetType().Name + ": " + e.Message); }
    }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        if (inputOk && local != null)
        {
            try
            {
                var w = Input.GetKey(KeyCode.W);
                if (w != wDown) { wDown = w; Say("INPUT W=" + w + " t=" + Rel(now)); }
            }
            catch (Exception e) { inputOk = false; Say("INPUT legacy Input threw " + e.GetType().Name + ": " + e.Message); }
        }
        try
        {
            if (mode == 4 && now < pushEnd) SetVelocity();
            if (pushEnd > 0f && now >= pushEnd) EndPush(now);
            if (trace && now >= nextTrace) { nextTrace = now + 0.25f; Trace(now); }
        }
        catch (Exception e) { Say("update threw: " + e.GetType().Name + ": " + e.Message); mode = 0; pushEnd = -1f; }
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
            if (ticks % 20 == 1) Say("cam=" + V(cp) + " players=" + n + (best != null ? " nearest=" + bestD.ToString("F1") + "m name=" + best.gameObject.name : ""));
            if (best == null || bestD > 4f) return;
            local = best; localT = best.transform;
            Say("LOCAL found pos=" + V(localT.position) + " cam=" + V(cp));
            Scan("at spawn", cam);
            MakeBoard(cam);
            return;
        }
        var up = cp.y - localT.position.y;
        if (wakeAt < 0f)
        {
            standTicks = up > 1.0f ? standTicks + 1 : 0;
            if (ticks % 10 == 0) Say("waiting for the player to stand: camera height=" + up.ToString("F2"));
            if (standTicks >= 2) { wakeAt = now; Say("AWAKE camera height=" + up.ToString("F2") + " player=" + V(localT.position)); }
            return;
        }
        var t = now - wakeAt;
        if (verdictAt > 0f && now >= verdictAt) Verdict(cam);
        if (stage == 0 && t >= 3f) { Scan("awake", cam); trace = true; stage = 1; Say("WALK WINDOW open until t=16"); }
        else if (stage >= 1 && stage <= 6 && t >= 16f + (stage - 1) * 12f) { StartPush(stage, cam, now); stage++; }
        else if (stage == 7 && t >= 88f && !done)
        {
            done = true; trace = false;
            System.IO.File.WriteAllText(System.IO.Path.Combine(Paths.PluginPath, "probe.done"), "done");
            Say("DONE");
        }
    }

    [HideFromIl2Cpp]
    private void Trace(float now)
    {
        var cam = Camera.main; if (cam == null || local == null) return;
        var cp = cam.transform.position;
        if (Vector3.Distance(cp, lastTraceCam) < 0.05f && now - lastTraceAt < 4f) return;
        lastTraceCam = cp; lastTraceAt = now;
        var s = "  t=" + Rel(now) + " cam=" + V(cp) + " player=" + V(localT.position);
        for (var i = 0; i < bodies.Count && i < 2; i++) { var b = bodies[i]; if (b != null) s += " b" + i + "=" + V(b.position) + " v=" + V(b.linearVelocity) + (b.isKinematic ? " kin" : ""); }
        Say(s);
    }

    [HideFromIl2Cpp]
    private void StartPush(int m, Camera cam, float now)
    {
        var f = cam.transform.forward; f.y = 0f; f = f.sqrMagnitude > 0.01f ? f.normalized : Vector3.forward;
        dir = m % 2 == 1 ? f : -f;
        camBefore = cam.transform.position; fixedSteps = 0; lastMode = m; mode = m; pushEnd = now + 3f;
        Say("PUSH " + m + " (" + Modes[m] + ") start t=" + Rel(now) + " cam=" + V(camBefore) + " dir=" + V(dir) + " bodies=" + bodies.Count + " walks=" + walks.Count);
    }

    [HideFromIl2Cpp]
    private void SetVelocity()
    {
        for (var i = 0; i < bodies.Count; i++)
        {
            var b = bodies[i]; if (b == null) continue;
            var v = dir * 5f; v.y = b.linearVelocity.y; b.linearVelocity = v;
        }
    }

    [HideFromIl2Cpp]
    private void EndPush(float now)
    {
        if (lastMode == 2) for (var i = 0; i < walks.Count; i++) if (walks[i] != null) walks[i].enabled = true;
        var cam = Camera.main; camAfter = cam != null ? cam.transform.position : camBefore;
        Say("PUSH " + lastMode + " end: camera moved " + H(camBefore, camAfter).ToString("F2") + " m in 3 s, fixed steps=" + fixedSteps);
        mode = 0; pushEnd = -1f; verdictAt = now + 6f;
    }

    [HideFromIl2Cpp]
    private void Verdict(Camera cam)
    {
        var moved = H(camBefore, camAfter); var kept = H(camBefore, cam.transform.position);
        Say("PUSH " + lastMode + " verdict: moved=" + moved.ToString("F1") + " m, 6 s later " + kept.ToString("F1") + " m from start => " + (moved < 1f ? "NO MOVEMENT" : kept > moved * 0.7f ? "KEPT" : "SNAPPED BACK"));
        verdictAt = -1f;
    }

    private void FixedUpdate()
    {
        if (mode < 1 || mode > 4) return;
        try
        {
            if (Time.realtimeSinceStartup >= pushEnd) return;
            fixedSteps++;
            for (var i = 0; i < bodies.Count; i++)
            {
                var b = bodies[i]; if (b == null) continue;
                if (fixedSteps <= 3 || fixedSteps == 20) Say("  fixed#" + fixedSteps + " b" + i + " before set: vel=" + V(b.linearVelocity) + " pos=" + V(b.position) + " kin=" + b.isKinematic);
                if (mode == 3) b.MovePosition(b.position + dir * 5f * Time.fixedDeltaTime);
                else { var v = dir * 5f; v.y = b.linearVelocity.y; b.linearVelocity = v; }
            }
            if (mode == 2) for (var i = 0; i < walks.Count; i++) if (walks[i] != null) walks[i].enabled = false;
        }
        catch (Exception e) { Say("push threw: " + e.GetType().Name + ": " + e.Message); mode = 0; }
    }

    private void LateUpdate()
    {
        try
        {
            var now = Time.realtimeSinceStartup;
            if (mode == 4 && now < pushEnd) SetVelocity();
            if (mode == 5 && now < pushEnd) localT.position = localT.position + dir * 5f * Time.deltaTime;
            if (mode == 6 && now < pushEnd)
                for (var i = 0; i < bodies.Count; i++) { var b = bodies[i]; if (b != null) b.transform.position = b.transform.position + dir * 5f * Time.deltaTime; }
            if (board != null && wakeAt > 0f && localT != null)
            {
                // The board rides at the player's feet and points where the camera looks.
                var cam = Camera.main;
                if (cam != null)
                {
                    var f = cam.transform.forward; f.y = 0f;
                    if (f.sqrMagnitude > 0.01f) board.transform.rotation = Quaternion.LookRotation(f.normalized, Vector3.up);
                }
                board.transform.position = localT.position + Vector3.up * 0.06f;
            }
        }
        catch (Exception e) { Say("late update threw: " + e.GetType().Name + ": " + e.Message); mode = 0; }
    }

    [HideFromIl2Cpp]
    private void MakeBoard(Camera cam)
    {
        try
        {
            // Board-sized box on the ground ahead of the player until the player stands up.
            var fwd = cam.transform.forward; fwd.y = 0f; fwd = fwd.sqrMagnitude > 0.01f ? fwd.normalized : Vector3.forward;
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
