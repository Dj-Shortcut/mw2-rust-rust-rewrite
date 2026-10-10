// Diagnostics. LoaderSteps are the loader checks from issue #289, kept for the next Rust update.
// ModelDump logs what the rider model is made of: components, Animator parameters, the humanoid
// bone mapping with positions, skinned meshes and the bone tree.
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

public static class ModelDump
{
    private static int budget;

    private static void Say(string m) { Out.Say("MODEL " + m); }

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

    public static void Run()
    {
        var playerT = SkateRig.LocalT; var pos = playerT.position;
        PlayerModel model = null;
        try
        {
            var ms = UnityEngine.Object.FindObjectsOfType<PlayerModel>();
            var bd = 999f;
            Say("player models=" + ms.Length);
            for (var i = 0; i < ms.Length; i++)
            {
                var d = Vector3.Distance(ms[i].transform.position, pos);
                if (d < 6f) Say("candidate go=" + ms[i].gameObject.name + " d=" + d.ToString("F2") + " parent=" + Out.Chain(ms[i].transform) + " comps:" + Comps(ms[i].gameObject));
                if (d < bd) { bd = d; model = ms[i]; }
            }
            if (bd > 2f) model = null;
        }
        catch (Exception e) { Say("search threw " + e.GetType().Name + ": " + e.Message); }
        if (model == null) { Say("none near the player"); return; }
        var m = model;
        Say("chosen go=" + m.gameObject.name + " pos=" + Out.V(m.transform.position) + " player=" + Out.V(pos) + " scale=" + Out.V(m.transform.lossyScale));
        Say("legsAnimator=" + Out.Try(() => m.legsAnimator == null ? "none" : Cls(m.legsAnimator) + " on " + m.legsAnimator.gameObject.name + " enabled=" + m.legsAnimator.enabled)
            + " tempPoseType=" + Out.Try(() => m.tempPoseType.ToString()) + " isPreview=" + Out.Try(() => m.isPreview.ToString()));
        Say("feet left=" + Out.Try(() => m.leftFootBone == null ? "none" : m.leftFootBone.name) + " right=" + Out.Try(() => m.rightFootBone == null ? "none" : m.rightFootBone.name)
            + " head=" + Out.Try(() => m.headBone == null ? "none" : m.headBone.name) + " neck=" + Out.Try(() => m.neckBone == null ? "none" : m.neckBone.name));
        Say("spine=" + Out.Try(() => { var s = ""; var a = m.SpineBones; for (var i = 0; a != null && i < a.Length; i++) s += (a[i] == null ? "null" : a[i].name) + " "; return s; }));

        Animator an = null;
        try { an = m.GetComponentInChildren<Animator>(true); } catch (Exception e) { Say("animator search threw " + e.GetType().Name + ": " + e.Message); }
        if (an == null) { Say("no animator under the model"); return; }
        var a2 = an;
        Say("animator on " + a2.gameObject.name + " enabled=" + a2.enabled + " isHuman=" + Out.Try(() => a2.isHuman.ToString()) + " updateMode=" + Out.Try(() => a2.updateMode.ToString())
            + " culling=" + Out.Try(() => a2.cullingMode.ToString()) + " layers=" + Out.Try(() => a2.layerCount.ToString()) + " avatar=" + Out.Try(() => a2.avatar == null ? "none" : a2.avatar.name + " human=" + a2.avatar.isHuman));
        Say("parameters: " + Out.Try(() => { var s = ""; var n = a2.parameterCount; for (var i = 0; i < n && i < 80; i++) { var p = a2.GetParameter(i); s += p.name + ":" + p.type + " "; } return n + " | " + s; }));
        var names = new[] { HumanBodyBones.Hips, HumanBodyBones.Spine, HumanBodyBones.Chest, HumanBodyBones.UpperChest, HumanBodyBones.Neck, HumanBodyBones.Head,
            HumanBodyBones.LeftUpperLeg, HumanBodyBones.LeftLowerLeg, HumanBodyBones.LeftFoot, HumanBodyBones.LeftToes, HumanBodyBones.RightUpperLeg, HumanBodyBones.RightLowerLeg, HumanBodyBones.RightFoot, HumanBodyBones.RightToes,
            HumanBodyBones.LeftShoulder, HumanBodyBones.LeftUpperArm, HumanBodyBones.LeftLowerArm, HumanBodyBones.LeftHand, HumanBodyBones.RightShoulder, HumanBodyBones.RightUpperArm, HumanBodyBones.RightLowerArm, HumanBodyBones.RightHand };
        foreach (var b in names)
        {
            var bone = b;
            Say("human " + bone + " = " + Out.Try(() => { var t = a2.GetBoneTransform(bone); return t == null ? "none" : t.name + " at " + Out.V3(playerT.InverseTransformPoint(t.position)); }));
        }
        try
        {
            var skins = m.GetComponentsInChildren<SkinnedMeshRenderer>(true);
            Say("skinned renderers=" + skins.Length);
            for (var i = 0; i < skins.Length && i < 40; i++)
            {
                var s = skins[i];
                Say("skin " + s.gameObject.name + " active=" + s.gameObject.activeInHierarchy + " enabled=" + s.enabled + " layer=" + s.gameObject.layer + " bones=" + Out.Try(() => s.bones == null ? "none" : s.bones.Length.ToString()));
            }
        }
        catch (Exception e) { Say("skin scan threw " + e.GetType().Name + ": " + e.Message); }
        budget = 260;
        Say("bones under " + a2.gameObject.name + " (name, local position, local euler):");
        try { Tree(a2.transform, 0); } catch (Exception e) { Say("bone dump threw " + e.GetType().Name + ": " + e.Message); }
        if (budget <= 0) Say("bone dump cut short");
    }

