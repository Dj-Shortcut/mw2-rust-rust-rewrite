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

[BepInPlugin("claude.loaderprobe", "Loader Probe", "0.8.0")]
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
            case "skate": SkateRig.Test = false; return "added=" + (AddComponent<SkateRig>() != null);
            case "skatetest": SkateRig.Test = true; return "added=" + (AddComponent<SkateRig>() != null);
            case "mute": SkateRig.Mute = true; AudioListener.volume = 0f; return "volume=" + AudioListener.volume;
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

// First skate controller. The game's own members are obfuscated in the client, so this uses only
// Unity engine API, game type names and Unity message names. What the probe runs established
// (10 October 2026, build 25824447, see issue #289):
//   - The local player is moved by a separate root object, assets/prefabs/player/player_movement.prefab:
//     PlayerWalkMovement, a dynamic Rigidbody (mass 0.5, no engine gravity, rotation frozen) and a
//     CapsuleCollider. The BasePlayer object follows it.
//   - The game's fixed step cancels any velocity written before it. A component created after the
//     walk component exists gets its FixedUpdate after the game's, and a velocity written there
//     moves the player; the game keeps handling gravity, jumping and the model.
//   - Disabling the walk component is not usable: the player is then pulled back constantly.
//   - Above walking pace the server pulls the player back unless its anti-hack leaves them alone.
// K, or two quick presses of the jump key, mounts and dismounts. Mounted: W pushes, S brakes, the
// board turns toward where the camera looks, A and D carve harder, the game's own jump still works.
public class SkateRig : MonoBehaviour
{
    public SkateRig(IntPtr p) : base(p) { }

    public static bool Mute, Test;
    private float nextTick, nextLog, wakeAt = -1f, mountAt = -1f;
    private int ticks, standTicks, phase;
    private BasePlayer local;
    private Transform localT;
    private Rigidbody body;
    private GameObject board;
    private LateDriver late;
    private bool lateAdded, kDown, spaceDown, keysOk = true, done;
    private float yaw0, lastJumpTap = -1f;

