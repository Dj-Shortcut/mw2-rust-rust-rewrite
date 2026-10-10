// Finds the local player and the object that moves it, handles mounting and the board, and runs
// the scripted ride. The game's own members are obfuscated in the client, so this uses only Unity
// engine API, game type names and Unity message names. What the probe runs established
// (10 October 2026, build 25824447, issue #289):
//   - The local player is moved by a separate root object, assets/prefabs/player/player_movement.prefab:
//     PlayerWalkMovement, a dynamic Rigidbody (mass 0.5, no engine gravity, rotation frozen) and a
//     CapsuleCollider. The BasePlayer object follows it.
//   - That object is replaced when the player dies or reconnects, so it is looked up again.
// K, or two quick presses of the jump key, mounts and dismounts.
using System;
using Il2CppInterop.Runtime.Attributes;
using UnityEngine;
using UnityEngine.InputSystem;

public class SkateRig : MonoBehaviour
{
    public SkateRig(IntPtr p) : base(p) { }

    public static bool Mute, Test, Quiet;
    public static BasePlayer Local;
    public static Transform LocalT;
    public static Rigidbody Body;
    public static float AwakeAt = -1f;
    public static bool Ready { get { return Body != null && AwakeAt > 0f; } }

    private float nextTick, nextLog, mountAt = -1f, yaw0, lastJumpTap = -1f;
    private int standTicks, phase;
    private GameObject board;
    private LateDriver late;
    private bool lateAdded, kDown, spaceDown, keysOk = true, done;

    private static void Say(string m) { Out.Say("SKATE " + m); }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        if (keysOk && !Test && !Quiet && Ready)
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
        if (SkateRide.On && now >= nextLog) { nextLog = now + 1f; Say(SkateRide.Status()); }
        if (Test && mountAt > 0f) Script(now - mountAt);
        if (now < nextTick) return;
        nextTick = now + 1f;
        try { Tick(now); }
        catch (Exception e) { Say("tick threw: " + e.GetType().Name + ": " + e.Message); nextTick = now + 5f; }
    }

    [HideFromIl2Cpp]
    private void Toggle()
    {
        var cam = Camera.main;
        if (SkateRide.On) { SkateRide.Dismount(); Say("DISMOUNT " + SkateRide.Status()); }
        else if (cam != null) { SkateRide.Mount(Body, cam.transform.eulerAngles.y); Say("MOUNT yaw=" + SkateRide.Yaw.ToString("F0") + " speed=" + SkateRide.Speed.ToString("F1")); }
    }

    // Scripted ride for an unattended check: out along the reverse of the view direction, back again.
    [HideFromIl2Cpp]
    private void Script(float s)
    {
        var next = s < 2f ? 1 : s < 4f ? 2 : s < 5.5f ? 3 : s < 7.5f ? 4 : s < 9.5f ? 5 : s < 11.5f ? 6 : s < 13f ? 7 : s < 15f ? 8 : 9;
        if (next == phase) return;
        phase = next;
        SkateRide.SynthPush = phase == 1 || phase == 5;
        SkateRide.SynthBrake = phase == 3 || phase == 7;
        SkateRide.SynthYaw = phase <= 3 ? yaw0 + 180f : yaw0;
        Say("SCRIPT phase " + phase + (phase == 1 ? " push out" : phase == 2 ? " coast" : phase == 3 ? " brake" : phase == 4 ? " turn round" : phase == 5 ? " push back" : phase == 6 ? " coast" : phase == 7 ? " brake" : phase == 8 ? " dismount" : " finish") + " | " + SkateRide.Status());
        if (phase == 8 && SkateRide.On) SkateRide.Dismount();
        if (phase == 9 && !done)
        {
            done = true;
            System.IO.File.WriteAllText(System.IO.Path.Combine(BepInEx.Paths.PluginPath, "probe.done"), "done");
            Say("DONE top speed=" + SkateRide.Top.ToString("F1") + " pull-backs=" + SkateRide.Resets + " late calls=" + LateDriver.Calls);
        }
    }

    [HideFromIl2Cpp]
    private void Tick(float now)
    {
        if (Mute) AudioListener.volume = 0f;
        var cam = Camera.main;
        if (cam == null) return;
        var cp = cam.transform.position;
        if (Local != null && Body == null && AwakeAt > 0f)
        {
            Say("movement object gone; looking again");
            SkateRide.Dismount(); Local = null; LocalT = null; AwakeAt = -1f; standTicks = 0; lateAdded = false;
        }
        if (Local == null)
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
            Local = best; LocalT = best.transform; Body = w.gameObject.GetComponent<Rigidbody>();
            Say("LOCAL found player=" + Out.V1(LocalT.position) + " body=" + (Body != null));
            if (board == null) { try { board = SkateBoard.Build(); board.SetActive(false); } catch (Exception e) { Say("board threw " + e.GetType().Name + ": " + e.Message); } }
            return;
        }
        if (AwakeAt < 0f)
        {
            standTicks = cp.y - LocalT.position.y > 1.0f ? standTicks + 1 : 0;
            if (standTicks >= 2) { AwakeAt = now; Say("AWAKE player=" + Out.V1(LocalT.position) + (Test || Quiet ? "" : " | K or a double jump mounts")); }
            return;
        }
        if (Body == null) return;
        if (!lateAdded && now - AwakeAt >= 2f)
        {
            // A fresh driver for every movement object, so that it is always queued behind that object's walk component.
            lateAdded = true;
            try
            {
                if (late != null) UnityEngine.Object.Destroy(late);
                late = SkatePlugin.Instance.AddComponent<LateDriver>();
                Say("late driver added=" + (late != null));
            }
            catch (Exception e) { Say("late driver threw " + e.GetType().Name + ": " + e.Message); }
        }
        if (Test && mountAt < 0f && now - AwakeAt >= 4f)
        {
            yaw0 = cam.transform.eulerAngles.y;
            SkateRide.Synth = true; SkateRide.SynthYaw = yaw0 + 180f;
            SkateRide.Mount(Body, yaw0 + 180f);
            mountAt = now;
            Say("MOUNT (scripted) view yaw=" + yaw0.ToString("F0") + " at " + Out.V1(LocalT.position));
        }
    }

    private void LateUpdate()
    {
        if (board == null) return;
        try { SkateBoard.Follow(board, LocalT); }
        catch (Exception e) { Say("board update threw " + e.GetType().Name + ": " + e.Message); board = null; }
    }
}
