// The ride. While mounted the plugin owns the whole velocity of the movement body: a signed speed
// along a heading on the ground, a ballistic arc in the air, a line along an edge while grinding.
// Established by the probe runs (issues #289 and #337):
//   - The game's fixed step cancels any velocity written before it. A component created after the
//     walk component exists gets its FixedUpdate after the game's, and a velocity written there
//     moves the player. Disabling the walk component is not usable: the player is pulled back.
//   - Above walking pace the server pulls the player back unless its skate plugin accepts the
//     movement, so the speed cap backs off below whatever was refused and probes upward again.
// Ground: forward pushes, back brakes, the board turns toward where the camera looks, left and
// right carve harder; slopes speed the board up, slow it down and roll it backwards.
// Air: the heading is kept; a tap queues one whole flip or one half spin, accepted only when the
// arc is long enough to finish it. Touch-down, bails, combos and names come from the shared trick
// module. Plain arithmetic instead of UnityEngine.Mathf: engine methods the game never calls can
// be missing from the build.
using System;
using Shortcut.RustMod;
using UnityEngine;

public enum RideMode { Off, Ground, Air, Grind, Bail }

public static class SkateRide
{
    public const float PushAccel = 6f, MaxPush = 8f, BrakeDecel = 12f, RollDecel = 0.45f, MaxSpeed = 13f, MaxReverse = 6f, TurnRate = 150f, SlopeGain = 1.6f;
    public const float AirGravity = 16f, OlliePop = 6f, MaxFall = 30f, BailImpact = 12f, BailSeconds = 0.9f, PushPeriod = 0.8f, SeaLevel = -0.6f, FootPace = 5.5f;
    private const float RayLift = 0.6f, FeetProbe = 0.3f, AirProbe = 3f;
    // Everything except the player's own layers, triggers, water, ragdolls and invisible helpers.
    public static readonly int GroundMask = ~((1 << 12) | (1 << 17) | (1 << 18) | (1 << 4) | (1 << 10) | (1 << 9) | (1 << 2));

    public static RideMode Mode = RideMode.Off;
    public static bool On { get { return Mode != RideMode.Off; } }
    public static bool Grounded, Braking, Jumped, TrickAir, Grab;
    // Test sessions hold the ride still and set the state themselves, to look at each pose.
    public static bool Frozen;
    public static float Speed, Yaw, Lean, VSpeed, AirTime, AirPeak, Clearance = 99f, Actual, ActualV, Top, Cap = MaxSpeed, ModeAt, LandAt = -99f, LandImpact, PushPhase;
    public static Vector3 Normal = Vector3.up, Tangent = Vector3.forward, ViewNormal = Vector3.up, Position;
    // Board rotation since take-off as the shared trick module counts it: positive spin is
    // frontside, positive flip is a kickflip.
    public static double SpinDeg, FlipDeg;
    public static SkateTrickState Trick;
    public static string TrickName = "", LastAir = "", OffReason = "";
    public static long TrickPoints, Banked;
    public static float TrickAt = -99f, BankedAt = -99f;
    public static int Steps, Resets, Pops, Lands, Bails, Pushes, GrindStarts, GrindEnds, Refused;
    private static Rigidbody body;
    private static Vector3 lastPos, lastDir, velocity;
    private static float lastSpeed, lastPull = -999f, now, takeOffY, spinLeft, flipLeft, jumpWanted = -99f;
    private static bool hasLast;
    private static int stuck;

    public static Vector3 Dir(float yaw) { return Quaternion.Euler(0f, yaw, 0f) * Vector3.forward; }
    public static float Toward(float a, float b, float step) { return Math.Abs(b - a) <= step ? b : a + Math.Sign(b - a) * step; }
    public static float Delta(float from, float to) { var d = (to - from) % 360f; if (d > 180f) d -= 360f; if (d < -180f) d += 360f; return d; }
    public static float Clamp(float v, float lo, float hi) { return v < lo ? lo : v > hi ? hi : v; }
    // A rotation about an axis, built from its fields so that it needs no engine call.
    public static Quaternion Turn(float degrees, Vector3 axis)
    {
        var n = axis.normalized; var half = degrees * (float)Math.PI / 360f; var s = (float)Math.Sin(half);
        var q = default(Quaternion); q.x = n.x * s; q.y = n.y * s; q.z = n.z * s; q.w = (float)Math.Cos(half);
        return q;
    }

