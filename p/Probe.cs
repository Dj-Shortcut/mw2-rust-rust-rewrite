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
using UnityEngine;
using UnityEngine.InputSystem;

[BepInPlugin("claude.loaderprobe", "Loader Probe", "0.5.0")]
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
// Unity engine API and game type names. Run X (10 October 2026) showed that the local player is
// moved by a separate root object, assets/prefabs/player/player_movement.prefab, which carries
// PlayerWalkMovement, a dynamic Rigidbody and a CapsuleCollider; the BasePlayer object only follows
// it. BaseMovement has a Unity FixedUpdate, so disabling the walk component should free the body.
// This probe drives that body directly and reports whether the game and the server accept it.
// Timeline, in seconds after the player stands up:
//   2 capability checks, 4-14 key window (hold W), then six pushes of 3 s each at 16/28/40/52/64/76:
//   1 = 5 m/s with the walk component on, 2 = 5 m/s with it off, 3 and 4 = 8 m/s (faster than
//   sprinting) with it off, 5 = 5 m/s plus an upward kick, 6 = one 6 m/s shove and then coasting.
//   90 gamepad read, 94 done.
public class MoveProbe : MonoBehaviour
{
    public MoveProbe(IntPtr p) : base(p) { }

    public static bool Mute;
    private static readonly string[] Modes = { "", "5 m/s, walk component on", "5 m/s, walk component off", "8 m/s, walk component off",
        "8 m/s, walk component off", "5 m/s plus upward kick, walk component off", "single 6 m/s shove then coast, walk component off" };
    private static readonly float[] Speeds = { 0f, 5f, 5f, 8f, 8f, 5f, 6f };
    private float nextTick, nextTrace, lastTraceAt, wakeAt = -1f, pushEnd = -1f, verdictAt = -1f;
    private int ticks, standTicks, stage, mode, lastMode, fixedSteps;
    private BasePlayer local;
    private Transform localT, moveT;
    private PlayerWalkMovement walk;
    private Rigidbody body;
    private GameObject board;
    private bool keysOk = true, wDown, trace, done, padDone;
    private Vector3 dir, camBefore, camAfter, lastTraceCam;