    private static void Say(string m) { ProbePlugin.L.LogMessage("SKATE " + m); }
    private static string V(Vector3 v) { return v.x.ToString("F1") + "," + v.y.ToString("F1") + "," + v.z.ToString("F1"); }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        if (keysOk && !Test && body != null && wakeAt > 0f)
        {
            try
            {
                var kb = Keyboard.current;
                var k = kb != null && kb.kKey.isPressed;
                if (k && !kDown) Toggle();
                kDown = k;
                // Two quick presses of the jump key also mount and dismount, for controllers mapped to keys.
                var sp = kb != null && kb.spaceKey.isPressed;
                if (sp && !spaceDown)
                {
                    if (now - lastJumpTap < 0.4f) { Toggle(); lastJumpTap = -1f; }
                    else lastJumpTap = now;
                }
                spaceDown = sp;
            }
            catch (Exception e) { keysOk = false; Say("key read threw " + e.GetType().Name + ": " + e.Message); }
        }
        if (Skate.On && now >= nextLog) { nextLog = now + 1f; Say(Skate.Status()); }
        if (Test && mountAt > 0f) Script(now - mountAt);
        if (now < nextTick) return;
        nextTick = now + 1f; ticks++;
        try { Tick(now); }
        catch (Exception e) { Say("tick threw: " + e.GetType().Name + ": " + e.Message); nextTick = now + 5f; }
    }

    [HideFromIl2Cpp]
    private void Toggle()
    {
        var cam = Camera.main;
        if (Skate.On) { Skate.Dismount(); Say("DISMOUNT " + Skate.Status()); }
        else if (cam != null) { Skate.Mount(body, cam.transform.eulerAngles.y); Say("MOUNT yaw=" + Skate.Yaw.ToString("F0") + " speed=" + Skate.Speed.ToString("F1")); }
    }

    // Scripted ride for an unattended check: out along the reverse of the view direction, back again.
    [HideFromIl2Cpp]
    private void Script(float s)
    {
        var next = s < 2f ? 1 : s < 4f ? 2 : s < 5.5f ? 3 : s < 7.5f ? 4 : s < 9.5f ? 5 : s < 11.5f ? 6 : s < 13f ? 7 : s < 15f ? 8 : 9;
        if (next == phase) return;
        phase = next;
        Skate.SynthPush = phase == 1 || phase == 5;
        Skate.SynthBrake = phase == 3 || phase == 7;
        Skate.SynthYaw = phase <= 3 ? yaw0 + 180f : yaw0;
        Say("SCRIPT phase " + phase + (phase == 1 ? " push out" : phase == 2 ? " coast" : phase == 3 ? " brake" : phase == 4 ? " turn round" : phase == 5 ? " push back" : phase == 6 ? " coast" : phase == 7 ? " brake" : phase == 8 ? " dismount" : " finish") + " | " + Skate.Status());
        if (phase == 8 && Skate.On) Skate.Dismount();
        if (phase == 9 && !done)
        {
            done = true;
            System.IO.File.WriteAllText(System.IO.Path.Combine(Paths.PluginPath, "probe.done"), "done");
            Say("DONE top speed=" + Skate.Top.ToString("F1") + " pull-backs=" + Skate.Resets + " late calls=" + LateDriver.Calls);
        }
    }

    [HideFromIl2Cpp]
    private void Tick(float now)
    {
        if (Mute) AudioListener.volume = 0f;
        var cam = Camera.main;
        if (cam == null) return;
        var cp = cam.transform.position;
        if (local != null && body == null && wakeAt > 0f)
        {
            // The movement object is replaced when the player dies or reconnects.
            Say("movement object gone; looking again");
            Skate.Dismount(); local = null; wakeAt = -1f; standTicks = 0; lateAdded = false;
        }
        if (local == null)
        {
            var players = UnityEngine.Object.FindObjectsOfType<BasePlayer>();
            BasePlayer best = null; var bestD = 999f;
            for (var i = 0; i < players.Length; i++)
            {
                var d = Vector3.Distance(players[i].transform.position, cp);
                if (d < bestD) { bestD = d; best = players[i]; }
            }
            if (best == null || bestD > 4f) return;
            var walks = UnityEngine.Object.FindObjectsOfType<PlayerWalkMovement>();
            PlayerWalkMovement w = null; var wd = 999f;
            for (var i = 0; i < walks.Length; i++)
            {
                var d = Vector3.Distance(walks[i].transform.position, cp);
                if (d < wd) { wd = d; w = walks[i]; }
            }
            if (w == null || wd > 4f) return;
            local = best; localT = best.transform; body = w.gameObject.GetComponent<Rigidbody>();
            Say("LOCAL found player=" + V(localT.position) + " body=" + (body != null));
            if (board == null) { try { board = BuildBoard(); board.SetActive(false); } catch (Exception e) { Say("board threw " + e.GetType().Name + ": " + e.Message); } }
            return;
        }
        if (wakeAt < 0f)
        {
            standTicks = cp.y - localT.position.y > 1.0f ? standTicks + 1 : 0;
            if (standTicks >= 2) { wakeAt = now; Say("AWAKE player=" + V(localT.position) + (Test ? "" : " | K or a double jump mounts")); }
            return;
        }
        if (body == null) return;
        if (!lateAdded && now - wakeAt >= 2f)
        {
            // A fresh driver for every movement object, so that it is always queued behind that object's walk component.
            lateAdded = true;
            try
            {
                if (late != null) UnityEngine.Object.Destroy(late);
                late = ProbePlugin.Instance.AddComponent<LateDriver>();
                Say("late driver added=" + (late != null));
            }
            catch (Exception e) { Say("late driver threw " + e.GetType().Name + ": " + e.Message); }
        }
        if (Test && mountAt < 0f && now - wakeAt >= 4f)
        {
            yaw0 = cam.transform.eulerAngles.y;
            Skate.Synth = true; Skate.SynthYaw = yaw0 + 180f;
            Skate.Mount(body, yaw0 + 180f);
            mountAt = now;
            Say("MOUNT (scripted) view yaw=" + yaw0.ToString("F0") + " at " + V(localT.position));
        }
    }

    private void LateUpdate()
    {
        if (board == null) return;
        try
        {
            if (board.activeSelf != Skate.On) board.SetActive(Skate.On);
            if (!Skate.On || localT == null) return;
            var dir = Quaternion.Euler(0f, Skate.Yaw, 0f) * Vector3.forward;
            var n = Skate.Normal;
            var fwd = dir - n * Vector3.Dot(dir, n);
            board.transform.position = localT.position;
            board.transform.rotation = Quaternion.LookRotation(fwd.sqrMagnitude > 0.0001f ? fwd.normalized : dir, n) * Quaternion.Euler(0f, 0f, -Skate.Lean);
        }
        catch (Exception e) { Say("board update threw " + e.GetType().Name + ": " + e.Message); board = null; }
    }

    // The shared procedural board (mods/rust/shared/SkateBoardMesh.cs) when the engine's mesh setters
    // are present in this build, otherwise the same idea from primitives.
    private static GameObject BuildBoard()
    {
        try { var made = BuildMeshBoard(); Say("BOARD from the shared mesh"); return made; }
        catch (Exception e) { Say("shared mesh unavailable (" + e.GetType().Name + ": " + e.Message + "); using primitives"); return BuildPrimitiveBoard(); }
    }

    private static GameObject BuildMeshBoard()
    {
        var data = Shortcut.RustMod.SkateBoardMesh.Create();
        var n = data.Positions.Length / 3;
        var verts = new Vector3[n]; var norms = new Vector3[n]; var cols = new Color[n];
        for (var i = 0; i < n; i++)
        {
            verts[i] = new Vector3(data.Positions[i * 3], data.Positions[i * 3 + 1], data.Positions[i * 3 + 2]);
            norms[i] = new Vector3(data.Normals[i * 3], data.Normals[i * 3 + 1], data.Normals[i * 3 + 2]);
            cols[i] = new Color(data.Colours[i * 4], data.Colours[i * 4 + 1], data.Colours[i * 4 + 2], data.Colours[i * 4 + 3]);
        }
        var mesh = new Mesh();
        mesh.vertices = verts; mesh.normals = norms; mesh.colors = cols; mesh.triangles = data.Triangles;
        mesh.RecalculateBounds();
        var sh = Shader.Find("Hidden/Internal-Colored");
        if (sh == null) throw new InvalidOperationException("no vertex-colour shader");
        var mat = new Material(sh); mat.color = Color.white;
        var root = new GameObject("skate_board");
        root.AddComponent<MeshFilter>().sharedMesh = mesh;
        root.AddComponent<MeshRenderer>().material = mat;
        UnityEngine.Object.DontDestroyOnLoad(root);
        return root;
    }

    // A recognisable board from primitives: deck with raised nose and tail, two trucks, four wheels.
    // Origin at the ground contact centre, +Z forward. Unlit colours, because that shader is known to render here.
    private static GameObject BuildPrimitiveBoard()
    {
        var root = new GameObject("skate_board");
        var sh = Shader.Find("Hidden/Internal-Colored");
        var grip = new Color(0.07f, 0.07f, 0.08f); var kick = new Color(0.16f, 0.16f, 0.18f); var under = new Color(1f, 0.1f, 0.6f);
        var metal = new Color(0.55f, 0.57f, 0.6f); var wheel = new Color(0.95f, 0.93f, 0.85f);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.078f, 0f), new Vector3(0.20f, 0.010f, 0.60f), Quaternion.identity, grip, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.069f, 0f), new Vector3(0.20f, 0.008f, 0.60f), Quaternion.identity, under, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.084f, 0f), new Vector3(0.025f, 0.003f, 0.56f), Quaternion.identity, under, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.097f, 0.365f), new Vector3(0.20f, 0.012f, 0.15f), Quaternion.Euler(-17f, 0f, 0f), kick, sh);
        Part(root, PrimitiveType.Cube, new Vector3(0f, 0.097f, -0.365f), new Vector3(0.20f, 0.012f, 0.15f), Quaternion.Euler(17f, 0f, 0f), kick, sh);
        for (var z = -1; z <= 1; z += 2)
        {
            Part(root, PrimitiveType.Cube, new Vector3(0f, 0.046f, z * 0.23f), new Vector3(0.15f, 0.034f, 0.05f), Quaternion.identity, metal, sh);
            for (var x = -1; x <= 1; x += 2)
                Part(root, PrimitiveType.Cylinder, new Vector3(x * 0.097f, 0.0275f, z * 0.23f), new Vector3(0.055f, 0.017f, 0.055f), Quaternion.Euler(0f, 0f, 90f), wheel, sh);
        }
        UnityEngine.Object.DontDestroyOnLoad(root);
        return root;
    }

    private static void Part(GameObject root, PrimitiveType type, Vector3 pos, Vector3 scale, Quaternion rot, Color color, Shader sh)
    {
        var go = GameObject.CreatePrimitive(type);
        var col = go.GetComponent<Collider>();
        if (col != null) UnityEngine.Object.Destroy(col);
        go.transform.SetParent(root.transform, false);
        go.transform.localPosition = pos; go.transform.localRotation = rot; go.transform.localScale = scale;
        if (sh != null) { var m = new Material(sh); m.color = color; go.GetComponent<Renderer>().material = m; }
    }
}