    // Regular stance: a frontside spin turns the nose to the left.
    public static float SpinYaw { get { return -(float)SpinDeg; } }
    // Where the nose points. After an odd number of half spins the rider is switch: the nose trails.
    public static float NoseYaw { get { return Yaw + (Trick.Switch ? 180f : 0f) + SpinYaw; } }
    // True while the board moves tail first.
    public static bool TailFirst { get { return Trick.Switch != (Speed < -0.3f); } }
    public static bool Rotating { get { return Math.Abs(spinLeft) > 0.001f || Math.Abs(flipLeft) > 0.001f; } }

    public static void Mount(Rigidbody b, float yaw)
    {
        body = b; Yaw = yaw;
        // Whatever the player was doing carries over: a mount in mid-jump finishes that arc on the board.
        var v = b.linearVelocity; velocity = v; v.y = 0f;
        Speed = Vector3.Dot(v, Dir(yaw));
        SkateKeys.EndStep();
        hasLast = false; Steps = 0; Resets = 0; Top = 0f; Lean = 0f; Cap = MaxSpeed; lastPull = Time.realtimeSinceStartup;
        VSpeed = 0f; AirTime = 0f; spinLeft = flipLeft = 0f; SpinDeg = FlipDeg = 0; Jumped = TrickAir = Grab = Braking = false; PushPhase = 0f; stuck = 0;
        Trick = new SkateTrickState(false); TrickName = ""; TrickAt = BankedAt = LandAt = jumpWanted = -99f; OffReason = "";
        Normal = ViewNormal = Vector3.up; Position = b.position;
        SkateGrind.Reset();
        Enter(RideMode.Ground);
    }

    public static void Dismount(string reason)
    {
        if (Mode == RideMode.Off) return;
        Mode = RideMode.Off; OffReason = reason; Grab = false;
        SkateKeys.EndStep();
    }

    private static void Enter(RideMode m) { Mode = m; ModeAt = Time.realtimeSinceStartup; }

    public static string Status()
    {
        return Mode + " speed=" + Speed.ToString("F1") + " actual=" + Actual.ToString("F1") + " top=" + Top.ToString("F1") + " cap=" + Cap.ToString("F1") + " yaw=" + Yaw.ToString("F0")
            + " grounded=" + Grounded + " pull-backs=" + Resets + " total=" + Trick.TotalPoints + (body != null ? " pos=" + Out.V1(body.position) : "");
    }

    public static void Step()
    {
        if (Mode == RideMode.Off) return;
        try
        {
            if (body == null) { Dismount("the movement body is gone"); return; }
            if (Frozen) { hasLast = false; return; }
            var dt = Time.fixedDeltaTime;
            if (!(dt > 0.001f && dt <= 0.05f)) dt = 0.03125f;
            now = Time.realtimeSinceStartup;
            var pos = body.position; var game = body.linearVelocity;
            Position = pos;
            if (pos.y < SeaLevel && OpenSky(pos)) { Dismount("in the water"); return; }
            Measure(pos, dt);
            Sense(pos);
            if (Mode != RideMode.Grind) SkateGrind.Tick(dt);
            if (SkateKeys.JumpPressed) jumpWanted = now;
            if (Mode == RideMode.Ground) GroundStep(dt);
            else if (Mode == RideMode.Air) AirStep(pos, dt);
            else if (Mode == RideMode.Grind) GrindStep(pos, dt);
            else BailStep(game, dt);
            if (Mode == RideMode.Off) return;
            if (Cap < MaxSpeed && now - lastPull > 20f) { Cap = Math.Min(MaxSpeed, Cap + 1f); lastPull = now; }
            if (Speed > Top) Top = Speed;
            body.linearVelocity = velocity;
            lastPos = pos; lastDir = Dir(Yaw); lastSpeed = Speed; hasLast = true; Steps++;
            var target = Mode == RideMode.Ground ? Normal : Vector3.up;
            ViewNormal = (ViewNormal + (target - ViewNormal) * 0.3f).normalized;
            SkateKeys.EndStep();
        }
        catch (Exception e) { Dismount("the step threw"); Out.Say("SKATE step threw " + e.GetType().Name + ": " + e.Message); }
    }

