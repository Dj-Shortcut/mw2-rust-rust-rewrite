// The ride: a signed speed along a heading, written into the movement body after the game's own
// fixed step. Established by the probe runs (issue #289):
//   - The game's fixed step cancels any velocity written before it. A component created after the
//     walk component exists gets its FixedUpdate after the game's, and a velocity written there
//     moves the player; the game keeps handling gravity, jumping and the model.
//   - Disabling the walk component is not usable: the player is then pulled back constantly.
//   - Above walking pace the server pulls the player back unless its anti-hack leaves them alone,
//     so the speed cap backs off below whatever was refused and probes upward again later.
// Mounted: forward pushes, back brakes, the board turns toward where the camera looks, left and
// right carve harder, the game's own jump still works. Slopes speed the board up, slow it down
// and can roll it backwards.
// Plain arithmetic is done here rather than through UnityEngine.Mathf, because engine methods the
// game itself never calls can be missing from the build.
using System;
using UnityEngine;
using UnityEngine.InputSystem;

public static class SkateRide
{
    public const float PushAccel = 6f, MaxPush = 8f, BrakeDecel = 12f, RollDecel = 0.45f, MaxSpeed = 13f, MaxReverse = 6f, TurnRate = 150f, SlopeGain = 1.6f;
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

    public static Vector3 Dir(float yaw) { return Quaternion.Euler(0f, yaw, 0f) * Vector3.forward; }
    private static float Toward(float a, float b, float step) { return Math.Abs(b - a) <= step ? b : a + Math.Sign(b - a) * step; }
    public static float Delta(float from, float to) { var d = (to - from) % 360f; if (d > 180f) d -= 360f; if (d < -180f) d += 360f; return d; }
    public static float Clamp(float v, float lo, float hi) { return v < lo ? lo : v > hi ? hi : v; }

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
            + " pull-backs=" + Resets + (body != null ? " pos=" + Out.V1(body.position) : "");
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
                if ((lastSpeed > 1f && Actual < -3f) || (lastSpeed < -1f && Actual > 3f))
                {
                    // The server refused the last stretch. Stay under the speed it refused, and try a little more later.
                    Resets++; lastPull = Time.realtimeSinceStartup;
                    Cap = Math.Max(2.6f, Math.Min(Cap, Math.Abs(lastSpeed) * 0.8f));
                }
                // An obstacle takes the speed away, in either direction of travel.
                if (Speed > 0f) { var limit = (Actual > 0f ? Actual : 0f) + 2f; if (Speed > limit) Speed = limit; }
                else if (Speed < 0f) { var limit = (Actual < 0f ? Actual : 0f) - 2f; if (Speed < limit) Speed = limit; }
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
            Speed = Clamp(Speed, -Math.Min(MaxReverse, Cap), Cap);
            if (Speed > Top) Top = Speed;
            var v = dir * Speed;
            // Follow the slope while rolling; leave the vertical part to the game when it is jumping or falling.
            v.y = Grounded && cur.y < 1.5f ? tangent.y * Speed : cur.y;
            body.linearVelocity = v;
            Lean += (Clamp(Delta(before, Yaw) / dt * 0.12f, -25f, 25f) - Lean) * 0.2f;
            lastPos = pos; lastDir = dir; lastSpeed = Speed; hasLast = true; Steps++;
        }
        catch (Exception e) { On = false; Out.Say("SKATE step threw " + e.GetType().Name + ": " + e.Message); }
    }
}

// Created only after the game's walk component exists, so that its FixedUpdate and LateUpdate are
// queued behind the game's own.
public class LateDriver : MonoBehaviour
{
    public LateDriver(IntPtr p) : base(p) { }

    public static int Calls;

    private void FixedUpdate()
    {
        Calls++;
        SkateRide.Step();
    }

    private void LateUpdate()
    {
        ModelProbeCode.Late();
    }
}
