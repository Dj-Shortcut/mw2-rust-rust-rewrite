// Diagnostics. LoaderSteps are the loader checks from issue #289, kept for the next Rust update.
// ModelProbe answers, in one scripted session, what the rider pose, a third-person camera, sound,
// on-screen text and a console command route can be built on in this client build.
using System;
using System.Diagnostics;
using System.Reflection;
using System.Runtime.InteropServices;
using Il2CppInterop.Runtime;
using Il2CppInterop.Runtime.Attributes;
using Il2CppInterop.Runtime.InteropTypes;
using UnityEngine;

public static class LoaderSteps
{
    private static long lo, hi;

    public static string Run(SkatePlugin plugin, string s)
    {
        if (hi == 0)
            foreach (ProcessModule m in Process.GetCurrentProcess().Modules)
                if (m.ModuleName.Equals("GameAssembly.dll", StringComparison.OrdinalIgnoreCase))
                { lo = (long)m.BaseAddress; hi = lo + m.ModuleMemorySize; }
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
            case "inject": return "added=" + (plugin.AddComponent<ProbeBehaviour>() != null);
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
            Out.Say("PROBE update frame " + n + " t=" + Time.realtimeSinceStartup);
    }
}

// Timeline, in seconds after the player has stood for three seconds:
//   0 dump the rider model, 1 mount and roll at walking pace, 2 own side camera,
//   6 Animator speed 0, 10 Animator disabled, 14 one leg bone overridden after animation,
//   18 own camera off and the main camera moved instead, 22 brake and a test tone,
//   24 console command through onSubmit (midnight), 29 through onEndEdit (noon), 34 noon again
//   through onSubmit, 36 done. The current phase is drawn on screen when IMGUI works.
public class ModelProbe : MonoBehaviour
{
    public ModelProbe(IntPtr p) : base(p) { }

    private float start = -1f;
    private int phase;
    private bool done, hudFailed;

    private void Update()
    {
        if (done || !SkateRig.Ready) return;
        var now = Time.realtimeSinceStartup;
        if (start < 0f) { if (now - SkateRig.AwakeAt < 3f) return; start = now; }
        var t = now - start;
        var next = t < 1f ? 1 : t < 2f ? 2 : t < 6f ? 3 : t < 10f ? 4 : t < 14f ? 5 : t < 18f ? 6 : t < 22f ? 7 : t < 24f ? 8 : t < 29f ? 9 : t < 34f ? 10 : t < 36f ? 11 : 12;
        if (next != phase)
        {
            phase = next;
            Out.Say("PROBE phase " + phase + " begins");
            try { done = ModelProbeCode.Enter(phase); }
            catch (Exception e) { Out.Say("PROBE phase " + phase + " threw " + e.GetType().Name + ": " + e.Message); }
        }
        if (SkateRide.On && SkateRide.Synth) SkateRide.SynthPush = phase <= 7 && SkateRide.Speed < 2.4f;
        try { ModelProbeCode.FollowOwnCamera(); }
        catch (Exception e) { Out.Say("PROBE own camera threw " + e.GetType().Name + ": " + e.Message); ModelProbeCode.Own = null; }
    }

    private void OnGUI()
    {
        if (hudFailed) return;
        try { ModelProbeCode.Hud(); }
        catch (Exception e) { hudFailed = true; Out.Say("PROBE text threw " + e.GetType().Name + ": " + e.Message); }
    }
}

public static class ModelProbeCode
{
    public static string Phase = "waiting";
    public static Animator Rider;
    public static Transform Thigh;
    public static Camera Own;
    public static bool OverrideBone, MoveMainCamera;
    private static GUIStyle style;
    private static bool hudLogged, lateFailed, fieldsLogged;

    private static void Say(string m) { Out.Say("PROBE " + m); }

    public static string Cls(Il2CppObjectBase o)
    {
        try { return Marshal.PtrToStringAnsi(IL2CPP.il2cpp_class_get_name(IL2CPP.il2cpp_object_get_class(o.Pointer))); }
        catch (Exception e) { return "?" + e.GetType().Name; }
    }

    public static string Comps(GameObject go)
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