// The skate model: a speed along a heading, written into the movement body after the game's own
// fixed step. Plain arithmetic is done here rather than through UnityEngine.Mathf, because engine
// methods the game itself never calls can be missing from the build.
public static class Skate
{
    public const float PushAccel = 6f, MaxPush = 8f, BrakeDecel = 12f, RollDecel = 0.45f, MaxSpeed = 13f, TurnRate = 150f, SlopeGain = 1.6f;
    // Everything except the player's own layers, triggers, water, ragdolls and invisible helpers.
    public static readonly int GroundMask = ~((1 << 12) | (1 << 17) | (1 << 18) | (1 << 4) | (1 << 10) | (1 << 9) | (1 << 2));
    public static bool On, Synth, SynthPush, SynthBrake, Grounded;
    public static float Speed, Yaw, Lean, SynthYaw, Actual, Top, Cap = MaxSpeed;
    public static Vector3 Normal = Vector3.up;
    public static int Steps, Resets;
    private static Rigidbody body;
    private static Vector3 lastPos, lastDir;
    private static float lastSpeed, lastPull = -999f;
    private static bool hasLast;

    private static Vector3 Dir(float yaw) { return Quaternion.Euler(0f, yaw, 0f) * Vector3.forward; }
    private static float Toward(float a, float b, float step) { return Math.Abs(b - a) <= step ? b : a + Math.Sign(b - a) * step; }
    private static float Delta(float from, float to) { var d = (to - from) % 360f; if (d > 180f) d -= 360f; if (d < -180f) d += 360f; return d; }
    private static float Clamp(float v, float lo, float hi) { return v < lo ? lo : v > hi ? hi : v; }

