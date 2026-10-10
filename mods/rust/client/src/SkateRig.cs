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

    public static bool Mute;
    public static BasePlayer Local;
    public static Transform LocalT;
    public static Rigidbody Body;
    public static float AwakeAt = -1f;
    public static bool Ready { get { return Body != null && AwakeAt > 0f; } }

    private float nextTick, nextLog, lastJumpTap = -1f;
    private int standTicks;
    private GameObject board;
    private LateDriver late;
    private bool lateAdded, kDown, spaceDown, keysOk = true, hudFailed;

    private static void Say(string m) { Out.Say("SKATE " + m); }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        if (keysOk && !Scenarios.Active && Ready)
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
        if (Scenarios.Active && Ready && late != null)
        {
            try { Scenarios.Update(now); }
            catch (Exception e) { Say("scenario threw " + e.GetType().Name + ": " + e.Message); Scenarios.Name = ""; }
        }
        if (now < nextTick) return;
        nextTick = now + 1f;
        try { Tick(now); }
        catch (Exception e) { Say("tick threw: " + e.GetType().Name + ": " + e.Message); nextTick = now + 5f; }
    }

    [HideFromIl2Cpp]
    private void Toggle()
    {
        if (SkateRide.On) { SkateRide.Dismount(); Say("DISMOUNT " + SkateRide.Status()); }
        else { MountNow(SkateCamera.LookYaw); Say("MOUNT yaw=" + SkateRide.Yaw.ToString("F0") + " speed=" + SkateRide.Speed.ToString("F1")); }
    }

    public static void MountNow(float yaw)
    {
        if (!RiderRig.Bound) { try { RiderRig.Bind(LocalT); } catch (Exception e) { Out.Say("RIDER bind threw " + e.GetType().Name + ": " + e.Message); } }
        SkateRide.Mount(Body, yaw);
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
            SkateRide.Dismount(); Local = null; LocalT = null; AwakeAt = -1f; standTicks = 0; lateAdded = false; RiderRig.Bound = false;
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
            if (standTicks >= 2) { AwakeAt = now; Say("AWAKE player=" + Out.V1(LocalT.position) + (Scenarios.Active ? " | scenario " + Scenarios.Name : " | K or a double jump mounts")); }
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
        if (SkateRide.On && !RiderRig.Bound) { try { RiderRig.Bind(LocalT); } catch (Exception e) { Say("rider bind threw " + e.GetType().Name + ": " + e.Message); } }
    }

    private void LateUpdate()
    {
        if (board == null) return;
        try { SkateBoard.Follow(board, LocalT); }
        catch (Exception e) { Say("board update threw " + e.GetType().Name + ": " + e.Message); board = null; }
    }

    private void OnGUI()
    {
        if (hudFailed) return;
        try { SkateHud.Draw(); }
        catch (Exception e) { hudFailed = true; Say("text threw " + e.GetType().Name + ": " + e.Message); }
    }
}