    // The rider model's skinned meshes and how each is drawn. The local player's model is set up
    // for first person; this shows what the chase camera has to work with.
    public static void Renderers()
    {
        try
        {
            if (!RiderRig.Bound && SkateRig.LocalT != null) RiderRig.Bind(SkateRig.LocalT);
            if (!RiderRig.Bound) { Say("no rider model bound"); return; }
            var skins = RiderRig.Root.GetComponentsInChildren<SkinnedMeshRenderer>(true);
            var line = "";
            for (var i = 0; i < skins.Length && i < 48; i++)
            {
                var s = skins[i];
                line += s.gameObject.name + "[" + (s.gameObject.activeInHierarchy ? "a" : "-") + (s.enabled ? "e" : "-") + " shadows=" + Out.Try(() => ((int)s.shadowCastingMode).ToString()) + "] ";
                if (line.Length > 900) { Say("renderers: " + line); line = ""; }
            }
            Say("renderers (" + skins.Length + "): " + line);
            Say("head bone scale=" + Out.V(RiderRig.Head.localScale) + " neck scale=" + Out.V(RiderRig.Neck.localScale) + " model root at " + Out.V3(SkateRig.LocalT.InverseTransformPoint(RiderRig.Root.position))
                + " head at " + Out.V3(SkateRig.LocalT.InverseTransformPoint(RiderRig.Head.position)) + " pelvis at " + Out.V3(SkateRig.LocalT.InverseTransformPoint(RiderRig.Pelvis.position)));
        }
        catch (Exception e) { Say("renderer scan threw " + e.GetType().Name + ": " + e.Message); }
    }

    private static void Tree(Transform t, int depth)
    {
        if (budget-- <= 0) return;
        Out.Say("B " + new string('.', depth) + t.gameObject.name + " p=" + Out.V3(t.localPosition) + " r=" + t.localEulerAngles.x.ToString("F0") + "," + t.localEulerAngles.y.ToString("F0") + "," + t.localEulerAngles.z.ToString("F0") + " s=" + t.localScale.x.ToString("F2") + (t.gameObject.activeSelf ? "" : " (inactive)"));
        var n = t.childCount;
        for (var i = 0; i < n; i++) Tree(t.GetChild(i), depth + 1);
    }
}