    // Returns true when the probe is finished.
    public static bool Enter(int phase)
    {
        switch (phase)
        {
            case 1: Phase = "1 dump"; Dump(); break;
            case 2:
                Phase = "2 mount";
                var cam = Camera.main;
                var yaw = cam != null ? cam.transform.eulerAngles.y : 0f;
                SkateRide.Synth = true; SkateRide.SynthYaw = yaw; SkateRide.SynthBrake = false;
                SkateRide.Mount(SkateRig.Body, yaw);
                break;
            case 3: Phase = "3 own camera, normal animation"; OwnCamera(true); break;
            case 4: Phase = "4 animator speed 0"; if (Rider != null) Rider.speed = 0f; break;
            case 5: Phase = "5 animator disabled"; if (Rider != null) { Rider.speed = 1f; Rider.enabled = false; } break;
            case 6: Phase = "6 leg bone overridden"; if (Rider != null) Rider.enabled = true; OverrideBone = true; break;
            case 7: Phase = "7 main camera moved"; OverrideBone = false; OwnCamera(false); MoveMainCamera = true; break;
            case 8: Phase = "8 brake and tone"; MoveMainCamera = false; SkateRide.SynthBrake = true; Audio(); break;
            case 9: Phase = "9 console onSubmit: midnight"; SkateRide.Dismount(); SkateRide.Synth = false; Console("env.time 0", 1); break;
            case 10: Phase = "10 console onEndEdit: noon"; Console("env.time 12", 2); break;
            case 11: Phase = "11 console onSubmit: noon"; Console("env.time 12", 1); break;
            default:
                Phase = "done";
                System.IO.File.WriteAllText(System.IO.Path.Combine(BepInEx.Paths.PluginPath, "probe.done"), "done");
                Say("DONE");
                return true;
        }
        return false;
    }

    public static void Dump()
    {
        var playerT = SkateRig.LocalT; var pos = playerT.position;
        PlayerModel model = null;
        try
        {
            var ms = UnityEngine.Object.FindObjectsOfType<PlayerModel>();
            var bd = 999f;
            Say("MODEL player models=" + ms.Length);
            for (var i = 0; i < ms.Length; i++)
            {
                var d = Vector3.Distance(ms[i].transform.position, pos);
                if (d < 6f) Say("MODEL candidate go=" + ms[i].gameObject.name + " d=" + d.ToString("F2") + " parent=" + Out.Chain(ms[i].transform) + " comps:" + Comps(ms[i].gameObject));
                if (d < bd) { bd = d; model = ms[i]; }
            }
            if (bd > 2f) model = null;
        }
        catch (Exception e) { Say("MODEL search threw " + e.GetType().Name + ": " + e.Message); }
        if (model == null) { Say("MODEL none near the player"); return; }
        var m = model;
        Say("MODEL chosen go=" + m.gameObject.name + " pos=" + Out.V(m.transform.position) + " player=" + Out.V(pos) + " scale=" + Out.V(m.transform.lossyScale));
        Say("MODEL legsAnimator=" + Out.Try(() => m.legsAnimator == null ? "none" : Cls(m.legsAnimator) + " on " + m.legsAnimator.gameObject.name + " enabled=" + m.legsAnimator.enabled)
            + " tempPoseType=" + Out.Try(() => m.tempPoseType.ToString()) + " isPreview=" + Out.Try(() => m.isPreview.ToString()));
        Say("MODEL feet left=" + Out.Try(() => m.leftFootBone == null ? "none" : m.leftFootBone.name) + " right=" + Out.Try(() => m.rightFootBone == null ? "none" : m.rightFootBone.name)
            + " head=" + Out.Try(() => m.headBone == null ? "none" : m.headBone.name) + " neck=" + Out.Try(() => m.neckBone == null ? "none" : m.neckBone.name)
            + " clavicleL=" + Out.Try(() => m.leftClavicleBone == null ? "none" : m.leftClavicleBone.name));
        Say("MODEL spine=" + Out.Try(() => { var s = ""; var a = m.SpineBones; for (var i = 0; a != null && i < a.Length; i++) s += (a[i] == null ? "null" : a[i].name) + " "; return s; })
            + "| shoulders=" + Out.Try(() => { var s = ""; var a = m.Shoulders; for (var i = 0; a != null && i < a.Length; i++) s += (a[i] == null ? "null" : a[i].name) + " "; return s; }));
        Say("MODEL foot targets L=" + Out.Try(() => Out.V(m.leftFootTargetPosition)) + " R=" + Out.Try(() => Out.V(m.rightFootTargetPosition)) + " hand targets L=" + Out.Try(() => Out.V(m.leftHandTargetPosition)) + " R=" + Out.Try(() => Out.V(m.rightHandTargetPosition)));

        Animator an = null;
        try { an = m.GetComponentInChildren<Animator>(true); } catch (Exception e) { Say("ANIM search threw " + e.GetType().Name + ": " + e.Message); }
        if (an == null) { Say("ANIM none under the model"); return; }
        Rider = an;
        var a2 = an;
        Say("ANIM on " + a2.gameObject.name + " parent=" + Out.Chain(a2.transform) + " enabled=" + a2.enabled + " isHuman=" + Out.Try(() => a2.isHuman.ToString()) + " speed=" + Out.Try(() => a2.speed.ToString("F2"))
            + " updateMode=" + Out.Try(() => a2.updateMode.ToString()) + " culling=" + Out.Try(() => a2.cullingMode.ToString()) + " layers=" + Out.Try(() => a2.layerCount.ToString())
            + " controller=" + Out.Try(() => a2.runtimeAnimatorController == null ? "none" : a2.runtimeAnimatorController.name) + " avatar=" + Out.Try(() => a2.avatar == null ? "none" : a2.avatar.name + " human=" + a2.avatar.isHuman));
        Say("ANIM parameters: " + Out.Try(() => { var s = ""; var n = a2.parameterCount; for (var i = 0; i < n && i < 80; i++) { var p = a2.GetParameter(i); s += p.name + ":" + p.type + " "; } return n + " | " + s; }));

        var root = a2.transform;
        var names = new[] { HumanBodyBones.Hips, HumanBodyBones.Spine, HumanBodyBones.Chest, HumanBodyBones.UpperChest, HumanBodyBones.Neck, HumanBodyBones.Head,
            HumanBodyBones.LeftUpperLeg, HumanBodyBones.LeftLowerLeg, HumanBodyBones.LeftFoot, HumanBodyBones.LeftToes, HumanBodyBones.RightUpperLeg, HumanBodyBones.RightLowerLeg, HumanBodyBones.RightFoot, HumanBodyBones.RightToes,
            HumanBodyBones.LeftShoulder, HumanBodyBones.LeftUpperArm, HumanBodyBones.LeftLowerArm, HumanBodyBones.LeftHand, HumanBodyBones.RightShoulder, HumanBodyBones.RightUpperArm, HumanBodyBones.RightLowerArm, HumanBodyBones.RightHand };
        foreach (var b in names)
        {
            var bone = b;
            Say("HUMAN " + bone + " = " + Out.Try(() => { var t = a2.GetBoneTransform(bone); return t == null ? "none" : t.name + " at " + Out.V3(playerT.InverseTransformPoint(t.position)); }));
        }
        try { Thigh = a2.GetBoneTransform(HumanBodyBones.LeftUpperLeg); } catch (Exception e) { Say("thigh lookup threw " + e.GetType().Name); }

        try
        {
            var skins = m.GetComponentsInChildren<SkinnedMeshRenderer>(true);
            Say("SKIN renderers=" + skins.Length);
            for (var i = 0; i < skins.Length && i < 24; i++)
            {
                var s = skins[i];
                Say("SKIN " + s.gameObject.name + " active=" + s.gameObject.activeInHierarchy + " enabled=" + s.enabled + " mesh=" + Out.Try(() => s.sharedMesh == null ? "none" : s.sharedMesh.name)
                    + " bones=" + Out.Try(() => s.bones == null ? "none" : s.bones.Length.ToString()) + " root=" + Out.Try(() => s.rootBone == null ? "none" : s.rootBone.name));
            }
        }
        catch (Exception e) { Say("SKIN scan threw " + e.GetType().Name + ": " + e.Message); }

        budget = 260;
        Say("BONES under " + root.gameObject.name + " (name, local position, local euler):");
        try { Tree(root, 0); } catch (Exception e) { Say("BONES dump threw " + e.GetType().Name + ": " + e.Message); }
        if (budget <= 0) Say("BONES dump cut short");
    }