    // What the body really did since the last write: a wall, or the server pulling the player back.
    private static void Measure(Vector3 pos, float dt)
    {
        if (!hasLast) return;
        var moved = pos - lastPos;
        ActualV = moved.y / dt; moved.y = 0f;
        Actual = Vector3.Dot(moved, lastDir) / dt;
        if ((lastSpeed > 1f && Actual < -3f) || (lastSpeed < -1f && Actual > 3f))
        {
            // The server put the player back. Above the pace it allows on foot that is a refused
            // speed: stay under it and try a little more later. At or below that pace it is something
            // else (it happens in the first seconds after waking up) and the cap is left alone.
            Resets++;
            if (Math.Abs(lastSpeed) > FootPace) { lastPull = now; Cap = Math.Max(FootPace, Math.Min(Cap, Math.Abs(lastSpeed) * 0.8f)); }
        }
        // An obstacle takes the speed away, in either direction of travel.
        if (Speed > 0f) { var limit = (Actual > 0f ? Actual : 0f) + 2f; if (Speed > limit) Speed = limit; }
        else if (Speed < 0f) { var limit = (Actual < 0f ? Actual : 0f) - 2f; if (Speed < limit) Speed = limit; }
    }

    // Land below sea level is under water unless something is overhead (tunnels, basements).
    private static bool OpenSky(Vector3 pos)
    {
        RaycastHit hit;
        return !Physics.Raycast(pos + Vector3.up * 2f, Vector3.up, out hit, 60f, GroundMask, QueryTriggerInteraction.Ignore);
    }

    private static void Sense(Vector3 pos)
    {
        RaycastHit hit;
        var reach = Mode == RideMode.Air ? AirProbe : FeetProbe;
        var got = Physics.Raycast(pos + Vector3.up * RayLift, Vector3.down, out hit, RayLift + reach, GroundMask, QueryTriggerInteraction.Ignore);
        Clearance = got ? hit.m_Distance - RayLift : 99f;
        Grounded = got && Clearance <= FeetProbe;
        Normal = Grounded && hit.m_Normal.y > 0.3f ? hit.m_Normal : Vector3.up;
    }

    private static void GroundStep(float dt)
    {
        // The ground fell away (a drop, the lip of a ramp): carry on as a ballistic arc.
        if (!Grounded) { TakeOff(false, velocity.y + AirGravity * dt); Sense(Position); AirStep(Position, dt); return; }
        var before = Yaw;
        var steer = (SkateKeys.Left ? -1f : 0f) + (SkateKeys.Right ? 1f : 0f);
        var maxTurn = TurnRate / (1f + Math.Abs(Speed) / 8f) * dt;
        Yaw += Clamp(Delta(Yaw, SkateKeys.LookYaw + steer * 40f), -maxTurn, maxTurn);
        var dir = Dir(Yaw);
        var tangent = dir - Normal * Vector3.Dot(dir, Normal);
        tangent = tangent.sqrMagnitude > 0.0001f ? tangent.normalized : dir;
        Tangent = tangent; Braking = SkateKeys.Brake;
        // A press up to a moment before touch-down still counts, so tricks can be chained.
        if (now - jumpWanted < 0.12f)
        {
            Pops++; jumpWanted = -99f;
            TakeOff(true, tangent.y * Speed + OlliePop * Normal.y + AirGravity * dt);
            AirStep(Position, dt);
            return;
        }
        Speed += Vector3.Dot(Physics.gravity, tangent) * SlopeGain * dt;
        var pushing = SkateKeys.Push && !SkateKeys.Brake && Speed < MaxPush;
        if (pushing) Speed = Math.Min(MaxPush, Speed + PushAccel * dt);
        if (SkateKeys.Brake) Speed = Toward(Speed, 0f, BrakeDecel * dt);
        Speed = Toward(Speed, 0f, RollDecel * dt);
        Speed = Clamp(Speed, -Math.Min(MaxReverse, Cap), Cap);
        // The pushing foot finishes its stroke once started.
        if (pushing || PushPhase > 0f)
        {
            if (PushPhase <= 0f) Pushes++;
            PushPhase += dt / PushPeriod;
            if (PushPhase >= 1f) PushPhase = 0f;
        }
        velocity = dir * Speed;
        // Follow the slope, and settle onto the ground when hovering just above it.
        velocity.y = tangent.y * Speed - Clamp(Clearance * 8f, 0f, 3f);
        Lean += (Clamp(Delta(before, Yaw) / dt * 0.12f, -25f, 25f) - Lean) * 0.2f;
        TrickTick(false, false, 0, 0, 0, dt);
    }

    private static void TakeOff(bool jumped, float vertical)
    {
        Enter(RideMode.Air);
        Jumped = jumped; TrickAir = jumped; VSpeed = vertical; AirTime = 0f; AirPeak = 0f; takeOffY = Position.y;
        spinLeft = flipLeft = 0f; PushPhase = 0f; Braking = false; stuck = 0;
    }