    public static void Mount(Rigidbody b, float yaw)
    {
        body = b; Yaw = yaw;
        var v = b.linearVelocity; v.y = 0f;
        Speed = Vector3.Dot(v, Dir(yaw));
        hasLast = false; Steps = 0; Resets = 0; Top = 0f; Lean = 0f; Cap = MaxSpeed; lastPull = Time.realtimeSinceStartup; On = true;
    }

    public static void Dismount() { On = false; }

    public static string Status()
    {
        return "speed=" + Speed.ToString("F1") + " actual=" + Actual.ToString("F1") + " top=" + Top.ToString("F1") + " cap=" + Cap.ToString("F1") + " yaw=" + Yaw.ToString("F0") + " grounded=" + Grounded
            + " pull-backs=" + Resets + (body != null ? " pos=" + body.position.x.ToString("F1") + "," + body.position.y.ToString("F1") + "," + body.position.z.ToString("F1") : "");
    }

    public static void Step()
    {
        if (!On) return;
        try
        {
            if (body == null) { On = false; return; }
            var dt = Time.fixedDeltaTime;
            bool push = false, brake = false; var steer = 0f; var look = Yaw;
            if (Synth) { push = SynthPush; brake = SynthBrake; look = SynthYaw; }
            else
            {
                var cam = Camera.main;
                if (cam != null) look = cam.transform.eulerAngles.y;
                var kb = Keyboard.current;
                if (kb != null)
                {
                    push = kb.wKey.isPressed; brake = kb.sKey.isPressed;
                    if (kb.aKey.isPressed) steer -= 1f;
                    if (kb.dKey.isPressed) steer += 1f;
                }
            }
            var pos = body.position; var cur = body.linearVelocity;
            if (hasLast)
            {
                // What the body really did since the last write: a wall, or the server pulling the player back.
                var moved = pos - lastPos; moved.y = 0f;
                Actual = Vector3.Dot(moved, lastDir) / dt;
                if (lastSpeed > 1f && Actual < -3f)
                {
                    // The server refused the last stretch. Stay under the speed it refused, and try a little more later.
                    Resets++; lastPull = Time.realtimeSinceStartup;
                    Cap = Math.Max(2.6f, Math.Min(Cap, lastSpeed * 0.8f));
                }
                var limit = (Actual > 0f ? Actual : 0f) + 2f;
                if (Speed > limit) Speed = limit;
            }
            RaycastHit hit;
            Grounded = Physics.Raycast(pos + Vector3.up * 0.5f, Vector3.down, out hit, 0.75f, GroundMask, QueryTriggerInteraction.Ignore);
            Normal = Grounded ? hit.m_Normal : Vector3.up;
            var before = Yaw;
            var maxTurn = TurnRate / (1f + Math.Abs(Speed) / 8f) * dt;
            Yaw += Clamp(Delta(Yaw, look + steer * 40f), -maxTurn, maxTurn);
            var dir = Dir(Yaw);
            var tangent = dir - Normal * Vector3.Dot(dir, Normal);
            tangent = tangent.sqrMagnitude > 0.0001f ? tangent.normalized : dir;
            if (Grounded)
            {
                Speed += Vector3.Dot(Physics.gravity, tangent) * SlopeGain * dt;
                if (push && Speed < MaxPush) Speed = Math.Min(MaxPush, Speed + PushAccel * dt);
                if (brake) Speed = Toward(Speed, 0f, BrakeDecel * dt);
                Speed = Toward(Speed, 0f, RollDecel * dt);
            }
            if (Cap < MaxSpeed && Time.realtimeSinceStartup - lastPull > 20f) { Cap = Math.Min(MaxSpeed, Cap + 1f); lastPull = Time.realtimeSinceStartup; }
            Speed = Clamp(Speed, -6f, Cap);
            if (Speed > Top) Top = Speed;
            var v = dir * Speed;
            // Follow the slope while rolling; leave the vertical part to the game when it is jumping or falling.
            v.y = Grounded && cur.y < 1.5f ? tangent.y * Speed : cur.y;
            body.linearVelocity = v;
            Lean += (Clamp(Delta(before, Yaw) / dt * 0.12f, -25f, 25f) - Lean) * 0.2f;
            lastPos = pos; lastDir = dir; lastSpeed = Speed; hasLast = true; Steps++;
        }
        catch (Exception e) { On = false; ProbePlugin.L.LogMessage("SKATE step threw " + e.GetType().Name + ": " + e.Message); }
    }
}

// Created only after the game's walk component exists, so that its FixedUpdate is queued behind
// the game's own fixed step.
public class LateDriver : MonoBehaviour
{
    public LateDriver(IntPtr p) : base(p) { }

    public static int Calls;

    private void FixedUpdate()
    {
        Calls++;
        Skate.Step();
    }
}