    private static int budget;

    private static void Tree(Transform t, int depth)
    {
        if (budget-- <= 0) return;
        Out.Say("B " + new string('.', depth) + t.gameObject.name + " p=" + Out.V3(t.localPosition) + " r=" + t.localEulerAngles.x.ToString("F0") + "," + t.localEulerAngles.y.ToString("F0") + "," + t.localEulerAngles.z.ToString("F0") + (t.gameObject.activeSelf ? "" : " (inactive)"));
        var n = t.childCount;
        for (var i = 0; i < n; i++) Tree(t.GetChild(i), depth + 1);
    }

    private static Vector3 SideView(out Vector3 target)
    {
        var p = SkateRig.LocalT.position;
        var side = SkateRide.Dir(SkateRide.Yaw + 90f);
        target = p + Vector3.up * 0.9f;
        return p + side * 2.8f + Vector3.up * 1.3f + SkateRide.Dir(SkateRide.Yaw) * 0.6f;
    }

    public static void OwnCamera(bool on)
    {
        if (!on) { if (Own != null) { UnityEngine.Object.Destroy(Own.gameObject); Own = null; Say("CAMERA own camera removed"); } return; }
        var main = Camera.main;
        var go = new GameObject("skate_probe_camera");
        var c = go.AddComponent<Camera>();
        Say("CAMERA copy from main: " + Out.Try(() => { c.CopyFrom(main); return "ok"; }));
        Say("CAMERA main depth=" + Out.Try(() => main.depth.ToString("F1")) + " mask=" + Out.Try(() => main.cullingMask.ToString("X")) + " clear=" + Out.Try(() => main.clearFlags.ToString()) + " fov=" + Out.Try(() => main.fieldOfView.ToString("F0")));
        c.depth = main.depth + 5f;
        UnityEngine.Object.DontDestroyOnLoad(go);
        Own = c;
        Say("CAMERA own camera created");
    }