    // Seconds until the feet reach the ground under the rider on the current arc.
    private static float TimeToGround()
    {
        var h = Clearance > AirProbe ? AirProbe : Clearance < 0f ? 0f : Clearance;
        return (VSpeed + (float)Math.Sqrt(VSpeed * VSpeed + 2f * AirGravity * h)) / AirGravity;
    }

    private static void AirStep(Vector3 pos, float dt)
    {
        AirTime += dt;
        VSpeed -= AirGravity * dt; if (VSpeed < -MaxFall) VSpeed = -MaxFall;
        if (pos.y - takeOffY > AirPeak) AirPeak = pos.y - takeOffY;
        var dir = Dir(Yaw);
        Tangent = dir; Lean += (0f - Lean) * 0.2f;

        // A tap queues one whole flip, a held or tapped side key one half spin; each is accepted
        // only when the arc lasts long enough to finish it.
        var left = (float)(TimeToGround() - 0.03);
        if (SkateKeys.FlipUp != SkateKeys.FlipDown)
        {
            var sign = SkateKeys.FlipUp ? 1f : -1f;
            if (Math.Abs(flipLeft) < 0.001f || Math.Sign(flipLeft) == Math.Sign(sign))
            {
                if (left >= (Math.Abs(flipLeft) + 360f) / (float)SkateMotion.FlipRate) { flipLeft += 360f * sign; TrickAir = true; }
                else Refused++;
            }
        }
        var side = Jumped ? (SkateKeys.Left ? -1f : 0f) + (SkateKeys.Right ? 1f : 0f) : (SkateKeys.LeftPressed ? -1f : 0f) + (SkateKeys.RightPressed ? 1f : 0f);
        if (side != 0f && Math.Abs(spinLeft) < 0.001f)
        {
            if (left >= 180f / (float)SkateMotion.RotationRate) { spinLeft = -180f * side; TrickAir = true; }
            else if (SkateKeys.LeftPressed || SkateKeys.RightPressed) Refused++;
        }
        Grab = TrickAir && SkateKeys.Crouch;

        var spinStep = (float)SkateMotion.RotationRate * dt; var flipStep = (float)SkateMotion.FlipRate * dt;
        var spinAxis = Clamp(spinLeft / spinStep, -1f, 1f); var flipAxis = Clamp(flipLeft / flipStep, -1f, 1f);

        // Touch-down: the feet reach the ground within this step, or the body has come to rest on something.
        stuck = VSpeed < -2f && Math.Abs(ActualV) < 0.3f && AirTime > 0.2f ? stuck + 1 : 0;
        if ((VSpeed <= 0f && Clearance <= Math.Max(0.04f, -VSpeed * dt + 0.02f)) || stuck >= 3)
        {
            var impact = stuck >= 3 ? 0f : -VSpeed;
            Land(impact, dt);
            if (Mode == RideMode.Ground)
            {
                velocity = dir * Speed;
                velocity.y = -Clamp(Clearance / dt, 0f, impact);
            }
            else BailStep(velocity, dt);
            return;
        }
        velocity = dir * Speed + Vector3.up * VSpeed;
        if (VSpeed < 0f && !Rotating && SkateGrind.TryCapture(pos, velocity, NoseYaw, dt))
        {
            LastAir = AirLine("grind");
            Enter(RideMode.Grind); GrindStarts++;
            Yaw = SkateGrind.Heading; Speed = SkateGrind.Speed; VSpeed = 0f;
            TrickTick(false, true, 0, 0, Math.Min(-velocity.y, (float)SkateMotion.MaximumLandingFallSpeed), dt);
            TrickAir = Jumped = false; AirTime = 0f;
            GrindVelocity(pos);
            return;
        }
        if (TrickAir)
        {
            TrickTick(true, false, spinAxis, flipAxis, 0, dt);
            spinLeft -= spinAxis * spinStep; flipLeft -= flipAxis * flipStep;
        }
        else TrickTick(false, false, 0, 0, 0, dt);
    }

    private static string AirLine(string end)
    {
        return "air " + end + " time=" + AirTime.ToString("F2") + " peak=" + AirPeak.ToString("F2") + " impact=" + (-VSpeed).ToString("F1") + " spin=" + SpinDeg.ToString("F0") + " flip=" + FlipDeg.ToString("F0")
            + " jumped=" + Jumped + " speed=" + Speed.ToString("F1");
    }

