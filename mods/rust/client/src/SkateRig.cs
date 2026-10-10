// The client's game code is obfuscated: only Unity engine API, game component type names and
// Unity message names can be relied on. The local player is moved by a separate object (the walk
// component, a dynamic Rigidbody, a capsule) that is replaced on death and reconnect, so it is
// looked up again and gets a fresh late driver each time.
using System;
using Il2CppInterop.Runtime.Attributes;
using UnityEngine;

public class SkateRig : MonoBehaviour
{
    public SkateRig(IntPtr p) : base(p) { }

    public static bool Mute;
    public static BasePlayer Local;
    public static Transform LocalT;
    public static Rigidbody Body;
    public static PlayerWalkMovement Walk;
    public static float AwakeAt = -1f;
    public static bool Ready { get { return Body != null && AwakeAt > 0f; } }
    public static bool CanMount { get { return Ready && late != null && !SkateRide.On && Time.realtimeSinceStartup - offAt > 0.7f; } }

    private static GameObject board;
    private static LateDriver late;
    private static float offAt = -99f, lastFrame;
    private static bool wasOn;
    private float nextTick, nextLog, lastJumpTap = -1f, crouchSince = -1f;
    private int standTicks;
    private bool lateAdded, hudFailed;

    private static void Say(string m) { Out.Say("SKATE " + m); }

    private void Update()
    {
        var now = Time.realtimeSinceStartup;
        SkateKeys.Poll();
        if (!Scenarios.Active && Ready)
        {
            if (SkateRide.On)
            {
                if (SkateKeys.CrouchTap && SkateRide.Mode == RideMode.Ground) crouchSince = now;
                if (!SkateKeys.Crouch || SkateRide.Mode != RideMode.Ground) crouchSince = -1f;
                if (SkateKeys.ToggleTap || (crouchSince > 0f && now - crouchSince > 0.6f)) { crouchSince = -1f; SkateRide.Dismount("stepped off"); }
                if (SkateKeys.CameraTap) SkateCamera.Chase = !SkateCamera.Chase;
            }
            else
            {
                var twice = false;
                if (SkateKeys.JumpTap) { twice = now - lastJumpTap < 0.4f; lastJumpTap = twice ? -1f : now; }
                if ((SkateKeys.ToggleTap || twice) && CanMount) MountNow(SkateCamera.LookYaw);
            }
        }
        if (SkateRide.On && now >= nextLog) { nextLog = now + 1f; Say(SkateRide.Status()); }
        if (wasOn && !SkateRide.On) { offAt = now; Say("OFF (" + SkateRide.OffReason + ") top=" + SkateRide.Top.ToString("F1") + " pull-backs=" + SkateRide.Resets + " score=" + SkateRide.Trick.TotalPoints); }
        wasOn = SkateRide.On;
        if (Scenarios.Active && Ready && late != null)
        {
            try { Scenarios.Update(now); }
            catch (Exception e) { Say("scenario threw " + e.GetType().Name + ": " + e.Message); Scenarios.Abort(); }
        }
        SkateSfx.Update(Time.deltaTime);
        if (now < nextTick) return;
        nextTick = now + 1f;
        try { Tick(now); }
        catch (Exception e) { Say("tick threw: " + e.GetType().Name + ": " + e.Message); nextTick = now + 5f; }
    }

    public static void MountNow(float yaw)
    {
        if (Body == null) return;
        if (!RiderRig.Bound) { try { RiderRig.Bind(LocalT); } catch (Exception e) { Out.Say("RIDER bind threw " + e.GetType().Name + ": " + e.Message); } }
        SkateSfx.Start();
        SkateRide.Mount(Body, yaw);
        SkateHud.MountedAt = Time.realtimeSinceStartup;
        Say("MOUNT yaw=" + SkateRide.Yaw.ToString("F0") + " speed=" + SkateRide.Speed.ToString("F1") + " rider=" + RiderRig.Bound + " audio=" + SkateSfx.State);
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
            SkateRide.Dismount("the movement object is gone"); Local = null; LocalT = null; Walk = null; AwakeAt = -1f; standTicks = 0; lateAdded = false; RiderRig.Bound = false;
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
            Local = best; LocalT = best.transform; Walk = w; Body = w.gameObject.GetComponent<Rigidbody>();
            Say("LOCAL found player=" + Out.V1(LocalT.position) + " body=" + (Body != null));
            if (board == null) { try { board = SkateBoard.Build(); board.SetActive(false); } catch (Exception e) { Say("board threw " + e.GetType().Name + ": " + e.Message); } }
            return;
        }
        if (AwakeAt < 0f)
        {
            // The camera lies on the ground while the player sleeps.
            standTicks = cp.y - LocalT.position.y > 1.0f ? standTicks + 1 : 0;
            if (standTicks >= 2) { AwakeAt = now; Say("AWAKE player=" + Out.V1(LocalT.position) + (Scenarios.Active ? " | scenario " + Scenarios.Name : " | K or a double jump gets on")); }
            return;
        }
        if (Body == null) return;
        // The game switches its walk component off when something else moves the player (a seat, a vehicle).
        if (SkateRide.On && Walk != null && !Walk.enabled) SkateRide.Dismount("the game took over the movement");
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

    public static void LateFrame()
    {
        var now = Time.realtimeSinceStartup;
        var frame = SkateRide.Clamp(now - lastFrame, 0.001f, 0.1f); lastFrame = now;
        SkateCamera.Sample();
        if (!SkateKeys.Scripted) SkateKeys.LookYaw = SkateCamera.LookYaw;
        if (board != null)
        {
            try { SkateBoard.Follow(board, LocalT); }
            catch (Exception e) { Say("board update threw " + e.GetType().Name + ": " + e.Message); board = null; }
        }
        var third = SkateRide.On && LocalT != null && (SkateCamera.Chase || SkateCamera.Fixed);
        RiderRig.Look(third);
        if (!SkateRide.On || LocalT == null) return;
        var at = SkateBoard.Contact(LocalT);
        RiderRig.Frame(at, SkateRide.ViewNormal, !third, frame);
        SkateCamera.Apply(at);
    }

    private void OnGUI()
    {
        if (hudFailed) return;
        try { SkateHud.Draw(); }
        catch (Exception e) { hudFailed = true; Say("text threw " + e.GetType().Name + ": " + e.Message); }
    }
}