    public static void FollowOwnCamera()
    {
        if (Own == null || SkateRig.LocalT == null) return;
        Vector3 target;
        Own.transform.position = SideView(out target);
        Own.transform.LookAt(target);
    }

    // Called from LateDriver.LateUpdate, which is queued behind the game's own late updates.
    public static void Late()
    {
        if (lateFailed) return;
        try
        {
            if (OverrideBone && Thigh != null) Thigh.localRotation = Thigh.localRotation * Quaternion.Euler(0f, 0f, 70f);
            if (MoveMainCamera && SkateRig.LocalT != null)
            {
                var main = Camera.main;
                if (main != null) { Vector3 target; main.transform.position = SideView(out target); main.transform.LookAt(target); }
            }
        }
        catch (Exception e) { lateFailed = true; Say("late overrides threw " + e.GetType().Name + ": " + e.Message); }
    }

    public static void Audio()
    {
        Say("AUDIO begin");
        const int rate = 22050;
        var data = new float[rate];
        for (var i = 0; i < rate; i++) data[i] = (float)(Math.Sin(i * 2 * Math.PI * 440 / rate) * 0.25 * (1.0 - i / (double)rate));
        var clip = AudioClip.Create("skate_probe_tone", rate, 1, rate, false);
        Say("AUDIO clip created=" + (clip != null));
        Say("AUDIO set data=" + clip.SetData(data, 0));
        var go = new GameObject("skate_probe_audio");
        var src = go.AddComponent<AudioSource>();
        src.spatialBlend = 0f; src.volume = 1f;
        src.PlayOneShot(clip);
        Say("AUDIO one-shot started, listener volume=" + AudioListener.volume.ToString("F2"));
    }

    public static void Console(string command, int method)
    {
        TMPro.TMP_InputField target = null;
        var all = Resources.FindObjectsOfTypeAll(Il2CppType.Of<TMPro.TMP_InputField>());
        if (!fieldsLogged) Say("CONSOLE text input fields=" + all.Length);
        for (var i = 0; i < all.Length; i++)
        {
            var f = all[i].TryCast<TMPro.TMP_InputField>();
            if (f == null) continue;
            var hint = Out.Try(() => { var g = f.placeholder; var t = g == null ? null : g.TryCast<TMPro.TMP_Text>(); return t == null ? "" : t.text; });
            if (!fieldsLogged) Say("CONSOLE field go=" + f.gameObject.name + " active=" + f.gameObject.activeInHierarchy + " hint='" + hint + "' parent=" + Out.Chain(f.transform));
            if (target == null && (hint == "Enter Command" || f.gameObject.name.ToLowerInvariant().Contains("console"))) target = f;
        }
        if (!fieldsLogged)
        {
            var old = Resources.FindObjectsOfTypeAll(Il2CppType.Of<UnityEngine.UI.InputField>());
            Say("CONSOLE legacy input fields=" + old.Length);
            for (var i = 0; i < old.Length && i < 12; i++)
            {
                var f = old[i].TryCast<UnityEngine.UI.InputField>();
                if (f != null) Say("CONSOLE legacy field go=" + f.gameObject.name + " active=" + f.gameObject.activeInHierarchy + " parent=" + Out.Chain(f.transform));
            }
        }
        fieldsLogged = true;
        if (target == null) { Say("CONSOLE no console field found; '" + command + "' not sent"); return; }
        target.text = command;
        if (method == 1) target.onSubmit.Invoke(command); else target.onEndEdit.Invoke(command);
        Say("CONSOLE sent '" + command + "' through " + (method == 1 ? "onSubmit" : "onEndEdit") + " of " + target.gameObject.name);
    }

    public static void Hud()
    {
        if (style == null)
        {
            style = new GUIStyle(GUI.skin.label);
            style.fontSize = 30; style.fontStyle = FontStyle.Bold;
            style.normal.textColor = Color.white;
        }
        GUI.Label(new Rect(40f, 110f, 1400f, 50f), "SKATE PROBE  " + Phase, style);
        if (!hudLogged) { hudLogged = true; Say("TEXT first label drawn"); }
    }
}