    private static void Land(float impact, float dt)
    {
        LastAir = AirLine("land");
        if (AirTime > 0.12f || impact > 1.5f) { Lands++; LandAt = now; LandImpact = impact; }
        Enter(RideMode.Ground);
        if (TrickAir)
        {
            // The shared module bails above 8 m/s, a drop of under two metres; the client's own limit
            // is BailImpact, so anything between the two is reported as the module's maximum.
            var reported = impact > BailImpact ? impact : Math.Min(impact, (float)SkateMotion.MaximumLandingFallSpeed);
            TrickTick(false, false, 0, 0, reported, dt);
        }
        else if (impact > BailImpact) Bail();
        else TrickTick(false, false, 0, 0, 0, dt);
        spinLeft = flipLeft = 0f; TrickAir = Jumped = Grab = false; AirTime = 0f;
        if (Mode == RideMode.Ground) { SpinDeg = 0; FlipDeg = 0; }
    }

    // One observation for the shared trick module; its events become the lines the HUD shows.
    private static void TrickTick(bool airborne, bool grinding, float spinAxis, float flipAxis, float impact, float dt)
    {
        SkateTrickResult r; string error;
        if (!SkateTricks.TryStep(Trick, new SkateTrickInput(spinAxis, flipAxis, Grab, airborne, grinding, impact), dt, out r, out error))
        {
            Out.Say("SKATE trick step refused: " + error);
            return;
        }
        Trick = r.State;
        SpinDeg = airborne ? r.BoardSpin : 0; FlipDeg = airborne ? r.BoardFlip : 0;
        if ((r.Events & SkateTrickEvents.Bailed) != 0) { Bail(); return; }
        if ((r.Events & (SkateTrickEvents.Landed | SkateTrickEvents.GrindEnded)) != 0)
        {
            TrickName = r.TrickName;
            TrickPoints = r.Points; TrickAt = now;
            Out.Say("SKATE trick " + r.TrickName + " +" + r.Points + " combo=" + r.ComboPoints + " x" + Trick.ComboCount + " switch=" + Trick.Switch);
        }
        if ((r.Events & SkateTrickEvents.ComboBanked) != 0)
        {
            Banked = r.BankedPoints; BankedAt = now;
            Out.Say("SKATE combo banked +" + r.BankedPoints + " total=" + Trick.TotalPoints);
        }
    }

    public static void Bail()
    {
        if (Mode == RideMode.Bail || Mode == RideMode.Off) return;
        SkateTrickResult r; string error;
        if (SkateTricks.TryBail(Trick, out r, out error)) Trick = r.State;
        if (Mode == RideMode.Grind) GrindEnds++;
        Enter(RideMode.Bail); Bails++;
        TrickName = "Bail"; TrickPoints = 0; TrickAt = now;
        spinLeft = flipLeft = 0f; TrickAir = Jumped = Grab = false;
        Out.Say("SKATE bail | " + LastAir);
    }

    // The rider is off the board: the game's own gravity takes the fall, the roll dies out, then walking resumes.
    private static void BailStep(Vector3 game, float dt)
    {
        Speed = Toward(Speed, 0f, 10f * dt);
        velocity = Dir(Yaw) * Speed; velocity.y = game.y < 0f ? game.y : 0f;
        if (now - ModeAt > BailSeconds) Dismount("bail");
    }

    private static void GrindStep(Vector3 pos, float dt)
    {
        bool ended;
        if (now - jumpWanted < 0.12f)
        {
            // Ollie out of the grind; the combo carries on.
            Pops++; GrindEnds++; jumpWanted = -99f; SkateGrind.Release(dt);
            TakeOff(true, OlliePop + AirGravity * dt);
            AirStep(pos, dt);
            return;
        }
        if (!SkateGrind.TryContinue(pos, dt, out ended) || ended)
        {
            GrindEnds++; SkateGrind.Release(dt);
            Speed = SkateGrind.Speed;
            TakeOff(false, 0f);
            velocity = Dir(Yaw) * Speed;
            TrickTick(false, false, 0, 0, 0, dt);
            return;
        }
        Speed = SkateGrind.Speed; Yaw = SkateGrind.Heading;
        Tangent = Dir(Yaw);
        GrindVelocity(pos);
        TrickTick(false, true, 0, 0, 0, dt);
    }

    // Along the edge, pulled gently onto its line and held at its height.
    private static void GrindVelocity(Vector3 pos)
    {
        var to = SkateGrind.Target - pos;
        var side = to; side.y = 0f;
        var dir = Dir(Yaw);
        side = side - dir * Vector3.Dot(side, dir);
        var pull = side * 8f;
        if (pull.sqrMagnitude > 6.25f) pull = pull.normalized * 2.5f;
        velocity = dir * Speed + pull;
        velocity.y = Clamp(to.y * 10f, -3f, 3f);
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
        SkateRig.LateFrame();
    }
}