    private static void Say(string m) { ProbePlugin.L.LogMessage("MOVE " + m); }
    private static string V(Vector3 v) { return v.x.ToString("F2") + "," + v.y.ToString("F2") + "," + v.z.ToString("F2"); }
    private static float H(Vector3 a, Vector3 b) { a.y = 0f; b.y = 0f; return Vector3.Distance(a, b); }
    [HideFromIl2Cpp]
    private string Rel(float now) { return (wakeAt > 0f ? now - wakeAt : 0f).ToString("F1"); }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        if (keysOk && wakeAt > 0f)
        {
            try
            {
                var kb = Keyboard.current;
                var w = kb != null && kb.wKey.isPressed;
                if (w != wDown) { wDown = w; Say("KEY W=" + w + " t=" + Rel(now)); }
            }
            catch (Exception e) { keysOk = false; Say("KEY read threw " + e.GetType().Name + ": " + e.Message); }
        }
        try
        {
            if (pushEnd > 0f && now >= pushEnd) EndPush(now);
            if (trace && now >= nextTrace) { nextTrace = now + 0.25f; Trace(now); }
        }
        catch (Exception e) { Say("update threw: " + e.GetType().Name + ": " + e.Message); Release(); }
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
            Say("LOCAL found player=" + V(localT.position) + " movement object=" + w.gameObject.name + " at " + V(moveT.position) + " body=" + (body != null) + " cam=" + V(cp));
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
        if (body == null) { if (!done) { done = true; Say("no rigidbody on the movement object; nothing to push"); Finish(); } return; }
        var t = now - wakeAt;
        if (verdictAt > 0f && now >= verdictAt) Verdict(cam);
        if (stage == 0 && t >= 2f) { Caps.Run(cam, body, moveT, walk); trace = true; stage = 1; Say("KEY WINDOW open until t=14 (hold W)"); }
        else if (stage >= 1 && stage <= 6 && t >= 16f + (stage - 1) * 12f) { StartPush(stage, cam, now); stage++; }
        else if (stage == 7 && t >= 90f && !padDone)
        {
            padDone = true;
            Say("PAD read begins");
            Caps.Pad();
        }
        else if (stage == 7 && t >= 94f && !done) { done = true; trace = false; Finish(); }
    }

    [HideFromIl2Cpp]
    private void Finish()
    {
        Release();
        System.IO.File.WriteAllText(System.IO.Path.Combine(Paths.PluginPath, "probe.done"), "done");
        Say("DONE");
    }

    [HideFromIl2Cpp]
    private void Release()
    {
        mode = 0; pushEnd = -1f;
        try { if (walk != null && !walk.enabled) walk.enabled = true; } catch (Exception e) { Say("could not re-enable the walk component: " + e.Message); }
    }

    [HideFromIl2Cpp]
    private void Trace(float now)
    {
        var cam = Camera.main; if (cam == null || local == null) return;
        var cp = cam.transform.position;
        if (Vector3.Distance(cp, lastTraceCam) < 0.05f && now - lastTraceAt < 4f) return;
        lastTraceCam = cp; lastTraceAt = now;
        Say("  t=" + Rel(now) + " cam=" + V(cp) + " player=" + V(localT.position) + " mover=" + V(moveT.position) + (body != null ? " vel=" + V(body.linearVelocity) + (body.isKinematic ? " KINEMATIC" : "") : "") + (walk.enabled ? "" : " walk-off"));
    }

    [HideFromIl2Cpp]
    private void StartPush(int m, Camera cam, float now)
    {
        var f = Quaternion.Euler(0f, cam.transform.eulerAngles.y, 0f) * Vector3.forward;
        dir = m % 2 == 1 ? f : -f;
        camBefore = cam.transform.position; fixedSteps = 0; lastMode = m; mode = m; pushEnd = now + (m == 6 ? 4f : 3f);
        Say("PUSH " + m + " (" + Modes[m] + ") start t=" + Rel(now) + " cam=" + V(camBefore) + " dir=" + V(dir) + " vel=" + V(body.linearVelocity) + " kinematic=" + body.isKinematic);
    }

    [HideFromIl2Cpp]
    private void EndPush(float now)
    {
        var cam = Camera.main; camAfter = cam != null ? cam.transform.position : camBefore;
        Say("PUSH " + lastMode + " end: camera moved " + H(camBefore, camAfter).ToString("F2") + " m, height change " + (camAfter.y - camBefore.y).ToString("F2") + " m, fixed steps=" + fixedSteps + " vel=" + V(body.linearVelocity) + " kinematic=" + body.isKinematic);
        Release();
        verdictAt = now + 6f;
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
        if (mode < 1 || body == null) return;
        try
        {
            if (Time.realtimeSinceStartup >= pushEnd) return;
            fixedSteps++;
            var cur = body.linearVelocity;
            if (fixedSteps <= 4 || fixedSteps == 20 || fixedSteps == 60) Say("  fixed#" + fixedSteps + " before set: vel=" + V(cur) + " pos=" + V(body.position) + " kinematic=" + body.isKinematic + " walkEnabled=" + walk.enabled);
            if (mode >= 2 && walk.enabled) walk.enabled = false;
            if (mode == 6 && fixedSteps > 1) return;
            var v = dir * Speeds[mode];
            // With the walk component off nothing may be applying gravity, so add it here.
            v.y = mode == 1 ? cur.y : cur.y - 9.81f * Time.fixedDeltaTime;
            if (mode == 5 && fixedSteps == 1) v.y = 5f;
            body.linearVelocity = v;
        }
        catch (Exception e) { Say("push threw: " + e.GetType().Name + ": " + e.Message); Release(); }
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

// Capability checks for the skate controller. Kept outside the injected MonoBehaviour so that the
// compiler-generated lambda methods are not registered with the game runtime. Each check stands
// alone, so one stripped engine method does not hide the rest.
public static class Caps
{
    private static void Say(string m) { ProbePlugin.L.LogMessage("MOVE " + m); }
    private static string V(Vector3 v) { return v.x.ToString("F2") + "," + v.y.ToString("F2") + "," + v.z.ToString("F2"); }

    public static string Try(Func<string> f)
    {
        try { return f(); }
        catch (Exception e) { return "unavailable(" + (e.Message != null && e.Message.Contains("unstripping") ? "stripped" : e.GetType().Name) + ")"; }
    }

    public static void Run(Camera cam, Rigidbody body, Transform moveT, PlayerWalkMovement walk)
    {
        Say("CAPS body: kinematic=" + Try(() => body.isKinematic.ToString()) + " mass=" + Try(() => body.mass.ToString("F1")) + " useGravity=" + Try(() => body.useGravity.ToString())
            + " constraints=" + Try(() => body.constraints.ToString()) + " interpolation=" + Try(() => body.interpolation.ToString()) + " linearDamping=" + Try(() => body.linearDamping.ToString("F2"))
            + " freezeRotation=" + Try(() => body.freezeRotation.ToString()) + " collisionDetection=" + Try(() => body.collisionDetectionMode.ToString()));
        var go = moveT.gameObject;
        Say("CAPS capsule: " + Try(() => go.GetComponent<CapsuleCollider>() != null ? "present" : "none") + " height=" + Try(() => go.GetComponent<CapsuleCollider>().height.ToString("F2")) + " radius=" + Try(() => go.GetComponent<CapsuleCollider>().radius.ToString("F2"))
            + " center=" + Try(() => V(go.GetComponent<CapsuleCollider>().center)) + " material=" + Try(() => { var m = go.GetComponent<CapsuleCollider>().sharedMaterial; return m == null ? "none" : m.name; }));
        Say("CAPS walk: enabled=" + walk.enabled + " zeroFriction=" + Try(() => { var m = walk.zeroFrictionMaterial; return m == null ? "none" : m.name + " dyn=" + m.dynamicFriction.ToString("F2") + " static=" + m.staticFriction.ToString("F2"); })
            + " highFriction=" + Try(() => { var m = walk.highFrictionMaterial; return m == null ? "none" : m.name + " dyn=" + m.dynamicFriction.ToString("F2") + " static=" + m.staticFriction.ToString("F2"); }));
        var origin = moveT.position + Vector3.up * 1.0f;
        var mask = ~((1 << 12) | (1 << 17) | (1 << 18) | (1 << 4) | (1 << 10) | (1 << 9) | (1 << 2));
        Say("CAPS raycast (origin, direction, out hit, distance, mask, triggers): " + Try(() => { RaycastHit h; var ok = Physics.Raycast(origin, Vector3.down, out h, 4f, mask, QueryTriggerInteraction.Ignore); return ok + " dist=" + h.m_Distance.ToString("F2") + " normal=" + V(h.m_Normal) + " point=" + V(h.m_Point); }));
        Say("CAPS raycast (origin, direction, out hit, distance, mask): " + Try(() => { RaycastHit h; var ok = Physics.Raycast(origin, Vector3.down, out h, 4f, mask); return ok + " dist=" + h.m_Distance.ToString("F2") + " normal=" + V(h.m_Normal); }));
        Say("CAPS raycast (ray, out hit, distance, mask): " + Try(() => { RaycastHit h; var ok = Physics.Raycast(new Ray(origin, Vector3.down), out h, 4f, mask); return ok + " dist=" + h.m_Distance.ToString("F2") + " normal=" + V(h.m_Normal); }));
        Say("CAPS spherecast: " + Try(() => { RaycastHit h; var ok = Physics.SphereCast(origin, 0.2f, Vector3.down, out h, 4f, mask, QueryTriggerInteraction.Ignore); return ok + " dist=" + h.m_Distance.ToString("F2") + " normal=" + V(h.m_Normal); }));
        foreach (var n in new[] { "Standard", "Rust/Standard", "Legacy Shaders/Diffuse", "Unlit/Color", "Sprites/Default", "Hidden/Internal-Colored" })
            Say("CAPS shader " + n + ": " + Try(() => Shader.Find(n) != null ? "found" : "missing"));
        Say("CAPS cylinder primitive: " + Try(() => { var g = GameObject.CreatePrimitive(PrimitiveType.Cylinder); var ok = g != null; UnityEngine.Object.Destroy(g); return ok.ToString(); }));
        Say("CAPS keyboard: " + Try(() => Keyboard.current == null ? "none" : "present w=" + Keyboard.current.wKey.isPressed + " space=" + Keyboard.current.spaceKey.isPressed));
        Say("CAPS gamepad present: " + Try(() => (Gamepad.current != null).ToString()));
        Say("CAPS camera yaw=" + cam.transform.eulerAngles.y.ToString("F0") + " fixedDt=" + Time.fixedDeltaTime.ToString("F4"));
    }

    public static void Pad()
    {
        Say("PAD " + Try(() => { var g = Gamepad.current; if (g == null) return "no gamepad"; return "left stick up=" + g.leftStick.up.isPressed + " down=" + g.leftStick.down.isPressed + " south=" + g.buttonSouth.isPressed; }));
        Say("PAD analog " + Try(() => { var g = Gamepad.current; if (g == null) return "no gamepad"; return "x=" + g.leftStick.x.ReadValue().ToString("F2") + " y=" + g.leftStick.y.ReadValue().ToString("F2"); }));
    }
}
